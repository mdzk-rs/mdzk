mod builder;

pub use crate::note::{link::Edge, Note, NoteId};
use crate::{note::NoteSerialized, utils::string::hex, IdMap};
pub use builder::VaultBuilder;

use rayon::prelude::*;
use serde::{Serialize, Serializer};
use std::collections::HashMap;

/// A directed graph, where the nodes are [`Note`]s.
///
/// The graph is represented as an adjacency matrix. This gives fast lookup times and allows the
/// discovery of backlinks to be very quick.
#[derive(Default, Debug)]
pub struct Vault {
    notes: IdMap<Note>,
    id_lookup: HashMap<String, NoteId>,
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

    /// Returns the [`NoteId`] to a note that has the supplied title or path.
    ///
    /// **Note**: If two notes share the same title, the [`NoteId`] returned is determined by which
    /// note is read last when generating the vault. This can make looking up IDs by title
    /// non-consistent. Paths are unique to each [`Note`] and should therefore be consistent.
    pub fn id_of(&self, title_or_path: impl AsRef<str>) -> Option<&NoteId> {
        self.id_lookup.get(title_or_path.as_ref())
    }

    /// Returns an iterator of backlink [`NoteId`]s.
    ///
    /// The IDs point to notes that link to the note identified by the paramater ID (`id`).
    /// Essentially, this gives you an iterator of
    /// [backlinks](https://en.wikipedia.org/wiki/Backlink).
    ///
    /// # Example
    ///
    /// This example prints all note titles and their backlinks.
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use mdzk::Vault;
    /// # let vault = Vault::default();
    /// for (id, note) in vault.iter() {
    ///     println!("Backlinks of {}:", note.title);
    ///     for backlink_id in vault.backlinks(*id) {
    ///         println!(" - {}", vault.get(backlink_id).unwrap().title);
    ///     }
    /// }
    /// ```
    pub fn backlinks(&self, id: NoteId) -> impl Iterator<Item = &NoteId> + '_ {
        self.iter()
            .filter_map(move |(backlink_id, note)| match note.adjacencies.get(&id) {
                Some(Edge::Connected(_)) => Some(backlink_id),
                _ => None,
            })
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

/// An iterator returning references to all notes in a vault in an arbitrary order.
///
/// The notes are indexed by a reference to their [`NoteId`].
pub struct Notes<'a> {
    base: std::collections::hash_map::Iter<'a, NoteId, Note>,
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
    base: std::collections::hash_map::IterMut<'a, NoteId, Note>,
}

impl<'a> Iterator for NotesMut<'a> {
    type Item = (&'a NoteId, &'a mut Note);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.base.next()
    }
}

#[derive(Serialize)]
struct VaultSerialized {
    notes: Vec<NoteSerialized>,
}

impl From<&Vault> for VaultSerialized {
    fn from(vault: &Vault) -> Self {
        Self {
            notes: vault
                .notes
                .par_iter()
                .map(|(id, note)| {
                    NoteSerialized::new(
                        hex(id),
                        note.clone(),
                        vault.backlinks(*id).map(hex).collect(),
                    )
                })
                .collect(),
        }
    }
}

impl Serialize for Vault {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serialized: VaultSerialized = self.into();
        serialized.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::{NoteId, Vault};
    use serde_json::json;
    use std::path::Path;
    use test::Bencher;

    fn setup() -> Vault {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("benchsuite");

        crate::VaultBuilder::default()
            .source(source.to_owned())
            .build()
            .unwrap()
    }

    #[bench]
    fn bench_serializer(b: &mut Bencher) {
        let vault = setup();
        b.iter(|| json!(vault));
    }

    #[bench]
    fn bench_backlinks(b: &mut Bencher) {
        let vault = setup();
        b.iter(|| {
            let _: Vec<&NoteId> = vault.backlinks(10479125933004782128).collect();
        })
    }
}
