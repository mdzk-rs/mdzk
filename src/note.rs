use crate::link::Edge;
use std::{
    collections::HashMap,
    path::PathBuf,
};

// This puts an upper bound on the amount of notes to 65536.
pub type NoteId = u16;

pub struct Note {
    title: String,
    path: Option<PathBuf>,
    tags: Vec<String>,
    date: Option<chrono::NaiveDateTime>,
    content: String,
    adjacencies: HashMap<NoteId, Edge>,
}
