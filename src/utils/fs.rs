use anyhow::{Context, Result};
use std::{
    fs::File,
    io::Read,
    path::Path,
};

/// Ease-of-use function for loading a file from a path.
pub(crate) fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut buf = String::new();

    File::open(&path)
        .with_context(|| format!("Unable to open {:?}.", path.as_ref()))?
        .read_to_string(&mut buf)
        .with_context(|| format!("Could not read file {:?}.", path.as_ref()))?;

    Ok(buf)
}
