mod wikilinks;

use anyhow::Result;
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

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        // Populate hashmap of each chapter's path.
        let mut path_map = HashMap::new();
        for chapter in book.iter().filter_map(get_chapter) {
            if !chapter.path.is_none() {
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
