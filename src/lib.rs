#![cfg_attr(feature = "test", feature(test))]

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod error;
pub mod note;
pub mod utils;
mod vault;

#[doc(inline)]
pub use note::{Note, NoteId};
pub use vault::{Vault, VaultBuilder};
