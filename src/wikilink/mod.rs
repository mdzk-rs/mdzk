mod preprocessor;
mod renderer;
pub use preprocessor::{MdzkPreprocessor, WikilinkPreprocessor};
pub use renderer::{CommonMarkRenderer, WikilinkRenderer};

use crate::{error::Result, Note};
use pulldown_cmark::{CowStr, Event, Tag};
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct WikilinkContext<'a> {
    pub text: &'a str,
    pub source: &'a Note,
    pub destination: Option<&'a Note>,
    pub alias: Option<String>,
    pub anchor: Anchor,
}

impl<'a> WikilinkContext<'a> {
    pub fn new(text: &'a str, source: &'a Note) -> Self {
        WikilinkContext {
            text,
            source,
            destination: None,
            alias: None,
            anchor: Anchor::None,
        }
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub enum Anchor {
    Header(String),
    Blockref(String),
    #[default]
    None,
}

/// Finds all wikilinks in `content` and applies the closure `handle_link` to them.
pub fn for_each_wikilink(
    content: &str,
    mut handle_link: impl FnMut(&str, Range<usize>) -> Result<()>,
) -> Result<()> {
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
    for (event, range) in parser.into_offset_iter() {
        match event {
            // Don't parse links in codeblocks, links and images
            Event::Start(Tag::CodeBlock(_) | Tag::Link(_, _, _) | Tag::Image(_, _, _)) => {
                current = Currently::Ignore;
            }
            Event::End(Tag::CodeBlock(_) | Tag::Link(_, _, _) | Tag::Image(_, _, _)) => {
                current = Currently::OutsideLink;
            }

            Event::Text(CowStr::Borrowed("[")) => match current {
                Currently::OutsideLink => current = Currently::MaybeOpen,
                Currently::MaybeOpen => current = Currently::MaybeInsideLink,
                Currently::MaybeInsideLink => current = Currently::OutsideLink,
                Currently::MaybeClose => {
                    buffer.clear();
                    current = Currently::OutsideLink;
                }
                Currently::Ignore => {}
            },

            Event::Text(CowStr::Borrowed("]")) => match current {
                Currently::MaybeOpen => current = Currently::OutsideLink,
                Currently::MaybeInsideLink => current = Currently::MaybeClose,
                Currently::MaybeClose => {
                    handle_link(buffer.trim(), range)?;
                    buffer.clear();
                    current = Currently::OutsideLink;
                }
                Currently::OutsideLink | Currently::Ignore => {}
            },

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
    Ok(())
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

| Header 1 | Header 2 |
| -------- | -------- |
| Tables can also have [[table links]] | more stuff |

Check if [[link with (parentheses) work as well]]. What [[about {curly braces}?]]"#;

        let mut gots = vec![];
        for_each_wikilink(content, |link_text, _| {
            gots.push(link_text.to_owned());
            Ok(())
        })
        .unwrap();

        let wants = vec![
            "link",
            "link#header",
            "link | a bit more complex",
            "link#header | more 😭 complex",
            "link in a blockquote",
            "list link",
            "table links",
            "link with (parentheses) work as well",
            "about {curly braces}?",
        ];

        for (got, want) in gots.iter().zip(wants.iter()) {
            assert_eq!(got, want);
        }
    }

    #[test]
    fn dont_detect_these() {
        let content = r#"Here are some non-correct links:
First a link [[with
newline]]
Then a link `inside [[inline code]]`, or inside. What about \[\[escaped brackets\]\]?

```rust
let link = "[[link_in_code]]".to_owned();
```
<p>
  This is some raw HTML. We don't want [[html links]] detected here.
</p>"#;

        let mut links = Vec::<String>::new();
        for_each_wikilink(content, |link_text, _| {
            links.push(link_text.to_owned());
            Ok(())
        })
        .unwrap();

        assert!(links.is_empty(), "Got links: {:?}", links);
    }
}
