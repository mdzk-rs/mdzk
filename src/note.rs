use crate::{error::Result, link::Edge};
use chrono::{DateTime, NaiveDate};
use gray_matter::{Matter, engine::YAML};
use serde::Deserialize;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    path::PathBuf,
};

#[derive(Clone, Eq, Hash, Debug, PartialEq)]
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

impl Note {
    pub fn process_content(&mut self, content: &str) -> Result<()> {
        #[derive(Deserialize)]
        struct FrontMatter {
            title: Option<String>,
            tags: Option<Vec<String>>,
            date: Option<String>,
        }

        let gray_matter = Matter::<YAML>::new().parse(content);

        if let Some(data) = gray_matter.data {
            if let Ok(front_matter) = data.deserialize::<FrontMatter>() {
                if let Some(title) = front_matter.title {
                    self.title = title;
                }
                if let Some(tags) = front_matter.tags {
                    self.tags = tags;
                }
                if let Some(datestring) = front_matter.date {
                    self.date = DateTime::parse_from_rfc3339(&datestring)
                        .or_else(|_| DateTime::parse_from_rfc3339(format!("{}Z", datestring).as_ref()))
                        .map(|s| s.naive_local())
                        .or_else(|_| NaiveDate::parse_from_str(&datestring, "%Y-%m-%d").map(|s| s.and_hms(0, 0, 0)))
                        .ok();
                }
            }
        }

        self.content = gray_matter.content;

        Ok(())
    }
}
