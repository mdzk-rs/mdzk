use crate::load_zk;
use mdbook::errors::*;
use std::path::PathBuf;

pub fn build(dir: Option<PathBuf>) -> Result<()> {
    let zk = load_zk(dir)?;

    zk.build()?;

    Ok(())
}

