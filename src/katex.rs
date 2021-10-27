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
                            if let Ok(_) = handle_display(&mut buf, ch) {
                                in_paragraph = false;
                                continue
                            }
                            buf.clear();
                            in_paragraph = false;
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

fn handle_display(buf: &mut String, ch: &mut Chapter) -> Result<(), ()> {
    if let Some(text) = buf.strip_prefix("$$") {
        if let Some(text) = text.strip_suffix("$$") {
            let opts = katex::OptsBuilder::default()
                .display_mode(true)
                .build()
                .unwrap(); // No idea why they return a Result. build should never fail.

            match katex::render_with_opts(text, opts) {
                Ok(math) => {
                    ch.content = ch.content.replacen(
                        buf.as_str(),
                        &math,
                        1
                    );
                },
                Err(e) => {
                    error!("Failed to render: {}\n    {}", text, e);
                }
            };
            buf.clear();
            return Ok(())
        };
    }
    Err(())
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
