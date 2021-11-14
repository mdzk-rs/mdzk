use crate::utils::escape_special_chars;
use anyhow::Result;
use mdbook::utils::id_from_content;
use pest::Parser;
use pulldown_cmark::{CowStr, Event};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Finds all wikilinks in `content` and applies the closure `handle_link` to them.
pub fn for_each_wikilink(content: &str, mut handle_link: impl FnMut(&str)) {
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
                    handle_link(buffer.trim());
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
}

#[derive(Parser)]
#[grammar = "preprocess/wikilink.pest"]
pub struct WikiLinkParser;

#[derive(Debug, PartialEq)]
pub struct WikiLink {
    pub path: Option<PathBuf>,
    pub title: String,
    pub anchor: Option<String>,
}

impl WikiLink {
    #[allow(dead_code)]
    pub fn from(internals: &str) -> Result<Self> {
        Self::from_with_index(internals, &HashMap::new())
    }

    pub fn from_with_index(internals: &str, index: &HashMap<String, PathBuf>) -> Result<Self> {
        let mut link = WikiLinkParser::parse(Rule::link, internals)?
            .next()
            .unwrap()
            .into_inner();

        // Handle destination
        let mut dest = link.next().unwrap().into_inner();
        let note = dest.next().unwrap().as_str();

        // Handle alias
        let title = if let Some(alias) = link.next() {
            alias.as_str() 
        } else {
            note
        };

        // Find path
        let path = if index.is_empty() {
            Some(PathBuf::from(note))
        } else {
            index.get(note).cloned()
        };

        Ok(Self {
            path,
            title: title.to_owned(),
            anchor: dest.next().map(|x| x.as_str()[1..].to_owned()),
        })
    }

    pub fn cmark(&self, cur_path: &Path) -> String {
        if let Some(path) = &self.path {
            let mut href = pathdiff::diff_paths(path, cur_path)
                .unwrap()
                .to_string_lossy()
                .to_string(); // Gotta love Rust <3

            // Handle anchor
            // TODO: Blockrefs are currently not handled here
            if let Some(anchor) = &self.anchor {
                let header_kebab = id_from_content(anchor.as_str());
                href.push_str(&format!("#{}", header_kebab));
            }

            format!("[{}](<{}>)", self.title, escape_special_chars(&href))
        } else {
            format!(
                "<span class=\"missing-link\" style=\"color:darkred;\">{}</span>",
                self.title
            )
        }
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
        for_each_wikilink(content, |link_text| {
            links.push(link_text.to_owned());
        });

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
        for_each_wikilink(content, |link_text| {
            links.push(link_text.to_owned());
        });

        assert!(links.is_empty(), "Got links: {:?}", links);
    }

    #[test]
    fn test_parse_link() {
        let cases = vec![
            (
                WikiLink {
                    path: Some(PathBuf::from("This is note")),
                    title: "Some alias".to_owned(),
                    anchor: None,
                },
                "This is note|Some alias",
            ),
            (
                WikiLink {
                    path: Some(PathBuf::from("Tïtlæ fôr nøte")),
                    title: "Tïtlæ fôr nøte".to_owned(),
                    anchor: Some("^id1234".to_owned()),
                },
                "Tïtlæ fôr nøte#^id1234",
            ),
        ];
        for (want, from) in cases {
            assert_eq!(want, WikiLink::from(from).unwrap());
        }
    }

    #[test]
    fn test_cmark_link() {
        let mut index = HashMap::new();
        index.insert("This is note".to_owned(), PathBuf::from("This is note.md"));
        index.insert("Tïtlæ fôr nøte".to_owned(), PathBuf::from("Tïtlæ fôr nøte.md"));

        let cases = vec![
            (
                "[Some alias](<../../This%20is%20note.md>)",
                WikiLink::from_with_index("This is note|Some alias", &index).unwrap(),
            ),
            (
                "[Tïtlæ fôr nøte](<../../T%C3%AFtl%C3%A6%20f%C3%B4r%20n%C3%B8te.md#id1234>)",
                WikiLink::from_with_index("Tïtlæ fôr nøte#^id1234", &index).unwrap(),
            ),
            (
                "<span class=\"missing-link\" style=\"color:darkred;\">This is missing note</span>",
                WikiLink::from_with_index("Missing note#header | This is missing note", &index).unwrap(),
            ),
        ];

        for (want, from) in cases {
            assert_eq!(want, from.cmark(Path::new("sub/subsub")))
        }
    }
}
