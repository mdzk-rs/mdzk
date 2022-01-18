use chrono::prelude::*;
use gray_matter::{
    engine::{Engine, TOML, YAML},
    Matter, ParsedEntityStruct,
};
use mdbook::book::Chapter;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct FrontMatter {
    title: Option<String>,
    date: Option<String>,
    draft: Option<bool>,
}

pub fn run(ch: &mut Chapter) {
    let content = ch.content.clone();

    let mut handle_front_matter = |parsed: ParsedEntityStruct<FrontMatter>| {
        ch.content = parsed.content;
        let config = parsed.data;

        // Set as draft chapter
        if let Some(true) = config.draft {
            ch.path = None
        }

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
            } else if let Ok(naive_datetime) =
                NaiveDateTime::parse_from_str(&datestring, "%Y-%m-%dT%H:%M")
            {
                // Timezone and seconds are not specified.
                let datetime = Local.from_local_datetime(&naive_datetime).unwrap();
                datetime.format(format).to_string()
            } else {
                // Could not parse date
                "Unknown date format".to_string()
            };

            ch.content.push_str("\n\n<div class=\"datetime\">");
            ch.content.push_str(&formatted_date);
            ch.content.push_str("</div>\n");
        }
    };

    if let Some(parsed) = parse_matter::<YAML>(&content) {
        handle_front_matter(parsed);
    } else if let Some(parsed) = parse_matter::<TOML>(&content) {
        handle_front_matter(parsed);
    }
}

fn parse_matter<T: Engine>(content: &str) -> Option<ParsedEntityStruct<FrontMatter>> {
    let matter: Matter<T> = Matter::new();
    matter.parse_with_struct::<FrontMatter>(content)
}
