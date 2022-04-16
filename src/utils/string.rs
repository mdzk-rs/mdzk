use crate::{error::Result, NoteId};
use pulldown_cmark::{escape::escape_href as ehref, CowStr};
use serde_json::{to_string, to_string_pretty, Value};

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
pub fn escape_href(text: impl AsRef<str>) -> String {
    let mut buf = String::new();
    ehref(&mut buf, text.as_ref()).ok();
    // Apparently, pulldown-cmark does not escape `?` into `%3F`, which means we
    // have to do it manually.
    buf.replace('?', "%3F")
}

/// Formats a [`NoteId`] into a hex string.
///
/// Padded by 0s to width 16. Simply a wrapper around the [`format`] macro.
pub fn hex(id: &NoteId) -> String {
    format!("{:016x}", id)
}

/// Format's a [`serde_json::Value`] into a string, according to the options.
///
/// If `val` is a `Value::String` or `Value::Array` and the `raw` option is specified, the strings
/// will get their surrounding quotes removed, and the array's values will get printed one by one
/// with a newline separating them. This makes the value compatible with shell scripts.
///
/// If `pretty` is specified, the JSON value will get pretty-printed.
pub fn format_json_value(val: &Value, raw: bool, pretty: bool) -> Result<String> {
    Ok(match (pretty, raw, val) {
        (_, true, Value::String(s)) => s.clone(),
        (_, true, Value::Array(a)) => {
            let mut acc: Vec<String> = Vec::new();
            for val in a {
                let val = format_json_value(val, true, false)?;
                acc.push(val);
            }
            acc.join("\n")
        }
        (true, _, _) => to_string_pretty(val)?,
        _ => to_string(val)?,
    })
}

/// Checks if the URL starts with a scheme (like `https` or `file`).
pub(crate) fn has_scheme(url: impl AsRef<str>) -> bool {
    if let Some((potential_scheme, _)) = url.as_ref().split_once("://") {
        let mut chars = potential_scheme.chars();
        chars.next().unwrap_or('�').is_ascii_alphabetic()
            && chars.all(|c| c.is_alphanumeric() || c == '+' || c == '.' || c == '-')
    } else {
        false
    }
}

/// Changes links with extension `.md` to `.html` if they don't have schemes. Anchors are kept.
pub(crate) fn fix_link(dest: CowStr) -> CowStr {
    if !(dest.starts_with('#') || has_scheme(&dest)) {
        if let Some((link, anchor)) = dest.split_once(".md") {
            let mut fixed_link = String::new();
            fixed_link.push_str(link);
            fixed_link.push_str(".html");
            if anchor.starts_with('#') {
                fixed_link.push_str(anchor);
            }
            return CowStr::from(fixed_link);
        }
    }
    dest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_link() {
        assert_eq!(
            fix_link("folder/subfolder/file.md".into()),
            "folder/subfolder/file.html".into()
        );
        assert_eq!(fix_link("https://mdzk.md".into()), "https://mdzk.md".into());
        assert_eq!(fix_link("file.md#anchor".into()), "file.html#anchor".into());
    }

    #[test]
    fn test_has_scheme() {
        assert!(has_scheme("https://mdzk.app"));
        assert!(has_scheme("file:///home/user/mdzk/file.md"));
        assert!(!has_scheme("folder/subfolder/file.md"));
    }

    #[test]
    fn test_format_json_value() {
        let want = r#"{
  "key": "value",
  "another-key": [
    1,
    2,
    3
  ]
}"#
        .to_string();
        let object = json!({ "key": "value", "another-key": [1, 2, 3]});
        assert_eq!(want, format_json_value(&object, false, true).unwrap());

        use serde_json::json;
        let string = json!("test");
        assert_eq!(
            "test".to_string(),
            format_json_value(&string, true, false).unwrap()
        );

        let arr = json!(["el1", "el2", "el3"]);
        assert_eq!(
            "el1\nel2\nel3".to_string(),
            format_json_value(&arr, true, false).unwrap()
        )
    }
}
