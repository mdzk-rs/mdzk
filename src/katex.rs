use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{Event, Tag};

const KATEX_CSS: &str = r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.13.18/dist/katex.min.css" integrity="sha384-zTROYFVGOfTw7JV7KUu8udsvW2fx4lWOsCEDqhBreBwlHI4ioVRtmIvEThzJHGET" crossorigin="anonymous">"#;

pub struct Katex;

impl Preprocessor for Katex {
    fn name(&self) -> &str {
        "katex"
    }

    fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                let copy = ch.content.clone();
                let parser = pulldown_cmark::Parser::new(&copy);
                let mut buf = String::new();
                let mut in_paragraph = false;
                for event in parser {
                    match event {
                        Event::Start(Tag::Paragraph) => {
                            in_paragraph = true;
                        }
                        Event::Text(text) => {
                            if in_paragraph {
                                buf.push_str(&text);
                            }
                        }
                        Event::End(Tag::Paragraph) => {
                            in_paragraph = false;
                            if let Some(math) = is_display(&buf) {
                                render_display(math, &buf, ch);
                            /* } else if let Some(maths) = has_inline(&buf) {
 paprintln!("{:?}", maths); */
                            }
                            buf.clear();
                        }
                        _ => {}
                    }
                }
                ch.content.push_str(&format!("\n\n{}", KATEX_CSS));
            }
        });
        Ok(book)
    }
}

/* fn has_inline(paragraph: &str) -> Option<Vec<&str>> {
    let mut maths = Vec::new();
    let mut buf = paragraph.clone();
    loop {
        match buf.find(" $") {
            Some(i) => {
                maths.push(&paragraph[i..]);
                buf = &buf[i..];
            }
            None => break
        }
    }
    Some(maths)
} */

fn is_display(paragraph: &str) -> Option<&str> {
    if let Some(math) = paragraph.strip_prefix("$$") {
        if let Some(math) = math.strip_suffix("$$") {
            return Some(math)
        }
    }
    None
}

/// Checks if paragraph is a math block, and renders this math to the chapter content if it is.
fn render_display(math: &str, paragraph: &str, ch: &mut Chapter) {
    // `\`s are really `\\`s, but they are escaped by pulldown-cmark. we want them present for
    // things like matrices, so replace them back.
    let math = math.replace(r#" \ "#, r#" \\ "#);

    let opts = katex::Opts::builder()
        .display_mode(true)
        .build()
        .unwrap(); // No idea why they return a Result. build should never fail.

    match katex::render_with_opts(&math, opts) {
        Ok(math) => {
            ch.content = ch.content.replacen(
                paragraph,
                &math,
                1
            );
        },
        Err(e) => {
            error!("Failed to render: {}\n    {}", math, e);
        }
    };
}

#[cfg(test)]
mod tests {
    // NOTE: These tests are redundant, since katex itself is pretty well tested. However, they
    // serve as peace of mind for the time being, and can be evolved into tests that check our
    // delimiter parsing as well.
    #[test]
    fn should_render() {
        let examples = vec![
            r#" \text{BIC} = \log N\cdot d - 2\ \text{loglik} ,"#,
            r#"S(\beta,\lambda) = \text{RSS}(\beta) + \lambda \sum_{j=1}^p\beta_j^2 = \sum_{i=1}^N (y_i - \hat y_i)^2 + \lambda \sum_{j=1}^p\beta_j^2"#,
            r#"Here is escaped \$"#,
        ];
        for ex in examples {
            katex::render(ex).unwrap();
        };
    }

    #[test]
    #[should_panic]
    fn shouldnt_render() {
        let examples = vec![
            r#"$ weird stuff \ matrix && entries"#,
            r#"\wrongsin (x)"#,
            r#"Ending with extra $"#,
        ];
        for ex in examples {
            katex::render(ex).unwrap();
        };
    }
}
