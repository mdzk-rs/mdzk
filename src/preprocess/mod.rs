mod backlinks;
mod frontmatter;
mod katex;
pub mod readme;
mod wikilinks;

use anyhow::Result;
use lazy_regex::regex;
use mdbook::{
    book::{Book, BookItem},
    preprocess::{Preprocessor, PreprocessorContext},
};
use std::collections::HashMap;
use toml::Value;

pub struct MdzkPreprocessor;

impl Preprocessor for MdzkPreprocessor {
    fn name(&self) -> &str {
        "mdzk"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        // Check which preprocessors are toggled on
        let wikilinks = get_toml_value(
            ctx.config.get("mdzk.wikilinks"),
            "Wikilink parsing is disabled.",
        );
        let math = get_toml_value(ctx.config.get("mdzk.math"), "Math rendering is disabled.");
        let readme = get_toml_value(
            ctx.config.get("mdzk.readme"),
            "\"README.md\" to \"index.md\" conversion disabled.",
        );
        let front_matter = get_toml_value(
            ctx.config.get("mdzk.front-matter"),
            "Front matter parsing is disabled.",
        );
        let backlinks = get_toml_value(ctx.config.get("mdzk.backlinks"), "Backlinks disabled.");

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
                if wikilinks && ch.path.is_some() {
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

                        // Popoulate backlinks_map with this chapter
                        if backlinks {
                            if let Some(ref path) = wikilink.path {
                                if let Some(backlinks) = backlinks_map.get_mut(path) {
                                    backlinks.push((ch.path.clone().unwrap(), ch.name.clone()));
                                }
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

        if backlinks {
            let prefix = backlinks::prefix(&ctx.config);

            // Third iteration
            book.for_each_mut(|item| {
                if let BookItem::Chapter(ch) = item {
                    backlinks::insert_backlinks(ch, &backlinks_map, &prefix);
                }
            });
        }

        Ok(book)
    }
}

fn get_toml_value(val: Option<&Value>, message: &str) -> bool {
    match val {
        Some(toml::Value::Boolean(false)) => {
            info!("{}", message);
            false
        }
        _ => true,
    }
}
