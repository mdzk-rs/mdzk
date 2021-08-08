use gray_matter::engine::yaml::YAML;
use gray_matter::matter::Matter;
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use regex::Regex;
use serde::Deserialize;

pub struct FrontMatter;

const YAML_RE: &str = r"(?sm)^\s*---(.*)---\s*$";

impl Preprocessor for FrontMatter {
    fn name(&self) -> &str {
        "front_matter"
    }

    fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                // TODO: The deserializer is strict, which means all fields have to be present for
                // the YAML to be parsed. Since we only have one field now, this is fine, but when
                // adding more fields, we need to find a workaround for this.
                #[derive(Deserialize, Debug)]
                struct Config {
                    title: String,
                }

                let yaml_matter: Matter<YAML> = Matter::new();
                let result = yaml_matter.matter(ch.content.clone());

                if let Ok(parsed) = result.data.deserialize::<Config>() {
                    let re = Regex::new(YAML_RE).unwrap();
                    ch.name = parsed.title;
                    ch.content = re.replace_all(&ch.content, "").to_string();
                }
            }
        });

        Ok(book)
    }
}
