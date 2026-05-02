pub mod error;
pub mod token;

use unicode_xid::UnicodeXID;

use crate::{
    lexer::{
        error::{Result, TokenizationError},
        token::{CharTokenExt, Grouping, Keyword, OpKind, Token},
    },
    source::{Source, Span, Spanned},
};

/// Converts source code into a stream of [`Token`]s.
pub struct Tokenizer {
    /// The source code being tokeinzed.
    source: Source,
    /// The current byte position within the `source`.
    cursor: usize,

    /// `true` if the eof token has already been emitted, `false` otherwise.
    eof_emitted: bool,
}

impl Tokenizer {
    /// Creates a new [`Tokenizer`].
    pub fn new(source: Source) -> Self {
        Self {
            source,
            cursor: 0,
            eof_emitted: false,
        }
    }

    /// Returns the remaining characters to be tokenized.
    fn remaining(&self) -> &str {
        &self.source.content[self.cursor..]
    }

    /// Peeks at the next character without advancing the cursor.
    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    /// Advances the cursor forward by one character, returning it if not at eof.
    fn advance(&mut self) -> Option<Spanned<char>> {
        let c = self.peek()?;

        let start = self.cursor;
        self.cursor += c.len_utf8();
        let end = self.cursor;

        Some(Spanned::wrap(c, Span::new(self.source, start..end)))
    }

    /// Advances the cursor past all whitespace characters.
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.advance();
        }
    }

    /// Advances the cursor while the predicate is met, returning the input and span consumed.
    fn advance_while(&mut self, predicate: impl Fn(&char) -> bool) -> Span {
        let start = self.cursor;

        while matches!(self.peek(), Some(c) if predicate(&c)) {
            self.advance();
        }

        Span::new(self.source, start..self.cursor)
    }

    /// Tokenizes a single symbol (keyword or identifier).
    fn next_symbol(&mut self) -> Spanned<Token> {
        let span = self.advance_while(|c| c.is_xid_continue());
        let symbol = span.text();

        let token = Keyword::try_from(symbol)
            .map(Token::Keyword)
            .unwrap_or(Token::Symbol(symbol));

        Spanned::wrap(token, span)
    }

    /// Tokenizes a single grouping symbol.
    fn next_grouping(&mut self) -> Spanned<Token> {
        self.advance()
            .unwrap()
            .map(|c| Grouping::try_from(c).unwrap().into())
    }

    /// Tokenizes a single operator (may span multiple characters).
    fn next_operator(&mut self) -> Spanned<Token> {
        let start = self.cursor;
        let operator = OpKind::try_from((self.advance().unwrap().value, self.peek())).unwrap();

        if operator.len() == 2 {
            self.advance();
        }

        let span = Span::new(self.source, start..self.cursor);

        Spanned::wrap(Token::Operator(operator), span)
    }

    /// Tokenizes a single integer literal.
    fn next_integer(&mut self) -> Result<Spanned<Token>> {
        let span = self.advance_while(|c| c.is_ascii_digit());

        if matches!(self.peek(), Some(c) if c.is_xid_continue()) {
            let error_span = self.advance_while(|c| c.is_xid_continue());
            let full_span = Span::new(self.source, span.start..error_span.end);

            return Err(Spanned::wrap(
                TokenizationError::InvalidIntegerLiteral(full_span.text()),
                full_span,
            ));
        }

        let literal = span.text();

        literal
            .parse()
            .map(|n| Spanned::wrap(Token::Int(n), span))
            .map_err(|_| Spanned::wrap(TokenizationError::InvalidIntegerLiteral(literal), span))
    }

    /// Tokenizes a single line string literal.
    fn next_string(&mut self) -> Result<Spanned<Token>> {
        let opening = self.advance().unwrap();
        let mut buf = String::new();
        let mut last_span = opening.span;

        loop {
            let Some(c) = self.advance() else {
                return Err(Spanned::wrap(
                    TokenizationError::UnterminatedStringLiteral,
                    Span::merge(opening.span, last_span),
                ));
            };

            last_span = c.span;

            match c.value {
                '\n' => {
                    return Err(Spanned::wrap(
                        TokenizationError::UnterminatedStringLiteral,
                        Span::merge(opening.span, last_span),
                    ));
                }
                '\\' => {
                    let Some(next) = self.advance() else {
                        return Err(c.map(|_| TokenizationError::InvalidEscapeSequence("\\(EOF)")));
                    };
                    last_span = next.span;
                    match next.value {
                        'n' => buf.push('\n'),
                        't' => buf.push('\t'),
                        '\\' => buf.push('\\'),
                        _ => {
                            let span = Span::merge(c.span, next.span);
                            return Err(Spanned::wrap(
                                TokenizationError::InvalidEscapeSequence(span.text()),
                                span,
                            ));
                        }
                    }
                }

                ch if ch == opening.value => {
                    let span = Span::merge(opening.span, last_span);
                    return Ok(Spanned::wrap(
                        Token::String(Box::leak(buf.into_boxed_str())),
                        span,
                    ));
                }

                ch => buf.push(ch),
            }
        }
    }
}

impl Iterator for Tokenizer {
    type Item = Result<Spanned<Token>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        if self.eof_emitted {
            return None;
        }

        let Some(c) = self.peek() else {
            self.eof_emitted = true;

            return Some(Ok(Spanned::wrap(
                Token::Eof,
                Span::new(self.source, self.cursor.saturating_sub(1)..self.cursor),
            )));
        };

        Some(match c {
            c if c == '_' || c.is_xid_start() => Ok(self.next_symbol()),

            c if c.is_ascii_digit() => self.next_integer(),

            c if c.is_operator_start() => Ok(self.next_operator()),

            c if c.is_grouping() => Ok(self.next_grouping()),

            ';' => Ok(self.advance().unwrap().map(|_| Token::Semicolon)),

            ',' => Ok(self.advance().unwrap().map(|_| Token::Comma)),

            '\'' | '\"' => self.next_string(),

            _ => {
                let span = self.advance_while(|c| !c.is_whitespace());

                Err(Spanned::wrap(
                    TokenizationError::UnknownSymbol(span.text()),
                    span,
                ))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::error::TokenizationError;
    use crate::lexer::token::{Grouping, OpKind, Token};
    use std::path::Path;

    fn make_source(content: &'static str) -> Source {
        Source {
            content,
            path: Path::new("test.hx"),
        }
    }

    fn tokenize(content: &'static str) -> Vec<Result<Spanned<Token>>> {
        Tokenizer::new(make_source(content))
            .filter(|t| {
                !matches!(
                    t,
                    Ok(Spanned {
                        value: Token::Eof,
                        ..
                    })
                )
            })
            .collect()
    }

    fn tokens_ok(content: &'static str) -> Vec<Token> {
        tokenize(content)
            .into_iter()
            .map(|r| r.unwrap().value)
            .collect()
    }

    #[test]
    fn test_integer() {
        assert_eq!(tokens_ok("42"), vec![Token::Int(42)]);
    }

    #[test]
    fn test_integer_adjacent_operator() {
        assert_eq!(
            tokens_ok("123+23"),
            vec![
                Token::Int(123),
                Token::Operator(OpKind::Plus),
                Token::Int(23),
            ]
        );
    }

    #[test]
    fn test_integer_with_trailing_alpha_is_error() {
        let results = tokenize("123abc");
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0].as_ref().unwrap_err().value,
            TokenizationError::InvalidIntegerLiteral("123abc")
        ));
    }

    #[test]
    fn test_integer_mixed_in_expression() {
        let results = tokenize("1+2abc");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].as_ref().unwrap().value, Token::Int(1));
        assert_eq!(
            results[1].as_ref().unwrap().value,
            Token::Operator(OpKind::Plus)
        );
        assert!(results[2].is_err());
    }

    #[test]
    fn test_symbol() {
        assert_eq!(tokens_ok("foo"), vec![Token::Symbol("foo")]);
    }

    #[test]
    fn test_keyword() {
        assert_eq!(
            tokens_ok("hello and or but not"),
            vec![
                Token::Symbol("hello"),
                Token::Keyword(Keyword::And),
                Token::Keyword(Keyword::Or),
                Token::Symbol("but"),
                Token::Symbol("not"),
            ]
        );
    }

    #[test]
    fn test_symbol_with_underscore_start() {
        assert_eq!(tokens_ok("_foo"), vec![Token::Symbol("_foo")]);
    }

    #[test]
    fn test_symbol_with_digits() {
        assert_eq!(tokens_ok("foo123"), vec![Token::Symbol("foo123")]);
    }

    #[test]
    fn test_single_char_operators() {
        assert_eq!(tokens_ok("+"), vec![Token::Operator(OpKind::Plus)]);
        assert_eq!(tokens_ok("-"), vec![Token::Operator(OpKind::Minus)]);
        assert_eq!(tokens_ok("*"), vec![Token::Operator(OpKind::Star)]);
        assert_eq!(tokens_ok("/"), vec![Token::Operator(OpKind::Slash)]);
        assert_eq!(tokens_ok(">"), vec![Token::Operator(OpKind::GreaterThan)]);
        assert_eq!(tokens_ok("<"), vec![Token::Operator(OpKind::LessThan)]);
    }

    #[test]
    fn test_double_char_operators() {
        assert_eq!(tokens_ok("!="), vec![Token::Operator(OpKind::NotEquals)]);
        assert_eq!(tokens_ok("=="), vec![Token::Operator(OpKind::Equals)]);
        assert_eq!(
            tokens_ok(">="),
            vec![Token::Operator(OpKind::GreaterThanEquals)]
        );
        assert_eq!(
            tokens_ok("<="),
            vec![Token::Operator(OpKind::LessThanEquals)]
        );
    }

    #[test]
    fn test_groupings() {
        assert_eq!(
            tokens_ok("("),
            vec![Token::Grouping(Grouping::OpeningParen)]
        );
        assert_eq!(
            tokens_ok(")"),
            vec![Token::Grouping(Grouping::ClosingParen)]
        );
    }

    #[test]
    fn test_whitespace_skipped() {
        assert_eq!(tokens_ok("  42  "), vec![Token::Int(42)]);
    }

    #[test]
    fn test_whitespace_between_tokens() {
        assert_eq!(
            tokens_ok("foo + 42"),
            vec![
                Token::Symbol("foo"),
                Token::Operator(OpKind::Plus),
                Token::Int(42),
            ]
        );
    }

    #[test]
    fn test_string_basic() {
        assert_eq!(tokens_ok(r#""hello""#), vec![Token::String("hello")]);
    }

    #[test]
    fn test_string_single_quotes() {
        assert_eq!(tokens_ok("'hello'"), vec![Token::String("hello")]);
    }

    #[test]
    fn test_string_empty() {
        assert_eq!(tokens_ok(r#""""#), vec![Token::String("")]);
    }

    #[test]
    fn test_string_escape_newline() {
        assert_eq!(tokens_ok(r#""\n""#), vec![Token::String("\n")]);
    }

    #[test]
    fn test_string_escape_tab() {
        assert_eq!(tokens_ok(r#""\t""#), vec![Token::String("\t")]);
    }

    #[test]
    fn test_string_escape_backslash() {
        assert_eq!(tokens_ok(r#""\\""#), vec![Token::String("\\")]);
    }

    #[test]
    fn test_string_unterminated_eof() {
        let results = tokenize(r#""hello"#);
        assert!(matches!(
            results[0].as_ref().unwrap_err().value,
            TokenizationError::UnterminatedStringLiteral
        ));
    }

    #[test]
    fn test_string_unterminated_newline() {
        let results = tokenize("\"hello\nworld\"");
        assert!(matches!(
            results[0].as_ref().unwrap_err().value,
            TokenizationError::UnterminatedStringLiteral
        ));
    }

    #[test]
    fn test_string_invalid_escape() {
        let results = tokenize(r#""\q""#);
        assert!(matches!(
            results[0].as_ref().unwrap_err().value,
            TokenizationError::InvalidEscapeSequence(_)
        ));
    }

    #[test]
    fn test_string_escape_at_eof() {
        let results = tokenize(r#""\"#);
        assert!(matches!(
            results[0].as_ref().unwrap_err().value,
            TokenizationError::InvalidEscapeSequence("\\(EOF)")
        ));
    }

    #[test]
    fn test_string_span() {
        let source = make_source(r#""hi""#);
        let tokens: Vec<_> = Tokenizer::new(source).collect();
        let t = tokens[0].as_ref().unwrap();
        assert_eq!(t.span.start, 0);
        assert_eq!(t.span.end, 4); // includes delimiters in span
    }

    #[test]
    fn test_string_in_expression() {
        assert_eq!(
            tokens_ok(r#"foo + "bar""#),
            vec![
                Token::Symbol("foo"),
                Token::Operator(OpKind::Plus),
                Token::String("bar"),
            ]
        );
    }

    #[test]
    fn test_string_mismatched_quotes() {
        // "foo' should be unterminated since delimiters must match
        let results = tokenize(r#""foo'"#);
        assert!(matches!(
            results[0].as_ref().unwrap_err().value,
            TokenizationError::UnterminatedStringLiteral
        ));
    }

    #[test]
    fn test_span_correctness() {
        let source = make_source("123+456");
        let tokens: Vec<_> = Tokenizer::new(source).collect();
        let first = tokens[0].as_ref().unwrap();
        assert_eq!(first.span.start, 0);
        assert_eq!(first.span.end, 3);
        let op = tokens[1].as_ref().unwrap();
        assert_eq!(op.span.start, 3);
        assert_eq!(op.span.end, 4);
    }

    #[test]
    fn test_unknown_symbol() {
        let results = tokenize("@");
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0].as_ref().unwrap_err().value,
            TokenizationError::UnknownSymbol("@")
        ));
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(tokenize("").len(), 0);
    }
}
