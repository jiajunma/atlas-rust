//! Compatibility-oriented Atlas language runtime.
//!
//! Public modules are intentionally small and map to observable language
//! boundaries. Implementation work follows the contracts in `docs/`.

pub mod diagnostic;
pub mod lex;
pub mod source;

/// Version of the language compatibility contract implemented by this crate.
pub const COMPATIBILITY_VERSION: &str = "atlas-language-v0";
