use super::toc;
use crate::utils;
use anyhow::Context;
use handlebars::{no_escape, Handlebars};
use lazy_regex::regex;
use mdbook::{book::Chapter, errors::*, renderer::RenderContext, BookItem, Renderer};
use pulldown_cmark::{html::push_html, CowStr, Event, Options, Parser, Tag};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;

/// The HTML backend for mdzk, implementing [`Renderer`](https://docs.rs/mdbook/0.4.12/mdbook/renderer/trait.Renderer.html).
#[derive(Default)]
pub struct HtmlMdzk;

impl HtmlMdzk {
    fn render_chapter(&self, ch: &Chapter, ctx: &RenderContext, hbs: &Handlebars) -> Result<()> {
        let path = ch.path.as_ref().unwrap();

        let html = render_markdown(&ch.content);

        // Make map with values for all handlebars keys
        let mut data = BTreeMap::new();
        data.insert("content", json!(html));
        data.insert("title", json!(ch.name));
        data.insert("language", json!("en"));
        data.insert("path_to_root", json!(utils::path_to_root(path)));

        // Render output
        println!("Before");
        let out = hbs.render("index", &data)?;
        println!("After");

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
        hbs.register_helper(
            "toc",
            Box::new(toc::RenderToc {
                no_section_label: ctx
                    .config
                    .html_config()
                    .unwrap_or_default()
                    .no_section_label,
            }),
        );

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
        utils::write_file(
            &css_path.join("main.css"),
            include_bytes!("theme/css/main.css"),
        )?;
        utils::write_file(
            &css_path.join("inter.css"),
            include_bytes!("theme/css/inter.css"),
        )?;
        utils::write_file(
            &css_path.join("katex.css"),
            include_bytes!("theme/css/katex.css"),
        )?;
        utils::write_file(
            &js_path.join("katex.min.js"),
            include_bytes!("theme/js/katex.js"),
        )?;
        utils::write_file(
            &js_path.join("auto-render.min.js"),
            include_bytes!("theme/js/auto-render.js"),
        )?;
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

fn render_markdown(text: &str) -> String {
    // Set up CommonMark parser
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(text, opts);
    let parser = parser.map(|event| match event {
        Event::Start(Tag::Link(link_type, dest, title)) => {
            Event::Start(Tag::Link(link_type, fix(dest), title))
        }
        Event::Start(Tag::Image(link_type, dest, title)) => {
            Event::Start(Tag::Image(link_type, fix(dest), title))
        }
        _ => event,
    });

    // Push HTML into string
    let mut content = String::new();
    push_html(&mut content, parser);
    content
}

fn fix(dest: CowStr) -> CowStr {
    let scheme_link_re = regex!(r"^[a-z][a-z0-9+.-]*:");
    let md_link_re = regex!(r"(?P<link>.*)\.md(?P<anchor>#.*)?");

    if dest.starts_with('#') {
        return dest;
    }
    // Don't modify links with schemes like `https`.
    if !scheme_link_re.is_match(&dest) {
        // This is a relative link, adjust it as necessary.
        let mut fixed_link = String::new();

        if let Some(caps) = md_link_re.captures(&dest) {
            fixed_link.push_str(&caps["link"]);
            fixed_link.push_str(".html");
            if let Some(anchor) = caps.name("anchor") {
                fixed_link.push_str(anchor.as_str());
            }
        } else {
            fixed_link.push_str(&dest);
        };
        return CowStr::from(fixed_link);
    }
    dest
}

const FONTS: [(&str, &[u8]); 21] = [
    ("Inter.ttf", include_bytes!("theme/css/fonts/Inter.ttf")),
    (
        "KaTeX_Main-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Main-Regular.woff2"),
    ),
    (
        "KaTeX_Main-Italic.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Main-Italic.woff2"),
    ),
    (
        "KaTeX_Main-Bold.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Main-Bold.woff2"),
    ),
    (
        "KaTeX_Main-BoldItalic.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Main-BoldItalic.woff2"),
    ),
    (
        "KaTeX_Math-Italic.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Math-Italic.woff2"),
    ),
    (
        "KaTeX_Math-BoldItalic.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Math-BoldItalic.woff2"),
    ),
    (
        "KaTeX_AMS-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_AMS-Regular.woff2"),
    ),
    (
        "KaTeX_Fraktur-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Fraktur-Regular.woff2"),
    ),
    (
        "KaTeX_Fraktur-Bold.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Fraktur-Bold.woff2"),
    ),
    (
        "KaTeX_SansSerif-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_SansSerif-Regular.woff2"),
    ),
    (
        "KaTeX_SansSerif-Italic.woff2",
        include_bytes!("theme/css/fonts/KaTeX_SansSerif-Italic.woff2"),
    ),
    (
        "KaTeX_SansSerif-Bold.woff2",
        include_bytes!("theme/css/fonts/KaTeX_SansSerif-Bold.woff2"),
    ),
    (
        "KaTeX_Typewriter-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Typewriter-Regular.woff2"),
    ),
    (
        "KaTeX_Caligraphic-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Caligraphic-Regular.woff2"),
    ),
    (
        "KaTeX_Caligraphic-Bold.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Caligraphic-Bold.woff2"),
    ),
    (
        "KaTeX_Script-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Script-Regular.woff2"),
    ),
    (
        "KaTeX_Size1-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Size1-Regular.woff2"),
    ),
    (
        "KaTeX_Size2-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Size2-Regular.woff2"),
    ),
    (
        "KaTeX_Size3-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Size3-Regular.woff2"),
    ),
    (
        "KaTeX_Size4-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Size4-Regular.woff2"),
    ),
];
