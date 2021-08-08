use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

pub struct FrontMatter;

impl Preprocessor for FrontMatter {
    fn name(&self) -> &str {
        "front_matter"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        Ok(book)
    }
}
