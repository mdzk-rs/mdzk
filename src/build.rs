use crate::{
    preprocessors::FrontMatter,
    utils::{find_zk_root, update_summary},
    // HtmlMdzk, 
    CONFIG_FILE, SUMMARY_FILE,
};
use anyhow::Context;
use mdbook::{book::parse_summary, errors::*, Config, MDBook};
use mdbook_backlinks::Backlinks;
use mdbook_katex::KatexProcessor;
use mdbook_wikilink::WikiLinks;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use toml::Value;

pub fn build(dir: Option<PathBuf>) -> Result<()> {
    let zk = load_zk(dir)?;

    zk.build()?;

    Ok(())
}

pub fn load_zk(dir: Option<PathBuf>) -> Result<MDBook, Error> {
    let root = match dir {
        Some(path) => path,
        None => find_zk_root().ok_or(Error::msg("Could not find the root of your Zettelkasten"))?,
    };
    debug!("Found root: {:?}", root);

    let config: Config = Config::from_disk(root.join(CONFIG_FILE)).context(format!(
        "Could not load config file {:?}",
        root.join(CONFIG_FILE)
    ))?;
    debug!("Successfully loaded config.");

    let book_source = &config.book.src;
    update_summary(&book_source)?;

    let summary_file = book_source.join(SUMMARY_FILE);
    let mut summary_content = String::new();
    File::open(&summary_file)
        .with_context(|| {
            format!(
                "Couldn't open {} in {:?} directory",
                SUMMARY_FILE, book_source
            )
        })?
        .read_to_string(&mut summary_content)?;

    let summary = parse_summary(&summary_content).context("Summary parsing failed")?;
    debug!("Parsed summary.");

    let mut zk = MDBook::load_with_config_and_summary(root, config, summary)?;
    info!("Successfully loaded mdzk: {:?}", zk.root);

    if !zk
        .config
        .get("disable_default_preprocessors")
        .unwrap_or(&Value::Boolean(false))
        .as_bool()
        .ok_or(Error::msg("use-mdzk-preprocessors should be a boolean"))?
    {
        zk.with_preprocessor(FrontMatter);
        zk.with_preprocessor(KatexProcessor);
        zk.with_preprocessor(Backlinks);
        zk.with_preprocessor(WikiLinks);
    } else {
        info!("Running without default mdzk preprocessors.")
    }

    // This will register our own renderer, but it does not fully work atm
    // zk.with_renderer(HtmlMdzk);

    Ok(zk)
}
