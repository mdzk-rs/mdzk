use anyhow::Result;
use crate::{Config, DEFAULT_CONFIG_FILE, Note, NoteId};
use ignore::{
    WalkBuilder,
    overrides::OverrideBuilder,
    types::TypesBuilder,
};
use std::{
    cmp::Ordering,
    collections::HashMap,
    path::Path, 
};

#[derive(Default)]
struct Vault {
    config: Config,
    notes: HashMap<NoteId, Note>,
}

impl Vault {
    fn from_dir(root: &Path) -> Result<Self> {
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
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => path1.cmp(path2),
            })
            .build();

        Ok(Self::default())
    }
}
