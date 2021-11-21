use crate::utils::diff_paths;
use mdbook::book::Chapter;
use std::collections::HashMap;
use std::path::PathBuf;
use toml::Value;

pub fn insert_backlinks(
    ch: &mut Chapter,
    backlinks_map: &HashMap<PathBuf, Vec<(PathBuf, String)>>,
    pre: &str,
) {
    if let Some(path) = &ch.path {
        if let Some(backlinks) = backlinks_map.get(path) {
            if !backlinks.is_empty() {
                ch.content.push_str(pre);
                for (dest, name) in backlinks.iter() {
                    let diff_path = diff_paths(dest, path.parent().unwrap()).unwrap();
                    ch.content.push_str(&format!(
                        "\n> - [{}](<{}>)",
                        name,
                        diff_path.to_string_lossy()
                    ));
                }
            };
        }
    }
}

pub fn prefix(config: &mdbook::Config) -> String {
    let mut prefix = "\n\n---\n\n".to_string();
    if let Some(Value::String(val)) = config.get("mdzk.backlinks-header") {
        prefix.push_str("## ");
        prefix.push_str(val);
        prefix.push_str("\n\n");
    }
    prefix
}
