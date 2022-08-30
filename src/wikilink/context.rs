use crate::{utils::string::kebab, Note, HashMap};
use pest::Parser;

#[derive(Debug)]
pub struct WikilinkContext<'a> {
    pub source: &'a Note,
    pub destination: Option<&'a Note>,
    pub alias: Option<String>,
    pub anchor: Anchor,
}

#[derive(Parser)]
#[grammar = "wikilink/grammar.pest"]
pub struct WikilinkParser;

impl<'a> WikilinkContext<'a> {
    pub(crate) fn estabilsh_context(text: &str, source: &'a Note, note_lookup: &'a HashMap<String, Note>) -> Self {
        if let Ok(mut link) = WikilinkParser::parse(Rule::link, text) {
            let mut link = link.next().unwrap().into_inner();

            // Handle destination title
            let mut dest = link.next().unwrap().into_inner();
            let destination_title = dest.next().unwrap().as_str();

            // Handle header and blockref
            let anchor = match dest.next() {
                Some(anchor) => match anchor.into_inner().next() {
                    Some(x) if x.as_rule() == Rule::header => Anchor::Header(kebab(x.as_str())),
                    Some(x) if x.as_rule() == Rule::blockref => Anchor::Blockref(kebab(x.as_str())),
                    _ => panic!("This should not happen..."),
                },
                None => Anchor::None,
            };

            WikilinkContext {
                source,
                destination: note_lookup.get(destination_title),
                alias: link.next().map(|pair| pair.as_str().to_owned()),
                anchor,
            }
        } else {
            WikilinkContext {
                source,
                destination: None,
                alias: None,
                anchor: Default::default(),
            }
        }
    }
}

#[derive(Default, Debug, Eq, PartialEq)]
pub enum Anchor {
    Header(String),
    Blockref(String),
    #[default]
    None,
}

// TODO: Write tests
