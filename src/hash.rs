use crate::NoteId;
use std::hash::{BuildHasherDefault, Hash, Hasher};

type MdzkHash = fnv::FnvHasher;

/// mdzk's internal [`HashMap`], using the [FNV](https://docs.rs/fnv/latest/fnv/) hasher.
///
/// mdzk uses a [`HashMap`] alias internally that allows us to exchange the hasher in use. This is
/// pretty overkill and does not provide *very* big performance improvements. However, it helps a
/// bit, and since we do some custom hashing magic (e.g. to create note IDs), having this
/// flexibility could prove useful in the future.
pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<MdzkHash>>;

/// An alias for a [`HashMap`](std::collections::HashMap) that uses a [`NoteId`] as it's key.
///
/// Since [`NoteId`] is just a [`u64`] that is meant to be the hashed representation of a path, we
/// do not need to hash it again. Therefore, for HashMaps indexed by NoteId's, we use an
/// identity hasher.
pub type IdMap<V> = std::collections::HashMap<NoteId, V, BuildHasherDefault<IdHasher>>;

pub(crate) trait FromHash {
    fn from_hashable(s: impl Hash) -> Self;
}

impl FromHash for NoteId {
    fn from_hashable(s: impl Hash) -> Self {
        let mut hasher = MdzkHash::default();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

/// A simple identity hasher, that can only hash u64s.
///
/// This is the hasher supplied to our [`IdMap`] alias, in order to avoid double hashing of note
/// IDs.
#[derive(Default)]
pub struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        panic!("IdHasher can only hash a u64")
    }

    fn write_u64(&mut self, i: u64) { self.0 = i; }

    fn finish(&self) -> u64 { self.0 }
}
