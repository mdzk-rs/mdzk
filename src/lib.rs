#![cfg_attr(feature = "nightly", feature(test))]

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod error;
mod utils;
mod vault;

pub use vault::{Edge, Note, NoteId, Vault, VaultBuilder, Notes, NotesMut};
