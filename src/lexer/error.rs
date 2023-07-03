use super::span::Span;

/// Represents an error that occurred during lexing.
#[derive(Debug)]
pub enum LexerError {
    /// When a number contains more than one dot.
    TooManyDots { range: Span },

    /// When the input contains an unknown symbol.
    UnknownSymbol { range: Span },

    /// When the input contains an unknown command.
    UnknownCommand { range: Span },
}
