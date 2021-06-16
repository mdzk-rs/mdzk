use mdzk::{build, init};
use std::path::PathBuf;
use quicli::prelude::*;
use structopt::StructOpt;

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

    #[structopt(name = "init")]
    Init {
        dir: Option<PathBuf>,
    }
}

fn main() -> Result<(), Error> {
    let args = MDZK::from_args();

    match args.cmd {
        Command::Build{dir} => build(dir),
        Command::Init{dir} => init(dir),
    }
}
