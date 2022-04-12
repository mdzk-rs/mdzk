use crate::{error::Result, NoteId};
use pulldown_cmark::escape::escape_href as ehref;
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

#[cfg(test)]
mod tests {
    use super::*;

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
