use crate::{Config, CONFIG_FILE, SUMMARY_FILE};

use mdbook::errors::*;
use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::Write;
use std::{
    env,
    path::{Component, Path, PathBuf},
    process::Command,
};
use ignore::{WalkBuilder, overrides::OverrideBuilder, types::TypesBuilder};

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
    for ignore in config.mdzk.ignore.iter() {
        if let Some(s) = ignore.strip_prefix('!') {
            overrides.add(s)?;
        } else {
            overrides.add(&format!("!{}", ignore))?;
        }
    }

    let walker = WalkBuilder::new(&book_source)
        .hidden(true)
        .overrides(overrides.build()?)
        .types(TypesBuilder::new().add_defaults().select("markdown").build()?)
        .sort_by_file_path(|path1, path2| {
            match (path1.is_dir(), path2.is_dir()) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => path1.cmp(path2),
            }
        })
        .build();

    let summary = walker
        .filter_map(|e| e.ok())
        // Don't include the book source directory
        .filter(|e| e.path() != book_source && e.path() != book_source.join(SUMMARY_FILE))
        .map(|e| {
            let stripped_path = e.path().strip_prefix(&book_source).unwrap();
            let file_stem = stripped_path.file_stem().unwrap().to_str().unwrap();
            let depth = (stripped_path.components().count() - 1) * PAD_SIZE;

            if e.path().is_dir() {
                format!("{1:>0$}- [{2}]()\n", depth, "", file_stem)
            } else {
                format!(
                    "{1:>0$}- [{2}](<{3}>)\n",
                    depth,
                    "",
                    file_stem,
                    escape_special_chars(stripped_path.to_str().unwrap())
                )
            }
        })
        .fold(String::new(), |acc, curr| acc + &curr);

    let mut summary_file = File::create(book_source.join(SUMMARY_FILE))?;
    write!(summary_file, "# Summary\n\n{}", summary)?;

    debug!("Updated summary file.");

    Ok(())
}

/// String escapes for HTML entity.
fn escape_special_chars(text: &str) -> String {
    text.replace(' ', "%20")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
        .unwrap() // FIXME: Should handle this more gracefully
        .components()
        .fold(String::new(), |mut s, c| {
            match c {
                Component::Normal(_) => s.push_str("../"),
                _ => {
                    debug!("Other path component... {:?}", c);
                }
            }
            s
        })
}
