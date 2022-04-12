pub(crate) mod link;

use crate::{
    note::link::Edge,
    utils::{self, string::hex},
    IdMap,
};
use gray_matter::{engine::YAML, Matter, Pod};
use pulldown_cmark::{Options, Parser};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Range,
    path::PathBuf,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

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
/// Since the content adheres to CommonMark, you can easily convert it into HTML by using a
/// Markdown parser like e.g. [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark).
///
/// ```
/// # use mdzk::Vault;
/// use pulldown_cmark::{Parser, html};
///
/// // Load a vault...
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
#[derive(Clone, Debug, PartialEq)]
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
    pub date: Option<OffsetDateTime>,
    /// The original content of the source file that produced this note.
    pub original_content: String,
    /// The full content of the note, in [CommonMark](https://commonmark.org/). All internal links
    /// are changed from the
    /// [wikilink style](https://en.wikipedia.org/wiki/Help:Link#Wikilinks_(internal_links)) to
    /// regular [CommonMark links](https://commonmark.org/help/tutorial/07-links.html).
    ///
    /// All CommonMark extensions supported by [`pulldown_cmark`] are also supported by mdzk,
    /// meaning these will be parsed as expected from `content`. You can see a list of the
    /// extensions in the [`pulldown_cmark::Options`] struct. Note that this allows mdzk to
    /// understand the current context and thus not parse internal links within e.g. code blocks.
    pub content: String,
    pub invalid_internal_links: Vec<(Range<usize>, String)>,
    pub(crate) adjacencies: IdMap<Edge>,
}

impl Note {
    pub(crate) fn process_front_matter(&mut self) {
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
                    self.date = utils::time::parse_datestring(&datestring)
                }
            }
        };

        let gray_matter = Matter::<YAML>::new().parse(&self.content);
        if let Some(data) = gray_matter.data {
            handle_front_matter(data);
        }

        self.content = gray_matter.content;
    }

    /// Uses [`pulldown_cmark`] to parse the [`Note::content`] into HTML.
    pub fn as_html(&self) -> String {
        let mut s = String::new();
        let parser = Parser::new_ext(&self.content, Options::all());
        pulldown_cmark::html::push_html(&mut s, parser);
        s
    }

    /// Returns an iterator of [`NoteId`]s for this note's outgoing links.
    pub fn links(&self) -> impl Iterator<Item = &NoteId> + '_ {
        self.adjacencies
            .iter()
            .filter(|(_, edge)| matches!(edge, Edge::Connected(_)))
            .map(|(id, _)| id)
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

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
            links: note.links().map(|id| hex(id)).collect::<Vec<String>>(),
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

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;
    use time::macros::datetime;
    use super::*;

    fn setup_note() -> Note {
        let mut adjacencies: IdMap<Edge> = IdMap::default();
        adjacencies.insert(1, Edge::NotConnected);
        adjacencies.insert(2, Edge::Connected(vec![]));
        Note {
            title: "Test note".to_owned(),
            path: None,
            date: Some(datetime!(2022-04-12 10:43 +2)),
            tags: vec!["tag1".to_owned(), "tag2".to_owned()],
            content: "This is the note content".to_owned(),
            original_content: "This is the note content".to_owned(),
            invalid_internal_links: vec![],
            adjacencies,
        }
    }

    #[bench]
    fn bench_links(b: &mut Bencher) {
        let note = setup_note();
        b.iter(|| {
            let _: Vec<&NoteId> = note.links().collect();
        })
    }
}
