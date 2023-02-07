use crate::error::Result;
use pulldown_cmark::{Event, Tag};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Arc {
    Connected,
    NotConnected,
}

/// Finds all links in `content` and applies the closure `handle_link` to them.
pub fn for_each_link(
    content: &str,
    mut handle_link: impl FnMut(&str, &str) -> Result<()>,
) -> Result<()> {
    for event in pulldown_cmark::Parser::new(content) {
        if let Event::Start(Tag::Link(_, dest, title)) = event {
            handle_link(&dest, &title)?;
        }
    }
    Ok(())
}
