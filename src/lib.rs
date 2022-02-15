#![cfg_attr(feature = "nightly", feature(test))]

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod error;
pub mod note;
mod utils;
mod vault;

pub use vault::{Vault, VaultBuilder};
pub use note::{Note, NoteId};
