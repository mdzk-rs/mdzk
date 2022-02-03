mod builder;
pub(crate) mod link;
mod note;

pub use builder::VaultBuilder;
pub use link::{Edge, InternalLink};
pub use note::{Note, NoteId};

use std::collections::HashMap;

#[derive(Default)]
pub struct Vault {
    notes: HashMap<NoteId, Note>,
    id_lookup: HashMap<String, NoteId>,
}

impl Vault {
    pub fn iter(&self) -> impl Iterator<Item = (&NoteId, &Note)> + '_ {
        self.notes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&NoteId, &mut Note)> + '_ {
        self.notes.iter_mut()
    }

    pub fn id_of(&self, title_or_path: impl AsRef<str>) -> Option<&NoteId> {
        self.id_lookup.get(title_or_path.as_ref())
    }

    pub fn backlinks(&self, id: &NoteId) -> Vec<NoteId> {
        self.iter()
            .filter_map(|(backlink_id, note)| {
                if matches!(note.adjacencies.get(&id), Some(Edge::Connected)) {
                    Some(backlink_id.to_owned())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl IntoIterator for Vault {
    type Item = (NoteId, Note);
    type IntoIter = std::collections::hash_map::IntoIter<NoteId, Note>;

    fn into_iter(self) -> Self::IntoIter {
        self.notes.into_iter()
    }
}
