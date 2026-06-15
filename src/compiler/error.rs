use thiserror::Error;

use crate::source::Spanned;

/// A type alias for the result of an operation that occurred during program compilation.
pub type Result<T, E = Spanned<CompilerError>> = std::result::Result<T, E>;

/// An error that occured during the compilation of a source.
#[derive(Debug, Clone, Error)]
pub enum CompilerError {
    #[error("variable binding `{symbol}` does not exist")]
    UnboundBinding {
        /// The symbol of the binding.
        symbol: &'static str,
    },
}
