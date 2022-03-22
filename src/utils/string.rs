use crate::NoteId;
use pulldown_cmark::escape::escape_href as ehref;

/// Make kebab-cased string
pub fn kebab(s: impl AsRef<str>) -> String {
    s.as_ref()
        .chars()
        .filter_map(|ch| {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

/// Escape characters for usage in URLs
pub fn escape_href(text: &str) -> String {
    let mut buf = String::new();
    ehref(&mut buf, text).ok();
    // Apparently, pulldown-cmark does not escape `?` into `%3F`, which means we
    // have to do it manually.
    buf.replace('?', "%3F")
}

/// Formats a [`NoteId`] into a hex string.
///
/// Simply a wrapper around the [`format`] macro.
pub fn hex(id: &NoteId) -> String {
    format!("{:x}", id)
}
