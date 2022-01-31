use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The source for a vault must be a directory.")]
    VaultSourceNotDir,
    #[error("Cannot find the destination for the internal link: {0}")]
    InvalidInternalLinkDestination(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
