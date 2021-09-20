use crate::SRC_DIR;

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::fs::File;
use std::io::Read;
use mdbook::config::{BuildConfig, RustConfig, BookConfig};
use mdbook::errors::{Error, Result};
use anyhow::Context;
use serde::{Deserialize, Deserializer};
use toml::Value;

/// This struct represents the configuration of an mdzk. It is loaded from the mdzk.toml file.
pub struct Config {
    pub vault: VaultConfig,
    pub build: BuildConfig,
    pub rust: RustConfig,
}

impl Config {
    pub fn from_disk<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut buffer = String::new();
        File::open(path)
            .context("Unable to open the configuration file")?
            .read_to_string(&mut buffer)
            .context("Couldn't read the file")?;

        Config::from_str(&buffer)
    }
}

impl FromStr for Config {
    type Err = Error;

    /// Load a `Config` from some string.
    fn from_str(src: &str) -> Result<Self> {
        toml::from_str(src).with_context(|| "Invalid configuration file")
    }
}

impl Into<mdbook::Config> for Config {
    fn into(self) -> mdbook::Config {
        let mut config = mdbook::Config::default();
        config.book = self.vault.into();
        config.build = self.build;
        config.rust = self.rust;
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

        let vault: VaultConfig = table
            .remove("vault")
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
            vault,
            build,
            rust,
        })
    }
}

#[derive(Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct VaultConfig {
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
}

impl Default for VaultConfig {
    fn default() -> Self {
        VaultConfig {
            title: None,
            authors: Vec::new(),
            description: None,
            src: PathBuf::from(SRC_DIR),
            ignore: Vec::new(),
            multilingual: false,
            language: Some("en".to_string())
        }
    }
}

impl Into<BookConfig> for VaultConfig {
    fn into(self) -> BookConfig {
        BookConfig {
            title: self.title,
            authors: self.authors,
            description: self.description,
            src: self.src,
            multilingual: self.multilingual,
            language: self.language,
        }
    }
}
