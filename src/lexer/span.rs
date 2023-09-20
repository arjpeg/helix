use std::{
    ops::{Index, Range},
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    /// The start of the span.
    pub start: usize,
    /// The end of the span.
    pub end: usize,
    /// The file that the span is in.
    pub file: Rc<str>,
}

impl Span {
    /// Creates a new span from a start and end position.
    pub fn new(start: usize, end: usize, file: Rc<str>) -> Span {
        Span { start, end, file }
    }

    /// Returns the length of the span.
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

impl From<(Range<usize>, Rc<str>)> for Span {
    fn from(range: (Range<usize>, Rc<str>)) -> Self {
        Span::new(range.0.start, range.0.end, range.1)
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
