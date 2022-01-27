use crate::DEFAULT_BUILD_DIR;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration describing the build process.
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub struct BuildConfig {
    pub build_dir: PathBuf,
    pub create_missing: bool,
    pub disable_default_preprocessors: bool,
    pub front_matter: bool,
    pub math: bool,
    pub readme: bool,
    pub wikilinks: bool,
    pub preprocessors: Vec<String>,
    /// A list of patterns to ignore notes. Based on gitignore syntax.
    pub ignore: Option<Vec<String>>,
}

impl Default for BuildConfig {
    fn default() -> BuildConfig {
        BuildConfig {
            build_dir: PathBuf::from(DEFAULT_BUILD_DIR),
            create_missing: true,
            disable_default_preprocessors: false,
            front_matter: true,
            math: true,
            readme: true,
            wikilinks: true,
            preprocessors: vec![],
            ignore: None,
        }
    }
}
