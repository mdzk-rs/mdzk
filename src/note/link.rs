use crate::{
    error::{Error, Result},
    utils::{
        fs::diff_paths,
        string::{escape_href, kebab},
    },
    HashMap, IdMap, NoteId,
};
use anyhow::Context;
use pest::Parser;
use pulldown_cmark::{CowStr, Event, Tag};
use std::{
    ops::Range,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Edge {
    Connected(Vec<Range<usize>>),
    NotConnected,
}

impl Edge {
    pub fn push_link_range(&mut self, range: Range<usize>) {
        match self {
            Edge::Connected(ranges) => ranges.push(range),
            Edge::NotConnected => *self = Edge::Connected(vec![range]),
        }
    }
}

/// Finds all wikilinks in `content` and applies the closure `handle_link` to them.
pub fn for_each_internal_link(
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

#[derive(Parser)]
#[grammar = "note/internal_link.pest"]
pub struct InternalLinkParser;

#[derive(Debug, PartialEq)]
enum Anchor {
    Header(String),
    Blockref(String),
    None,
}

#[derive(Debug, PartialEq)]
pub(crate) struct InternalLink {
    dest_title: String,
    dest_path: Option<PathBuf>,
    pub(crate) dest_id: NoteId,
    link_text: String,
    anchor: Anchor,
}

impl InternalLink {
    pub fn cmark<P>(&self, cur_dir: P) -> String
    where
        P: AsRef<Path>,
    {
        let mut href = if let Some(dest_path) = &self.dest_path {
            diff_paths(dest_path, cur_dir.as_ref())
                .unwrap()
                .to_string_lossy()
                .to_string() // Gotta love Rust <3
        } else {
            self.dest_title.clone()
        };

        // Handle anchor
        match &self.anchor {
            Anchor::Header(id) | Anchor::Blockref(id) => href.push_str(&format!("#{}", id)),
            Anchor::None => {}
        }

        format!("[{}]({})", self.link_text, escape_href(&href))
    }
}

pub(crate) fn create_link(
    link_string: &str,
    path_lookup: &IdMap<PathBuf>,
    id_lookup: &HashMap<String, NoteId>,
) -> Result<InternalLink> {
    let mut link = InternalLinkParser::parse(Rule::link, link_string)
        .with_context(|| format!("Failed parsing internal link from {}", link_string))?
        .next()
        .unwrap()
        .into_inner();

    // Handle destination
    let mut dest = link.next().unwrap().into_inner();
    let dest_title = dest.next().unwrap().as_str();

    if let Some(&dest_id) = id_lookup.get(dest_title) {
        // Handle alias
        let link_text = if let Some(alias) = link.next() {
            alias.as_str()
        } else {
            dest_title
        };

        // Handle header and blockref
        let anchor = match dest.next() {
            Some(anchor) => match anchor.into_inner().next() {
                Some(x) if x.as_rule() == Rule::header => Anchor::Header(kebab(x.as_str())),
                Some(x) if x.as_rule() == Rule::blockref => Anchor::Blockref(kebab(x.as_str())),
                _ => panic!("This should not happen..."),
            },
            None => Anchor::None,
        };

        Ok(InternalLink {
            dest_title: dest_title.to_owned(),
            dest_path: path_lookup.get(&dest_id).cloned(),
            dest_id,
            link_text: link_text.to_owned(),
            anchor,
        })
    } else {
        Err(Error::InvalidInternalLinkDestination(
            link_string.to_owned(),
        ))
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
| Tables can also have [[table links]] | more stuff |

Check if [[link with (parentheses) work as well]]. What [[about {curly braces}?]]"#;

        let mut gots = vec![];
        for_each_internal_link(content, |link_text, _| {
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
        for_each_internal_link(content, |link_text, _| {
            links.push(link_text.to_owned());
            Ok(())
        })
        .unwrap();

        assert!(links.is_empty(), "Got links: {:?}", links);
    }

    #[test]
    fn test_parse_link() {
        let cases = vec![
            (
                InternalLink {
                    dest_title: "This is note".to_owned(),
                    dest_path: Some(PathBuf::from("This is note")),
                    dest_id: 1,
                    link_text: "Some alias".to_owned(),
                    anchor: Anchor::None,
                },
                "This is note|Some alias",
            ),
            (
                InternalLink {
                    dest_title: "Tïtlæ fôr nøte".to_owned(),
                    dest_path: Some(PathBuf::from("Tïtlæ fôr nøte")),
                    dest_id: 2,
                    link_text: "Tïtlæ fôr nøte".to_owned(),
                    anchor: Anchor::Blockref("id1234".to_owned()),
                },
                "Tïtlæ fôr nøte#^id1234",
            ),
            (
                InternalLink {
                    dest_title: "🔈 Music".to_owned(),
                    dest_path: Some(PathBuf::from("🔈 Music")),
                    dest_id: 3,
                    link_text: "🔈 Music".to_owned(),
                    anchor: Anchor::Header("header-with-spaces".to_owned()),
                },
                "🔈 Music#Header with spaces",
            ),
        ];
        let mut id_lookup = HashMap::default();
        id_lookup.insert("This is note".to_owned(), 1);
        id_lookup.insert("Tïtlæ fôr nøte".to_owned(), 2);
        id_lookup.insert("🔈 Music".to_owned(), 3);

        let mut path_lookup = IdMap::<PathBuf>::default();
        path_lookup.insert(1, PathBuf::from("This is note"));
        path_lookup.insert(2, PathBuf::from("Tïtlæ fôr nøte"));
        path_lookup.insert(3, PathBuf::from("🔈 Music"));

        for (want, from) in cases {
            assert_eq!(want, create_link(from, &path_lookup, &id_lookup).unwrap());
        }
    }
}
