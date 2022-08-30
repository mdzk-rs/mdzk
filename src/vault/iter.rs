use crate::{vault::Arc, Note, NoteId, Vault};
use rayon::prelude::*;
use std::collections::hash_map;

/// An iterator returning references to all notes in a vault in an arbitrary order.
///
/// The notes are indexed by a reference to their [`NoteId`].
pub struct Notes<'a> {
    base: hash_map::Iter<'a, NoteId, Note>,
}

impl<'a> Iterator for Notes<'a> {
    type Item = (&'a NoteId, &'a Note);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.base.next()
    }
}

/// An iterator returning mutable references to all notes in a vault in an arbitrary order.
///
/// The notes are indexed by a reference to their [`NoteId`].
pub struct NotesMut<'a> {
    base: hash_map::IterMut<'a, NoteId, Note>,
}

impl<'a> Iterator for NotesMut<'a> {
    type Item = (&'a NoteId, &'a mut Note);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.base.next()
    }
}

impl Vault {
    /// An iterator visiting all pairs of IDs and corresponding notes in an arbitrary order.
    pub fn iter(&self) -> Notes {
        Notes {
            base: self.notes.iter(),
        }
    }

    /// An iterator visiting all pairs of IDs and corresponding notes in an arbitrary order, with
    /// mutable references to the notes.
    pub fn iter_mut(&mut self) -> NotesMut {
        NotesMut {
            base: self.notes.iter_mut(),
        }
    }

    pub fn par_iter(&self) -> rayon::collections::hash_map::Iter<u64, Note> {
        self.notes.par_iter()
    }

    /// Returns an iterator visiting all notes with an arc leading to the note with ID `id`.
    ///
    /// The returned iterator iterates over pairs of IDs and their corresponding notes. Essentially,
    /// this gives you an iterator of [backlinks](https://en.wikipedia.org/wiki/Backlink).
    ///
    /// If the `id` parameter does not correspond to any note present in this vault, this function
    /// will simply return an empty iterator.
    ///
    /// # Example
    ///
    /// This example prints all note titles in a vault, along with the backlinks' titles.
    ///
    /// ```
    /// # use mdzk::Vault;
    /// # let vault = Vault::default();
    /// for (id, note) in vault.iter() {
    ///     println!("Backlinks of {}:", note.title);
    ///     for (backlink_id, backlink_note) in vault.incoming_arcs(id) {
    ///         println!(" - {}", backlink_note.title);
    ///     }
    /// }
    /// ```
    pub fn incoming_arcs<'a>(&'a self, id: &'a NoteId) -> impl Iterator<Item = (&NoteId, &Note)> {
        self.iter()
            .filter_map(|(other_id, other)| match other.adjacencies.get(id) {
                Some(Arc::Connected(_)) => Some((other_id, other)),
                _ => None,
            })
    }

    /// Returns an iterator visiting all notes connected by an arc coming from the note with ID `id`.
    ///
    /// The returned iterator iterates over pairs of IDs and their corresponding notes.
    /// Essentially, this gives you an iterator of outgoing links.
    ///
    /// If the `id` parameter does not correspond to any note present in this vault, this function
    /// will simply return an empty iterator.
    pub fn outgoing_arcs<'a>(&'a self, id: &'a NoteId) -> impl Iterator<Item = (&NoteId, &Note)> {
        self.get(id)
            .map(|note| {
                note.adjacencies
                    .iter()
                    .filter(|(_, edge)| matches!(edge, Arc::Connected(_)))
                    .map(|(other_id, _)| (other_id, self.get(other_id).unwrap()))
            })
            .into_iter()
            .flatten()
    }
}
