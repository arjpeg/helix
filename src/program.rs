use slotmap::{DefaultKey, SlotMap};

use crate::{
    error::{Error, Result},
    interpreter::Interpreter,
    lexer::Lexer,
    parser::Parser,
    token::{ASTNode, Token},
    value::Value,
};

/// A source file that contains some source code, and potentially
/// parsed ast.
pub struct Source {
    /// The name of the source.
    pub name: String,
    /// The content of the source.
    pub content: String,
}

/// A program is a collection of (potentially parsed) source files,
/// and an interpreter for the program
pub struct Program {
    sources: SlotMap<DefaultKey, Source>,
}

impl Source {
    /// Lexes the source file.
    fn lex(&self, key: DefaultKey) -> Result<Vec<Token>> {
        Lexer::new(key, self).tokenize()
    }

    /// Lexes and parses the source file.
    pub fn parse(&self, key: DefaultKey) -> Result<ASTNode> {
        let tokens = self.lex(key)?;
        Parser::new(tokens).parse()
    }
}

impl Program {
    pub fn new() -> Self {
        Self {
            sources: SlotMap::new(),
        }
    }

    /// Register a new source file with the program.
    pub fn add_source(&mut self, name: String, content: String) -> DefaultKey {
        self.sources.insert(Source { name, content })
    }

    /// Excecutes the given source file by key.
    pub fn run(&mut self, key: DefaultKey) -> Result<Value> {
        let source = self.sources.get(key).expect("entry point does not exist");
        let ast = source.parse(key)?;

        let mut interpreter = Interpreter::new();

        interpreter.run(ast)
    }

    /// Pretty prints an error
    pub fn pretty_print_error(&self, Error { span, kind }: Error) {
        use owo_colors::OwoColorize;

        let source = &self
            .sources
            .get(span.source)
            .expect("registered source should be in sources");

        let line_start = match source.content[..span.start].rfind('\n') {
            Some(start) => start + 1,
            None => 0,
        };

        let line_end = source.content[span.end..]
            .find('\n')
            .map(|end| span.end + end)
            .unwrap_or(source.content.len());

        let line_number = source.content[..span.start].lines().count();

        let at = format!("{} line {}:", source.name, line_number);

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
