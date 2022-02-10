use crate::{
    error::{Error, Result},
    utils,
    vault::link::{create_link, for_each_internal_link},
    vault::note::FromHash,
    Edge, Note, NoteId, Vault,
};
use anyhow::Context;
use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Builder struct for making a Vault instance.
///
/// # Example
///
/// ```ignore
/// # use mdzk::VaultBuilder;
/// let vault = VaultBuilder::default()
///     .source("some/dir")
///     .ignores(vec!["some/dir/ignore-this".to_owned()])
///     .build()
///     .unwrap();
/// ```
///
/// **Note**: This example is not tested, since it is dependent on the file system.
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
            return Err(Error::VaultSourceNotDir);
        }

        let walker = {
            let mut builder = OverrideBuilder::new(&self.source);
            for ignore in self.ignores.iter() {
                builder
                    .add(ignore)
                    .with_context(|| format!("Invalid ignore pattern: {}", ignore))?;
            }
            let overrides = builder
                .build()
                .context("Building walker overrides failed.")?;

            let types = TypesBuilder::new()
                .add_defaults()
                .select("markdown")
                .build()
                .expect("Building default types should never fail.");

            WalkBuilder::new(&self.source)
                .hidden(true)
                .overrides(overrides)
                .types(types)
                .build_parallel()
        };

        let notes = Arc::new(Mutex::new(HashMap::<NoteId, Note>::new()));
        let adjacencies = Arc::new(Mutex::new(HashMap::<NoteId, Edge>::new()));
        let id_lookup = Arc::new(Mutex::new(HashMap::<String, NoteId>::new()));

        walker.run(|| {
            let notes = notes.clone();
            let adjacencies = adjacencies.clone();
            let id_lookup = id_lookup.clone();
            Box::new(move |e| {
                if let Ok(e) = e {
                    let path = e.path();
                    if !path.is_dir() {
                        let path_from_root = utils::fs::diff_paths(&path, &self.source).unwrap();
                        let id = NoteId::from_hash(&path_from_root);

                        let mut note = Note {
                            title: path.file_stem().unwrap().to_string_lossy().to_string(),
                            path: Some(path_from_root),
                            tags: vec![],
                            date: None,
                            content: utils::fs::read_file(path).unwrap(),
                            adjacencies: HashMap::new(),
                        };

                        note.process_front_matter().unwrap();

                        id_lookup.lock().unwrap().insert(note.title.clone(), id);
                        if let Some(ref path) = note.path {
                            // This allows linking by filename
                            id_lookup
                                .lock()
                                .unwrap()
                                .insert(path.to_string_lossy().to_string(), id);
                        }
                        adjacencies.lock().unwrap().insert(id, Edge::NotConnected);
                        notes.lock().unwrap().insert(id, note);
                    }
                }
                ignore::WalkState::Continue
            })
        });

        let id_lookup = id_lookup.lock().unwrap().to_owned();
        let adjacencies = adjacencies.lock().unwrap().to_owned();
        let mut notes = notes.lock().unwrap().to_owned();

        let path_lookup = notes
            .clone()
            .into_iter()
            .map(|(id, note)| (id, note.path.unwrap()))
            .collect();

        notes.iter_mut().try_for_each(|(_, note)| {
            note.adjacencies = adjacencies.clone();
            for_each_internal_link(&note.content.clone(), |link_string| {
                match create_link(link_string, &path_lookup, &id_lookup) {
                    Ok(link) => {
                        note.adjacencies
                            .entry(link.dest_id)
                            .and_modify(|adj| *adj = Edge::Connected);
                        note.content = note.content.replacen(
                            &format!("[[{}]]", link_string),
                            &link.cmark(note.path.as_ref().unwrap().parent().unwrap()),
                            1,
                        );
                    }
                    // NOTE: This error is currently ignored, but could be useful as a toggleable
                    // error, since that would allow users of mdzk to ensure all links have a valid
                    // destination on vault creation.
                    Err(Error::InvalidInternalLinkDestination(_)) => {}
                    Err(e) => return Err(e),
                }

                Ok(())
            })
        })?;

        Ok(Vault { notes, id_lookup })
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use std::path::PathBuf;
    use test::Bencher;

    #[bench]
    fn bench_builder(b: &mut Bencher) {
        b.iter(|| {
            let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("benchsuite")
                .join("lyt_kit");

            crate::VaultBuilder::default()
                .source(source)
                .build()
                .unwrap()
        });
    }
}
