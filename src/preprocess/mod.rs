mod katex;
pub mod readme;
mod wikilinks;

use anyhow::Result;
use lazy_regex::regex;
use mdbook::{
    book::{Book, BookItem, Chapter},
    preprocess::{Preprocessor, PreprocessorContext},
};
use std::collections::HashMap;

pub struct MdzkPreprocessor;

impl Preprocessor for MdzkPreprocessor {
    fn name(&self) -> &str {
        "mdzk"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        // Populate hashmap of each chapter's path.
        let mut path_map = HashMap::new();
        for chapter in book.iter().filter_map(get_chapter) {
            if chapter.path.is_some() {
                let key = chapter.name.clone();
                if path_map.contains_key(&key) {
                    warn!(r#"Duplicated page title found:

- {} ({:?})
- {} ({:?})

If links do not properly specify paths, they might lead to the wrong note..."#,
                        key, chapter.path,
                        key, path_map.get(&key),
                    );
                }
                path_map.insert(key, chapter.path.as_ref().unwrap().clone());
            }
        }

        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                wikilinks::for_each_wikilink(&ch.content.clone(), |link_text| {
                    let wikilink = match wikilinks::WikiLink::from(link_text) {
                        Ok(wl) => wl,
                        Err(e) => {
                            warn!("{}", e);
                            return
                        },
                    };

                    ch.content = ch.content.replacen(
                        &format!("[[{}]]", link_text),
                        &wikilink.cmark(ch.path.as_ref().unwrap().parent().unwrap(), &path_map),
                        1,
                    );
                });

                katex::render(ch);

                readme::convert_readme(
                    ch,
                    ctx.root.join(&ctx.config.book.src),
                    regex!(r"(?i)\[(.*?)\]\(<?README(?:(?:\.md)|(?:\.markdown))>?\)"),
                )

        ;
            }
        });

        Ok(book)
    }
}

fn get_chapter(it: &BookItem) -> Option<&Chapter> {
    if let BookItem::Chapter(ch) = it {
        Some(ch)
    } else {
        None
    }
}
