mod build;
pub use build::BuildConfig;
mod style;
pub use style::StyleConfig;

use anyhow::{Context, Error, Result};
use crate::DEFAULT_SRC_DIR;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};
use toml::{self, value::Table, Value};

/// This struct represents the configuration of an mdzk. It is loaded from the mdzk.toml file.
#[derive(Deserialize, Serialize)]
pub struct Config {
    /// The title of the vault.
    pub title: Option<String>,
    /// The author(s) of the vault.
    pub authors: Vec<String>,
    /// A description of the vault (optional).
    pub description: Option<String>,
    /// The source directory of the vault, relative to the location of the config file.
    pub src: PathBuf,
    /// The language of the vault.
    pub language: Option<String>,
    /// Configuration for the build process.
    pub build: BuildConfig,
    pub style: StyleConfig,
}

impl Config {
    /// Load an mdzk configuration file from a path.
    pub fn from_disk<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut buffer = String::new();
        File::open(&path)
            .with_context(|| format!("Unable to open {:?}.", path.as_ref()))?
            .read_to_string(&mut buffer)
            .context("Couldn't read the configuration file")?;

        let mut conf = Config::from_str(&buffer).with_context(|| {
            format!("Unable to load the configuration file {:?}", path.as_ref())
        })?;

        Ok(conf)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: None,
            authors: Vec::new(),
            description: None,
            src: PathBuf::from(DEFAULT_SRC_DIR),
            language: Some("en".to_string()),
            build: BuildConfig::default(),
            style: StyleConfig::default(),
        }
    }
}

impl FromStr for Config {
    type Err = Error;

    /// Load an mdzk configuration from some string.
    fn from_str(src: &str) -> Result<Self> {
        toml::from_str(src).with_context(|| format!("Invalid TOML:\n\n{}\n", src))
    }
}

trait Insert {
    fn insert(&mut self, key: &str, value: Value);
}

impl Insert for Value {
    fn insert(&mut self, key: &str, value: Value) {
        fn split(key: &str) -> Option<(&str, &str)> {
            let ix = key.find('.')?;

            let (head, tail) = key.split_at(ix);
            // splitting will leave the "."
            let tail = &tail[1..];

            Some((head, tail))
        }

        if !self.is_table() {
            *self = Value::Table(Table::new());
        }

        let table = self.as_table_mut().expect("unreachable");

        if let Some((head, tail)) = split(key) {
            table
                .entry(head)
                .or_insert_with(|| Value::Table(Table::new()))
                .insert(tail, value);
        } else {
            table.insert(key.to_string(), value);
        }
    }
}
