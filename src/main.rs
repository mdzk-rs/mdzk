use env_logger::Builder;

use mdbook::errors::Error;
use mdzk::{build, init, serve};
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "mdzk", about = "A Zettelkasten tool based on mdBook.")]
struct Mdzk {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "build", about = "Build a Zettelkasten")]
    Build {
        #[structopt(parse(from_os_str))]
        dir: Option<PathBuf>,
    },

    #[structopt(name = "init")]
    Init { dir: Option<PathBuf> },

    #[structopt(name = "serve")]
    Serve {
        #[structopt(parse(from_os_str))]
        dir: Option<PathBuf>,

        #[structopt(long = "port", short = "p", default_value = "3000")]
        port: i32,

        #[structopt(long = "bind", short = "b", default_value = "localhost")]
        bind: String,
    },
}

fn main() -> Result<(), Error> {
    init_logger();

    let args = Mdzk::from_args();

    match args.cmd {
        Command::Build { dir } => build(dir),
        Command::Init { dir } => init(dir),
        Command::Serve { dir, port, bind } => serve(dir, port, bind),
    }
}

fn init_logger() {
    let mut builder = Builder::new();
    builder.filter(None, log::LevelFilter::Info);
    builder.format(|formatter, record| {
        writeln!(formatter, "{:8>} - {}", record.level(), record.args())
    });

    builder.init();
}
