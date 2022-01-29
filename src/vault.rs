use crate::{Note, NoteId};
use anyhow::Result;
use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};
use std::{
    cmp::Ordering,
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Builder struct for making a Vault instance.
pub struct VaultBuilder {
    source: PathBuf,
    ignores: Vec<String>,
}

impl VaultBuilder {
    /// Set the source for the directory walker.
    ///
    /// The source has to be a directory, but this
    /// function does not check that. Rather, the [`VaultBuilder::build`] function will do the
    /// check and errors if the check fails.
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

        let notes: HashMap<NoteId, Note> = walker
            .filter_map(|e| e.ok())
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

        Ok(Vault { notes })
    }
}

#[derive(Default)]
pub struct Vault {
    pub notes: HashMap<NoteId, Note>,
}
