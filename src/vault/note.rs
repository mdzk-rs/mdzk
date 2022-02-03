use crate::{error::Result, Edge};
use chrono::{DateTime, NaiveDate};
use gray_matter::{engine::YAML, Matter, Pod};
use serde::Deserialize;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    path::PathBuf,
};

/// Alias for [`u64`]. Uniquely identifies a note.
pub type NoteId = u64;

pub(crate) trait FromHash {
    fn from_hash(s: impl Hash) -> Self; 
}

impl FromHash for NoteId {
    fn from_hash(s: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

/// A node in [`Vault`](crate::Vault). Represents a note in a Zettelkasten.
///
/// This struct is not supposed to be constructed in isolation, but rather interacted with as part
/// of a [`Vault`](crate::Vault) (by e.g. iterating over the [`Vault`](crate::Vault) or using the
/// [`get`](crate::Vault::get) method).
///
/// # Example
///
/// Since the content follows CommonMark, you can easily convert it into HTML by using a Markdown 
/// parser like e.g. [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark).
///
/// ```
/// # use mdzk::Vault;
/// use pulldown_cmark::{Parser, html};
/// 
/// # let vault = Vault::default();
/// for (_, note) in vault {
///     let parser = Parser::new(&note.content);
///     let mut html_output = String::new();
///     html::push_html(&mut html_output, parser);
///
///     assert!(!html_output.is_empty())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Note {
    /// The title of a note.
    pub title: String,
    /// The path to the note, if it exists.
    ///
    /// **Note**: This path is relative to the root of the directory the vault is loaded from.
    pub path: Option<PathBuf>,
    /// The tags assigned to the note.
    pub tags: Vec<String>,
    /// The date assigned to the note.
    pub date: Option<chrono::NaiveDateTime>,
    /// The full content of the note, in [CommonMark](https://commonmark.org/)[^1].
    ///
    /// [^1]: With some extensions. <!-- TODO: Describe which -->
    pub content: String,
    pub(crate) adjacencies: HashMap<NoteId, Edge>,
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
        write!(f, "{}", self.content)
    }
}
