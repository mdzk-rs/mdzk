use serde::{Deserialize, Serialize};
use toml::value::{Map, Value};

#[derive(Deserialize, Serialize, PartialEq, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct StyleConfig {
    variables: Option<Map<String, Value>>,
}

impl StyleConfig {
    pub fn css_bytes(&self) -> Option<Vec<u8>> {
        if let Some(ref variables) = self.variables {
            let mut css = String::new();
            css.push_str(":root {\n");
            for (key, value) in variables.iter() {
                if let Some(value) = value.as_str() {
                    css.push_str(&format!("--{key}: {value};\n"));
                }
            }
            css.push('}');
            Some(css.into_bytes())
        } else {
            None
        }
    }
}

impl From<mdbook::Config> for StyleConfig {
    fn from(conf: mdbook::Config) -> Self {
        Self {
            variables: conf
                .get("style")
                .and_then(Value::as_table)
                .map(|t| t.to_owned()),
        }
    }
}
