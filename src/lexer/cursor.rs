use std::str::Chars;

/// Represents a cursor into a source file.
/// Used to keep track of the current position in the file.
#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    /// The source file.
    content: Chars<'a>,
    /// The current position in the file.
    byte_pos: usize,
    /// The actual character that is being pointed to.
    current: Option<char>,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor from a source file.
    pub fn new(content: &'a str) -> Self {
        Self {
            content: content.chars(),
            byte_pos: 0,
            current: None,
        }
    }

    /// Peeks at the next character in the file.
    pub fn peek(&self) -> Option<char> {
        self.content.clone().next()
    }

    /// Returns the current character in the file.
    #[allow(dead_code)]
    pub fn current(&self) -> Option<char> {
        self.current
    }

    /// Advances the cursor by one character.
    pub fn advance(&mut self) -> Option<char> {
        let c = self.content.next();
        c.map(|c| self.byte_pos += c.len_utf8());

        self.current = c;

        c
    }

    /// Returns the current position in the file.
    pub fn pos(&self) -> usize {
        self.byte_pos
    }

    /// Advances the cursor as long as the predicate returns true.
    pub fn advance_while(&mut self, predicate: fn(char) -> bool) {
        while matches!(self.peek(), Some(c) if predicate(c)) {
            self.advance();
        }
    }
}
