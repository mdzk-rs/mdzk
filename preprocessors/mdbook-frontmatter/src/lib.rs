use chrono::prelude::*;
use gray_matter::{
    engine::{Engine, TOML, YAML},
    Matter,
};
use mdbook::{
    book::{Book, BookItem},
    errors::*,
    preprocess::{Preprocessor, PreprocessorContext},
};
use lazy_regex::regex;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    title: Option<String>,
    date: Option<String>,
}

pub struct FrontMatter;

impl Preprocessor for FrontMatter {
    fn name(&self) -> &str {
        "front_matter"
    }

    fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let re = regex!(r"(?sm)^\s*---(.*)---\s*$");
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                let content = ch.content.clone();

                let mut handle_front_matter = |config: Config| {
                    // Remove metadata from content
                    ch.content = re.replace_all(&ch.content, "").to_string();

                    // Set title
                    if let Some(title) = config.title {
                        ch.name = title;
                    }

                    // Set date
                    if let Some(datestring) = config.date {
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
                            "{}\n\n<div class=\"datetime\" style=\"text-align: center; color: gray; font-style: italic; font-size: 90%;\">{}</div>\n\n",
                            ch.content,
                            &formatted_date,
                        );
                    }
                };

                if let Some(config) = parse_matter::<YAML>(&content) {
                    handle_front_matter(config);
                } else if let Some(config) = parse_matter::<TOML>(&content) {
                    handle_front_matter(config);
                }
            }
        });

        Ok(book)
    }
}

fn parse_matter<T: Engine>(content: &str) -> Option<Config> {
    let matter: Matter<T> = Matter::new();
    Some(matter.parse_with_struct::<Config>(content)?.data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml() {
        let yaml_parsable_content = r#"---
title: Title
date: 2001-01-01T01:01
---

Markdown content."#;

        let yaml_unparsable_content = r#"---
title = "Title"
wrong_key: 42
---

Markdown content."#;

        let parsed = parse_matter::<YAML>(&yaml_parsable_content).unwrap();
        assert_eq!(parsed.title.unwrap(), "Title");
        assert!(parse_matter::<YAML>(&yaml_unparsable_content).is_none())
    }

    #[test]
    fn test_parse_toml() {
        let toml_parsable_content = r#"---
title = "Title"
date = "2001-01-01T01:01"
---

Markdown content."#;

        let toml_unparsable_content = r#"---
title: Title
wrong_key = 42
---

Markdown content."#;

        let parsed = parse_matter::<TOML>(&toml_parsable_content).unwrap();
        assert_eq!(parsed.title.unwrap(), "Title");
        assert!(parse_matter::<TOML>(&toml_unparsable_content).is_none())
    }
}
