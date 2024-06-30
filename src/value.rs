#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub kind: ValueKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    /// A floating point number.
    Float(f64),
    /// An integer.
    Integer(i64),
}
