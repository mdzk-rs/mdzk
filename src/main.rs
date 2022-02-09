use anyhow::Result;

fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next().unwrap();
    let source = args.next().unwrap_or_else(|| ".".to_owned());

    let vault = mdzk::VaultBuilder::default().source(source).build()?;

    for (id, note) in vault.iter() {
        println!("Backlinks of {}:", note.title);
        for backlink_id in vault.backlinks(*id) {
            println!(" - {}", vault.get(backlink_id).unwrap().title);
        }
    }

    Ok(())
}
