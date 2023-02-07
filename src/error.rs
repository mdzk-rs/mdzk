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

    #[error("Cannot find the destination for the arc: {0}")]
    /// An error for when an arc destination is not found.
    InvalidArcDestination(String),

    #[error("Could not format the value: {0}")]
    FormatError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error with ignore pattern: {0}")]
    IgnorePatternError(String),

    #[error("Walk error: {0}")]
    WalkError(ignore::Error),

    #[error("Path error: {0}")]
    PathError(#[from] std::path::StripPrefixError),

    #[cfg(feature = "ssg")]
    #[error("Template error: {0}")]
    TemplateError(#[from] sailfish::RenderError),

    #[cfg(feature = "ssg")]
    #[error("TOML deserialize error: {0}")]
    TomlDeserializeError(#[from] toml::de::Error),

    #[cfg(feature = "ssg")]
    #[error("TOML serialize error: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error(transparent)]
    /// Represents all other errors, likely from another crate.
    Other(#[from] anyhow::Error),
}

impl From<ignore::Error> for Error {
    fn from(e: ignore::Error) -> Self {
        match e {
            ignore::Error::Glob { glob, err } => {
                Self::IgnorePatternError(format!("{glob:?}, {err}"))
            }
            ignore::Error::InvalidDefinition => Self::IgnorePatternError(
                "invalid definition (format is type:glob, e.g., html:*.html)".to_owned(),
            ),
            _ => Self::WalkError(e),
        }
    }
}
