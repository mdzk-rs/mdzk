use crate::{error::Result, link::Edge};
use chrono::{DateTime, NaiveDate};
use gray_matter::{Matter, Pod, engine::YAML};
use serde::Deserialize;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    path::PathBuf,
};

#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq)]
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

#[derive(Clone, Debug)]
pub struct Note {
    pub title: String,
    pub path: Option<PathBuf>,
    pub tags: Vec<String>,
    pub date: Option<chrono::NaiveDateTime>,
    pub content: String,
    pub adjacencies: HashMap<NoteId, Edge>,
}

impl Note {
    pub fn process_front_matter(&mut self) -> Result<()> {
        #[derive(Deserialize)]
        struct FrontMatter {
            title: Option<String>,
            tags: Option<Vec<String>>,
            date: Option<String>,
        }

        let mut handle_front_matter = |data: Pod| {
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
        };

        let gray_matter = Matter::<YAML>::new().parse(&self.content);
        if let Some(data) = gray_matter.data {
            handle_front_matter(data);
        }

        self.content = gray_matter.content;

        Ok(())
    }
}
