use lazy_regex::{lazy_regex, Lazy};
use mdbook::{
    book::{Book, Chapter},
    errors::Error,
    preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext},
    BookItem,
};
use regex::{Captures, Regex};
use std::{collections::HashMap, io, path::PathBuf};
use pulldown_cmark::{CowStr, Event, Parser};

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
                pulldown_method(&chapter.content.clone(), |link_text| {
                    chapter.content = chapter.content.replacen(&format!("[[{}]]", link_text), "!!This was link!!", 1);
                });

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

#[derive(Debug)]
enum WikiTag {
    OutsideLink,
    MaybeOpen,
    MaybeInsideLink,
    MaybeClose,
    Ignore,
}

fn pulldown_method(content: &str, mut handle_link: impl FnMut(&str)) {
    let parser = Parser::new(content);

    let mut buffer = String::new();
    let mut current = WikiTag::OutsideLink;
    for event in parser {
        match event {
            Event::Text(CowStr::Borrowed("[")) => {
                match current {
                    WikiTag::OutsideLink => current = WikiTag::MaybeOpen,
                    WikiTag::MaybeOpen => current = WikiTag::MaybeInsideLink,
                    WikiTag::MaybeInsideLink => current = WikiTag::OutsideLink,
                    WikiTag::MaybeClose => {
                        buffer.clear();
                        current = WikiTag::OutsideLink;
                    }
                    WikiTag::Ignore => {}
                }
            }

            Event::Text(CowStr::Borrowed("]")) => {
                match current {
                    WikiTag::MaybeOpen => current = WikiTag::OutsideLink,
                    WikiTag::MaybeInsideLink => current = WikiTag::MaybeClose,
                    WikiTag::MaybeClose => {
                        if !buffer.contains('\n') {
                            handle_link(&buffer.trim());
                        }
                        buffer.clear();
                        current = WikiTag::OutsideLink;
                    }
                    WikiTag::OutsideLink => {},
                    WikiTag::Ignore => {}
                }
            }

            Event::Text(ref text) => {
                match current {
                    WikiTag::MaybeInsideLink => {
                        if buffer.is_empty() {
                            buffer.push_str(text);
                        } else {
                            // Buffer contains something, which means a newline or something else
                            // split it up. Clear buffer and don't consider this a link.
                            buffer.clear();
                            current = WikiTag::OutsideLink;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
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
    use super::*;

    #[test]
    fn detect_these() {
        let content = r#"This is a note with four links:

This one [[link]], this one [[ link#header ]], this one [[   link | a bit more complex]], and this one [[     link#header | more 😭 complex]].

> This is a [[link in a blockquote]]

- List item
- Second list item with [[list link]]

| Header 1 | Header 2 |
| -------- | -------- |
| Tables can also have [[table links]] | more stuff |"#;

        let mut links = vec![];
        pulldown_method(content, |link_text| { links.push(link_text.to_owned()); });

        assert_eq!(links, vec![
                   "link",
                   "link#header",
                   "link | a bit more complex",
                   "link#header | more 😭 complex",
                   "link in a blockquote",
                   "list link",
                   "table links"
        ]);
    }

    #[test]
    fn dont_detect_these() {
        let content = r#"Here are some non-correct links:

First a link [[with
newline]]

Then a link `inside [[inline code]]`, or inside <span class="katex-inline">inline [[math]]</span>. What about \[\[escaped brackets\]\]?

<div class="katex-display">
    f(x) = \text{[[display link]]}
</div>

```rust
let link = "[[link_in_code]]".to_owned();
```

<p>
  This is some raw HTML. We don't want [[html links]] detected here.
</p>"#;

        let mut links = Vec::<String>::new();
        pulldown_method(content, |link_text| { links.push(link_text.to_owned()); });

        assert!(links.is_empty(), "Got links: {:?}", links);
    }

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
