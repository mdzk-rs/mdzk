use mdbook::book::Chapter;
use regex::{Captures, Regex};
use std::path::{Path, PathBuf};

pub fn convert_readme(ch: &mut Chapter, source_dir: PathBuf, re: &Regex) {
    if let Some(ref mut path) = ch.path {
        if is_readme_file(&path) {
            let index_md = source_dir.join(path.with_file_name("index.md"));
            if index_md.exists() {
                let file_name = path.file_name().unwrap_or_default();
                let parent_dir = index_md
                    .parent()
                    .unwrap_or_else(|| index_md.as_ref());

                warn!(
                    r#"It seems that there are both {:?} and "index.md" under {:?}.

mdzk converts {0:?} into "index.md" by default, which may cause
conflicts. Consider moving one of the files to another directory or
give it another name.

"#,
                    file_name,
                    parent_dir
                );
            }

            path.set_file_name("index.md");
        } else {
            ch.content = re
                .replace_all(&ch.content, |caps: &Captures| {
                    format!("[{}](index.md)", &caps[1])
                })
                .to_string();
        }
    }
}

pub fn is_readme_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_lowercase()
        .eq("readme")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_stem_exactly_matches_readme_case_insensitively() {
        let path = "path/to/Readme.md";
        assert!(is_readme_file(path));

        let path = "path/to/README.md";
        assert!(is_readme_file(path));

        let path = "path/to/rEaDmE.md";
        assert!(is_readme_file(path));

        let path = "path/to/README.markdown";
        assert!(is_readme_file(path));

        let path = "path/to/README";
        assert!(is_readme_file(path));

        let path = "path/to/README-README.md";
        assert!(!is_readme_file(path));
    }
}
