use std::iter::Peekable;

/// A cursor that keeps track of the current position in some iterable sequence
pub struct Cursor<I: Iterator> {
    /// The thing beeing iterated over.
    iter: Peekable<I>,

    /// The current index of the iterator
    pub pos: usize,
}

impl<I: Iterator> Cursor<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
            pos: 0,
        }
    }

    /// Peek at the next character in the input string.
    pub fn peek(&mut self) -> Option<&I::Item> {
        self.iter.peek()
    }

    /// Advance the cursor by one character.
    pub fn advance(&mut self) -> Option<I::Item> {
        let value = self.iter.next();

        self.pos += value.as_ref().map_or(0, |_| 1);

        value
    }

    /// Skip the current character if it matches the given predicate.
    pub fn advance_while<F>(&mut self, predicate: F)
    where
        F: Fn(&I::Item) -> bool,
    {
        while matches!(self.peek(), Some(c) if predicate(c)) {
            self.advance();
        }
    }
}

impl<I: Iterator> Clone for Cursor<I>
where
    I: Clone,
    I::Item: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            pos: self.pos,
        }
    }
}
