use crate::{error::Result, utils::string::kebab};
use pest::Parser;
use super::{Anchor, WikilinkContext};

pub trait WikilinkPreprocessor {
    fn run(&self, ctx: &mut WikilinkContext) -> Result<()>;
}

pub struct MdzkPreprocessor;

#[derive(Parser)]
#[grammar = "wikilink/grammar.pest"]
pub struct WikilinkParser;

impl WikilinkPreprocessor for MdzkPreprocessor {
    fn run(&self, ctx: &mut WikilinkContext) -> Result<()> {
        if let Ok(mut link) = WikilinkParser::parse(Rule::link, ctx.text) {
            let mut link = link.next().unwrap().into_inner();

            // Handle destination title
            let mut dest = link.next().unwrap().into_inner();
            let _destination_title = dest.next().unwrap().as_str();

            // Handle header and blockref
            ctx.anchor = match dest.next() {
                Some(anchor) => match anchor.into_inner().next() {
                    Some(x) if x.as_rule() == Rule::header => Anchor::Header(kebab(x.as_str())),
                    Some(x) if x.as_rule() == Rule::blockref => Anchor::Blockref(kebab(x.as_str())),
                    _ => panic!("This should not happen..."),
                },
                None => Anchor::None,
            };

            // TODO: ctx.destination = note_lookup.get(destination_title);
            ctx.alias = link.next().map(|pair| pair.as_str().to_owned());
        }

        Ok(())
    }
}
