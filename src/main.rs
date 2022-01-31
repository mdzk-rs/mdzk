use anyhow::Result;

fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next().unwrap();
    let source = args.next().unwrap_or(".".to_owned());

    let vault = mdzk::VaultBuilder::default()
        .source(source)
        .build()?;

    println!("{:#?}", vault.notes);

    Ok(())
}
