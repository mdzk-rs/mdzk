use crate::utils::{fs::diff_paths, string::{kebab, escape_href}};
use anyhow::Result;
use pest::Parser;
use pulldown_cmark::{CowStr, Event};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub enum Edge {
    Connected,
    NotConnected,
}

/// Finds all wikilinks in `content` and applies the closure `handle_link` to them.
pub fn for_each_internal_link(content: &str, mut handle_link: impl FnMut(&str) -> Result<()>) -> Result<()> {
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
            Event::Html(CowStr::Borrowed("<span class=\"katex-inline\">")) => {
                current = Currently::Ignore
            }
            Event::Html(CowStr::Borrowed("</span>")) => current = Currently::OutsideLink,

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
                    handle_link(buffer.trim())?;
                    buffer.clear();
                    current = Currently::OutsideLink;
                }
                Currently::OutsideLink => {}
                Currently::Ignore => {}
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

#[derive(Parser)]
#[grammar = "./wikilink.pest"]
pub struct InternalLinkParser;

#[derive(Debug, PartialEq)]
pub enum Anchor {
    Header(String),
    Blockref(String),
    None,
}

#[derive(Debug, PartialEq)]
pub struct InternalLink {
    pub dest_title: String,
    pub dest_path: Option<PathBuf>,
    pub link_text: String,
    pub anchor: Anchor,
}

impl InternalLink {
    pub fn from(internals: &str) -> Result<Self> {
        let mut link = InternalLinkParser::parse(Rule::link, internals)?
            .next()
            .unwrap()
            .into_inner();

        // Handle destination
        let mut dest = link.next().unwrap().into_inner();
        let dest_title = dest.next().unwrap().as_str();

        // Handle alias
        let link_text = if let Some(alias) = link.next() {
            alias.as_str()
        } else {
            dest_title
        };

        // Handle header and blockref
        let anchor = match dest.next() {
            Some(anchor) => match anchor.into_inner().next() {
                Some(x) if x.as_rule() == Rule::header => {
                    Anchor::Header(kebab(x.as_str()))
                }
                Some(x) if x.as_rule() == Rule::blockref => {
                    Anchor::Blockref(kebab(x.as_str()))
                }
                _ => panic!("This should not happen..."),
            },
            None => Anchor::None,
        };

        Ok(Self {
            dest_title: dest_title.to_owned(),
            dest_path: None, // TODO: Find destination path
            link_text: link_text.to_owned(),
            anchor,
        })
    }

    pub fn cmark<P>(&self, cur_dir: P) -> String
    where
        P: AsRef<Path>,
    {
        let mut href = if let Some(dest_path) = &self.dest_path {
            diff_paths(dest_path, cur_dir.as_ref())
                .unwrap()
                .to_string_lossy()
                .to_string() // Gotta love Rust <3
        } else { self.dest_title.to_owned() };

        // Handle anchor
        // TODO: Blockrefs are currently not handled here
        match &self.anchor {
            Anchor::Header(id) | Anchor::Blockref(id) => href.push_str(&format!("#{}", id)),
            Anchor::None => {}
        }

        format!("[{}]({})", self.link_text, escape_href(&href))
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
| Header 1 | Header 2 |
| -------- | -------- |
| Tables can also have [[table links]] | more stuff |"#;

        let mut links = vec![];
        for_each_internal_link(content, |link_text| {
            links.push(link_text.to_owned());
            Ok(())
        }).unwrap();

        assert_eq!(
            links,
            vec![
                "link",
                "link#header",
                "link | a bit more complex",
                "link#header | more 😭 complex",
                "link in a blockquote",
                "list link",
                "table links"
            ]
        );
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
        for_each_internal_link(content, |link_text| {
            links.push(link_text.to_owned());
            Ok(())
        }).unwrap();

        assert!(links.is_empty(), "Got links: {:?}", links);
    }

    #[test]
    fn test_parse_link() {
        let cases = vec![
            (
                InternalLink {
                    dest_title: "This is note".to_owned(),
                    dest_path: None,
                    link_text: "Some alias".to_owned(),
                    anchor: Anchor::None,
                },
                "This is note|Some alias",
            ),
            (
                InternalLink {
                    dest_title: "Tïtlæ fôr nøte".to_owned(),
                    dest_path: None,
                    link_text: "Tïtlæ fôr nøte".to_owned(),
                    anchor: Anchor::Blockref("id1234".to_owned()),
                },
                "Tïtlæ fôr nøte#^id1234",
            ),
            (
                InternalLink {
                    dest_title: "🔈 Music".to_owned(),
                    dest_path: None,
                    link_text: "🔈 Music".to_owned(),
                    anchor: Anchor::Header("header-with-spaces".to_owned()),
                },
                "🔈 Music#Header with spaces",
            ),
        ];
        for (want, from) in cases {
            assert_eq!(want, InternalLink::from(from).unwrap());
        }
    }

    #[test]
    fn test_cmark_link() {
        let cases = vec![
            (
                "[Some alias](../../This%20is%20note.md)",
                InternalLink::from("This is note|Some alias").unwrap(),
            ),
            (
                "[Tïtlæ fôr nøte](../../T%C3%AFtl%C3%A6%20f%C3%B4r%20n%C3%B8te.md#id1234)",
                InternalLink::from("Tïtlæ fôr nøte#id1234").unwrap(),
            ),
        ];

        for (want, mut from) in cases {
            // FIXME: This is temporary while we work out the proper path derivation
            from.dest_path = Some(PathBuf::from(&format!("{}.md", from.dest_title)));
            assert_eq!(want, from.cmark(Path::new("sub/subsub")))
        }
    }
}
