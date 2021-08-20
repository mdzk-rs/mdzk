// This is simply a modified version of https://github.com/rust-lang/mdBook/blob/master/src/cmd/watch.rs
use crate::SUMMARY_FILE;

use mdbook::MDBook;
use notify::Watcher;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;

fn remove_ignored_files(book_root: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
    if paths.is_empty() {
        return vec![];
    }

    let paths: Vec<&PathBuf> = paths
        .iter()
        .filter(|x| x.file_name() != Some(OsStr::new(SUMMARY_FILE)))
        .collect();

    match find_gitignore(book_root) {
        Some(gitignore_path) => {
            match gitignore::File::new(gitignore_path.as_path()) {
                Ok(exclusion_checker) => filter_ignored_files(exclusion_checker, paths),
                Err(_) => {
                    // We're unable to read the .gitignore file, so we'll silently allow everything.
                    // Please see discussion: https://github.com/rust-lang/mdBook/pull/1051
                    paths.iter().map(|path| path.to_path_buf()).collect()
                }
            }
        }
        None => {
            // There is no .gitignore file.
            paths.iter().map(|path| path.to_path_buf()).collect()
        }
    }
}

fn find_gitignore(book_root: &Path) -> Option<PathBuf> {
    book_root
        .ancestors()
        .map(|p| p.join(".gitignore"))
        .find(|p| p.exists())
}

fn filter_ignored_files(exclusion_checker: gitignore::File, paths: Vec<&PathBuf>) -> Vec<PathBuf> {
    paths
        .iter()
        .filter(|path| match exclusion_checker.is_excluded(path) {
            Ok(exclude) => !exclude,
            Err(error) => {
                error!(
                    "Unable to determine if {:?} is excluded: {:?}. Including it.",
                    &path, error
                );
                true
            }
        })
        .map(|path| path.to_path_buf())
        .collect()
}

/// Calls the closure when a book source file is changed, blocking indefinitely.
pub fn trigger_on_change<F>(book: &MDBook, closure: F)
where
    F: Fn(Vec<PathBuf>, &Path),
{
    use notify::DebouncedEvent::*;
    use notify::RecursiveMode::*;

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
        Ok(w) => w,
        Err(e) => {
            error!("Error while trying to watch the files:\n\n\t{:?}", e);
            std::process::exit(1)
        }
    };

    // Add the source directory to the watcher
    if let Err(e) = watcher.watch(book.source_dir(), Recursive) {
        error!("Error while watching {:?}:\n    {:?}", book.source_dir(), e);
        std::process::exit(1);
    };

    let _ = watcher.watch(book.theme_dir(), Recursive);

    // Add the book.toml file to the watcher if it exists
    let _ = watcher.watch(book.root.join("zk.toml"), NonRecursive);

    info!("Listening for changes...");

    loop {
        let first_event = rx.recv().unwrap();
        sleep(Duration::from_millis(50));
        let other_events = rx.try_iter();

        let all_events = std::iter::once(first_event).chain(other_events);

        let paths = all_events
            .filter_map(|event| {
                debug!("Received filesystem event: {:?}", event);

                match event {
                    Create(path) | Write(path) | Remove(path) | Rename(_, path) => Some(path),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        let paths = remove_ignored_files(&book.root, &paths[..]);

        if !paths.is_empty() {
            closure(paths, &book.root);
        }
    }
}
