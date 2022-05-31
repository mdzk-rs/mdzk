pub mod config;
mod copy;
mod static_files;

use crate::{
    error::Result,
    hash::FromHash,
    utils::fs::{diff_paths, path_to_root, write_file},
    utils::time::READABLE,
    NoteId, Vault,
};
use anyhow::Context;
use config::Config;
use rayon::prelude::*;
use sailfish::{runtime::Buffer, TemplateOnce};
use std::path::{Path, PathBuf};

#[derive(TemplateOnce)]
#[template(path = "index.stpl")]
struct IndexTemplate<'a> {
    title_path_list: Vec<(&'a str, PathBuf)>,
    title: &'a str,
    body: String,
    description: Option<&'a str>,
    dark_mode: bool,
}

#[derive(TemplateOnce)]
#[template(path = "note.stpl")]
struct NoteTemplate<'a> {
    title: &'a str,
    description: Option<&'a str>,
    note_title: &'a str,
    body: String,
    backlinks: Vec<(String, PathBuf)>,
    path_to_root: String,
    date: Option<String>,
    dark_mode: bool,
}

pub fn build(vault: &Vault, destination: &Path) -> Result<()> {
    // Handle destination directory
    if destination.exists() {
        std::fs::remove_dir_all(destination)
            .with_context(|| format!("Deletion of destination dir {:?} failed.", destination))?;
    }
    std::fs::create_dir_all(destination)
        .with_context(|| format!("Failed creating destination dir {:?}.", destination))?;

    // Load config
    let config = match Config::from_disk(vault.root.join("mdzk.toml")) {
        Ok(conf) => conf,
        Err(crate::error::Error::Io(_)) => Config::default(),
        Err(e) => return Err(e),
    };

    // Write static files
    static_files::write_static_files(destination, &config)?;

    // Copy other files (images and such)
    copy::copy_other_files(&vault.root, destination)?;

    // Render index
    render_index(vault, destination, &config)?;

    // Render all notes
    render_notes(vault, destination, &config)
}

pub fn render_index(vault: &Vault, destination: &Path, config: &Config) -> Result<()> {
    let readme_id = NoteId::from_hashable(Path::new("README.md"));
    let dark_mode = config
        .style
        .as_ref()
        .and_then(|c| c.dark_mode)
        .unwrap_or(true);
    let body = vault
        .get(&readme_id)
        .map(|note| note.as_html())
        .unwrap_or_default();

    let mut title_path_list = Vec::with_capacity(vault.len());
    for (_, note) in vault.iter() {
        let path = note.path().with_extension("html");

        // Do not include files named `README` or `index` in index
        if path == Path::new("README.html") || path == Path::new("index.html") {
            continue;
        }

        title_path_list.push((note.title.as_ref(), path));
    }
    title_path_list.sort_unstable();

    let tmpl = IndexTemplate {
        title_path_list,
        title: &config.title,
        body,
        description: config.description.as_deref(),
        dark_mode,
    };
    render(tmpl, destination.join("index.html"))
}

pub fn render_notes(vault: &Vault, destination: &Path, config: &Config) -> Result<()> {
    let readme_id = NoteId::from_hashable(Path::new("README.md"));
    let dark_mode = config
        .style
        .as_ref()
        .and_then(|c| c.dark_mode)
        .unwrap_or(true);

    vault.par_iter().try_for_each(|(id, note)| {
        let path = note.path().with_extension("html");

        // Skip rendering files named `README` or `index`
        if path == Path::new("README.html") || path == Path::new("index.html") {
            return Ok(());
        }

        let full_path = destination.join(&path);

        let backlinks = vault
            .incoming(id)
            // Don't render backlinks to the index
            .filter(|(id, _)| *id != &readme_id)
            .map(|(_, other)| {
                (
                    other.title.to_owned(),
                    // NOTE: These unwraps should be safe
                    diff_paths(other.path().with_extension("html"), path.parent().unwrap())
                        .unwrap(),
                )
            })
            .collect();

        let tmpl = NoteTemplate {
            title: &config.title,
            description: config.description.as_deref(),
            note_title: note.title.as_ref(),
            body: note.as_html(),
            backlinks,
            path_to_root: path_to_root(path),
            date: note.date.and_then(|date| date.format(READABLE).ok()),
            dark_mode,
        };

        render(tmpl, full_path)?;

        Ok(())
    })
}

fn render(tmpl: impl TemplateOnce, path: impl AsRef<Path>) -> Result<()> {
    let mut buf = Buffer::new();
    tmpl.render_once_to(&mut buf)?;

    // Uh oh, our first unsafe 😱
    let rendered = unsafe { std::slice::from_raw_parts(buf.as_mut_ptr(), buf.len()) };
    write_file(path, rendered)?;
    Ok(())
}
