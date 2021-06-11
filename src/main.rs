use std::{
    fs,
    path::PathBuf,
};
use quicli::prelude::*;
use structopt::StructOpt;
use mdbook::MDBook;
use mdbook_katex::KatexProcessor;
use wikilink::AutoTitle;

#[derive(StructOpt)]
#[structopt(name = "mdzk", about = "A Zettelkasten tool based on mdBook.")]
struct MDZK {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "build", about = "Build a Zettelkasten")]
    Build {
        #[structopt(parse(from_os_str))]
        dir: Option<PathBuf>
    },

    #[structopt(name = "new")]
    New {
        name: String,
    }
}

fn main() -> CliResult {
    let args = MDZK::from_args();

    match args.cmd {
        Command::Build{dir} => match dir {
            Some(path) => {
                let mut md = MDBook::load(path).expect("Unable to load the book");
                md.with_preprocessor(KatexProcessor);
                md.with_preprocessor(AutoTitle::new());
                md.build().expect("Builing failed");
            },
            None => {
                let mut md = MDBook::load(".").expect("Unable to load the book");
                md.with_preprocessor(KatexProcessor);
                md.with_preprocessor(AutoTitle::new());
                md.build().expect("Builing failed");
            },
        },
        Command::New{name} => println!("Creating new: {}", name),
    }

    Ok(())
}
