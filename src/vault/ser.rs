use crate::{note::ser::NoteSerialized, utils::string::hex, Vault};
use rayon::prelude::*;
use serde::{Serialize, Serializer};

type VaultSerialized = Vec<NoteSerialized>;

impl From<&Vault> for VaultSerialized {
    fn from(vault: &Vault) -> Self {
        vault
            .par_iter()
            .map(|(id, note)| {
                NoteSerialized::new(
                    hex(id),
                    note.clone(),
                    vault.incoming_arcs(id).map(|(id, _)| hex(id)).collect(),
                )
            })
            .collect()
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
