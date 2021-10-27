use mdbook_wikilinks::WikiLinks;

use clap::{App, Arg, SubCommand};
use mdbook::{
    preprocess::{CmdPreprocessor, Preprocessor},
    errors::Error,
};
use std::io;

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

pub fn handle_preprocessing(pre: impl Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
