use anyhow::{bail, Result};
use clap::Parser;
use jql::walker;
use mdzk::utils::string::format_json_value;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(version, about)]
struct Mdzk {
    #[clap(default_value = ".", hide_default_value = true)]
    /// JSON querying string
    query: String,

    #[clap(short, long, default_value = "3", value_name = "LEVEL", global = true)]
    /// The logging level of mdzk
    log_level: usize,

    #[clap(
        short,
        long,
        default_value = ".",
        value_name = "DIR",
        hide_default_value = true,
        global = true
    )]
    /// Path to a vault source
    source: PathBuf,

    #[clap(short, long, action = clap::ArgAction::Append)]
    /// List of gitignore patterns for the directory walker
    ignore: Vec<String>,

    #[clap(long)]
    /// Prettify the JSON output
    pretty: bool,

    #[clap(short, long)]
    /// Output shell-compatible raw strings and arrays
    raw: bool,
}

fn main() {
    if let Err(err) = run() {
        let chain: String = err.chain().skip(1).map(|e| format!("{e}\n")).collect();

        eprintln!("{}\n{}", err, chain);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Mdzk::parse();

    let mut builder = mdzk::VaultBuilder::default().source(args.source);
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
