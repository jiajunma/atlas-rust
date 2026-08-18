//! Compatibility-oriented Atlas language runtime.
//!
//! Public modules are intentionally small and map to observable language
//! boundaries. Implementation work follows the contracts in `docs/`.

pub mod coercions;
pub mod diagnostic;
pub mod domain_builtins;
pub mod formula;
pub mod frames;
pub mod lex;
pub mod linear_values;
pub(crate) mod matreduc;
pub mod session;
pub mod session_frame;
pub mod source;
pub mod syntax;
pub mod typed;
pub mod types;
pub mod value;

#[cfg(test)]
mod session_fixture_tests;

/// Version of the language compatibility contract implemented by this crate.
pub const COMPATIBILITY_VERSION: &str = "atlas-language-v0";
