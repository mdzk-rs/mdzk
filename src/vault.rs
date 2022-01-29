use crate::{Note, NoteId, error::Error, link::Edge, utils};
use anyhow::Result;
use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};
use std::{
    cmp::Ordering,
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Builder struct for making a Vault instance.
#[derive(Default)]
pub struct VaultBuilder {
    source: PathBuf,
    ignores: Vec<String>,
}

impl VaultBuilder {
    /// Set the source for the directory walker.
    ///
    /// The source has to be a directory, but this function does not check that. Rather, the
    /// [`build`](VaultBuilder::build) function will do a check and errors if it fails.
    #[must_use]
    pub fn source(self, source: impl AsRef<Path>) -> Self {
        Self {
            source: source.as_ref().to_owned(),
            ..self
        }
    }

    /// Set the ignore patterns for the directory walker.
    ///
    /// The patterns follows the [gitignore format](https://git-scm.com/docs/gitignore).
    #[must_use]
    pub fn ignores(self, ignores: Vec<String>) -> Self {
        Self { ignores, ..self }
    }

    /// Build a [`Vault`] from the options supplied to the builder.
    pub fn build(&self) -> Result<Vault> {
        if !self.source.is_dir() {
            return Err(Error::VaultSourceNotDir.into())
        }

        let mut overrides = OverrideBuilder::new(&self.source);
        for ignore in self.ignores.iter() {
            if let Some(s) = ignore.strip_prefix('!') {
                overrides.add(s)?;
            } else {
                overrides.add(&format!("!{}", ignore))?;
            }
        }

        let walker = WalkBuilder::new(&self.source)
            .hidden(true)
            .overrides(overrides.build()?)
            .types(
                TypesBuilder::new()
                    .add_defaults()
                    .select("markdown")
                    .build()?,
            )
            .sort_by_file_path(|path1, path2| match (path1.is_dir(), path2.is_dir()) {
                // Sort alphabetically, directories first
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => path1.cmp(path2),
            })
            .build();

        let mut notes: HashMap<NoteId, Note> = walker
            .filter_map(|e| e.ok())
            .filter(|e| !e.path().is_dir())
            .map(|e| e.into_path())
            .map(|path| {
                (
                    NoteId::from(path.to_string_lossy()),
                    Note {
                        title: path.file_stem().unwrap().to_string_lossy().to_string(),
                        path: Some(path.to_owned()),
                        tags: vec![],
                        date: None,
                        content: "".to_owned(),
                        adjacencies: HashMap::new(),
                    },
                )
            })
            .collect();

        let adjacencies: HashMap<NoteId, Edge> = notes
            .keys()
            .clone()
            .cloned()
            .map(|id| (id, Edge::NotConnected))
            .collect();

        notes.iter_mut()
            .for_each(|(_, note)| {
                note.adjacencies = adjacencies.clone();
                note.content = utils::fs::read_file(note.path.as_ref().unwrap()).unwrap(); // TODO: Handle error
            });

        Ok(Vault { notes })
    }
}

#[derive(Default)]
pub struct Vault {
    pub notes: HashMap<NoteId, Note>,
}
