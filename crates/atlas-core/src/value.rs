//! Scalar values produced by the Atlas evaluator.

use std::fmt;

use num_bigint::BigInt;
use num_rational::BigRational;

/// The scalar portion of the Atlas value model.
///
/// Containers, functions, and domain handles will be added as separate
/// variants once their language contracts have been frozen against Atlas.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Integer(BigInt),
    Rational(BigRational),
    Boolean(bool),
    String(String),
}

/// Descriptive alias for callers that prefer the language-level name.
pub type AtlasValue = Value;

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(formatter, "{value}"),
            Self::Rational(value) => {
                write!(formatter, "{}/{}", value.numer(), value.denom())
            }
            Self::Boolean(value) => write!(formatter, "{value}"),
            Self::String(value) => write!(formatter, "\"{value}\""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_string_values_like_the_atlas_oracle() {
        assert_eq!(Value::String("a\"b".into()).to_string(), "\"a\"b\"");
    }
}
