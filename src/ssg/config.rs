use crate::{error::Result, utils::fs::read_file};
use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt::Write, path::Path};
use toml::value::{Map, Value};

#[derive(Deserialize, Serialize)]
pub struct Config {
    /// The title of the vault.
    pub title: String,
    /// A description of the vault (optional).
    pub description: Option<String>,
    /// Configuration options for styling the generated webpage
    pub style: Option<StyleConfig>,
}

impl Config {
    /// Load an mdzk configuration file from a path.
    pub fn from_disk<P: AsRef<Path>>(path: P) -> Result<Config> {
        Ok(toml::from_str(&read_file(path)?)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "My vault".to_owned(),
            description: None,
            style: None,
        }
    }
}

#[derive(Serialize, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub struct StyleConfig {
    pub dark_mode: Option<bool>,
    css: Option<String>,
    variables: Option<Map<String, Value>>,
}

impl StyleConfig {
    pub fn css_bytes(&self) -> Option<Vec<u8>> {
        let mut css = String::new();

        if let Some(ref variables) = self.variables {
            css.push_str(":root {\n");
            for (key, value) in variables.iter() {
                if let Some(value) = value.as_str() {
                    // Safe unwrap. Writing to this string will always work
                    writeln!(&mut css, "--{key}: {value} !important;").unwrap()
                }
            }
            css.push('}');
        }

        if let Some(ref additional_css) = self.css {
            match &read_file(additional_css) {
                Ok(s) => css.push_str(s),
                // FIXME: This ignores ALL errors. That means if the `css` field is indeed a valid file,
                // but perhaps the read permissions are wrong, the user will not get any error
                // message indicating it. This is a very slim scenario though, so I am not prioritizing it, but
                // it's worth noting for the future.
                Err(_) => css.push_str(additional_css),
            }
        }

        if css.is_empty() {
            None
        } else {
            Some(css.into_bytes())
        }
    }
}

impl<'de> Deserialize<'de> for StyleConfig {
    fn deserialize<D: Deserializer<'de>>(de: D) -> std::result::Result<Self, D::Error> {
        let raw = Value::deserialize(de)?;

        use serde::de::Error;
        let mut table = match raw {
            Value::Table(t) => t,
            _ => {
                return Err(D::Error::custom(
                    "A style config should always be a TOML table",
                ));
            }
        };

        let dark_mode = table.remove("dark-mode").as_ref().and_then(Value::as_bool);
        let css = table
            .remove("css")
            .as_ref()
            .and_then(Value::as_str)
            .map(String::from);

        let variables = if table.is_empty() { None } else { Some(table) };

        Ok(Self {
            css,
            dark_mode,
            variables,
        })
    }
}
