use anyhow::{Context, Result};
use std::{
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
};

/// Ease-of-use function for loading a file from a path.
pub(crate) fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut buf = String::new();

    File::open(&path)
        .with_context(|| format!("Unable to open {:?}.", path.as_ref()))?
        .read_to_string(&mut buf)
        .with_context(|| format!("Could not read file {:?}.", path.as_ref()))?;

    Ok(buf)
}

// Slightly modified version of pathdiff::diff_paths:
// https://github.com/Manishearth/pathdiff/blob/08fabb3b2864a19d1b870d0d1fe887abad351930/src/lib.rs#L34-L77
/// Returns a relative path from `from` to `to`.
pub(crate) fn diff_paths<T, F>(to: T, from: F) -> Option<PathBuf>
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
