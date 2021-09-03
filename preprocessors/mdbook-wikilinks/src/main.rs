use mdbook_wikilinks::{handle_preprocessing, WikiLinks};

use clap::{App, Arg, SubCommand};
use mdbook::errors::Error;

pub fn make_app() -> App<'static, 'static> {
    App::new("wikilink")
        .about("A mdbook preprocessor which does autotitle")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() -> Result<(), Error> {
    let matches = make_app().get_matches();

    if let Some(_sub_args) = matches.subcommand_matches("supports") {
        Ok(())
    } else {
        handle_preprocessing(WikiLinks)
    }
}
