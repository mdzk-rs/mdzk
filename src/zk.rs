use crate::{
    Config,
    utils::{find_mdzk_root, update_summary},
    CONFIG_FILE,
    SUMMARY_FILE,
};

use anyhow::Context;
use mdbook::{book::parse_summary, errors::*, MDBook};
use mdbook_backlinks::Backlinks;
use mdbook_frontmatter::FrontMatter;
use mdbook_katex::KatexProcessor;
use mdbook_wikilinks::WikiLinks;
use mdbook_readme::ReadmePreprocessor;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn load_zk(dir: Option<PathBuf>) -> Result<MDBook, Error> {
    let root = match dir {
        Some(path) => path,
        None => find_mdzk_root()
            .ok_or_else(|| Error::msg("Could not find the root of your Zettelkasten"))?,
    };
    debug!("Found root: {:?}", root);

    let config: Config = Config::from_disk(root.join(CONFIG_FILE))
        .with_context(|| format!(
            "Could not load config file {:?}",
            root.join(CONFIG_FILE)
        ))?;
    debug!("Successfully loaded config.");

    update_summary(&config, &root)?;

    let summary_file = config.mdzk.src.join(SUMMARY_FILE);
    let mut summary_content = String::new();
    File::open(&summary_file)
        .with_context(|| format!("Couldn't open {:?}", summary_file))?
        .read_to_string(&mut summary_content)?;

    let summary = parse_summary(&summary_content).context("Summary parsing failed")?;
    debug!("Parsed summary.");

    let disable_default_preprocessors = config.build.disable_default_preprocessors.clone();
    let mut zk = MDBook::load_with_config_and_summary(root, config.into(), summary)?;
    info!("Successfully loaded mdzk in: {:?}", zk.root);

    if !disable_default_preprocessors {
        zk.with_preprocessor(FrontMatter);
        zk.with_preprocessor(KatexProcessor);
        zk.with_preprocessor(Backlinks);
        zk.with_preprocessor(WikiLinks);
        zk.with_preprocessor(ReadmePreprocessor);
    } else {
        info!("Running without default mdzk preprocessors.")
    }

    Ok(zk)
}
