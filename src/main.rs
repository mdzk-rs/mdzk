use anyhow::Result;

fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next().unwrap();
    let source = args.next().unwrap_or_else(|| ".".to_owned());

    let vault = mdzk::VaultBuilder::default().source(source).build()?;

    for backlink in vault.backlinks(vault.id_of("Bias").unwrap()) {
        println!("{}", backlink);
    }

    Ok(())
}
