use crate::token::Span;

macro_rules! impl_binary_operator {
    (($name:ident, $operator:ident, {
        $( ($lhs:pat, $rhs:pat) => $body:expr),*
    })) => {
        impl Value {
            pub fn $name(&self, other: &Value) -> Option<Value> {
                use $crate::value::ValueKind as VK;
                use VK::*;

                let kind: VK = match (&self.kind, &other.kind) {
                    $( ($lhs, $rhs) => {
                        $body
                    })*
                    _ => todo!(),
                };

                let span = Span::new(self.span.start..other.span.end, self.span.source);

                Some($crate::value::Value {
                    kind,
                    span
                })
            }
        }
    };
}

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
    /// A string.
    String(String),
    /// A boolean.
    Boolean(bool),
}

impl Value {
    /// Create a new value.
    pub fn new(kind: ValueKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl_binary_operator! {
    (add, Add, {
        (Float(a), Float(b)) => Float(a + b)
    })
}
