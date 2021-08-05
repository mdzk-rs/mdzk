use crate::utils::find_zk_root;
use std::path::PathBuf;
use std::io::prelude::*;
use std::fs::File;
use mdbook::{MDBook, Config};
use mdbook_katex::KatexProcessor;
use mdbook_backlinks::Backlinks;
use mdbook_wikilink::WikiLinks;
use failure::{Error, err_msg};
use walkdir::WalkDir;

const PAD_SIZE: usize = 4;

pub fn build(dir: Option<PathBuf>) -> Result<(), Error> {
    let path = match dir {
        Some(path) => path,
        None => find_zk_root().ok_or(err_msg("Could not find the root of your Zettelkasten"))?,
    };

    let config_path: PathBuf = [path.to_str().unwrap(), "book.toml"].iter().collect();
    let config: Config = match Config::from_disk(config_path) {
        Ok(config) => config,
        Err(_) => return Err(err_msg("Could not load a config file.")),
    };

    let book_source = &config.book.src;
    let summary = WalkDir::new(book_source)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path() != book_source)
        .map(|e| {
            let stripped_path = e.path().strip_prefix(&book_source).unwrap();
            let file_stem = stripped_path.file_stem().unwrap().to_str().unwrap();
            let depth = (stripped_path.components().count() - 1) * PAD_SIZE;

            if e.path().is_dir() {
                return Some(format!("{1:>0$}- [{2}]()\n", depth, "", file_stem));
            }

            let file_ext = e.path().extension().unwrap().to_str().unwrap();
            if file_ext == "md" {
                return Some(format!(
                    "{1:>0$}- [{2}](<{3}>)\n",
                    depth,
                    "",
                    file_stem,
                    stripped_path.to_str().unwrap()
                ));
            } else {
                None
            }
        })
        .filter_map(|e| e)
        .fold(String::new(), |acc, curr| acc + &curr);

    let mut summary_file = File::create([&config.book.src, &PathBuf::from("SUMMARY.md")].iter().collect::<PathBuf>())?;
    summary_file.write_all(summary.as_bytes())?;

    let mut zk = match MDBook::load_with_config(path, config) {
        Ok(val) => val,
        Err(e) => return Err(err_msg(e.to_string())),
    };

    zk.with_preprocessor(KatexProcessor);
    zk.with_preprocessor(Backlinks);
    zk.with_preprocessor(WikiLinks);
    zk.build().expect("Builing failed");

    Ok(())
}
