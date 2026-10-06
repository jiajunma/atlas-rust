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
    String(AtlasString),
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
    /// A captured builtin keeps its registry identity and argument policy;
    /// it is not a user closure and has no captured lexical frame.
    BuiltinFunction(Rc<BuiltinFunction>),
}

/// Opaque outside the core: only the typed builtin registry constructs these.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuiltinFunction {
    pub(crate) index: usize,
    pub(crate) print_name: String,
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

impl SlotShape {
    pub(crate) fn binds_slots(&self) -> bool {
        match self {
            Self::Leaf => true,
            Self::Discard => false,
            Self::Tuple { elements, whole } => *whole || elements.iter().any(Self::binds_slots),
        }
    }
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

/// Atlas strings are owned bytes, including fragments that are not UTF-8.
/// Display is for human diagnostics only; interpreter output uses `as_bytes`.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct AtlasString(Vec<u8>);

impl AtlasString {
    pub fn as_bytes(&self) -> &[u8] { &self.0 }
    pub fn into_bytes(self) -> Vec<u8> { self.0 }
    pub fn as_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn push_bytes(&mut self, bytes: &[u8]) { self.0.extend_from_slice(bytes); }
    pub fn push_str(&mut self, text: &str) { self.push_bytes(text.as_bytes()); }
}

impl From<String> for AtlasString {
    fn from(text: String) -> Self { Self(text.into_bytes()) }
}
impl From<&str> for AtlasString {
    fn from(text: &str) -> Self { Self(text.as_bytes().to_vec()) }
}
impl From<Vec<u8>> for AtlasString {
    fn from(bytes: Vec<u8>) -> Self { Self(bytes) }
}
impl PartialEq<str> for AtlasString {
    fn eq(&self, other: &str) -> bool { self.as_bytes() == other.as_bytes() }
}
impl PartialEq<&str> for AtlasString {
    fn eq(&self, other: &&str) -> bool { self == *other }
}
impl PartialEq<String> for AtlasString {
    fn eq(&self, other: &String) -> bool { self == other.as_str() }
}
impl PartialEq<AtlasString> for String {
    fn eq(&self, other: &AtlasString) -> bool { self.as_bytes() == other.as_bytes() }
}
impl fmt::Display for AtlasString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rest = self.as_bytes();
        while !rest.is_empty() {
            match std::str::from_utf8(rest) {
                Ok(text) => return f.write_str(text),
                Err(error) => {
                    let (valid, invalid) = rest.split_at(error.valid_up_to());
                    f.write_str(std::str::from_utf8(valid).expect("validated prefix"))?;
                    let n = error.error_len().unwrap_or(invalid.len());
                    for byte in &invalid[..n] { write!(f, "\\x{byte:02x}")?; }
                    rest = &invalid[n..];
                }
            }
        }
        Ok(())
    }
}

impl Value {
    /// Standard Atlas value printer, without a Unicode conversion boundary.
    pub fn atlas_text(&self) -> AtlasString {
        let mut out = AtlasString::default();
        self.append_atlas_text(&mut out);
        out
    }

    pub fn append_atlas_text(&self, out: &mut AtlasString) {
        match self {
            Self::String(text) => {
                out.push_str("\"");
                out.push_bytes(text.as_bytes());
                out.push_str("\"");
            }
            Self::Tuple(values) | Self::List(values) => {
                let tuple = matches!(self, Self::Tuple(_));
                out.push_str(if tuple { "(" } else { "[" });
                for (index, value) in values.iter().enumerate() {
                    if index > 0 { out.push_str(","); }
                    value.append_atlas_text(out);
                }
                out.push_str(if tuple { ")" } else { "]" });
            }
            Self::Union { injector_name, value, .. } => {
                value.append_atlas_text(out);
                out.push_str(".");
                out.push_str(injector_name);
            }
            _ => out.push_str(&self.to_string()),
        }
    }
}

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
            // (axis.w:3254-3271). Display has no access to the source-name
            // table, so it prints only the head; the back-trace frame dump
            // (typed.rs `closure_trace_string`) renders the full multi-line
            // form.
            Self::Closure(_) => write!(formatter, "Function defined"),
            Self::BuiltinFunction(function) => write!(formatter, "{{{}}}", function.print_name),
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
    fn byte_strings_render_nested_values_without_replacement() {
        let left = Value::String(vec![0xc3].into());
        let right = Value::String(vec![0xa9].into());
        let value = Value::Tuple(vec![Value::List(vec![left.clone(), right]), Value::Union {
            tag: 0, injector_name: "byte".into(), value: Box::new(left),
        }]);
        assert_eq!(value.atlas_text().as_bytes(), b"([\"\xc3\",\"\xa9\"],\"\xc3\".byte)");
        let mut joined = AtlasString::from(vec![0xc3]);
        joined.push_bytes(&[0xa9]);
        assert_eq!(joined.as_utf8(), Ok("é"));
        assert_eq!(joined.len(), 2);
        // Editor formatting may escape invalid bytes, but must not replace
        // them or be used as the interpreter's output representation.
        assert_eq!(AtlasString::from(vec![0xc3]).to_string(), "\\xc3");
    }

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
