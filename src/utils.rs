use crate::{Config, CONFIG_FILE, SUMMARY_FILE};

use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};
use mdbook::errors::*;
use pulldown_cmark::escape::escape_href;
use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::Write;
use std::{
    env,
    path::{Component, Path, PathBuf},
    process::Command,
};

const PAD_SIZE: usize = 4;

/// Search up the filesystem to find a `mdzk.toml` file, and return it's parent directory.
/// This is simply used to find the root of any mdzk vault.
pub fn find_mdzk_root() -> Option<PathBuf> {
    let mut path: PathBuf = env::current_dir().unwrap();
    let file = Path::new(CONFIG_FILE);

    loop {
        path.push(file);

        if path.is_file() {
            path.pop();
            break Some(path);
        }

        if !(path.pop() && path.pop()) {
            break None;
        }
    }
}

// Function fetched from https://github.com/rust-lang/mdBook/blob/master/src/cmd/init.rs
/// Obtains author name from git config file by running the `git config` command.
pub fn get_author_name() -> Option<String> {
    let output = Command::new("git")
        .args(&["config", "--get", "user.name"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        None
    }
}

/// Searches for Markdown-files in the source directory, and updates the summary file
/// correspondingly. Ignores the summary file itself.
pub fn update_summary(config: &Config, root: &Path) -> Result<(), Error> {
    let book_source = root.join(&config.mdzk.src);

    let mut overrides = OverrideBuilder::new(&book_source);
    if let Some(ref ignores) = config.mdzk.ignore {
        for ignore in ignores.iter() {
            if let Some(s) = ignore.strip_prefix('!') {
                overrides.add(s)?;
            } else {
                overrides.add(&format!("!{}", ignore))?;
            }
        }
    }

    let walker = WalkBuilder::new(&book_source)
        .hidden(true)
        .overrides(overrides.build()?)
        .types(
            TypesBuilder::new()
                .add_defaults()
                .select("markdown")
                .build()?,
        )
        .sort_by_file_path(|path1, path2| match (path1.is_dir(), path2.is_dir()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => path1.cmp(path2),
        })
        .build();

    let summary = walker
        .filter_map(|e| e.ok())
        // Don't include the book source directory
        .filter(|e| e.path() != book_source && e.path() != book_source.join(SUMMARY_FILE))
        // Filter files which share the name of it's parent directory.
        .filter(|e| {
            if let Some(dir_stem) = e.path().parent() {
                let dir_stem = dir_stem.file_stem().unwrap_or_default();
                let file_stem = e.path().file_stem().unwrap_or_default();
                file_stem != dir_stem
            } else {
                true
            }
        })
        .map(|e| {
            let path = e.path();
            let stripped_path = path.strip_prefix(&book_source).unwrap();
            let file_stem = stripped_path.file_stem().unwrap().to_str().unwrap();
            let depth = (stripped_path.components().count() - 1) * PAD_SIZE;

            if path.is_dir() {
                let dir_file = stripped_path
                    .join(&stripped_path.file_stem().unwrap_or_default())
                    .with_extension("md");

                if book_source.join(&dir_file).is_file() {
                    // Directory contains equally named file
                    format!(
                        "{1:>0$}- [{2}](<{3}>)\n",
                        depth,
                        "",
                        file_stem,
                        dir_file.to_str().unwrap()
                    )
                } else {
                    // Directory has no equally named file
                    format!("{1:>0$}- [{2}]()\n", depth, "", file_stem)
                }
            } else {
                format!(
                    "{1:>0$}- [{2}](<{3}>)\n",
                    depth,
                    "",
                    file_stem,
                    stripped_path.to_str().unwrap()
                )
            }
        })
        .fold(String::new(), |acc, curr| acc + &curr);

    let mut summary_file = File::create(book_source.join(SUMMARY_FILE))?;
    write!(summary_file, "# Summary\n\n{}", summary)?;

    info!("Updated summary file.");

    Ok(())
}

/// Escape characters for usage in URLs
pub fn escape_special_chars(text: &str) -> String {
    let mut buf = String::new();
    escape_href(&mut buf, text).ok();
    buf
}

/// Ease-of-use function for creating a file and writing bytes to it
pub fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    // Create file
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    let mut f = File::create(path)?;

    // Write bytes
    f.write_all(bytes)?;

    Ok(())
}

/// Takes a path and returns a path containing just enough `../` to point to
/// the root of the given path.
pub fn path_to_root<P: Into<PathBuf>>(path: P) -> String {
    path.into()
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .components()
        .fold(String::new(), |mut s, c| {
            match c {
                Component::Normal(_) => s.push_str("../"),
                _ => debug!("Other path component... {:?}", c),
            }
            s
        })
}

// Slightly modified version of pathdiff::diff_paths:
// https://github.com/Manishearth/pathdiff/blob/08fabb3b2864a19d1b870d0d1fe887abad351930/src/lib.rs#L34-L77
/// Returns a relative path from `from` to `to`.
pub fn diff_paths<T, F>(to: T, from: F) -> Option<PathBuf>
where
    T: AsRef<Path>,
    F: AsRef<Path>,
{
    let to = to.as_ref();
    let from = from.as_ref();

    match (to.is_absolute(), from.is_absolute()) {
        (true, false) => Some(PathBuf::from(to)),
        (false, true) => None,
        _ => {
            let mut to_iter = to.components();
            let mut from_iter = from.components();
            let mut components = vec![];
            loop {
                match (to_iter.next(), from_iter.next()) {
                    (None, None) => break,
                    (Some(a), None) => {
                        components.push(a);
                        components.extend(to_iter);
                        break;
                    }
                    (None, _) => components.push(Component::ParentDir),
                    (Some(a), Some(b)) if components.is_empty() && a == b => {}
                    (Some(a), Some(Component::CurDir)) => components.push(a),
                    (Some(_), Some(Component::ParentDir)) => return None,
                    (Some(a), Some(_)) => {
                        components.extend(
                            std::iter::repeat(Component::ParentDir).take(from_iter.count() + 1),
                        );
                        components.push(a);
                        components.extend(to_iter);
                        break;
                    }
                }
            }
            Some(components.iter().map(|comp| comp.as_os_str()).collect())
        }
    }
}

/// Checks if the URL starts with a scheme (like `https:`).
pub fn has_scheme(url: &str) -> bool {
    if let Some((potential_scheme, _)) = url.split_once("://") {
        let mut chars = potential_scheme.chars();
        chars.next().unwrap_or('�').is_ascii_alphabetic()
        && chars.all(|c| c.is_alphanumeric() || c == '+' || c == '.' || c == '-')
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_scheme() {
        assert!(has_scheme("https://mdzk.app"));
        assert!(has_scheme("file:///home/user/mdzk/file.md"));
        assert!(!has_scheme("folder/subfolder/file.md"));
    }

    #[test]
    fn test_escape_special_chars() {
        assert_eq!(
            escape_special_chars("w3ir∂ førmättÎñg"),
            "w3ir%E2%88%82%20f%C3%B8rm%C3%A4tt%C3%8E%C3%B1g"
        )
    }

    // Diff paths
    fn assert_diff_paths(path: &str, base: &str, expected: Option<&str>) {
        assert_eq!(diff_paths(path, base), expected.map(|s| s.into()));
    }

    #[test]
    fn test_absolute() {
        assert_diff_paths("/foo", "/bar", Some("../foo"));
        assert_diff_paths("/foo", "bar", Some("/foo"));
        assert_diff_paths("foo", "/bar", None);
        assert_diff_paths("foo", "bar", Some("../foo"));
    }

    #[test]
    fn test_identity() {
        assert_diff_paths(".", ".", Some(""));
        assert_diff_paths("../foo", "../foo", Some(""));
        assert_diff_paths("./foo", "./foo", Some(""));
        assert_diff_paths("/foo", "/foo", Some(""));
        assert_diff_paths("foo", "foo", Some(""));

        assert_diff_paths("../foo/bar/baz", "../foo/bar/baz", Some("".into()));
        assert_diff_paths("foo/bar/baz", "foo/bar/baz", Some(""));
    }

    #[test]
    fn test_subset() {
        assert_diff_paths("foo", "fo", Some("../foo"));
        assert_diff_paths("fo", "foo", Some("../fo"));
    }

    #[test]
    fn test_empty() {
        assert_diff_paths("", "", Some(""));
        assert_diff_paths("foo", "", Some("foo"));
        assert_diff_paths("", "foo", Some(".."));
    }

    #[test]
    fn test_relative() {
        assert_diff_paths("../foo", "../bar", Some("../foo"));
        assert_diff_paths("../foo", "../foo/bar/baz", Some("../.."));
        assert_diff_paths("../foo/bar/baz", "../foo", Some("bar/baz"));

        assert_diff_paths("foo/bar/baz", "foo", Some("bar/baz"));
        assert_diff_paths("foo/bar/baz", "foo/bar", Some("baz"));
        assert_diff_paths("foo/bar/baz", "foo/bar/baz", Some(""));
        assert_diff_paths("foo/bar/baz", "foo/bar/baz/", Some(""));

        assert_diff_paths("foo/bar/baz/", "foo", Some("bar/baz"));
        assert_diff_paths("foo/bar/baz/", "foo/bar", Some("baz"));
        assert_diff_paths("foo/bar/baz/", "foo/bar/baz", Some(""));
        assert_diff_paths("foo/bar/baz/", "foo/bar/baz/", Some(""));

        assert_diff_paths("foo/bar/baz", "foo/", Some("bar/baz"));
        assert_diff_paths("foo/bar/baz", "foo/bar/", Some("baz"));
        assert_diff_paths("foo/bar/baz", "foo/bar/baz", Some(""));
    }

    #[test]
    fn test_current_directory() {
        assert_diff_paths(".", "foo", Some("../."));
        assert_diff_paths("foo", ".", Some("foo"));
        assert_diff_paths("/foo", "/.", Some("foo"));
    }
}
