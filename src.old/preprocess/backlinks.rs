use crate::utils::{diff_paths, escape_special_chars};
use mdbook::book::Chapter;
use std::collections::HashMap;
use std::path::PathBuf;
use toml::Value;

pub fn insert_backlinks(
    ch: &mut Chapter,
    backlinks_map: &mut HashMap<PathBuf, Vec<(PathBuf, String)>>,
    header: &str,
) {
    if let Some(path) = &ch.path {
        if let Some(backlinks) = backlinks_map.get_mut(path) {
            if !backlinks.is_empty() {
                backlinks.sort_unstable();
                backlinks.dedup();

                ch.content.push_str("\n<div class=\"backlinks\">\n\n");
                ch.content.push_str(header);

                for (dest, name) in backlinks.iter() {
                    let link = diff_paths(dest, path.parent().unwrap()).unwrap();
                    ch.content.push_str(&format!(
                        "\n[{}]({})",
                        name,
                        escape_special_chars(&link.to_string_lossy()),
                    ));
                }

                ch.content.push_str("\n</div>");
            };
        }
    }
}

pub fn backlinks_header(config: &mdbook::Config) -> String {
    let mut header = String::new();
    if let Some(Value::String(val)) = config.get("mdzk.backlinks-header") {
        header.push_str("### ");
        header.push_str(val);
        header.push_str(" {.backlinks-header}");
    }
    header
}
