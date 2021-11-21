mod frontmatter;
mod katex;
pub mod readme;
mod wikilinks;

use crate::utils::diff_paths;

use anyhow::Result;
use lazy_regex::regex;
use mdbook::{
    book::{Book, BookItem},
    preprocess::{Preprocessor, PreprocessorContext},
};
use std::collections::HashMap;

pub struct MdzkPreprocessor;

impl Preprocessor for MdzkPreprocessor {
    fn name(&self) -> &str {
        "mdzk"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        // Check which preprocessors are toggled on
        let wikilinks = match ctx.config.get("mdzk.wikilinks") {
            Some(toml::Value::Boolean(false)) => {
                info!("Wikilink parsing is disabled.");
                false
            }
            _ => true,
        };
        let math = match ctx.config.get("mdzk.math") {
            Some(toml::Value::Boolean(false)) => {
                info!("Math rendering is disabled.");
                false
            }
            _ => true,
        };
        let readme = match ctx.config.get("mdzk.readme") {
            Some(toml::Value::Boolean(false)) => {
                info!("\"README.md\" to \"index.md\" conversion disabled.");
                false
            }
            _ => true,
        };
        let front_matter = match ctx.config.get("mdzk.front-matter") {
            Some(toml::Value::Boolean(false)) => {
                info!("Front matter parsing is disabled.");
                false
            }
            _ => true,
        };

        // First iteration
        let mut path_map = HashMap::new();
        let mut backlinks_map = HashMap::new();
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                // Handle front matter
                if front_matter {
                    frontmatter::run(ch);
                }

                // Convert README.md into index.md
                if readme {
                    readme::run(
                        ch,
                        ctx.root.join(&ctx.config.book.src),
                        regex!(r"(?i)\[(.*?)\]\(<?README(?:(?:\.md)|(?:\.markdown))>?\)"),
                    )
                }

                // Populate path_map with note names and paths.
                // Populate backlinks_map with note paths and an empty vector.
                if ch.path.is_some() {
                    let key = ch.name.clone();
                    let path = ch.path.as_ref().unwrap().to_owned();
                    if path_map.contains_key(&key) {
                        warn!(
                            r#"Duplicated page title found:

- {} ({:?})
- {0} ({:?})

If links do not properly specify paths, they might lead to the wrong note..."#,
                            key,
                            path,
                            path_map.get(&key),
                        );
                    }
                    path_map.insert(key, path.clone());
                    backlinks_map.insert(path, Vec::new());
                }
            }
        });

        // Second iteration
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                // Parse and convert all wikilinks to CommonMark
                if wikilinks {
                    wikilinks::for_each_wikilink(&ch.content.clone(), |link_text| {
                        let wikilink =
                            match wikilinks::WikiLink::from_with_index(link_text, &path_map) {
                                Ok(wl) => wl,
                                Err(e) => {
                                    warn!("{}", e);
                                    return;
                                }
                            };

                        if let Some(ref path) = wikilink.path {
                            if let Some(backlinks) = backlinks_map.get_mut(path) {
                                backlinks.push((ch.path.clone().unwrap(), ch.name.clone()));
                            }
                        }

                        ch.content = ch.content.replacen(
                            &format!("[[{}]]", link_text),
                            &wikilink.cmark(ch.path.as_ref().unwrap().parent().unwrap()),
                            1,
                        );
                    });
                }

                // Generate KaTeX spans/divs
                if math {
                    katex::run(ch);
                }
            }
        });

        // Third iteration
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                if let Some(path) = &ch.path {
                    if let Some(backlinks) = backlinks_map.get(path) {
                        if !backlinks.is_empty() {
                            // ch.content += &backlink_prefix;
                            for (dest, name) in backlinks.iter() {
                                let diff_path = diff_paths(dest, path.parent().unwrap()).unwrap();
                                ch.content += &format!(
                                    "> - [{}](<{}>)\n",
                                    name,
                                    diff_path.to_string_lossy()
                                );
                            }
                        };

                    }
                }
            }
        });

        Ok(book)
    }
}
