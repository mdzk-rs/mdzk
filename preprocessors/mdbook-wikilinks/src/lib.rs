extern crate pest;
#[macro_use]
extern crate pest_derive;

use mdbook::{
    book::{Book, Chapter},
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
    utils::id_from_content,
};
use pest::Parser;
use std::collections::HashMap;
use pulldown_cmark::{CowStr, Event, escape::escape_href};

#[derive(Parser)]
#[grammar = "wikilink.pest"]
pub struct WikiLinkParser;

pub struct WikiLinks;

impl Preprocessor for WikiLinks {
    fn name(&self) -> &str {
        "wikilink-preprocessor"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let mut path_map = HashMap::new();
        for chapter in book.iter().filter_map(get_chapter) {
            let key = chapter.name.clone();
            if chapter.path.is_none() { continue; }
            if path_map.contains_key(&key) {
                eprintln!("Duplicated page title found: {} at {:?}", key, chapter.path);
            }
            path_map.insert(key, chapter.path.as_ref().unwrap().clone());
        }

        book.for_each_mut(|it| {
            if let BookItem::Chapter(chapter) = it {
                for_each_link(&chapter.content.clone(), |link_text| {
                    let mut link = match WikiLinkParser::parse(Rule::link, link_text) {
                        Ok(parsed) => parsed,
                        Err(e) => {
                            eprintln!("Failed parsing wikilink internals: {}", e);
                            return
                        },
                    }.next()
                     .unwrap()
                     .into_inner();

                    // Handle destination
                    let mut dest = link.next().unwrap().into_inner();
                    let note = dest.next().unwrap().as_str();

                    // Handle link text
                    let title = match link.next() {
                        Some(alias) => alias.as_str(),
                        None => note.as_ref(),
                    };

                    let cmark_link = if !path_map.contains_key(note) {
                        format!(
                            "<span class=\"missing-link\" style=\"color:darkred;\">{}</span>", 
                            title
                        )
                    } else {
                        let mut href = pathdiff::diff_paths(
                            path_map.get(note).unwrap(),
                            chapter.path.as_ref().unwrap().parent().unwrap(),
                        ).unwrap().to_string_lossy().to_string(); // Gotta love Rust <3

                        // Handle anchor
                        // TODO: Blockrefs are currently not handled here
                        if let Some(anchor) = dest.next() {
                            let header_kebab = id_from_content(&anchor.as_str()[1..]);
                            href.push_str(&format!("#{}", header_kebab));
                        }

                        format!("[{}](<{}>)", title, escape_special_chars(&href))
                    };

                    chapter.content = chapter.content
                        .replacen(&format!("[[{}]]", link_text), &cmark_link, 1);
                });
            }
        });

        Ok(book)
    }
}

fn for_each_link(content: &str, mut handle_link: impl FnMut(&str)) {
    enum Currently {
        OutsideLink,
        MaybeOpen,
        MaybeInsideLink,
        MaybeClose,
        Ignore,
    }

    let parser = pulldown_cmark::Parser::new(content);

    let mut buffer = String::new();
    let mut current = Currently::OutsideLink;
    for event in parser {
        match event {
            // Ignore KaTeX spans
            Event::Html(CowStr::Borrowed("<span class=\"katex-inline\">")) => current = Currently::Ignore,
            Event::Html(CowStr::Borrowed("</span>")) => current = Currently::OutsideLink,

            Event::Text(CowStr::Borrowed("[")) => {
                match current {
                    Currently::OutsideLink => current = Currently::MaybeOpen,
                    Currently::MaybeOpen => current = Currently::MaybeInsideLink,
                    Currently::MaybeInsideLink => current = Currently::OutsideLink,
                    Currently::MaybeClose => {
                        buffer.clear();
                        current = Currently::OutsideLink;
                    }
                    Currently::Ignore => {}
                }
            }

            Event::Text(CowStr::Borrowed("]")) => {
                match current {
                    Currently::MaybeOpen => current = Currently::OutsideLink,
                    Currently::MaybeInsideLink => current = Currently::MaybeClose,
                    Currently::MaybeClose => {
                        handle_link(&buffer.trim());
                        buffer.clear();
                        current = Currently::OutsideLink;
                    }
                    Currently::OutsideLink => {},
                    Currently::Ignore => {}
                }
            }

            Event::Text(ref text) => {
                if let Currently::MaybeInsideLink = current {
                    if buffer.is_empty() {
                        buffer.push_str(text);
                    } else {
                        // Buffer contains something, which means a newline or something else
                        // split it up. Clear buffer and don't consider this a link.
                        buffer.clear();
                        current = Currently::OutsideLink;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Escape characters for usage in URLs
fn escape_special_chars(text: &str) -> String {
    let mut buf = String::new();
    escape_href(&mut buf, text).ok();
    buf
}

fn get_chapter(it: &BookItem) -> Option<&Chapter> {
    if let BookItem::Chapter(ch) = it {
        Some(ch)
    } else {
        None
    }
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
        for_each_link(content, |link_text| { links.push(link_text.to_owned()); });

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
        for_each_link(content, |link_text| { links.push(link_text.to_owned()); });

        assert!(links.is_empty(), "Got links: {:?}", links);
    }

    #[test]
    fn escapel_special_chars() {
        assert_eq!(
            escape_special_chars("w3ir∂ førmättÎñg"),
            "w3ir%E2%88%82%20f%C3%B8rm%C3%A4tt%C3%8E%C3%B1g"
        )
    }
}
