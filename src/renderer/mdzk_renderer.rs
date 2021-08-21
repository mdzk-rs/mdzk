use crate::utils;
use mdbook::{Renderer, renderer::RenderContext, errors::*, BookItem, book::Chapter};
use anyhow::Context;
use std::fs;
use pulldown_cmark::{Options, Parser, html::push_html};

/// The HTML backend for mdzk, implementing [`Renderer`](https://docs.rs/mdbook/0.4.12/mdbook/renderer/trait.Renderer.html).
#[derive(Default)]
pub struct HtmlMdzk;

impl HtmlMdzk {
    fn render_chapter(&self, ch: &Chapter, ctx: &RenderContext) -> Result<()> {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_FOOTNOTES);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TASKLISTS);
        let parser = Parser::new_ext(&ch.content, opts);

        let mut html = String::new();
        push_html(&mut html, parser);

        let path = &ctx.destination.join(ch.path.as_ref().unwrap().with_extension("html"));
        debug!("Writing {:?}", &ctx.destination.join(&path));
        utils::write_file(path, html.as_bytes())?;

        Ok(())
    }
}

impl Renderer for HtmlMdzk {
    fn name(&self) -> &str {
        "mdzk"
    }

    fn render(&self, ctx: &RenderContext) -> Result<()> {
        let destination = &ctx.destination;
        let book = &ctx.book;

        if destination.exists() {
            // If output directory exists already, remove all content inside
            mdbook::utils::fs::remove_dir_content(destination)
                .with_context(|| "Unable to remove old output")?;
        } else {
            // If output direcory doesn't exist, create it
            fs::create_dir_all(&destination)
                .with_context(|| "Unexpected error when constructing destination path")?;
        }

        // Render out each note
        trace!("mdzk render");
        for item in book.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                if !ch.is_draft_chapter() {
                    self.render_chapter(ch, ctx)?;
                }
            }
        }

        Ok(())
    }
}
