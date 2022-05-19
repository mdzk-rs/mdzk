use anyhow::{bail, Result};
use clap::Parser;
use jql::walker;
use mdzk::utils::string::format_json_value;
use std::path::PathBuf;

#[macro_use]
pub mod log;

#[cfg(feature = "ssg")]
use clap::Subcommand;

#[derive(Parser)]
#[clap(version, about, override_usage = "mdzk [OPTIONS] [QUERY]")]
struct Args {
    #[clap(default_value = ".", hide_default_value = true)]
    /// JSON querying string
    ///
    /// See https://github.com/yamafaktory/jql for a syntax reference.
    query: String,

    #[clap(short = 'l', long = "log-level", default_value = "3", global = true)]
    /// The logging level of mdzk
    ///
    /// The logging levels use the following nomenclature:
    /// [0: off, 1: error, 2: warning and success,  3: info, >4: debug]
    log_level: usize,

    #[clap(
        short = 's',
        long = "source",
        default_value = ".",
        hide_default_value = true,
        global = true
    )]
    /// Path to a vault
    ///
    /// This value must be a directory.
    source: PathBuf,

    #[clap(short = 'i', long = "ignore", multiple_values = true)]
    /// List of ignore patterns for the directory walker
    ///
    /// The ignore patterns are in the gitignore format.
    /// See https://git-scm.com/docs/gitignore#_pattern_format for a reference.
    ignore: Vec<String>,

    #[clap(long = "pretty")]
    /// Prettify the JSON output
    ///
    /// If both --pretty and --raw are specified, the --raw flag takes precedence for strings and arrays.
    pretty: bool,

    #[clap(short = 'r', long = "raw")]
    /// Output shell-compatible raw strings and arrays
    ///
    /// If the output is a string, it will get formatted without quotes and if the output is an
    /// array, it's values will get delimited by newlines and printed without square brackets.
    raw: bool,

    #[cfg(feature = "ssg")]
    #[clap(subcommand)]
    cmd: Option<Commands>,
}

#[cfg(feature = "ssg")]
#[derive(Subcommand)]
enum Commands {
    /// Generate a static site from the vault
    Build {
        #[clap(short = 'o', long = "out", default_value = "./html")]
        /// Path to build the static site to.
        ///
        /// Will create a new directory if the path does not exist. If the path exists already,
        /// this will remove all the contents of that directory.
        destination: PathBuf,
    },
}

fn main() {
    if let Err(err) = run() {
        let chain: String = err.chain().skip(1).map(|e| format!("{}\n", e)).collect();

        error!("{}\n{}", err, chain);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    use clap::ErrorKind::*;
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) if matches!(e.kind(), DisplayHelp | DisplayVersion) => {
            print!("{}", e);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    log::set_max_level(args.log_level);

    let mut builder = mdzk::VaultBuilder::default().source(args.source);
    for ignore in args.ignore {
        builder.add_ignore(ignore)?;
    }
    let vault = builder.build()?;

    #[cfg(feature = "ssg")]
    if let Some(Commands::Build { destination }) = args.cmd {
        mdzk::ssg::build(&vault, &destination)?;
        return Ok(());
    }

    let json = serde_json::to_value(vault)?;

    match walker(&json, &args.query) {
        Ok(out) => println!("{}", format_json_value(&out, args.raw, args.pretty)?),
        Err(msg) => bail!(msg),
    }

    Ok(())
}
