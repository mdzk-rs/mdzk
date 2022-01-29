use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The source for a vault must be a directory.")]
    VaultSourceNotDir,
}
