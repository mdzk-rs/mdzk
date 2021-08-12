use crate::{
    utils,
    CONFIG_FILE,
    SRC_DIR,
    BUILD_DIR,
    DEFAULT_ZK_TITLE,
    SUMMARY_FILE,
};

use mdbook::errors::*;
use mdbook::Config;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use anyhow::Context;

// Initialize an mdzk vault
pub fn init(dir: Option<PathBuf>) -> Result<()> {
    let root = match dir {
        Some(path) => path,
        None => PathBuf::from("."),
    };

    let mut config = Config::default();

    config.book.title = Some(DEFAULT_ZK_TITLE.to_string());
    config.book.src = PathBuf::from(SRC_DIR);
    config.build.build_dir = PathBuf::from(BUILD_DIR);
    if let Some(author) = utils::get_author_name() {
        config.book.authors.push(author);
    }

    // Create directory structure
    debug!("Creating directory tree");
    fs::create_dir_all(&root)?;
    fs::create_dir_all(&root.join(&config.book.src)).context("Could not create source directory.")?;
    fs::create_dir_all(&root.join(&config.build.build_dir)).context("Could not create output directory.")?;

    // Create gitignore
    debug!("Creating .gitignore");
    let mut f = File::create(root.join(".gitignore"))?;
    writeln!(f, "{}", config.build.build_dir.display())?;

    // Create Summary
    let summary = root.join(&config.book.src).join(SUMMARY_FILE);
    if !summary.exists() {
        trace!("Creating summary file.");
        File::create(&summary)?;
    } else {
        trace!("Existing summary found, no need to create a new one.");
    }

    // Create config file
    debug!("Writing {}", CONFIG_FILE);
    let config_path = root.join(CONFIG_FILE);
    let config_bytes = toml::to_vec(&config).with_context(|| "Unable to serialize the config")?;
    File::create(config_path)
        .with_context(|| format!("Couldn't create {}", CONFIG_FILE))?
        .write_all(&config_bytes)
        .with_context(|| format!("Unable to write config to {}", CONFIG_FILE))?;

    Ok(())
}
