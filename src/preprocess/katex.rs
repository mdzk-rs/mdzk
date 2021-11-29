use mdbook::book::Chapter;
use pulldown_cmark::{Event, Parser, Tag};

const KATEX_CSS: &str = r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.13.18/dist/katex.min.css" integrity="sha384-zTROYFVGOfTw7JV7KUu8udsvW2fx4lWOsCEDqhBreBwlHI4ioVRtmIvEThzJHGET" crossorigin="anonymous">"#;
const KATEX_JS: &str = r#"<script defer src="https://cdn.jsdelivr.net/npm/katex@0.13.20/dist/katex.min.js" integrity="sha384-ov99pRO2tAc0JuxTVzf63RHHeQTJ0CIawbDZFiFTzB07aqFZwEu2pz4uzqL+5OPG" crossorigin="anonymous"></script>"#;
const RENDER_SCRIPT: &str = r#"<script>
document.addEventListener('DOMContentLoaded', function() {
    var math = document.querySelectorAll("[class^=katex]");
    for (var i = 0; i < math.length; i++) {
      // Convert escaped ampersands before rendering
      var toRender = math[i].textContent.replace(/&amp;/g, '&');
      katex.render(toRender, math[i]);
    }
}, false)
</script>"#;

pub fn run(ch: &mut Chapter) {
    let copy = ch.content.clone();

    let parser = Parser::new(&copy);
    let mut buf = String::new();
    for (event, range) in parser.into_offset_iter() {
        match event {
            // Start math rendering
            Event::Start(Tag::Paragraph)
            | Event::Start(Tag::Heading(_))
            | Event::Start(Tag::List(_))
            | Event::Start(Tag::Item)
            | Event::Start(Tag::BlockQuote) => {
                buf.push_str(&copy[range]);

                if let Some(new_buf) = inline(&buf) {
                    replace(ch, &buf, &new_buf);
                    buf = new_buf;
                }

                if let Some(display) = display(&buf) {
                    replace(ch, &buf, &display);
                }

                safeclear(&mut buf);
            }

            // If math is found in verbatim, replace it back to original
            Event::Code(verbatim) => {
                if let Some(inline) = inline(&verbatim) {
                    replace(ch, &inline, &verbatim);
                }
                if let Some(display) = display(&verbatim) {
                    replace(ch, &display, &verbatim);
                }
                buf.clear();
            }

            _ => {}
        }
    }

    ch.content.push_str(&format!("\n\n{}", KATEX_CSS));
    ch.content.push_str(&format!("\n\n{}", KATEX_JS));
    ch.content.push_str(&format!("\n\n{}", RENDER_SCRIPT));
}

fn inline(text: &str) -> Option<String> {
    let mut out = text.to_string();
    let mut splits = text.split('$');
    let mut escaped = match splits.next() {
        Some(first_split) => first_split.ends_with('\\'),
        None => return None, // No dollars, do early return
    };

    for split in splits {
        if split.is_empty() {
            escaped = true; // Two $s after each other. Do not consider inline.
            continue;
        } else if escaped
            || split.starts_with(' ')
            || split.ends_with(' ')
            || split.chars().next().unwrap().is_numeric()
        {
            if split.ends_with('\\') {
                escaped = true;
            } else {
                escaped = false;
            }
        } else if !text.ends_with(split) {
            // Hacky way of checking if this is the last split
            out = out.replacen(
                &format!("${}$", split),
                &format!("<span class=\"katex-inline\">{}</span>", split),
                1,
            );
        }
    }
    if out != text {
        Some(out)
    } else {
        None
    }
}

fn display(text: &str) -> Option<String> {
    let mut out = text.to_string();
    let splits = text.split("$$");

    for split in splits.skip(1).step_by(2) {
        out = out.replacen(
            &format!("$${}$$", split),
            &format!("<span class=\"katex-display\">{}</span>", split),
            1,
        );
    }

    if out != text {
        Some(out)
    } else {
        None
    }
}

fn fix(text: impl std::ops::Deref<Target = str>) -> String {
    text.replace(r#" \ "#, r#" \\ "#)
}

fn replace(ch: &mut Chapter, old: &str, new: &str) {
    ch.content = ch.content.replacen(&fix(old), &fix(new), 1);
}

fn safeclear(buf: &mut String) {
    let mut cache = buf.clone();
    let mut parity = true;
    buf.clear();

    while let Some(mut c) = cache.pop() {
        if c == '\\' {
            if !parity {
                buf.push(c);
            }
            if let Some(d) = cache.pop() {
                c = d;
            }
        } else if c == '$' {
            parity = !parity;
            buf.clear();
        }
        if !parity {
            buf.push(c);
        }
    }
}
