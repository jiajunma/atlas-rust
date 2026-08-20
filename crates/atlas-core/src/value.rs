//! Values produced by the Atlas evaluator.

use std::fmt;
use std::rc::Rc;

use malachite::{Integer as BigInt, Rational as BigRational};

use crate::diagnostic::SourceSpan;
use crate::frames::Frame;
use crate::typed::TypedExpr;

pub use crate::domain_builtins::DomainValue;
pub use crate::linear_values::{Matrix, RatVec, Vec32};

/// The Atlas value model currently covered by the evaluator.
///
/// Domain handles arrived with the domain slice; closures (non-recursive
/// functions) with the B3a function slice.
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
    /// A non-recursive closure: the shared typed body plus the captured
    /// frame chain (upstream `closure_value`, axis.w:3209-3236).
    Closure(Rc<Closure>),
}

/// The payload of a closure value. The body is shared between every closure
/// created from the same lambda literal; the captured chain keeps the
/// defining scope's frames alive after it pops.
pub struct Closure {
    /// Number of argument slots a call binds; 0 means parameterless, and
    /// the call pushes no frame beyond the self slot below.
    pub parameters: usize,
    /// How each argument value distributes into frame slots (one entry per
    /// parameter; upstream `bind_pattern`).
    pub shapes: Rc<[SlotShape]>,
    /// A recursive closure: a call binds the closure itself at slot 0 of
    /// the call frame, ahead of the argument slots (upstream `maybe_push`).
    pub recursive: bool,
    pub body: Rc<TypedExpr>,
    pub frame: Option<Rc<Frame>>,
    /// The lambda's source location, reported as `defined at ...` in
    /// back-trace call lines (upstream `lambda_struct::loc`).
    pub span: SourceSpan,
    /// Frame slot names in bind order (a recursive closure's slot 0 is the
    /// self name), for the back-trace frame dump (axis.w:2896-2909). Empty
    /// for closures whose call pushes no traced frame (upstream
    /// `parameterless` closures, and the builtin-backed member closures).
    pub param_names: Rc<[String]>,
}

/// How one bound value distributes into frame slots (upstream
/// `bind_pattern`): a leaf takes one slot, a discard none, a tuple
/// destructures its value per element. The whole-value name of a
/// `(a, b): t` pattern occupies the first slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotShape {
    /// Bind the value to one frame slot.
    Leaf,
    /// Bind nothing (a `()` or empty pat_list slot, an anonymous `type .`).
    Discard,
    /// Destructure a tuple value per element; `whole` additionally binds
    /// the undestructured value ahead of the element slots.
    Tuple {
        elements: Vec<SlotShape>,
        whole: bool,
    },
}

impl fmt::Debug for Closure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Closure")
            .field("parameters", &self.parameters)
            .field("shapes", &self.shapes)
            .field("recursive", &self.recursive)
            .field("body", &self.body)
            .field("frame", &self.frame.as_ref().map(Rc::as_ptr))
            .field("span", &self.span)
            .field("param_names", &self.param_names)
            .finish()
    }
}

// Two closures are only ever the same value when they share a body and a
// captured chain; runtime values are otherwise unordered.
impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        self.parameters == other.parameters
            && self.recursive == other.recursive
            && Rc::ptr_eq(&self.body, &other.body)
            && match (&self.frame, &other.frame) {
                (None, None) => true,
                (Some(own), Some(other)) => Rc::ptr_eq(own, other),
                _ => false,
            }
    }
}

impl Eq for Closure {}

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
            // Upstream prints `Function defined <loc>` plus the lambda text
            // (axis.w:3254-3271); the source location is not carried yet, so
            // only the head is printed.
            Self::Closure(_) => write!(formatter, "Function defined"),
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
