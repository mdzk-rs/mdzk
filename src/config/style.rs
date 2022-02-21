use serde::{Deserialize, Serialize};
use std::path::Path;
use toml::value::{Map, Value};

#[derive(Deserialize, Serialize, PartialEq, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct StyleConfig {
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

impl From<mdbook::Config> for StyleConfig {
    fn from(conf: mdbook::Config) -> Self {
        if let Some(mut style) = conf
            .get("style")
            .and_then(Value::as_table)
            .map(|t| t.to_owned())
        {
            let additional_css = if let Some(additional) = style.remove("additional-css") {
                Value::as_str(&additional).map(String::from)
            } else {
                None
            };

            Self {
                additional_css,
                variables: Some(style),
            }
        } else {
            Self {
                additional_css: None,
                variables: None,
            }
        }
    }
}
