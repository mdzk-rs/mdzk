use crate::{
    error::{Error, Result},
    note::link::{create_link, for_each_internal_link, Edge},
    note::FromHash,
    utils, Note, NoteId, Vault,
};
use anyhow::Context;
use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::mpsc,
};

/// Builder struct for making a Vault instance from a directory.
///
/// # Example
///
/// ```ignore
/// # use mdzk::VaultBuilder;
/// let vault = VaultBuilder::default()
///     .source("some/dir")
///     .ignores(["draft-*", "*not-finished*"])
///     .build()
///     .unwrap();
/// ```
///
/// **Note**: This example is not tested, since it is dependent on the file system.
pub struct VaultBuilder {
    source: PathBuf,
    override_builder: OverrideBuilder,
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

    /// Adds multiple ignore patterns for the directory walker.
    ///
    /// The patterns follow the [gitignore format](https://git-scm.com/docs/gitignore).
    ///
    /// ## Panics
    ///
    /// This function will panic on any invalid ignores. For a safer way of adding ignores, see the
    /// [`add_ignore`](VaultBuilder::add_ignore) function.
    #[must_use]
    pub fn ignores(mut self, ignores: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        for ignore in ignores {
            self.add_ignore(ignore).unwrap();
        }
        self
    }

    /// Adds an ignore pattern for the directory walker.
    ///
    /// The patterns follow the [gitignore format](https://git-scm.com/docs/gitignore). Any invalid
    /// pattern will return an error.
    pub fn add_ignore(&mut self, ignore: impl AsRef<str>) -> Result<()> {
        let ignore = ignore.as_ref();
        let s = match ignore.strip_prefix('!') {
            Some(s) => s.to_owned(),
            None => format!("!{}", ignore),
        };
        self.override_builder
            .add(&s)
            .with_context(|| format!("Invalid ignore pattern: {}", ignore))?;

        Ok(())
    }

    /// Build a [`Vault`] from the options supplied to the builder.
    pub fn build(&self) -> Result<Vault> {
        if !self.source.is_dir() {
            return Err(Error::VaultSourceNotDir);
        }

        if !self.source.exists() {
            return Err(Error::PathNotFound(self.source.clone()));
        }

        let walker = {
            let overrides = self
                .override_builder
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

        let (sender, reciever) = mpsc::channel();

        walker.run(|| {
            let sender = sender.clone();
            Box::new(move |e| {
                if let Ok(e) = e {
                    let path = e.path();
                    if !path.is_dir() {
                        let path_from_root = utils::fs::diff_paths(path, &self.source).unwrap();
                        let id = NoteId::from_hash(&path_from_root);
                        let content = match utils::fs::read_file(path) {
                            Ok(content) => content,
                            Err(e) => {
                                sender.send(Err(e)).unwrap();
                                return ignore::WalkState::Quit;
                            }
                        };

                        let mut note = Note {
                            title: path.file_stem().unwrap().to_string_lossy().to_string(),
                            path: Some(path_from_root),
                            tags: vec![],
                            date: None,
                            content,
                            adjacencies: HashMap::new(),
                        };

                        note.process_front_matter();

                        sender.send(Ok((id, note))).unwrap();
                    }
                }
                ignore::WalkState::Continue
            })
        });

        drop(sender);

        let mut id_lookup = HashMap::<String, NoteId>::new();
        let mut adjacencies = HashMap::<NoteId, Edge>::new();
        let mut path_lookup = HashMap::<NoteId, PathBuf>::new();
        let mut notes = HashMap::<NoteId, Note>::new();

        for res in reciever {
            match res {
                Ok((id, note)) => {
                    id_lookup.insert(note.title.clone(), id);
                    if let Some(ref path) = note.path {
                        // This allows linking by filename
                        id_lookup.insert(path.to_string_lossy().to_string(), id);
                    }
                    adjacencies.insert(id, Edge::NotConnected);
                    path_lookup.insert(id, note.path.clone().unwrap());
                    notes.insert(id, note);
                }
                Err(e) => return Err(e.into()),
            }
        }

        notes.par_iter_mut().try_for_each(|(_, note)| {
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

impl Default for VaultBuilder {
    fn default() -> Self {
        let source = PathBuf::default();
        Self {
            override_builder: OverrideBuilder::new(&source),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use std::path::Path;
    use test::Bencher;

    #[bench]
    fn bench_builder(b: &mut Bencher) {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("benchsuite")
            .join("lyt_kit");

        b.iter(|| {
            crate::VaultBuilder::default()
                .source(source.to_owned())
                .build()
                .unwrap()
        });
    }

    #[test]
    fn test_ignores() {
        let ignores = vec![
            "ignore-this-dir",
            "also-ignore-this",
            "!dont-ignore-this-dir",
        ];

        let _ = crate::VaultBuilder::default().ignores(ignores);
    }
}
