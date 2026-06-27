use std::{
    fmt::{Debug, Display},
    sync::LazyLock,
};

use lasso::{Spur, ThreadedRodeo};

/// Manages efficiently creating and querying for interned strings thoughout the program lifecycle.
pub struct Interner(ThreadedRodeo);

/// A unique, cheap handle for an interned string in the source code.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(Spur);

/// The global [`Interner`], allocated once at program initialization.
static INTERNER: LazyLock<Interner> = LazyLock::new(|| Interner(ThreadedRodeo::new()));

impl Interner {
    /// Creates a [`Symbol`] for the given string.
    pub fn intern(s: &str) -> Symbol {
        Symbol(INTERNER.0.get_or_intern(s))
    }

    /// Resolves the associated text for the given [`Symbol`].
    pub fn resolve(symbol: Symbol) -> &'static str {
        INTERNER.0.resolve(&symbol.0)
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Interner::resolve(*self))
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Interner::resolve(*self))
    }
}
