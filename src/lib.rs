#![cfg_attr(feature = "test", feature(test))]

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod error;
pub mod note;
mod utils;
mod vault;

pub use note::{Note, NoteId};
pub use vault::{Vault, VaultBuilder};
