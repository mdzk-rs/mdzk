use crate::{BUILD_DIR, SRC_DIR};

use anyhow::Context;
use mdbook::config::{BookConfig, RustConfig};
use mdbook::errors::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use toml::{self, value::Table, Value};

/// This struct represents the configuration of an mdzk. It is loaded from the mdzk.toml file.
pub struct Config {
    /// Metadata describing the mdzk.
    pub mdzk: MdzkConfig,
    /// Configuration for the build process.
    pub build: BuildConfig,
    /// Configuration for the Rust language support.
    pub rust: RustConfig,
    rest: Value,
}

impl Config {
    /// Load an mdzk configuration file from a path.
    pub fn from_disk<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut buffer = String::new();
        File::open(path)
            .context("Unable to open the configuration file")?
            .read_to_string(&mut buffer)
            .context("Couldn't read the file")?;

        Config::from_str(&buffer)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mdzk: MdzkConfig::default(),
            build: BuildConfig::default(),
            rust: RustConfig::default(),
            rest: Value::Table(Table::default()),
        }
    }
}

impl FromStr for Config {
    type Err = Error;

    /// Load an mdzk configuration from some string.
    fn from_str(src: &str) -> Result<Self> {
        toml::from_str(src).with_context(|| "Invalid configuration file")
    }
}

impl From<Config> for mdbook::Config {
    fn from(conf: Config) -> Self {
        let mut config = mdbook::Config::default();

        config.set("mdzk.backlinks-header", conf.mdzk.backlinks_header.clone()).ok();

        config.book = conf.mdzk.into();
        config.build = conf.build.into();
        config.rust = conf.rust;

        for (key, value) in conf.rest.as_table().unwrap().iter() {
            config.set(key, value).ok();
        }

        config
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(de: D) -> std::result::Result<Self, D::Error> {
        let raw = Value::deserialize(de)?;

        use serde::de::Error;
        let mut table = match raw {
            Value::Table(t) => t,
            _ => {
                return Err(D::Error::custom(
                    "A config file should always be a toml table",
                ));
            }
        };

        let mdzk: MdzkConfig = table
            .remove("mdzk")
            .map(|book| book.try_into().map_err(D::Error::custom))
            .transpose()?
            .unwrap_or_default();

        let build: BuildConfig = table
            .remove("build")
            .map(|build| build.try_into().map_err(D::Error::custom))
            .transpose()?
            .unwrap_or_default();

        let rust: RustConfig = table
            .remove("rust")
            .map(|rust| rust.try_into().map_err(D::Error::custom))
            .transpose()?
            .unwrap_or_default();

        Ok(Config {
            mdzk,
            build,
            rust,
            rest: Value::Table(table),
        })
    }
}

impl Serialize for Config {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut table = self.rest.clone();

        let book_config = Value::try_from(&self.mdzk).expect("should always be serializable");
        table.insert("mdzk", book_config);

        if self.build != BuildConfig::default() {
            let build_config = Value::try_from(&self.build).expect("should always be serializable");
            table.insert("build", build_config);
        }

        if self.rust != RustConfig::default() {
            let rust_config = Value::try_from(&self.rust).expect("should always be serializable");
            table.insert("rust", rust_config);
        }

        table.serialize(s)
    }
}

/// This struct holds the configuration for the metadata of the mdzk, as well as what it should
/// include.
#[derive(Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct MdzkConfig {
    /// The title of the vault.
    pub title: Option<String>,
    /// The author(s) of the vault.
    pub authors: Vec<String>,
    /// A description of the vault (optional).
    pub description: Option<String>,
    /// The source directory of the vault, relative to the location of the config file.
    pub src: PathBuf,
    /// A list of patterns to ignore notes. Based on gitignore syntax.
    pub ignore: Vec<String>,
    /// Whether the vault is multilingual or not.
    pub multilingual: bool,
    /// The language of the vault.
    pub language: Option<String>,
    /// The header before the backlinks list
    pub backlinks_header: Option<String>,
}

impl Default for MdzkConfig {
    fn default() -> Self {
        MdzkConfig {
            title: None,
            authors: Vec::new(),
            description: None,
            src: PathBuf::from(SRC_DIR),
            ignore: Vec::new(),
            multilingual: false,
            language: Some("en".to_string()),
            backlinks_header: None,
        }
    }
}

impl From<MdzkConfig> for BookConfig {
    fn from(conf: MdzkConfig) -> Self {
        Self {
            title: conf.title,
            authors: conf.authors,
            description: conf.description,
            src: conf.src,
            multilingual: conf.multilingual,
            language: conf.language,
        }
    }
}

/// Configuration describing the build process.
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub struct BuildConfig {
    pub build_dir: PathBuf,
    pub create_missing: bool,
    pub disable_default_preprocessors: bool,
}

impl Default for BuildConfig {
    fn default() -> BuildConfig {
        BuildConfig {
            build_dir: PathBuf::from(BUILD_DIR),
            create_missing: true,
            disable_default_preprocessors: false,
        }
    }
}

impl From<BuildConfig> for mdbook::config::BuildConfig {
    fn from(conf: BuildConfig) -> Self {
        Self {
            build_dir: conf.build_dir,
            create_missing: conf.create_missing,
            // Override use_default_preprocessors config option. We are using our own versions of
            // the defaults, controlled with `disable_default_preprocessors`.
            use_default_preprocessors: false,
        }
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
