use lazy_regex::{lazy_regex, Lazy};
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pathdiff;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use toml::value::Value;

static WIKILINK_REGEX: Lazy<Regex> =
    lazy_regex!(r"\[\[(?P<link>[^\]\|]+)(?:\|(?P<title>[^\]]+))?\]\]");

pub struct Backlinks;

impl Preprocessor for Backlinks {
    fn name(&self) -> &str {
        "backlinks"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // index maps each chapters source_path to its backlinks
        let mut index = create_backlink_map(&book);

        // Populate index backlinks
        for item in book.iter() {
            if let BookItem::Chapter(ch) = item {
                // Skip if source_path is None (because nothing to link to)
                if let Some(_) = &ch.source_path {
                    // Loop over the internal links found in the chapter
                    for link in find_links(&ch.content, &index) {
                        // Push current chapter into Vec corresponding to the chapter it links to
                        if let Some(backlinks) = index.get_mut(&link) {
                            backlinks.push(ch.clone());
                        }
                    }
                }
            }
        }

        // Should probably clean up this, but why bother?...
        let backlink_prefix = if let Some(header) = ctx.config.get("preprocessor.backlinks.header")
        {
            if let Value::String(val) = header {
                format!("\n\n---\n\n## {}\n\n", val)
            } else {
                eprintln!(
                    "Warning: You must use a string value as the backlink header. Skipping..."
                );
                String::from("\n\n---\n\n")
            }
        } else {
            String::from("\n\n")
        };

        // Add backlinks to each chapter
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                if let Some(source_path) = &ch.source_path {
                    if let Some(backlinks) = index.get(source_path) {
                        if backlinks.len() >= 1 {
                            ch.content += &backlink_prefix;
                        };

                        for backlink in backlinks.iter() {
                            // This is really ugly, but the unwraps should be safe
                            let dest = backlink.source_path.clone().unwrap();
                            let diff_path =
                                pathdiff::diff_paths(dest, source_path.parent().unwrap()).unwrap();
                            ch.content += &format!(
                                "> - [{}](<{}>)\n",
                                backlink.name,
                                escape_special_chars(diff_path.to_str().unwrap())
                            );
                        }
                    }
                }
            }
        });

        Ok(book)
    }
}

/// Finds all the links in content linking to chapters listed in index.
fn find_links(content: &str, index: &HashMap<PathBuf, Vec<Chapter>>) -> Vec<PathBuf> {
    let mut links: Vec<PathBuf> = Vec::new();

    for cap in WIKILINK_REGEX.captures_iter(&content) {
        if let Some(dest) = cap.get(1) {
            let mut path = PathBuf::from(dest.as_str());
            path.set_extension("md");
            if index.contains_key(&path) {
                links.push(path);
            }
        }
    }
    links.sort_unstable();
    links.dedup();
    links
}

/// Creates an index that maps all the book's chapter's source_path to an empty vector of backlinks
fn create_backlink_map(book: &Book) -> HashMap<PathBuf, Vec<Chapter>> {
    let mut map = HashMap::new();
    for item in book.iter() {
        if let BookItem::Chapter(ch) = item {
            if let Some(source_path) = &ch.source_path {
                map.insert(source_path.clone(), Vec::new());
            }
        }
    }
    map
}

fn escape_special_chars(text: &str) -> String {
    text.replace(" ", "%20")
        .replace("<", "%3C")
        .replace(">", "%3E")
        .replace('?', "%3F")
}
