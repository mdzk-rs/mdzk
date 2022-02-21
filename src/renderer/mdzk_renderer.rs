use super::toc;
use crate::{config::StyleConfig, utils};
use anyhow::Context;
use handlebars::{no_escape, Handlebars};
use mdbook::{book::Chapter, errors::*, renderer::RenderContext, BookItem, Renderer};
use pulldown_cmark::{html::push_html, CowStr, Event, Options, Parser, Tag};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;

/// The HTML backend for mdzk, implementing [`Renderer`](https://docs.rs/mdbook/0.4.12/mdbook/renderer/trait.Renderer.html).
#[derive(Default)]
pub struct HtmlMdzk;

impl HtmlMdzk {
    fn render_note(
        &self,
        ch: &Chapter,
        ctx: &RenderContext,
        data: &mut BTreeMap<&str, Value>,
        hbs: &Handlebars,
    ) -> Result<()> {
        let path = ch.path.as_ref().unwrap();

        let html = render_markdown(&ch.content);

        // Make map with values for all handlebars keys
        data.insert("content", json!(html));
        data.insert("title", json!(ch.name));
        data.insert("path", json!(path));
        data.insert("path_to_root", json!(utils::path_to_root(path)));

        if let Some(section_number) = &ch.number {
            data.insert("section", json!(section_number));
        }

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
        let css_path = destination.join("css");
        let js_path = destination.join("js");
        let font_path = css_path.join("fonts");

        let book = &ctx.book;
        let style_config: StyleConfig = ctx.config.clone().into();
        let html_config = match ctx.config.get_deserialized_opt("output.html") {
            Ok(Some(html_config)) => html_config,
            Ok(None) => mdbook::config::HtmlConfig::default(),
            Err(e) => {
                warn!("Failed to parse HTML config: {}", e);
                mdbook::config::HtmlConfig::default()
            }
        };

        if destination.exists() {
            // If output directory exists already, remove all content inside
            mdbook::utils::fs::remove_dir_content(destination)
                .context("Unable to remove old output")?;
        } else {
            // If output directory doesn't exist, create it
            fs::create_dir_all(destination)
                .context("Unexpected error when constructing destination path")?;
        }

        // Create data for all notes
        let mut data = BTreeMap::new();
        data.insert(
            "language",
            json!(ctx.config.book.language.clone().unwrap_or_default()),
        );
        data.insert(
            "mdzk_title",
            json!(ctx.config.book.title.clone().unwrap_or_default()),
        );
        data.insert(
            "description",
            json!(ctx.config.book.description.clone().unwrap_or_default()),
        );
        if let Some(ref livereload) = html_config.livereload_url {
            data.insert("livereload", json!(livereload));
        }
        data.insert("fold_enable", json!(html_config.fold.enable));
        data.insert("fold_level", json!(html_config.fold.level));

        // Add chapter titles for use in TOC
        let mut chapters = vec![];
        for item in book.iter() {
            // Create the data to inject in the template
            let mut chapter = BTreeMap::new();

            match *item {
                BookItem::PartTitle(ref title) => {
                    chapter.insert("part", json!(title));
                }
                BookItem::Chapter(ref ch) => {
                    if let Some(ref section) = ch.number {
                        chapter.insert("section", json!(section.to_string()));
                    }

                    chapter.insert(
                        "has_sub_items",
                        json!((!ch.sub_items.is_empty()).to_string()),
                    );

                    chapter.insert("name", json!(ch.name));
                    if let Some(ref path) = ch.path {
                        let p = path
                            .to_str()
                            .with_context(|| "Could not convert path to str")?;
                        chapter.insert("path", json!(p));
                    }
                }
                BookItem::Separator => {
                    chapter.insert("spacer", json!("_spacer_"));
                }
            }

            chapters.push(chapter);
        }
        data.insert("chapters", json!(chapters));

        // Write static files
        if let Some(user_css) = style_config.css_bytes() {
            utils::write_file(&css_path.join("user.css"), &user_css)?;
            data.insert("has_user_css", json!(true));
        }
        utils::write_file(
            &css_path.join("main.css"),
            include_bytes!("theme/css/main.css"),
        )?;
        utils::write_file(
            &css_path.join("chrome.css"),
            include_bytes!("theme/css/chrome.css"),
        )?;
        utils::write_file(
            &css_path.join("variables.css"),
            include_bytes!("theme/css/variables.css"),
        )?;
        utils::write_file(
            &css_path.join("atom-one-light.css"),
            include_bytes!("theme/css/atom-one-light.css"),
        )?;
        utils::write_file(
            &css_path.join("fonts.css"),
            include_bytes!("theme/css/fonts.css"),
        )?;
        utils::write_file(
            &css_path.join("katex.min.css"),
            include_bytes!("theme/css/katex.min.css"),
        )?;
        utils::write_file(
            &js_path.join("katex.min.js"),
            include_bytes!("theme/js/katex.min.js"),
        )?;
        utils::write_file(
            &js_path.join("highlight.min.js"),
            include_bytes!("theme/js/highlight.min.js"),
        )?;
        utils::write_file(&js_path.join("page.js"), include_bytes!("theme/js/page.js"))?;
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
        utils::write_file(
            &destination.join("favicon.png"),
            include_bytes!("theme/favicon.png"),
        )?;
        utils::write_file(
            &destination.join("favicon.svg"),
            include_bytes!("theme/favicon.svg"),
        )?;

        if html_config.search.unwrap_or_default().enable {
            let search_config = mdbook::config::Search::default();
            super::search::create_files(&search_config, destination, book)?;
        }

        // Make Handlebars template
        let mut hbs = Handlebars::new();
        hbs.register_escape_fn(no_escape);
        hbs.register_template_string("index", include_str!("theme/index.hbs"))?;
        hbs.register_helper(
            "toc",
            Box::new(toc::RenderToc {
                no_section_label: html_config.no_section_label,
                fold_enable: html_config.fold.enable,
                fold_level: html_config.fold.level,
            }),
        );

        // Render out each note
        for item in book.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                if !ch.is_draft_chapter() {
                    self.render_note(ch, ctx, &mut data, &hbs)?;
                }
            }
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
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(text, opts);
    let parser = parser.map(|event| match event {
        Event::Start(Tag::Link(link_type, dest, title)) => {
            Event::Start(Tag::Link(link_type, fix_link(dest), title))
        }
        Event::Start(Tag::Image(link_type, dest, title)) => {
            Event::Start(Tag::Image(link_type, fix_link(dest), title))
        }
        _ => event,
    });

    // Push HTML into string
    let mut content = String::new();
    push_html(&mut content, parser);
    content
}

/// Changes links with extension `.md` to `.html` if they don't have schemes. Keep any anchors.
fn fix_link(dest: CowStr) -> CowStr {
    if !(dest.starts_with('#') || crate::utils::has_scheme(&dest)) {
        if let Some((link, anchor)) = dest.split_once(".md") {
            let mut fixed_link = String::new();
            fixed_link.push_str(link);
            fixed_link.push_str(".html");
            if anchor.starts_with('#') {
                fixed_link.push_str(anchor);
            }
            return CowStr::from(fixed_link);
        }
    }
    dest
}

const FONTS: [(&str, &[u8]); 62] = [
    (
        "Inter.var.woff2",
        include_bytes!("theme/css/fonts/Inter.var.woff2"),
    ),
    (
        "Source_Code_Pro_v11_All_Charsets_500.woff2",
        include_bytes!("theme/css/fonts/Source_Code_Pro_v11_All_Charsets_500.woff2"),
    ),
    // woff2
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
    // woff
    (
        "KaTeX_Main-Regular.woff",
        include_bytes!("theme/css/fonts/KaTeX_Main-Regular.woff"),
    ),
    (
        "KaTeX_Main-Italic.woff",
        include_bytes!("theme/css/fonts/KaTeX_Main-Italic.woff"),
    ),
    (
        "KaTeX_Main-Bold.woff",
        include_bytes!("theme/css/fonts/KaTeX_Main-Bold.woff"),
    ),
    (
        "KaTeX_Main-BoldItalic.woff",
        include_bytes!("theme/css/fonts/KaTeX_Main-BoldItalic.woff"),
    ),
    (
        "KaTeX_Math-Italic.woff",
        include_bytes!("theme/css/fonts/KaTeX_Math-Italic.woff"),
    ),
    (
        "KaTeX_Math-BoldItalic.woff",
        include_bytes!("theme/css/fonts/KaTeX_Math-BoldItalic.woff"),
    ),
    (
        "KaTeX_AMS-Regular.woff",
        include_bytes!("theme/css/fonts/KaTeX_AMS-Regular.woff"),
    ),
    (
        "KaTeX_Fraktur-Regular.woff",
        include_bytes!("theme/css/fonts/KaTeX_Fraktur-Regular.woff"),
    ),
    (
        "KaTeX_Fraktur-Bold.woff",
        include_bytes!("theme/css/fonts/KaTeX_Fraktur-Bold.woff"),
    ),
    (
        "KaTeX_SansSerif-Regular.woff",
        include_bytes!("theme/css/fonts/KaTeX_SansSerif-Regular.woff"),
    ),
    (
        "KaTeX_SansSerif-Italic.woff",
        include_bytes!("theme/css/fonts/KaTeX_SansSerif-Italic.woff"),
    ),
    (
        "KaTeX_SansSerif-Bold.woff",
        include_bytes!("theme/css/fonts/KaTeX_SansSerif-Bold.woff"),
    ),
    (
        "KaTeX_Typewriter-Regular.woff",
        include_bytes!("theme/css/fonts/KaTeX_Typewriter-Regular.woff"),
    ),
    (
        "KaTeX_Caligraphic-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Caligraphic-Regular.woff"),
    ),
    (
        "KaTeX_Caligraphic-Bold.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Caligraphic-Bold.woff"),
    ),
    (
        "KaTeX_Script-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Script-Regular.woff"),
    ),
    (
        "KaTeX_Size1-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Size1-Regular.woff"),
    ),
    (
        "KaTeX_Size2-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Size2-Regular.woff"),
    ),
    (
        "KaTeX_Size3-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Size3-Regular.woff"),
    ),
    (
        "KaTeX_Size4-Regular.woff2",
        include_bytes!("theme/css/fonts/KaTeX_Size4-Regular.woff"),
    ),
    // ttf
    (
        "KaTeX_Main-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Main-Regular.ttf"),
    ),
    (
        "KaTeX_Main-Italic.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Main-Italic.ttf"),
    ),
    (
        "KaTeX_Main-Bold.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Main-Bold.ttf"),
    ),
    (
        "KaTeX_Main-BoldItalic.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Main-BoldItalic.ttf"),
    ),
    (
        "KaTeX_Math-Italic.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Math-Italic.ttf"),
    ),
    (
        "KaTeX_Math-BoldItalic.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Math-BoldItalic.ttf"),
    ),
    (
        "KaTeX_AMS-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_AMS-Regular.ttf"),
    ),
    (
        "KaTeX_Fraktur-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Fraktur-Regular.ttf"),
    ),
    (
        "KaTeX_Fraktur-Bold.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Fraktur-Bold.ttf"),
    ),
    (
        "KaTeX_SansSerif-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_SansSerif-Regular.ttf"),
    ),
    (
        "KaTeX_SansSerif-Italic.ttf",
        include_bytes!("theme/css/fonts/KaTeX_SansSerif-Italic.ttf"),
    ),
    (
        "KaTeX_SansSerif-Bold.ttf",
        include_bytes!("theme/css/fonts/KaTeX_SansSerif-Bold.ttf"),
    ),
    (
        "KaTeX_Typewriter-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Typewriter-Regular.ttf"),
    ),
    (
        "KaTeX_Caligraphic-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Caligraphic-Regular.ttf"),
    ),
    (
        "KaTeX_Caligraphic-Bold.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Caligraphic-Bold.ttf"),
    ),
    (
        "KaTeX_Script-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Script-Regular.ttf"),
    ),
    (
        "KaTeX_Size1-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Size1-Regular.ttf"),
    ),
    (
        "KaTeX_Size2-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Size2-Regular.ttf"),
    ),
    (
        "KaTeX_Size3-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Size3-Regular.ttf"),
    ),
    (
        "KaTeX_Size4-Regular.ttf",
        include_bytes!("theme/css/fonts/KaTeX_Size4-Regular.ttf"),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_link() {
        assert_eq!(
            fix_link("folder/subfolder/file.md".into()),
            "folder/subfolder/file.html".into()
        );
        assert_eq!(fix_link("https://mdzk.md".into()), "https://mdzk.md".into());
        assert_eq!(fix_link("file.md#anchor".into()), "file.html#anchor".into());
    }
}
