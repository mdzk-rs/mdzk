use crate::{load_zk, renderer::HtmlMdzk};
use mdbook::{
    errors::*,
    renderer::{HtmlHandlebars, MarkdownRenderer},
};
use std::path::PathBuf;

pub fn build(dir: Option<PathBuf>, renderer: String) -> Result<()> {
    let zk = load_zk(dir)?;

    match renderer.as_str() {
        "markdown" => zk.execute_build_process(&MarkdownRenderer)?,
        "mdbook" => zk.execute_build_process(&HtmlHandlebars)?,
        _ => zk.execute_build_process(&HtmlMdzk)?,
    }

    success!("Successfully built mdzk!");

    Ok(())
}
