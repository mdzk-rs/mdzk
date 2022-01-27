use crate::{Note, NoteId};
use std::collections::HashMap;

struct Vault {
    notes: HashMap<NoteId, Note>,
}
