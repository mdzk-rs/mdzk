use serde::{Deserialize, Serialize};
use toml::value::{Map, Value};

#[derive(Deserialize, Serialize, PartialEq, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct StyleConfig {
    variables: Option<Map<String, Value>>,
}

impl StyleConfig {
    pub fn css_bytes(&self) -> Vec<u8> {
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
        css.into_bytes()
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
