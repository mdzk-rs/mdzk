extern crate pest;
#[macro_use]
extern crate pest_derive;

mod error;
mod utils;
mod vault;

pub use vault::{InternalLink, Edge, Note, NoteId, Vault, VaultBuilder};
