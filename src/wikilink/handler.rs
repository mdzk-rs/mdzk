use super::Anchor;
use crate::{error::Result, utils::{fs::diff_paths, string::escape_href}};
use super::WikilinkContext;
use std::fmt::Write;

pub trait WikilinkHandler {
    fn run(&self, text: &mut String, ctx: &WikilinkContext) -> Result<()>;
}

pub struct CommonMarkHandler;

impl WikilinkHandler for CommonMarkHandler {
    fn run(&self, text: &mut String, ctx: &WikilinkContext) -> Result<()> {
        if let Some(destination) = ctx.destination {
            let mut href = diff_paths(destination.path(), &ctx.source.path().parent().unwrap())
                    .unwrap()
                    .to_string_lossy()
                    .to_string(); // Gotta love Rust <3

            // Handle anchor
            match &ctx.anchor {
                // Safe unwrap, writing to this string will always work
                Anchor::Header(id) | Anchor::Blockref(id) => write!(&mut href, "#{}", id).unwrap(),
                Anchor::None => {}
            }

            let title = ctx.alias.as_ref().unwrap_or(&destination.title);

            *text = format!("[{}]({})", title, escape_href(&href));
        }

        Ok(())
    }
}
