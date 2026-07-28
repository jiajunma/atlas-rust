//! Values produced by the Atlas evaluator.

use std::fmt;

use malachite::{Integer as BigInt, Rational as BigRational};

pub use crate::domain_builtins::DomainValue;
pub use crate::linear_values::{Matrix, RatVec, Vec32};

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
    /// An Atlas `vec`: machine 32-bit entries, printed right-aligned.
    Vector(Vec32),
    /// An Atlas `mat`: column-major machine-int entries.
    Matrix(Matrix),
    /// An Atlas `ratvec`: normalised numerators over one denominator.
    RatVector(RatVec),
    /// A union value: the injected component with its variant tag and the
    /// injector's name (printed as `value.injectorname`).
    Union {
        tag: u16,
        injector_name: String,
        value: Box<Value>,
    },
    Domain(DomainValue),
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
            Self::Vector(value) => write!(formatter, "{value}"),
            Self::Matrix(value) => write!(formatter, "{value}"),
            Self::RatVector(value) => write!(formatter, "{value}"),
            Self::Union {
                injector_name,
                value,
                ..
            } => write!(formatter, "{value}.{injector_name}"),
            Self::Domain(value) => write!(formatter, "{value}"),
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

    #[test]
    fn linear_and_union_values_use_their_upstream_prints() {
        assert_eq!(Value::Vector(Vec32(vec![1, 22])).to_string(), "[  1, 22 ]");
        assert_eq!(
            Value::RatVector(RatVec::new(vec![1, 2], 2).expect("valid")).to_string(),
            "[ 1, 2 ]/2"
        );
        assert_eq!(
            Value::Matrix(Matrix::from_columns(1, 2, vec![3, 4]).expect("valid")).to_string(),
            "\n| 3, 4 |\n"
        );
        assert_eq!(
            Value::Union {
                tag: 1,
                injector_name: "solution".into(),
                value: Box::new(Value::Vector(Vec32(vec![5]))),
            }
            .to_string(),
            "[ 5 ].solution"
        );
    }
}
