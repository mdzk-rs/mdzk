use serde::{Deserialize, Deserializer, Serialize};
use std::path::Path;
use toml::value::{Map, Value};

#[derive(Serialize, PartialEq, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct StyleConfig {
    /// The header before the backlinks list
    pub backlinks_header: Option<String>,
    additional_css: Option<String>,
    variables: Option<Map<String, Value>>,
}

impl StyleConfig {
    pub fn css_bytes(&self) -> Option<Vec<u8>> {
        let mut css = String::new();

        if let Some(ref variables) = self.variables {
            css.push_str(":root {\n");
            for (key, value) in variables.iter() {
                if let Some(value) = value.as_str() {
                    css.push_str(&format!("--{key}: {value};\n"));
                }
            }
            css.push('}');
        }

        if let Some(ref additional_css) = self.additional_css {
            if Path::new(additional_css).exists() {
                match &std::fs::read_to_string(additional_css) {
                    Ok(s) => css.push_str(s),
                    Err(e) => {
                        warn!("Could not read file {}. {}", additional_css, e);
                        css.push_str(additional_css);
                    }
                }
            } else {
                css.push_str(additional_css);
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
                    "A style config file should always be a TOML table",
                ));
            }
        };

        let backlinks_header = table.remove("backlinks-header").map(|v| v.to_string());

        let additional_css = table.remove("additional-css").map(|v| v.to_string());

        Ok(Self {
            backlinks_header,
            additional_css,
            variables: Some(table),
        })
    }
}
