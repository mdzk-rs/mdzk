use anyhow::{bail, Result};
use clap::Parser;
use jql::walker;
use mdzk::utils::string::format_json_value;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(version, about, override_usage = "mdzk [OPTIONS] [QUERY]")]
struct Args {
    #[clap(default_value = ".", hide_default_value = true)]
    /// JSON querying string
    ///
    /// See https://github.com/yamafaktory/jql for a syntax reference.
    query: String,

    #[clap(
        short = 'p',
        long = "path",
        default_value = ".",
        hide_default_value = true
    )]
    /// Path to a vault
    ///
    /// This value must be a directory.
    directory: PathBuf,

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
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::try_parse()?;

    let mut builder = mdzk::VaultBuilder::default().source(args.directory);
    for ignore in args.ignore {
        builder.add_ignore(ignore)?;
    }
    let vault = builder.build()?;

    let json = serde_json::to_value(vault)?;

    match walker(&json, &args.query) {
        Ok(out) => println!("{}", format_json_value(&out, args.raw, args.pretty)?),
        Err(msg) => bail!(msg),
    }

    Ok(())
}
