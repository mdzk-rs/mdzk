#[macro_use]
pub mod log;

mod config;
mod link;
mod note;
mod vault;

pub use config::Config;
pub use note::{Note, NoteId};

pub const DEFAULT_BUILD_DIR: &str = "html";
pub const DEFAULT_CONFIG_FILE: &str = "mdzk.toml";
pub const DEFAULT_SRC_DIR: &str = "notes";
