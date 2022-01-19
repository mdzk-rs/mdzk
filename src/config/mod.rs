mod build;
pub use build::BuildConfig;
mod mdzk;
pub use mdzk::MdzkConfig;
mod style;
pub use style::StyleConfig;

use anyhow::Context;
use mdbook::config::RustConfig;
use mdbook::errors::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs::File;
use std::io::Read;
use std::path::Path; 
use std::str::FromStr;
use toml::{self, value::Table, Value};

/// This struct represents the configuration of an mdzk. It is loaded from the mdzk.toml file.
pub struct Config {
    /// Metadata describing the mdzk.
    pub mdzk: MdzkConfig,
    /// Configuration for the build process.
    pub build: BuildConfig,
    /// Styling configuration.
    pub style: StyleConfig,
    /// Configuration for the Rust language support.
    pub rust: RustConfig,
    rest: Value,
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

        if conf.rest.get_mut("book").is_some() {
            warn!("Found a '[book]' section on your 'mdzk.toml' file. You might want to replace it with '[mdzk]' ;-)")
        }

        if let Some(preprocessors) = conf
            .rest
            .get_mut("preprocessor")
            .and_then(Value::as_table_mut)
        {
            for (name, _) in preprocessors.iter() {
                conf.build.preprocessors.push(name.to_owned());
            }
            preprocessors.clear();
        }

        Ok(conf)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mdzk: MdzkConfig::default(),
            build: BuildConfig::default(),
            style: StyleConfig::default(),
            rust: RustConfig::default(),
            rest: Value::Table(Table::default()),
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

impl From<Config> for mdbook::Config {
    fn from(conf: Config) -> Self {
        let mut config = mdbook::Config::default();

        // Explicitly set some values in the book config
        config
            .set("mdzk.backlinks-header", conf.mdzk.backlinks_header.clone())
            .ok();
        config
            .set("mdzk.front-matter", conf.build.front_matter)
            .ok();
        config.set("mdzk.math", conf.build.math).ok();
        config.set("mdzk.readme", conf.build.readme).ok();
        config.set("mdzk.wikilinks", conf.build.wikilinks).ok();
        config.set("style", conf.style).ok();

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
            .map(|mdzk| mdzk.try_into().map_err(D::Error::custom))
            .transpose()?
            .unwrap_or_default();

        let build: BuildConfig = table
            .remove("build")
            .map(|build| build.try_into().map_err(D::Error::custom))
            .transpose()?
            .unwrap_or_default();

        let style: StyleConfig = table
            .remove("style")
            .map(|style| style.try_into().map_err(D::Error::custom))
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
            style,
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
