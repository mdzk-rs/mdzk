extern crate pest;
#[macro_use]
extern crate pest_derive;

mod error;
mod link;
mod note;
mod utils;
mod vault;

pub use note::{Note, NoteId};
pub use vault::{Vault, VaultBuilder};
