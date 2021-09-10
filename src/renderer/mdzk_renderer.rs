use crate::utils;
use mdbook::{Renderer, renderer::RenderContext, errors::*, BookItem, book::Chapter};
use anyhow::Context;
use std::fs;
use std::collections::BTreeMap;
use pulldown_cmark::{Options, Parser, html::push_html};
use handlebars::{Handlebars, no_escape};
use serde_json::json;

/// The HTML backend for mdzk, implementing [`Renderer`](https://docs.rs/mdbook/0.4.12/mdbook/renderer/trait.Renderer.html).
#[derive(Default)]
pub struct HtmlMdzk;

impl HtmlMdzk {
    fn render_chapter(&self, ch: &Chapter, ctx: &RenderContext, hbs: &Handlebars) -> Result<()> {
        let path = ch.path.as_ref().unwrap();

        // Set up CommonMark parser
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_FOOTNOTES);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TASKLISTS);
        let parser = Parser::new_ext(&ch.content, opts);

        // Push HTML into string
        let mut content = String::new();
        push_html(&mut content, parser);

        // Make map with values for all handlebars keys
        let mut data = BTreeMap::new();
        data.insert("content", json!(content));
        data.insert("title", json!(ch.name));
        data.insert("language", json!("en"));
        data.insert("path_to_root", json!(utils::path_to_root(path)));

        // Render output
        let out = hbs.render("index", &data)?;
        
        // Write to file
        let path = &ctx.destination.join(path.with_extension("html"));
        debug!("Writing {:?}", &ctx.destination.join(&path));
        utils::write_file(path, out.as_bytes())?;

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
                .context("Unable to remove old output")?;
        } else {
            // If output direcory doesn't exist, create it
            fs::create_dir_all(destination)
                .context("Unexpected error when constructing destination path")?;
        }

        let mut hbs = Handlebars::new();
        hbs.register_escape_fn(no_escape);
        hbs.register_template_string("index", include_str!("theme/index.hbs"))?;

        // Render out each note
        trace!("mdzk render");
        for item in book.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                if !ch.is_draft_chapter() {
                    self.render_chapter(ch, ctx, &hbs)?;
                }
            }
        }

        // Write static files
        let css_path = destination.join("css");
        let js_path = destination.join("js");
        let font_path = css_path.join("fonts");
        utils::write_file(&css_path.join("main.css"), include_bytes!("theme/css/main.css"))?;
        utils::write_file(&css_path.join("inter.css"), include_bytes!("theme/css/inter.css"))?;
        utils::write_file(&css_path.join("katex.min.css"), include_bytes!("theme/css/katex.min.css"))?;
        utils::write_file(&js_path.join("katex.min.js"), include_bytes!("theme/js/katex.min.js"))?;
        utils::write_file(&js_path.join("auto-render.min.js"), include_bytes!("theme/js/auto-render.min.js"))?;
        if font_path.exists() {
            mdbook::utils::fs::remove_dir_content(&font_path)
                .context("Unable to remove old fonts")?;
        } else {
            fs::create_dir_all(&font_path)
                .context("Unexpected error when constructing fonts path")?;
        }
        for (font, bytes) in FONTS {
            utils::write_file(&font_path.join(font), bytes)?;
        }

        Ok(())
    }
}

const FONTS: [(&str, &[u8]); 21] = [
    ("Inter.ttf", include_bytes!("theme/css/fonts/Inter.ttf")),
    ("KaTeX_Main-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_Main-Regular.woff2")),
    ("KaTeX_Main-Italic.woff2", include_bytes!("theme/css/fonts/KaTeX_Main-Italic.woff2")),
    ("KaTeX_Main-Bold.woff2", include_bytes!("theme/css/fonts/KaTeX_Main-Bold.woff2")),
    ("KaTeX_Main-BoldItalic.woff2", include_bytes!("theme/css/fonts/KaTeX_Main-BoldItalic.woff2")),
    ("KaTeX_Math-Italic.woff2", include_bytes!("theme/css/fonts/KaTeX_Math-Italic.woff2")),
    ("KaTeX_Math-BoldItalic.woff2", include_bytes!("theme/css/fonts/KaTeX_Math-BoldItalic.woff2")),
    ("KaTeX_AMS-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_AMS-Regular.woff2")),
    ("KaTeX_Fraktur-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_Fraktur-Regular.woff2")),
    ("KaTeX_Fraktur-Bold.woff2", include_bytes!("theme/css/fonts/KaTeX_Fraktur-Bold.woff2")),
    ("KaTeX_SansSerif-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_SansSerif-Regular.woff2")),
    ("KaTeX_SansSerif-Italic.woff2", include_bytes!("theme/css/fonts/KaTeX_SansSerif-Italic.woff2")),
    ("KaTeX_SansSerif-Bold.woff2", include_bytes!("theme/css/fonts/KaTeX_SansSerif-Bold.woff2")),
    ("KaTeX_TypeWriter-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_TypeWriter-Regular.woff2")),
    ("KaTeX_Caligraphic-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_Caligraphic-Regular.woff2")),
    ("KaTeX_Caligraphic-Bold.woff2", include_bytes!("theme/css/fonts/KaTeX_Caligraphic-Bold.woff2")),
    ("KaTeX_Script-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_Script-Regular.woff2")),
    ("KaTeX_Size1-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_Size1-Regular.woff2")),
    ("KaTeX_Size2-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_Size2-Regular.woff2")),
    ("KaTeX_Size3-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_Size3-Regular.woff2")),
    ("KaTeX_Size4-Regular.woff2", include_bytes!("theme/css/fonts/KaTeX_Size4-Regular.woff2")),
];
