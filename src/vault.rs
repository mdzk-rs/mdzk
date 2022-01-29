use crate::{Config, Note, NoteId, DEFAULT_CONFIG_FILE};
use anyhow::Result;
use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};
use std::{cmp::Ordering, collections::HashMap, path::Path};

#[derive(Default)]
pub struct Vault {
    pub config: Config,
    pub notes: HashMap<NoteId, Note>,
}

impl Vault {
    pub fn load(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
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

        Ok(Self { notes, config })
    }
}
