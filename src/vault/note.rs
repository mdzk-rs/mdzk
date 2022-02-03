use crate::{error::Result, Edge};
use chrono::{DateTime, NaiveDate};
use gray_matter::{engine::YAML, Matter, Pod};
use serde::Deserialize;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    path::PathBuf,
};

pub type NoteId = u64;

pub trait FromHash {
    fn from_hash(s: impl Hash) -> Self; 
}

impl FromHash for NoteId {
    fn from_hash(s: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
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
    pub(crate) fn process_front_matter(&mut self) -> Result<()> {
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
                        .or_else(|_| {
                            DateTime::parse_from_rfc3339(format!("{}Z", datestring).as_ref())
                        })
                        .map(|s| s.naive_local())
                        .or_else(|_| {
                            NaiveDate::parse_from_str(&datestring, "%Y-%m-%d")
                                .map(|s| s.and_hms(0, 0, 0))
                        })
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

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}
