use anyhow::Result;
use jql::walker;

fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next().unwrap();
    let query = args.next().unwrap_or_else(|| ".".to_owned());

    let vault = mdzk::VaultBuilder::default().source(".").build()?;

    let json = serde_json::to_value(vault)?;

    println!("{}", walker(&json, &query).unwrap());

    Ok(())
}
