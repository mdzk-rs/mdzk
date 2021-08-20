use mdbook::{Renderer, renderer::RenderContext, errors::*, utils, BookItem};
use anyhow::Context;
use std::fs;

/// The HTML backend for mdzk, implementing [`Renderer`](https://docs.rs/mdbook/0.4.12/mdbook/renderer/trait.Renderer.html).
#[derive(Default)]
pub struct HtmlMdzk;

impl Renderer for HtmlMdzk {
    fn name(&self) -> &str {
        "mdzk"
    }

    fn render(&self, ctx: &RenderContext) -> Result<()> {
        let destination = &ctx.destination;
        let book = &ctx.book;

        if destination.exists() {
            utils::fs::remove_dir_content(destination)
                .with_context(|| "Unable to remove stale Markdown output")?;
        } else {
            fs::create_dir_all(&destination)
                .with_context(|| "Unexpected error when constructing destination path")?;
        }

        trace!("markdown render");
        for item in book.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                if !ch.is_draft_chapter() {
                    utils::fs::write_file(
                        &ctx.destination,
                        &ch.path.as_ref().expect("Checked path exists before"),
                        ch.content.as_bytes(),
                    )?;
                }
            }
        }

        Ok(())
    }
}
