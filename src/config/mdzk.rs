use crate::SRC_DIR;
use mdbook::config::BookConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub ignore: Option<Vec<String>>,
    /// Whether the vault is multilingual or not.
    pub multilingual: bool,
    /// The language of the vault.
    pub language: Option<String>,
    /// The header before the backlinks list
    pub backlinks_header: Option<String>,
    /// Whether the summary will be auto-generated or not.
    pub generate_summary: Option<bool>,
}

impl Default for MdzkConfig {
    fn default() -> Self {
        Self {
            title: None,
            authors: Vec::new(),
            description: None,
            src: PathBuf::from(SRC_DIR),
            ignore: None,
            multilingual: false,
            language: Some("en".to_string()),
            backlinks_header: None,
            generate_summary: None,
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
