mod builder;
pub(crate) mod link;
mod note;

pub use builder::VaultBuilder;
pub use link::{Edge, InternalLink};
pub use note::{Note, NoteId};

use std::collections::HashMap;

#[derive(Default)]
pub struct Vault {
    pub notes: HashMap<NoteId, Note>,
}
