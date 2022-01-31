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
    pub(crate) id_lookup: HashMap<String, NoteId>,
}

impl Vault {
    pub fn iter(&self) -> impl Iterator<Item = (&NoteId, &Note)> + '_ {
        self.notes.iter()
    }

    pub fn id_of(&self, title_or_path: impl AsRef<str>) -> Option<&NoteId> {
        self.id_lookup.get(title_or_path.as_ref())
    }

    pub fn backlinks(&self, id: &NoteId) -> Vec<Note> {
        self.iter()
            .filter_map(|(_, note)| {
                if matches!(note.adjacencies.get(&id), Some(Edge::Connected)) {
                    Some(note.to_owned())
                } else {
                    None
                }
            })
            .collect()
    }
}
