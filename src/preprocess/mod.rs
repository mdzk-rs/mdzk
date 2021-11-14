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

                // Populate index of all paths for use with wikilinks
                if ch.path.is_some() {
                    let key = ch.name.clone();
                    if path_map.contains_key(&key) {
                        warn!(
                            r#"Duplicated page title found:

- {} ({:?})
- {} ({:?})

If links do not properly specify paths, they might lead to the wrong note..."#,
                            key,
                            ch.path,
                            key,
                            path_map.get(&key),
                        );
                    }
                    path_map.insert(key, ch.path.as_ref().unwrap().clone());
                }

            }
        });

        // Second iteration
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                // Parse and convert all wikilinks to CommonMark
                if wikilinks {
                    wikilinks::for_each_wikilink(&ch.content.clone(), |link_text| {
                        let wikilink = match wikilinks::WikiLink::from_with_index(link_text, &path_map) {
                            Ok(wl) => wl,
                            Err(e) => {
                                warn!("{}", e);
                                return;
                            }
                        };

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

        Ok(book)
    }
}
