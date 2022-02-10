// Add nightly test crate if compiling tests
#![cfg_attr(test, feature(test))]

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod error;
mod utils;
mod vault;

pub use vault::{Edge, InternalLink, Note, NoteId, Vault, VaultBuilder};
