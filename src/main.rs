use anyhow::Result;
use mdzk::{build, init, serve};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "mdzk", about = "A Zettelkasten tool based on mdBook.")]
struct Mdzk {
    #[structopt(subcommand)]
    cmd: Command,

    #[structopt(
        long = "log-level",
        short = "l",
        default_value = "3",
        help = "Set the logging level"
    )]
    log_level: usize,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "build", about = "Builds an mdzk")]
    Build {
        #[structopt(parse(from_os_str))]
        dir: Option<PathBuf>,

        #[structopt(long = "renderer", short = "r", default_value = "mdbook")]
        renderer: String,
    },

    #[structopt(name = "init", about = "Initializes an mdzk")]
    Init { dir: Option<PathBuf> },

    #[structopt(name = "serve", about = "Serves the generated files")]
    Serve {
        #[structopt(parse(from_os_str))]
        dir: Option<PathBuf>,

        #[structopt(long = "port", short = "p", default_value = "3000")]
        port: i32,

        #[structopt(long = "bind", short = "b", default_value = "localhost")]
        bind: String,

        #[structopt(long = "renderer", short = "r", default_value = "mdzk")]
        renderer: String,
    },
}

fn main() {
    if let Err(e) = run() {
        mdzk::error!("{}{}", e, mdzk::log::format_chain(e.chain()));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Mdzk::from_args();

    mdzk::log::set_max_level(args.log_level);

    match args.cmd {
        Command::Build { dir, renderer } => build(dir, renderer),
        Command::Init { dir } => init(dir),
        Command::Serve {
            dir,
            port,
            bind,
            renderer,
        } => serve(dir, port, bind, renderer),
    }
}
