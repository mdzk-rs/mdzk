mod builder;
pub mod iter;
mod ser;

use crate::IdMap;
pub use crate::{
    arc::Arc,
    note::{Note, NoteId},
};
pub use builder::VaultBuilder;

use std::path::PathBuf;

/// A directed graph, where the nodes are [`Note`]s.
///
/// The graph is represented as an adjacency matrix. This gives fast lookup times and allows the
/// discovery of incoming arcs to be very quick.
#[derive(Default, Debug)]
pub struct Vault {
    /// The root directory of the vault.
    ///
    /// Since `Vault` can only be built by [`VaultBuilder`], this [`PathBuf`] is guaranteed to be a
    /// directory.
    pub root: PathBuf,
    pub(crate) notes: IdMap<Note>,
}

impl Vault {
    /// Gets a reference to a [`Note`] by it's [`NoteId`].
    ///
    /// Returns the reference as `Some(note_ref)`. If the given [`NoteId`] does not correspond to
    /// any [`Notes`](Note), this function will return `None`.
    pub fn get(&self, id: &NoteId) -> Option<&Note> {
        self.notes.get(id)
    }

    /// Gets a mutable reference to a [`Note`] by it's [`NoteId`].
    ///
    /// Returns the reference as `Some`. If the given [`NoteId`] does not correspond to any
    /// [`Note`]s, this function will return `None`.
    pub fn get_mut(&mut self, id: &NoteId) -> Option<&mut Note> {
        self.notes.get_mut(id)
    }

    /// Returns the amount of [`Note`]s in the vault.
    pub fn len(&self) -> usize {
        self.notes.len()
    }

    /// Returns `true` if the vault does not contain any notes.
    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }
}

impl IntoIterator for Vault {
    type Item = (NoteId, Note);
    type IntoIter = std::collections::hash_map::IntoIter<NoteId, Note>;

    fn into_iter(self) -> Self::IntoIter {
        self.notes.into_iter()
    }
}

impl PartialEq for Vault {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(id, note)| other.get(id).map_or(false, |n| *note == *n))
    }
}

impl Eq for Vault {}
