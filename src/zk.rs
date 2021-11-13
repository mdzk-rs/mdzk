use crate::{
    utils::{find_mdzk_root, update_summary},
    Config, CONFIG_FILE, SUMMARY_FILE,
};

use crate::preprocess::MdzkPreprocessor;
use anyhow::{anyhow, Context, Result};
use mdbook::{book::parse_summary, preprocess::CmdPreprocessor, MDBook};
use mdbook_backlinks::Backlinks;
use mdbook_frontmatter::FrontMatter;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn load_zk(dir: Option<PathBuf>) -> Result<MDBook> {
    let root = match dir {
        Some(path) => path,
        None => find_mdzk_root()
            .ok_or(anyhow!(r#"Could not find an mdzk.

If your mdzk is not located in this directory or any if it's parents, consider
specifying the path with e.g. `mdzk <command> [<path>]`. If you have not created
an mdzk yet, you can initialize one with `mdzk init`."#))?,
    };

    info!("Loading mdzk in {:?}.", root);

    let config: Config = Config::from_disk(root.join(CONFIG_FILE))?;
    info!("Loaded configuration.");

    if config.mdzk.generate_summary.unwrap_or(true) {
        update_summary(&config, &root)?;
    };

    let summary_file = config.mdzk.src.join(SUMMARY_FILE);
    let mut summary_content = String::new();
    File::open(&summary_file)
        .with_context(|| format!("Couldn't open {:?}.", summary_file))?
        .read_to_string(&mut summary_content)?;

    let summary = parse_summary(&summary_content).context("Summary parsing failed.")?;
    info!("Parsed summary.");

    // Cloning some values to avoid cloning the whole config
    let disable_default_preprocessors = config.build.disable_default_preprocessors;
    let preprocessors = config.build.preprocessors.clone();

    let mut zk = MDBook::load_with_config_and_summary(root, config.into(), summary)?;

    if disable_default_preprocessors {
        info!("Running without default mdzk preprocessors.")
    } else {
        zk.with_preprocessor(FrontMatter);
        zk.with_preprocessor(Backlinks);
        zk.with_preprocessor(MdzkPreprocessor);
    }

    for p in preprocessors {
        zk.with_preprocessor(CmdPreprocessor::new(p.to_owned(), format!("mdbook-{}", p)));
    }

    Ok(zk)
}
