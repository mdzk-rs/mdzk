use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
/// Represents errors that are produced by mdzk.
pub enum Error {
    #[error("The source for a vault must be a directory.")]
    /// Thrown by [`VaultBuilder`](crate::VaultBuilder) when the source is not a directory.
    VaultSourceNotDir,

    #[error("The path {0} was not found.")]
    /// An error stating a path does not exist in the filesystem.
    PathNotFound(PathBuf),

    #[error("Cannot find the destination for the internal link: {0}")]
    /// An error for when a destination for an internal link cannot be found.
    InvalidInternalLinkDestination(String),

    #[error("Could not format the value: {0}")]
    FormatError(#[from] serde_json::Error),

    #[error(transparent)]
    /// Represents all other errors, likely from another crate.
    Other(#[from] anyhow::Error),
}
