use gray_matter::engine::yaml::YAML;
use gray_matter::matter::Matter;
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use regex::Regex;
use serde::Deserialize;
use chrono::prelude::*;

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
                    title: Option<String>,
                    date: Option<String>
                }

                let yaml_matter: Matter<YAML> = Matter::new();
                let result = yaml_matter.matter(ch.content.clone());

                if let Ok(parsed) = result.data.deserialize::<Config>() {
                    // Remove metadata from content
                    let re = Regex::new(YAML_RE).unwrap();
                    ch.content = re.replace_all(&ch.content, "").to_string();

                    // Set title
                    if let Some(title) = parsed.title {
                        ch.name = title;
                    }

                    // Set date
                    if let Some(datestring) = parsed.date {
                        let format = "%H:%M %A %d %B %Y";

                        // Parse the supplied datestring (NOTE: Very verbose. There should be a more elegant way)
                        let formatted_date = if let Ok(datetime) = datestring.parse::<DateTime<FixedOffset>>() {
                            // Timezone is specified in datestring. Use that timezone
                            datetime.format(format).to_string()
                        } else if let Ok(naive_datetime) = datestring.parse::<NaiveDateTime>() {
                            // Timezone is not specified, use local timezone
                            let datetime = Local.from_local_datetime(&naive_datetime).unwrap();
                            datetime.format(format).to_string()
                        } else if let Ok(naive_datetime) = NaiveDateTime::parse_from_str(&datestring, "%Y-%m-%dT%H:%M") {
                            // Timezone and seconds are not specified.
                            let datetime = Local.from_local_datetime(&naive_datetime).unwrap();
                            datetime.format(format).to_string()
                        } else {
                            // Could not parse date
                            "Unknown date format".to_string()
                        };

                        ch.content = format!(
                            "\n\n<div class=\"datetime\" style=\"text-align: center; color: gray; font-style: italic; font-size: 90%;\">{}</div>\n\n{}",
                            &formatted_date,
                            ch.content
                        );
                    }
                }
            }
        });

        Ok(book)
    }
}
