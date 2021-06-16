use std::path::PathBuf;
use crate::utils;
use mdbook::MDBook;
use mdbook::Config;
use failure::{Error, err_msg};

pub fn init(dir: Option<PathBuf>) -> Result<(), Error> {
    let path = match dir {
        Some(path) => path,
        None => PathBuf::from("."),
    };

    let mut builder = MDBook::init(path); 
    let mut config = Config::default();

    config.book.title = Some("Zettelkasten".to_string());
    config.book.src = PathBuf::from("zettels");
    config.build.build_dir = PathBuf::from("out");
    if let Some(author) = utils::get_author_name() {
        config.book.authors.push(author);
    }

    builder.with_config(config);
    match builder.build() {
        Ok(_) => Ok(()),
        Err(e) => Err(err_msg(e.to_string())),
    }
}
