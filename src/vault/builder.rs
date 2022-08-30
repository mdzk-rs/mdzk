use crate::{
    vault::Arc,
    wikilink::{for_each_wikilink, CommonMarkHandler, WikilinkContext, WikilinkHandler},
    error::{Error, Result},
    hash::FromHash,
    utils, HashMap, IdMap, Note, NoteId, Vault,
};
use anyhow::Context;
use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};
use rayon::prelude::*;
use std::{
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
    wikilink_handlers: Vec<Box<dyn WikilinkHandler + Sync>>,
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

    #[must_use]
    pub fn wikilink_handlers(self, wikilink_handlers: Vec<Box<dyn WikilinkHandler + Sync>>) -> Self {
        Self {
            wikilink_handlers,
            ..self
        }
    }

    pub fn add_wikilink_handler(&mut self, wikilink_handler: Box<dyn WikilinkHandler + Sync>) {
        self.wikilink_handlers.push(wikilink_handler);
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

        crate::utils::time::store_timezone();

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
                        let id = NoteId::from_hashable(&path_from_root);
                        let content = match utils::fs::read_file(path) {
                            Ok(content) => content,
                            Err(e) => {
                                sender.send(Err(e)).unwrap();
                                return ignore::WalkState::Quit;
                            }
                        };

                        let mut note = Note {
                            id,
                            title: path.file_stem().unwrap().to_string_lossy().to_string(),
                            path: Some(path_from_root),
                            tags: vec![],
                            date: None,
                            extra: Default::default(),
                            original_content: content.clone(),
                            content,
                            invalid_arcs: Vec::new(),
                            adjacencies: IdMap::<Arc>::default(),
                        };

                        note.process_front_matter();

                        sender.send(Ok((id, note))).unwrap();
                    }
                }
                ignore::WalkState::Continue
            })
        });

        drop(sender);

        let mut note_lookup = HashMap::<String, Note>::default();
        let mut adjacencies = IdMap::<Arc>::default();
        let mut notes = IdMap::<Note>::default();

        for res in reciever {
            match res {
                Ok((id, note)) => {
                    // TODO: insert overwrites any previous values. Consider another method to
                    // allow checking for ambiguous links.
                    note_lookup.insert(note.title.clone(), note.clone());
                    if let Some(ref path) = note.path {
                        // This allows linking by filename
                        note_lookup.insert(path.to_string_lossy().to_string(), note.clone());
                    }

                    adjacencies.insert(id, Arc::NotConnected);
                    notes.insert(id, note);
                }
                Err(e) => return Err(e),
            }
        }

        notes.par_iter_mut().try_for_each(|(_, note)| {
            note.adjacencies = adjacencies.clone();
            for_each_wikilink(&note.content.clone(), |link_string, range| {
                let mut final_link_string = link_string.to_owned();
                let ctx = WikilinkContext::estabilsh_context(link_string, note, &note_lookup);

                for handler in self.wikilink_handlers.iter() {
                    handler.run(&mut final_link_string, &ctx)?;
                }

                if let Some(dest) = ctx.destination {
                    note.adjacencies
                        .entry(dest.id)
                        .and_modify(|adj| adj.push_link_range(range));
                }

                note.content = note.content.replacen(
                    &format!("[[{}]]", link_string),
                    &final_link_string,
                    1,
                );

                Ok(())
            })
        })?;

        Ok(Vault {
            root: self.source.to_owned(),
            notes,
            id_lookup: note_lookup.iter().map(|(s, n)| (s.clone(), n.id)).collect(),
        })
    }
}

impl Default for VaultBuilder {
    fn default() -> Self {
        let source = PathBuf::default();
        Self {
            override_builder: OverrideBuilder::new(&source),
            source,
            wikilink_handlers: vec![Box::new(CommonMarkHandler)],
        }
    }
}

#[cfg(test)]
mod tests {
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
