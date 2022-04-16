use crate::error::Result;
use anyhow::Context;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

/// Ease-of-use function for loading a file from a path.
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut buf = String::new();

    File::open(&path)
        .with_context(|| format!("Unable to open {:?}.", path.as_ref()))?
        .read_to_string(&mut buf)
        .with_context(|| format!("Could not read file {:?}.", path.as_ref()))?;

    Ok(buf)
}

/// Ease-of-use function for creating a file and writing bytes to it
pub fn write_file(path: impl AsRef<Path>, bytes: &[u8]) -> Result<()> {
    let path = path.as_ref();

    // Create file
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut f = File::create(path)?;

    // Write bytes
    f.write_all(bytes)?;

    Ok(())
}

/// Returns a relative path from `from` to `to`.
///
/// If `to` is absolute, but `from` is not, `None` is returned.
///
/// This is a slightly modified version of
/// [`pathdiff::diff_paths`](https://docs.rs/pathdiff/latest/pathdiff/fn.diff_paths.html):
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

/// Takes a path and returns a string containing just enough `../` to point to
/// the root of the given path.
pub fn path_to_root<P: AsRef<Path>>(path: P) -> String {
    let s = path
        .as_ref()
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .components()
        .fold(String::new(), |mut s, c| {
            if let Component::Normal(_) = c {
                s.push_str("../");
            }
            s
        });

    if s.is_empty() {
        "./".into()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
