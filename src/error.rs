use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The source for a vault must be a directory.")]
    VaultSourceNotDir,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
