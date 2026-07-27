//! Values produced by the Atlas evaluator.

use std::fmt;

use malachite::{Integer as BigInt, Rational as BigRational};

/// The Atlas value model currently covered by the evaluator.
///
/// Functions and domain handles will be added as separate variants once their
/// language contracts have been frozen against Atlas.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Integer(BigInt),
    Rational(BigRational),
    Boolean(bool),
    String(String),
    Tuple(Vec<Value>),
    List(Vec<Value>),
}

/// Descriptive alias for callers that prefer the language-level name.
pub type AtlasValue = Value;

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(formatter, "{value}"),
            Self::Rational(value) => {
                // Malachite stores the sign separately from a non-negative
                // numerator. Atlas prints a denominator even when it is one.
                if value < &BigRational::from(0) {
                    write!(
                        formatter,
                        "-{}/{}",
                        value.numerator_ref(),
                        value.denominator_ref()
                    )
                } else {
                    write!(
                        formatter,
                        "{}/{}",
                        value.numerator_ref(),
                        value.denominator_ref()
                    )
                }
            }
            Self::Boolean(value) => write!(formatter, "{value}"),
            Self::String(value) => write!(formatter, "\"{value}\""),
            Self::Tuple(values) => {
                write!(formatter, "(")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ",")?;
                    }
                    write!(formatter, "{value}")?;
                }
                write!(formatter, ")")
            }
            Self::List(values) => {
                write!(formatter, "[")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ",")?;
                    }
                    write!(formatter, "{value}")?;
                }
                write!(formatter, "]")
            }
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

    #[test]
    fn displays_tuple_and_list_values_like_source_literals() {
        assert_eq!(
            Value::Tuple(vec![Value::Integer(1.into()), Value::Boolean(true)]).to_string(),
            "(1,true)"
        );
        assert_eq!(
            Value::List(vec![Value::Integer(1.into()), Value::Integer(2.into())]).to_string(),
            "[1,2]"
        );
    }
}
