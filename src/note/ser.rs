use crate::{utils::string::hex, Note};
use serde::Serialize;
use std::path::PathBuf;
use time::format_description::well_known::Rfc3339;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct NoteSerialized {
    id: String,
    title: String,
    path: Option<PathBuf>,
    tags: Vec<String>,
    date: Option<String>,
    content: String,
    original_content: String,
    links: Vec<String>,
    backlinks: Vec<String>,
}

impl NoteSerialized {
    pub(crate) fn new(id: String, note: Note, backlinks: Vec<String>) -> Self {
        Self {
            id,
            links: note.links().map(hex).collect::<Vec<String>>(),
            title: note.title,
            path: note.path,
            tags: note.tags,
            date: note.date.and_then(|date| date.format(&Rfc3339).ok()),
            content: note.content,
            original_content: note.original_content,
            backlinks,
        }
    }
}
