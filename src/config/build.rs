use crate::BUILD_DIR;
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
}

impl Default for BuildConfig {
    fn default() -> BuildConfig {
        BuildConfig {
            build_dir: PathBuf::from(BUILD_DIR),
            create_missing: true,
            disable_default_preprocessors: false,
            front_matter: true,
            math: true,
            readme: true,
            wikilinks: true,
            preprocessors: vec![],
        }
    }
}

impl From<BuildConfig> for mdbook::config::BuildConfig {
    fn from(conf: BuildConfig) -> Self {
        Self {
            build_dir: conf.build_dir,
            create_missing: conf.create_missing,
            // Override use_default_preprocessors config option. We are using our own versions of
            // the defaults, controlled with `disable_default_preprocessors`.
            use_default_preprocessors: false,
        }
    }
}
