use anyhow::Result;
use crate::utils::escape_special_chars;
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

pub struct WikiLink {
    pub note: String,
    pub anchor: Option<String>,
    pub alias: Option<String>,
}

impl WikiLink {
    pub fn from(internals: &str) -> Result<Self> {
        let mut link = WikiLinkParser::parse(Rule::link, internals)?
            .next()
            .unwrap()
            .into_inner();

        // Handle destination
        let mut dest = link.next().unwrap().into_inner();

        Ok(Self {
            note: dest.next().unwrap().to_string(),
            anchor: dest.next().map(|x| x.to_string()),
            alias: link.next().map(|x| x.to_string()),
        })
    }

    pub fn title(&self) -> &str {
        if let Some(alias) = &self.alias {
            &alias
        } else {
            &self.note
        }
    }

    pub fn cmark(&self, cur_path: &Path, path_map: &HashMap<String, PathBuf>) -> String {
        if !path_map.contains_key(&self.note) {
            format!(
                "<span class=\"missing-link\" style=\"color:darkred;\">{}</span>",
                self.title()
            )
        } else {
            let mut href = pathdiff::diff_paths(
                path_map.get(&self.note).unwrap(),
                cur_path,
            )
            .unwrap()
            .to_string_lossy()
            .to_string(); // Gotta love Rust <3

            // Handle anchor
            // TODO: Blockrefs are currently not handled here
            if let Some(anchor) = &self.anchor {
                let header_kebab = id_from_content(&anchor.as_str()[1..]);
                href.push_str(&format!("#{}", header_kebab));
            }

            format!("[{}](<{}>)", self.title(), escape_special_chars(&href))
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
}
