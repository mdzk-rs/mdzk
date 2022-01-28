use crate::link::Edge;
use std::{collections::HashMap, path::PathBuf};

// This puts an upper bound on the amount of notes to 65536.
pub type NoteId = u16;

pub struct Note {
    pub title: String,
    pub path: Option<PathBuf>,
    pub tags: Vec<String>,
    pub date: Option<chrono::NaiveDateTime>,
    pub content: String,
    pub adjacencies: HashMap<NoteId, Edge>,
}
