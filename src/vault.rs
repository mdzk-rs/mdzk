use anyhow::{bail, Result};
use crate::{Config, DEFAULT_CONFIG_FILE, Note, NoteId, link::Edge};
use ignore::{
    WalkBuilder,
    overrides::OverrideBuilder,
    types::TypesBuilder,
};
use std::{
    cmp::Ordering,
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
struct Vault {
    config: Config,
    notes: HashMap<NoteId, Note>,
}

impl Vault {
    fn load(root: &Path) -> Result<Self> {
        let config = Config::from_disk(root.join(DEFAULT_CONFIG_FILE))?;
        let book_source = root.join(&config.src);

        let mut overrides = OverrideBuilder::new(&book_source);
        if let Some(ref ignores) = config.build.ignore {
            for ignore in ignores.iter() {
                if let Some(s) = ignore.strip_prefix('!') {
                    overrides.add(s)?;
                } else {
                    overrides.add(&format!("!{}", ignore))?;
                }
            }
        }

        let walker = WalkBuilder::new(&book_source)
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

        let paths: Vec<PathBuf> = walker
            .filter_map(|e| e.ok())
            .map(|e| e.into_path())
            .collect();

        let ids = if paths.len() > NoteId::MAX.into() {
            0..paths.len() as u16
        } else {
            bail!("Too many notes!");
        };

        let notes_map: HashMap<NoteId, Note> = ids
            .clone()
            .zip(paths.iter()
                .map(|path| Note {
                    title: path.file_stem().unwrap().to_string_lossy().to_string(),
                    path: Some(path.to_owned()),
                    tags: vec![],
                    date: None,
                    content: "".to_owned(),
                    adjacencies: ids.to_owned().zip(std::iter::repeat(Edge::NotConnected)).collect(),
                })
            )
            .collect();

        Ok(Self::default())
    }
}
