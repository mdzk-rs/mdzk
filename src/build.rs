use crate::{
    preprocessors::FrontMatter,
    utils::{find_zk_root, update_summary},
    CONFIG_FILE,
    SUMMARY_FILE,
};
use mdbook::{
    errors::Error,
    Config,
    MDBook,
    book::parse_summary,
};
use mdbook_backlinks::Backlinks;
use mdbook_katex::KatexProcessor;
use mdbook_wikilink::WikiLinks;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use toml::Value;
use anyhow::Context;

pub fn build(dir: Option<PathBuf>) -> Result<(), Error> {
    let zk = load_zk(dir)?;

    zk.build().expect("Builing failed");

    Ok(())
}

pub fn load_zk(dir: Option<PathBuf>) -> Result<MDBook, Error> {
    let root = match dir {
        Some(path) => path,
        None => find_zk_root().ok_or(Error::msg("Could not find the root of your Zettelkasten"))?,
    };

    let config: Config = Config::from_disk(root.join(CONFIG_FILE))?;

    let book_source = &config.book.src;
    update_summary(&book_source)?;

    let summary_file = book_source.join(SUMMARY_FILE);
    let mut summary_content = String::new();
    File::open(&summary_file)
        .with_context(|| format!("Couldn't open SUMMARY.md in {:?} directory", book_source))?
        .read_to_string(&mut summary_content)?;

    let summary = parse_summary(&summary_content)
        .with_context(|| format!("Summary parsing failed for file={:?}", summary_file))?;

    let mut zk = MDBook::load_with_config_and_summary(root, config, summary)?;
    if !zk
        .config
        .get("disable_default_preprocessors")
        .unwrap_or(&Value::Boolean(false))
        .as_bool()
        .ok_or(Error::msg(
            "disable_default_preprocessors should be a boolean",
        ))?
    {
        zk.with_preprocessor(FrontMatter);
        zk.with_preprocessor(KatexProcessor);
        zk.with_preprocessor(Backlinks);
        zk.with_preprocessor(WikiLinks);
    }

    Ok(zk)
}
