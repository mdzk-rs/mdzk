use crate::{error::Result, utils::{fs::diff_paths, string::escape_href}};
use super::{Anchor, WikilinkContext};
use std::fmt::Write;

pub trait WikilinkRenderer {
    fn render(&self, ctx: &WikilinkContext) -> Result<String>;
}

pub struct CommonMarkRenderer;

impl WikilinkRenderer for CommonMarkRenderer {
    fn render(&self, ctx: &WikilinkContext) -> Result<String> {
        if let Some(destination) = ctx.destination {
            let mut href = diff_paths(destination.path(), &ctx.source.path().parent().unwrap())
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

            // Handle anchor
            match &ctx.anchor {
                // Safe unwrap, writing to this string will always work
                Anchor::Header(id) | Anchor::Blockref(id) => write!(&mut href, "#{}", id)?,
                Anchor::None => {}
            }

            let title = ctx.alias.as_ref().unwrap_or(&destination.title);

            Ok(format!("[{}]({})", title, escape_href(&href)))
        } else {
            Ok(ctx.text.to_owned())
        }
    }
}
