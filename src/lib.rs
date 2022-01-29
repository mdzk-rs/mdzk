#[macro_use]
pub mod log;

mod link;
mod note;
mod vault;

pub use note::{Note, NoteId};
pub use vault::Vault;
