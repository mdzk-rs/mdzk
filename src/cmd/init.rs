use crate::{utils, Config, BUILD_DIR, CONFIG_FILE, DEFAULT_ZK_TITLE, SRC_DIR};

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

/// Initialize an mdzk vault
pub fn init(dir: Option<PathBuf>) -> Result<()> {
    let root = match dir {
        Some(path) => path,
        None => PathBuf::from("."),
    };

    info!("Creating an mdzk vault for you in {:?}.", root);

    let mut config = Config::default();

    config.mdzk.title = Some(DEFAULT_ZK_TITLE.to_string());
    config.mdzk.src = PathBuf::from(SRC_DIR);
    config.build.build_dir = PathBuf::from(BUILD_DIR);
    if let Some(author) = utils::get_author_name() {
        config.mdzk.authors.push(author);
    }

    // Create directory structure
    debug!("Creating directory tree");
    fs::create_dir_all(&root)?;
    fs::create_dir_all(&root.join(&config.mdzk.src))
        .context("Could not create source directory.")?;
    info!("Created a directory to put your notes.");

    // Create gitignore
    debug!("Creating .gitignore");
    let mut f = File::create(root.join(".gitignore"))?;
    writeln!(f, "{}", config.build.build_dir.display())?;
    info!("Generated a gitignore.");

    // Create config file
    debug!("Writing {}", CONFIG_FILE);
    let config_path = root.join(CONFIG_FILE);
    let config_bytes = toml::to_vec(&config).context("Unable to serialize the config")?;
    File::create(config_path)
        .context("Couldn't create the configuration file.")?
        .write_all(&config_bytes)
        .context("Unable to write to the configuration file.")?;

    success!(
        r#"Your mdzk is now initialized!

To start using mdzk, write some notes in {:?} or move your existing notes there.
You can then run `mdzk serve` to start a webserver with live updating.

If you need more help, you can always run `mdzk help` or view the documentation
online at https://mdzk.app/docs."#,
        config.mdzk.src
    );

    Ok(())
}
