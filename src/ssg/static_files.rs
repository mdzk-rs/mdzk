use super::config::Config;
use crate::{error::Result, utils::fs::write_file};
use std::path::Path;

pub(crate) fn write_static_files(destination: impl AsRef<Path>, config: &Config) -> Result<()> {
    for (path, bytes) in STATIC_FILES {
        write_file(destination.as_ref().join(path), bytes)?;
    }

    let mdzk_css = include_bytes!("./css/mdzk.css");
    let mdzk_css_path = destination.as_ref().join("css/mdzk.css");

    if let Some(style) = &config.style {
        if let Some(custom_css) = style.css_bytes() {
            let bytes = [mdzk_css, custom_css.as_slice()].concat();
            return write_file(mdzk_css_path, &bytes);
        }
    }
    write_file(mdzk_css_path, mdzk_css)
}

const STATIC_FILES: [(&str, &[u8]); 30] = [
    // CSS
    ("./css/katex.min.css", include_bytes!("./css/katex.min.css")),
    ("./css/prism.min.css", include_bytes!("./css/prism.min.css")),
    // ("./css/mdzk.css", include_bytes!("./css/mdzk.css")),
    (
        "./css/normalize.min.css",
        include_bytes!("./css/normalize.min.css"),
    ),
    // JavaScript
    (
        "./js/auto-render.min.js",
        include_bytes!("./js/auto-render.min.js"),
    ),
    ("./js/prism.min.js", include_bytes!("./js/prism.min.js")),
    ("./js/katex.min.js", include_bytes!("./js/katex.min.js")),
    // Fonts
    (
        "./css/fonts/SourceSans3VF-Roman.ttf.woff2",
        include_bytes!("./fonts/SourceSans3VF-Roman.ttf.woff2"),
    ),
    (
        "./css/fonts/SourceSans3VF-Italic.ttf.woff2",
        include_bytes!("./fonts/SourceSans3VF-Italic.ttf.woff2"),
    ),
    (
        "./css/fonts/SourceSerif4VF-Roman.ttf.woff2",
        include_bytes!("./fonts/SourceSerif4VF-Roman.ttf.woff2"),
    ),
    (
        "./css/fonts/SourceSerif4VF-Italic.ttf.woff2",
        include_bytes!("./fonts/SourceSerif4VF-Italic.ttf.woff2"),
    ),
    (
        "./css/fonts/KaTeX_AMS-Regular.woff2",
        include_bytes!("./fonts/KaTeX_AMS-Regular.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Caligraphic-Bold.woff2",
        include_bytes!("./fonts/KaTeX_Caligraphic-Bold.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Caligraphic-Regular.woff2",
        include_bytes!("./fonts/KaTeX_Caligraphic-Regular.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Fraktur-Bold.woff2",
        include_bytes!("./fonts/KaTeX_Fraktur-Bold.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Fraktur-Regular.woff2",
        include_bytes!("./fonts/KaTeX_Fraktur-Regular.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Main-Bold.woff2",
        include_bytes!("./fonts/KaTeX_Main-Bold.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Main-BoldItalic.woff2",
        include_bytes!("./fonts/KaTeX_Main-BoldItalic.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Main-Italic.woff2",
        include_bytes!("./fonts/KaTeX_Main-Italic.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Main-Regular.woff2",
        include_bytes!("./fonts/KaTeX_Main-Regular.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Math-BoldItalic.woff2",
        include_bytes!("./fonts/KaTeX_Math-BoldItalic.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Math-Italic.woff2",
        include_bytes!("./fonts/KaTeX_Math-Italic.woff2"),
    ),
    (
        "./css/fonts/KaTeX_SansSerif-Bold.woff2",
        include_bytes!("./fonts/KaTeX_SansSerif-Bold.woff2"),
    ),
    (
        "./css/fonts/KaTeX_SansSerif-Italic.woff2",
        include_bytes!("./fonts/KaTeX_SansSerif-Italic.woff2"),
    ),
    (
        "./css/fonts/KaTeX_SansSerif-Regular.woff2",
        include_bytes!("./fonts/KaTeX_SansSerif-Regular.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Script-Regular.woff2",
        include_bytes!("./fonts/KaTeX_Script-Regular.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Size1-Regular.woff2",
        include_bytes!("./fonts/KaTeX_Size1-Regular.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Size2-Regular.woff2",
        include_bytes!("./fonts/KaTeX_Size2-Regular.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Size3-Regular.woff2",
        include_bytes!("./fonts/KaTeX_Size3-Regular.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Size4-Regular.woff2",
        include_bytes!("./fonts/KaTeX_Size4-Regular.woff2"),
    ),
    (
        "./css/fonts/KaTeX_Typewriter-Regular.woff2",
        include_bytes!("./fonts/KaTeX_Typewriter-Regular.woff2"),
    ),
];
