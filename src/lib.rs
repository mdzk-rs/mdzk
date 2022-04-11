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

/// An alias for a [`HashMap`](std::collections::HashMap) that uses a [`NoteId`] as it's key.
///
/// Since [`NoteId`] is just a [`u64`] that is meant to be the hashed representation of a path, we
/// do not need to hash it again. Therefore, we for HashMaps indexed by NoteId's, we use an
/// identity hasher by default.
pub type IdMap<T> = std::collections::HashMap<NoteId, T, nohash_hasher::BuildNoHashHasher<NoteId>>;
