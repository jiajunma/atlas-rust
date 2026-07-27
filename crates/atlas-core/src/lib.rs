//! Compatibility-oriented Atlas language runtime.
//!
//! Public modules are intentionally small and map to observable language
//! boundaries. Implementation work follows the contracts in `docs/`.

pub mod diagnostic;
pub mod eval;
pub mod formula;
pub mod lex;
pub mod session;
pub mod source;
pub mod syntax;
pub mod value;

/// Version of the language compatibility contract implemented by this crate.
pub const COMPATIBILITY_VERSION: &str = "atlas-language-v0";
