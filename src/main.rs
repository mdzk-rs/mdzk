mod preprocessors;

use mdzk::{
    build, init, serve,
};
use preprocessors::FrontMatter;
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
        dir: Option<PathBuf>,

        #[structopt(long="port", short="p", default_value="3000")]
        port: i32,

        #[structopt(long="bind", short="b", default_value="localhost")]
        bind: String,
    },
}

fn main() -> Result<(), Error> {
    let args = MDZK::from_args();

    match args.cmd {
        Command::Build{dir} => build(dir),
        Command::Init{dir} => init(dir),
        Command::Serve{dir, port, bind} => serve(dir, port, bind),
    }
}
