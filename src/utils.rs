use mdbook::errors::Error;
use std::fs::File;
use std::io::Write;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

const PAD_SIZE: usize = 4;

/// Search up the filesystem to find a zk.toml file and return it's parent directory.
pub fn find_zk_root() -> Option<PathBuf> {
    let mut path: PathBuf = env::current_dir().unwrap().into();
    let file = Path::new("zk.toml");

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

pub fn update_summary(book_source: &PathBuf) -> Result<(), Error> {
    let summary = WalkDir::new(book_source)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path() != book_source)
        .map(|e| {
            let stripped_path = e.path().strip_prefix(&book_source).unwrap();
            let file_stem = stripped_path.file_stem().unwrap().to_str().unwrap();
            let depth = (stripped_path.components().count() - 1) * PAD_SIZE;

            if e.path().is_dir() {
                return Some(format!("{1:>0$}- [{2}]()\n", depth, "", file_stem));
            }

            let file_ext = match e.path().extension() {
                Some(ext) => ext.to_str()?,
                None => {
                    // one example being .gitkeep files
                    println!("Warning: File '{}' has no extension.", e.path().to_str()?);
                    return None;
                }
            };

            if file_ext == "md" {
                return Some(format!(
                    "{1:>0$}- [{2}](<{3}>)\n",
                    depth,
                    "",
                    file_stem,
                    stripped_path
                        .to_str()
                        .unwrap()
                        .replace(" ", "%20")
                        .replace("<", "&lt;")
                        .replace(">", "&gt;")
                ));
            }

            return None;
        })
        .filter_map(|e| e)
        .fold(String::new(), |acc, curr| acc + &curr);

    let mut summary_file = File::create(
        [book_source, &PathBuf::from("SUMMARY.md")]
            .iter()
            .collect::<PathBuf>(),
    )?;

    write!(summary_file, "# Summary\n{}", summary)?;
    Ok(())
}
