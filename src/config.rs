use crate::{SRC_DIR, BUILD_DIR};

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::fs::File;
use std::io::Read;
use mdbook::config::{RustConfig, BookConfig};
use mdbook::errors::{Error, Result};
use anyhow::Context;
use serde::{Deserialize, Deserializer};
use toml::Value;

/// This struct represents the configuration of an mdzk. It is loaded from the mdzk.toml file.
pub struct Config {
    /// Metadata describing the mdzk.
    pub mdzk: MdzkConfig,
    /// Configuration for the build process.
    pub build: BuildConfig,
    /// Configuration for the Rust language support.
    pub rust: RustConfig,
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

impl FromStr for Config {
    type Err = Error;

    /// Load an mdzk configuration from some string.
    fn from_str(src: &str) -> Result<Self> {
        toml::from_str(src).with_context(|| "Invalid configuration file")
    }
}

impl Into<mdbook::Config> for Config {
    fn into(self) -> mdbook::Config {
        let mut config = mdbook::Config::default();
        config.book = self.mdzk.into();
        config.build = self.build.into();
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

        let mdzk: MdzkConfig = table
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
            mdzk,
            build,
            rust,
        })
    }
}

/// This struct holds the configuration for the metadata of the mdzk, as well as what it should
/// include.
#[derive(Deserialize)]
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
            language: Some("en".to_string())
        }
    }
}

impl Into<BookConfig> for MdzkConfig {
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

/// Configuration describing the build process.
#[derive(Deserialize)]
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

impl Into<mdbook::config::BuildConfig> for BuildConfig {
    fn into(self) -> mdbook::config::BuildConfig {
        mdbook::config::BuildConfig {
            build_dir: self.build_dir,
            create_missing: self.create_missing,
            // Override use_default_preprocessors config option. We are using our own versions of
            // the defaults, controlled with `disable_default_preprocessors`.
            use_default_preprocessors: false,
        }
    }
}
