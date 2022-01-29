use crate::link::Edge;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    path::PathBuf,
};

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct NoteId(u64);

impl NoteId {
    pub fn from(s: impl Into<String>) -> Self {
        let mut hasher = DefaultHasher::new();
        s.into().hash(&mut hasher);
        Self(hasher.finish())
    }
}

impl std::fmt::Display for NoteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug)]
pub struct Note {
    pub title: String,
    pub path: Option<PathBuf>,
    pub tags: Vec<String>,
    pub date: Option<chrono::NaiveDateTime>,
    pub content: String,
    pub adjacencies: HashMap<NoteId, Edge>,
}
