use mdzk::{build, init, serve};
use mdbook::errors::Error;
use std::path::PathBuf;
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
    },

    #[structopt(name = "serve")]
    Serve {
        #[structopt(parse(from_os_str))]
        dir: Option<PathBuf>
    },
}

fn main() -> Result<(), Error> {
    let args = MDZK::from_args();

    match args.cmd {
        Command::Build{dir} => build(dir),
        Command::Init{dir} => init(dir),
        Command::Serve{dir} => serve(dir),
    }
}
