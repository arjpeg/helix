use crate::{error::Error, lexer::Lexer, token::Token};

/// A source file that contains some source code, and potentially
/// parsed ast
pub struct Source {
    /// The ID of the source (index into when it was registered)
    pub index: usize,

    /// The name of the source
    pub name: String,
    /// The content of the source
    pub content: String,
}

/// A program is a collection of (potentially parsed) source files,
/// and an interpreter for the program
pub struct Program {
    sources: Vec<Source>,
}

impl Source {
    /// Lexes the source file
    fn lex(&self) -> Result<Vec<Token>, Error> {
        Lexer::new(self).tokenize()
    }

    /// Lexes and parses the source file
    pub fn parse(&self) -> Result<(), Error> {
        dbg!(self.lex()?);

        Ok(())
    }
}

impl Program {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// Register a new source file with the program
    pub fn register_source(&mut self, name: String, content: String) {
        self.sources.push(Source {
            name,
            content,
            index: self.sources.len(),
        });
    }

    /// Runs the program from the given entry point
    pub fn run(&mut self, entry: usize) -> Result<(), Error> {
        let source = self.sources.get(entry).expect("entry point does not exist");

        source.parse()?;

        Ok(())
    }

    /// Pretty prints an error
    pub fn pretty_print_error(&self, Error { span, kind }: Error) {
        use owo_colors::OwoColorize;

        let source = &self.sources[span.source];

        let line_start = source.content[..span.start].rfind('\n').unwrap_or(0) + 1;
        let line_end = source.content[span.end..]
            .find('\n')
            .unwrap_or(source.content.len());

        let line_number = source.content[..span.start].lines().count();

        let at = format!("at {} line {}:", source.name, line_number);

        let arrow_offset = 2 + at.len() + span.start - line_start;

        eprintln!("{}: {}", "Error".red().bold(), kind.bold());
        eprintln!();

        eprint!("  {}", at.black());

        eprintln!("  {}", &source.content[line_start..line_end]);
        eprintln!(
            "  {}{}",
            " ".repeat(arrow_offset),
            "^".repeat(span.end - span.start)
        );
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
