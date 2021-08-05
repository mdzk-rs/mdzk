use crate::utils::{find_zk_root, update_summary};
use std::path::PathBuf;
use mdbook::{MDBook, Config, errors::Error};
use mdbook_katex::KatexProcessor;
use mdbook_backlinks::Backlinks;
use mdbook_wikilink::WikiLinks;

pub fn build(dir: Option<PathBuf>) -> Result<(), Error> {
    let mut zk = init_zk(dir)?;

    zk.with_preprocessor(KatexProcessor);
    zk.with_preprocessor(Backlinks);
    zk.with_preprocessor(WikiLinks);
    zk.build().expect("Builing failed");

    Ok(())
}

pub fn init_zk(dir: Option<PathBuf>) -> Result<MDBook, Error> {
    let path = match dir {
        Some(path) => path,
        None => find_zk_root().ok_or(Error::msg("Could not find the root of your Zettelkasten"))?,
    };

    let config_path: PathBuf = [path.to_str().unwrap(), "zk.toml"].iter().collect();
    let config: Config = Config::from_disk(config_path)?;

    let book_source = &config.book.src;
    update_summary(&book_source)?;

    Ok(MDBook::load_with_config(path, config)?)
}
