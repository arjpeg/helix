use std::{ops::Range, path::Path};

/// The file path and content of a helix source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Source {
    /// The text content of the file.
    pub content: &'static str,
    /// The path of the file.
    pub path: &'static Path,
}

/// Associates a value with a specific region of source text.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Spanned<T> {
    /// The associated value (for example, a token or AST node).
    pub value: T,
    /// The span of text in the source that produced this value.
    pub span: Span,
}

/// Represents a region of text within a source file.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    /// The [`Source`] file from which this span originates.
    pub source: Source,
    /// Inclusive start position (in bytes).
    pub start: usize,
    /// Exclusive end position (in bytes).
    pub end: usize,
}

impl Span {
    /// Creates a new [`Span`].
    pub fn new(source: Source, range: Range<usize>) -> Self {
        Self {
            source,
            start: range.start,
            end: range.end,
        }
    }

    /// Returns the corresponding text associated with this span.
    pub fn text(&self) -> &'static str {
        &self.source.content[self.start..self.end]
    }

    /// Merges two [`Span`]'s by using the start from "left" and the end from "right".
    pub fn merge(left: Self, right: Self) -> Self {
        assert!(
            left.source == right.source,
            "can't merge spans from different source files"
        );

        Self {
            source: left.source,
            start: left.start,
            end: right.end,
        }
    }
}

impl<T> Spanned<T> {
    /// Wraps the input argument inside a new [`Spanned`] object.
    pub fn wrap(value: T, span: Span) -> Self {
        Self { value, span }
    }

    /// Maps the the current value while maintaining the same [`Span`].
    pub fn map<V>(self, f: impl FnOnce(T) -> V) -> Spanned<V> {
        Spanned {
            value: f(self.value),
            span: self.span,
        }
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span")
            // skip source
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}
