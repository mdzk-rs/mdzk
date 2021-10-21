use lazy_regex::{lazy_regex, Lazy};
use mdbook::{
    book::{Book, Chapter},
    errors::Error,
    preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext},
    BookItem,
};
use regex::{Captures, Regex};
use std::{collections::HashMap, io, path::PathBuf};
use pulldown_cmark::{html::push_html, CowStr, Event, Options, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;

static WIKILINK_REGEX: Lazy<Regex> =
    lazy_regex!(r"\[\[(?P<link>[^\]\|]+)(?:\|(?P<title>[^\]]+))?\]\]");

pub fn handle_preprocessing(pre: impl Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    //serde_json::to_writer_pretty(io::stderr(), &processed_book);
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn chapter(it: &BookItem) -> Option<&Chapter> {
    if let BookItem::Chapter(ch) = it {
        Some(ch)
    } else {
        None
    }
}

pub struct WikiLinks;

impl Preprocessor for WikiLinks {
    fn name(&self) -> &str {
        "wikilink-preprocessor"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let mut chapters_map = HashMap::new();
        for chapter in book.iter().filter_map(chapter) {
            let key = chapter.name.clone();
            if chapter.path.is_none() {
                continue;
            }
            if chapters_map.contains_key(&key) {
                eprintln!("duplicated page title found: {} at {:?}", key, chapter.path);
            }
            chapters_map.insert(key, chapter.path.as_ref().unwrap().clone());
        }

        book.for_each_mut(|it| {
            if let BookItem::Chapter(chapter) = it {
                chapter.content = pulldown_method(&chapter.content);

                // chapter.content = regex_method(&chapter, &chapters_map);
            }
        });

        Ok(book)
    }
}

fn regex_method(chapter: &Chapter, chapters_map: &HashMap<String, PathBuf>) -> String {
    WIKILINK_REGEX
        .replace_all(&chapter.content, |it: &Captures| -> String {
            let key = it.name("link").unwrap().as_str().trim();
            if !chapters_map.contains_key(key) {
                return (maud::html! {
                    span.missing-link style="color:darkred;" {
                       (key)
                    }
                })
                .into_string();
            }
            let title = it
                .name("title")
                .map(|x| x.as_str().trim().to_string())
                .unwrap_or(key.to_string());
            let diff_path = pathdiff::diff_paths(
                chapters_map.get(key).unwrap(),
                chapter.path.as_ref().unwrap().parent().unwrap(),
            )
            .unwrap();

            return format!(
                "[{}](<{}>)",
                title,
                escape_special_chars(&diff_path.to_string_lossy())
            );
        })
        .to_string()
}

enum WikiTag {
    Link,
    MaybeLink,
    Waiting,
    Ignore,
}

fn pulldown_method(content: &str) -> String {
    let mut out = content.clone().to_owned();

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(content, opts);
    let mut link = WikiTag::Ignore;
    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {
                link = WikiTag::Waiting;
            }
            Event::End(Tag::Paragraph) => {
                link = WikiTag::Ignore;
            }
            Event::Text(CowStr::Borrowed("[")) => {
                match link {
                    WikiTag::Waiting => link = WikiTag::MaybeLink,
                    WikiTag::MaybeLink => link = WikiTag::Link,
                    WikiTag::Link => link = WikiTag::Waiting,
                    WikiTag::Ignore => {}
                }
            }
            Event::Text(ref text) => {
                // Really messy solution, need to match closing brackets as well.
                // This works as a proof of concept in my own vault, since I never use `[[`
                // anywhere but for wikilinks.
                match link {
                    WikiTag::Link => {
                        out = out.replace(
                            &format!("[[{}]]", text),
                            &format!("!!This was link!!")
                        );
                        link = WikiTag::Waiting;
                    }
                    WikiTag::MaybeLink => link = WikiTag::Waiting,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    out
}

/// Escape characters for usage in URLs
fn escape_special_chars(text: &str) -> String {
    text.replace(" ", "%20")
        .replace("<", "%3C")
        .replace(">", "%3E")
        .replace('?', "%3F")
}

#[cfg(test)]
mod tests {
    use crate::WIKILINK_REGEX;

    #[test]
    fn extract_link_regex() {
        let cases = [
            ("[[Link]]", "Link"),
            ("[[🪴 Sowing<Your>Garden]]", "🪴 Sowing<Your>Garden"),
            (
                "[[/Templates/🪴 Sowing<Your>Garden]]",
                "/Templates/🪴 Sowing<Your>Garden",
            ),
        ];

        for (case, expected) in &cases {
            let got = WIKILINK_REGEX
                .captures(case)
                .unwrap()
                .name("link")
                .unwrap()
                .as_str();
            assert_eq!(got.trim(), *expected);
        }
    }

    #[test]
    fn extract_title_regex() {
        let cases = [
            ("[[Link | My New Link]]", "My New Link"),
            ("[[🪴 Sowing<Your>Garden | 🪴 Emoji Link]]", "🪴 Emoji Link"),
            ("[[🪴 Sowing<Your>Garden | 🪴/Emoji/Link]]", "🪴/Emoji/Link"),
        ];

        for (case, expected) in &cases {
            let got = WIKILINK_REGEX
                .captures(case)
                .unwrap()
                .name("title")
                .unwrap()
                .as_str();
            assert_eq!(got.trim(), *expected)
        }
    }
}
