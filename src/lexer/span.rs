use std::ops::{Index, Range};

#[derive(Debug, Clone, Copy)]
pub struct Span {
    /// The start of the span.
    pub start: usize,
    /// The end of the span.
    pub end: usize,
}

impl Span {
    /// Creates a new span from a start and end position.
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    /// Returns the length of the span.
    pub fn len(&self) -> usize {
        dbg!(self.end, self.start);
        self.end - self.start
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Span::new(range.start, range.end)
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}

impl Index<Span> for String {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}
