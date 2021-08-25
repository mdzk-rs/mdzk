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

    #[structopt(long = "log-level", short = "l", default_value = "3", help = "Set the logging level")]
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
    },
}

fn main() -> Result<(), Error> {
    let args = Mdzk::from_args();

    init_logger(args.log_level);

    match args.cmd {
        Command::Build { dir, renderer } => build(dir, renderer),
        Command::Init { dir } => init(dir),
        Command::Serve { dir, port, bind } => serve(dir, port, bind),
    }
}

fn init_logger(log_level: usize) {
    let mut builder = Builder::new();

    builder.filter(None, match log_level {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        5 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    });

    builder.format(|formatter, record| {
        writeln!(formatter, "{:8>} - {}", record.level(), record.args())
    });

    builder.init();
}
