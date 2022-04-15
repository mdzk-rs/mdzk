use crate::{Vault, note::NoteSerialized, utils::string::hex};
use serde::{Serialize, Serializer};
use rayon::prelude::*;

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
                        vault.incoming(id).map(|(id, _)| hex(id)).collect(),
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
