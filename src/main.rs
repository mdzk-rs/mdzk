use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let vault = mdzk::VaultBuilder::default()
        .source(args.path.unwrap_or(PathBuf::from(".")))
        .build()?;

    for (id, note) in vault {
        println!("\"{id:x}\": {}", note.title);
    }

    Ok(())
}
