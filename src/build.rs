use crate::utils::find_zk_root;
use std::path::PathBuf;
use mdbook::MDBook;
use mdbook_katex::KatexProcessor;
use wikilink::AutoTitle;

pub fn build(dir: Option<PathBuf>) -> Result<(), String> {
    let path = match dir {
        Some(path) => path,
        None => find_zk_root().ok_or("Could not find a Zettelkasten root.".to_string())?,
    };

    let mut md = MDBook::load(path).expect("Unable to load the book");
    md.with_preprocessor(KatexProcessor);
    md.with_preprocessor(AutoTitle::new());
    md.build().expect("Builing failed");

    Ok(())
}
