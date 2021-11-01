use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{Event, Tag, Parser};

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

pub struct Katex;

impl Preprocessor for Katex {
    fn name(&self) -> &str {
        "katex"
    }

    fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                let copy = ch.content.clone();
                let parser = Parser::new(&copy);
                let mut buf = String::new();
                let mut render_math = false;
                for event in parser {
                    match event {
                        Event::Start(Tag::Paragraph)
                        | Event::Start(Tag::Heading(_))
                        | Event::Start(Tag::List(_))
                        | Event::Start(Tag::Item)
                        | Event::Start(Tag::Table(_))
                        | Event::Start(Tag::BlockQuote) => {
                            render_math = true;
                        }
                        Event::Text(text) => {
                            if render_math {
                                buf.push_str(&text);
                                if let Some(new_buf) = inline(&buf) {
                                    replace(ch, &buf, &new_buf);
                                    buf.clear()
                                }
                            }
                        }
                        Event::Start(Tag::Emphasis)
                        | Event::End(Tag::Emphasis) => if render_math {
                            buf.push_str("*")
                        }
                        Event::Start(Tag::Strong)
                        | Event::End(Tag::Strong) => if render_math {
                            buf.push_str("**")
                        }
                        Event::Code(_) => buf.clear(),
                        Event::End(Tag::Paragraph)
                        | Event::End(Tag::Heading(_))
                        | Event::End(Tag::List(_))
                        | Event::End(Tag::Item)
                        | Event::End(Tag::Table(_))
                        | Event::End(Tag::BlockQuote) => {
                            if let Some(display) = display(&buf) {
                                replace(ch, &buf, &display);
                            } else if let Some(new_text) = inline(&buf) {
                                replace(ch, &buf, &new_text);
                            }
                            buf.clear();
                            render_math = false;
                        }
                        _ => {}
                    }
                }
                ch.content.push_str(&format!("\n\n{}", KATEX_CSS));
                ch.content.push_str(&format!("\n\n{}", KATEX_JS));
                ch.content.push_str(&format!("\n\n{}", RENDER_SCRIPT));
            }
        });
        Ok(book)
    }
}

fn inline(text: &str) -> Option<String> {
    let mut out = text.to_string();
    let mut splits = text.split("$");
    let mut escaped = match splits.next() {
        Some(first_split) => first_split.ends_with("\\"),
        None => return None // No dollars, do early return
    };

    for split in splits {
        if split.is_empty() {
            escaped = true; // Two $s after each other. Do not consider inline.
            continue
        } else if escaped
        || split.starts_with(" ")
        || split.ends_with(" ")
        || split.chars().next().unwrap().is_numeric()
        {
            if split.ends_with("\\") {
                escaped = true;
            } else {
                escaped = false;
            }
        } else if !text.ends_with(split) { // Hacky way of checking if this is the last split
            out = out.replacen(
                &format!("${}$", split),
                &format!("<span class=\"katex-inline\">{}</span>", split),
                1
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
    if let Some(math) = text.strip_prefix("$$") {
        if let Some(math) = math.strip_suffix("$$") {
            return Some(text.replacen(
                text,
                &format!("<div class=\"katex-display\">{}</div>", math),
                1
            ))
        }
    }
    None
}

fn fix(text: impl std::ops::Deref<Target = str>) -> String {
    text.replace(r#" \ "#, r#" \\ "#)
}

fn replace(ch: &mut Chapter, old: &str, new: &str) {
    ch.content = ch.content.replacen(&fix(old), &fix(new), 1);
}
