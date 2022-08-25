pub(crate) mod ser;

use crate::{
    arc::Arc,
    utils::{string::fix_link, time::parse_datestring},
    IdMap,
};
use gray_matter::{engine::YAML, Matter, Pod};
use pulldown_cmark::{html::push_html, Event, Options, Parser, Tag};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::{ops::Range, path::PathBuf};
use time::OffsetDateTime;

/// Alias for [`u64`]. Uniquely identifies a note.
pub type NoteId = u64;

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
    /// Extra metadata from the front matter, kept as-were
    pub extra: HashMap<String, Value>,
    /// The original content of the source file that produced this note.
    pub original_content: String,
    /// The full content of the note, in [CommonMark](https://commonmark.org/). All wikilinks
    /// are changed from the
    /// [wikilink style](https://en.wikipedia.org/wiki/Help:Link#Wikilinks_(internal_links)) to
    /// regular [CommonMark links](https://commonmark.org/help/tutorial/07-links.html).
    ///
    /// All CommonMark extensions supported by [`pulldown_cmark`] are also supported by mdzk,
    /// meaning these will be parsed as expected from `content`. You can see a list of the
    /// extensions in the [`pulldown_cmark::Options`] struct. Note that this allows mdzk to
    /// understand the current context and thus not parse wikilinks within e.g. code blocks.
    pub content: String,
    pub invalid_arcs: Vec<(Range<usize>, String)>,
    pub(crate) adjacencies: IdMap<Arc>,
}

impl Note {
    pub(crate) fn process_front_matter(&mut self) {
        #[derive(Deserialize)]
        struct FrontMatter {
            title: Option<String>,
            tags: Option<Vec<String>>,
            date: Option<String>,
            #[serde(flatten)]
            extra: HashMap<String, Value>,
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
                    self.date = parse_datestring(&datestring)
                }
                self.extra = front_matter.extra;
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
        let parser = Parser::new_ext(&self.content, Options::all()).map(|event| match event {
            Event::Start(Tag::Link(link_type, dest, title)) => {
                Event::Start(Tag::Link(link_type, fix_link(dest), title))
            }
            Event::Start(Tag::Image(link_type, dest, title)) => {
                Event::Start(Tag::Image(link_type, fix_link(dest), title))
            }
            _ => event,
        });

        let mut s = String::new();
        push_html(&mut s, parser);
        s
    }

    /// Returns an iterator of [`NoteId`]s for this note's outgoing arcs.
    ///
    /// This function gives the same as running [`Vault::outgoing`](crate::Vault::outgoing) on this
    /// note's ID.
    pub fn outgoing_arcs(&self) -> impl Iterator<Item = &NoteId> + '_ {
        self.adjacencies
            .iter()
            .filter(|(_, edge)| matches!(edge, Arc::Connected(_)))
            .map(|(id, _)| id)
    }

    pub fn path(&self) -> PathBuf {
        self.path
            .clone()
            .unwrap_or_else(|| PathBuf::from(&self.title))
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}
