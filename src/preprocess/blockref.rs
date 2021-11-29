use mdbook::book::Chapter;

/// Wraps paragraphs in divs if they end with a blockreference ID. The div ID corresponds to the
/// blockreference ID, which means one can link to it.
///
/// The paragraphs are found by splitting on double newlines. This means some Markdown features
/// like lists do not support blockrefs.
pub fn wrap_blocks(ch: &mut Chapter) {
    for split in ch.content.clone().split("\n\n") {
        if let Some(id) = blockref_id(split) {
            let new_split = format!(
                "<div id=\"{}\">{}</div>",
                id,
                split.trim_end_matches(&id).trim_end_matches('^')
            );

            ch.content = ch.content.replacen(split, &new_split, 1);
        }
    }
}

/// Check if a string ends with `^` and then 6 ID-friendly chars. If so,
/// return the 6 digits as an id.
fn blockref_id(paragraph: &str) -> Option<String> {
    let len = paragraph.len();
    if len > 7 {
        let mut iter = paragraph.chars().skip(len - 7);
        if iter.next() == Some('^') && iter.clone().all(is_id_friendly) {
            return Some(iter.collect());
        }
    }
    None
}

/// Check if char can be used in a blockreference ID
///
/// 0-9, A-Z, a-z, _ and - are allowed
fn is_id_friendly(s: char) -> bool {
    s == '_' || s == '-' || char::is_digit(s, 36)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_friendly() {
        assert!(is_id_friendly('a'));
        assert!(is_id_friendly('Z'));
        assert!(is_id_friendly('7'));
        assert!(is_id_friendly('-'));
        assert!(!is_id_friendly('%'));
        assert!(!is_id_friendly('å'));
    }

    #[test]
    fn find_id() {
        let cases = vec![
            (
                "This is text with blockref at end ^1234fe",
                Some("1234fe".to_owned()),
            ),
            ("This is text with invalid blockref at end [^fn]...", None),
        ];

        for (from, want) in cases.iter() {
            assert_eq!(want, &blockref_id(from));
        }
    }
}
