extern crate pest;
#[macro_use]
extern crate pest_derive;

mod arc;
pub mod error;
mod hash;
mod note;
pub mod utils;
pub mod vault;

pub use hash::{HashMap, IdMap};
#[doc(inline)]
pub use note::{Note, NoteId};
#[doc(inline)]
pub use vault::{Vault, VaultBuilder};

#[cfg(feature = "ssg")]
pub mod ssg;
