//! The typed pipeline: conversion of parsed expressions into typed
//! executables, and their evaluation (phase B stage B2, growing in place
//! until it replaces the dynamic evaluator).
//!
//! `convert_expr` mirrors upstream (axis.w:272-487): one pass does both
//! checking and synthesis against an in/out type pattern that mutates only
//! through `specialise`; `conform_types` = specialise-else-coerce-else-
//! type-error; a list display in non-row context consults `row_coercion`
//! (so `mat: [[1,2]]` types its elements as `vec`). Conversion nodes
//! evaluate the registered conversion functions; integer narrowing uses
//! the EXACT upstream error text (typo included, bigint.cpp:142-162).

use malachite::{Integer as BigInt, Rational as BigRational};

use crate::coercions::{coercion_between, row_coercion};
use crate::diagnostic::{Diagnostic, ErrorKind, SourceSpan};
use crate::frames::{EvaluationContext, GlobalCell};
use crate::linear_values::{Matrix, RatVec, Vec32};
use crate::syntax::Expr;
use crate::types::{Prim, Type, TypeTable};
use crate::value::Value;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Why evaluation stopped early. Loops consume `Break(0)` and rethrow
/// decremented; closure application consumes `Return`; runtime errors
/// carry their diagnostic. A `Break`/`Return` reaching top level is an
/// internal error — analysis guarantees legality.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Control {
    Break(usize),
    Return(Value),
    Runtime(Diagnostic),
}

/// How much value the context wants (upstream `expression_base::level`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Level {
    NoValue,
    SingleValue,
}

/// A typed executable expression.
#[derive(Clone, Debug, PartialEq)]
pub enum TypedExpr {
    Denotation(Value),
    TupleDisplay(Vec<TypedExpr>),
    ListDisplay(Vec<TypedExpr>),
    /// A registered coercion applied to a fully converted inner expression.
    Conversion {
        tag: &'static str,
        inner: Box<TypedExpr>,
        span: SourceSpan,
    },
    /// A global read through the cell captured at analysis time.
    GlobalIdent {
        name: String,
        cell: GlobalCell,
        span: SourceSpan,
    },
    /// A statically resolved builtin call (index into the registry).
    BuiltinCall {
        builtin: usize,
        arguments: Vec<TypedExpr>,
        span: SourceSpan,
    },
}

/// Conversion-time context (locals arrive with lambdas in B3).
pub struct Analysis<'a> {
    pub types: &'a TypeTable,
    pub globals: &'a IdTable,
}

/// The global identifier table: one binding per name, each definition
/// holding the FRESH cell it allocated (converted code keeps the cell it
/// captured; re-definition rebinds the name only).
#[derive(Default)]
pub struct IdTable {
    entries: BTreeMap<String, (Type, GlobalCell)>,
}

impl IdTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(&mut self, name: impl Into<String>, type_: Type, cell: GlobalCell) {
        self.entries.insert(name.into(), (type_, cell));
    }

    pub fn lookup(&self, name: &str) -> Option<&(Type, GlobalCell)> {
        self.entries.get(name)
    }
}

fn type_error(message: String, span: SourceSpan) -> Diagnostic {
    Diagnostic::new(ErrorKind::Type, message, Some(span))
}

/// specialise-else-coerce-else-error (upstream `conform_types`,
/// axis-types.w:3095-3100). Coercion to void always succeeds without a
/// node (the caller voids); otherwise a matching table entry wraps the
/// converted expression.
fn conform_types(
    found: &Type,
    required: &mut Type,
    converted: TypedExpr,
    span: SourceSpan,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    if required.specialise(found, analysis.types) {
        return Ok(converted);
    }
    if required.is_void() {
        return Ok(converted);
    }
    if let Some(coercion) = coercion_between(found, required, analysis.types) {
        return Ok(TypedExpr::Conversion {
            tag: coercion.tag,
            inner: Box::new(converted),
            span,
        });
    }
    Err(type_error(
        format!(
            "type {} does not match required pattern {}",
            found.display(analysis.types),
            required.display(analysis.types),
        ),
        span,
    ))
}

/// Convert `expression` against the in/out `required` pattern.
pub fn convert_expr(
    expression: &Expr,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    match expression {
        Expr::Integer { value, span } => {
            let found = Type::Primitive(Prim::Int);
            conform_types(
                &found,
                required,
                TypedExpr::Denotation(Value::Integer(value.clone())),
                *span,
                analysis,
            )
        }
        Expr::Boolean { value, span } => conform_types(
            &Type::Primitive(Prim::Bool),
            required,
            TypedExpr::Denotation(Value::Boolean(*value)),
            *span,
            analysis,
        ),
        Expr::String { value, span } => conform_types(
            &Type::Primitive(Prim::String),
            required,
            TypedExpr::Denotation(Value::String(value.clone())),
            *span,
            analysis,
        ),
        Expr::Group { inner, .. } => convert_expr(inner, required, analysis),
        Expr::Cast { target, body, .. } => {
            // The cast's whole effect is conversion-time: convert the body
            // against the denoted type, then conform THAT to the context.
            let mut cast_type = target.resolve();
            let converted = convert_expr(body, &mut cast_type, analysis)?;
            conform_types(&cast_type, required, converted, expression.span(), analysis)
        }
        Expr::Tuple { elements, span } => {
            // Prepare a tuple pattern of the right arity, or fail.
            let mut pattern = Type::Tuple(vec![Type::Undetermined; elements.len()]);
            if !pattern.can_specialise(required, analysis.types) {
                return Err(type_error(
                    format!(
                        "tuple display of {} components does not match required pattern {}",
                        elements.len(),
                        required.display(analysis.types),
                    ),
                    *span,
                ));
            }
            pattern.specialise(required, analysis.types);
            let components = match &mut pattern {
                Type::Tuple(components) => components,
                // A 1-element display collapsed; treat as the single type.
                single => {
                    let converted = elements
                        .first()
                        .map(|element| convert_expr(element, single, analysis))
                        .transpose()?;
                    let found = single.clone();
                    return conform_types(
                        &found,
                        required,
                        converted.expect("collapse implies one element"),
                        *span,
                        analysis,
                    );
                }
            };
            let converted = elements
                .iter()
                .zip(components.iter_mut())
                .map(|(element, component)| convert_expr(element, component, analysis))
                .collect::<Result<Vec<_>, _>>()?;
            conform_types(
                &pattern,
                required,
                TypedExpr::TupleDisplay(converted),
                *span,
                analysis,
            )
        }
        Expr::List { elements, span } => {
            // In row context (or undetermined), elements share the component
            // pattern; in a non-row context the FIRST row coercion for that
            // target decides the component type (mat context -> vec).
            let (mut component, coercion_tag) = match &*required {
                Type::Undetermined => (Type::Undetermined, None),
                Type::Row(component) => (component.as_ref().clone(), None),
                other => match row_coercion(other, analysis.types) {
                    Some((coercion, component)) => (component.clone(), Some(coercion.tag)),
                    None => {
                        return Err(type_error(
                            format!(
                                "list display does not match required pattern {}",
                                other.display(analysis.types),
                            ),
                            *span,
                        ))
                    }
                },
            };
            let converted = elements
                .iter()
                .map(|element| convert_expr(element, &mut component, analysis))
                .collect::<Result<Vec<_>, _>>()?;
            let display = TypedExpr::ListDisplay(converted);
            match coercion_tag {
                Some(tag) => Ok(TypedExpr::Conversion {
                    tag,
                    inner: Box::new(display),
                    span: *span,
                }),
                None => conform_types(&Type::row(component), required, display, *span, analysis),
            }
        }
        Expr::Identifier { name, span } => {
            let Some((type_, cell)) = analysis.globals.lookup(name) else {
                return Err(Diagnostic::new(
                    ErrorKind::Name,
                    format!("undefined identifier `{name}`"),
                    Some(*span),
                ));
            };
            let found = type_.clone();
            conform_types(
                &found,
                required,
                TypedExpr::GlobalIdent {
                    name: name.clone(),
                    cell: cell.clone(),
                    span: *span,
                },
                *span,
                analysis,
            )
        }
        Expr::OperatorCall {
            operator,
            arguments,
            span,
        } => {
            // The a-priori-type design (axis.w:1552-1599): convert each
            // argument once in undetermined context, then one pass over the
            // variants — exact match wins immediately, else the FIRST
            // variant the a-priori type coerces into is taken, with each
            // divergent argument re-converted against its expected type.
            let mut converted = Vec::new();
            let mut a_priori = Vec::new();
            for argument in arguments {
                let mut slot = Type::Undetermined;
                converted.push(convert_expr(argument, &mut slot, analysis)?);
                a_priori.push(slot);
            }
            let a_priori_type = Type::tuple(a_priori.clone());
            let variants = overload_variants(&operator.symbol);
            let mut inexact = None;
            let mut chosen = None;
            for &index in variants {
                let builtin = &builtin_registry()[index];
                if builtin.arg_type == a_priori_type {
                    chosen = Some(index);
                    break;
                }
                if inexact.is_none()
                    && crate::coercions::is_close(&a_priori_type, &builtin.arg_type, analysis.types)
                        & 0x1
                        != 0
                {
                    inexact = Some(index);
                }
            }
            let index = match chosen.or(inexact) {
                Some(index) => index,
                None => {
                    return Err(type_error(
                        format!(
                            "no instance of operator '{}' matches argument type {}",
                            operator.symbol,
                            a_priori_type.display(analysis.types),
                        ),
                        *span,
                    ))
                }
            };
            let builtin = &builtin_registry()[index];
            // Re-convert divergent arguments against the expected components.
            let expected: Vec<Type> = match &builtin.arg_type {
                Type::Tuple(components) => components.clone(),
                single => vec![single.clone()],
            };
            let arguments = converted
                .into_iter()
                .zip(a_priori)
                .zip(expected)
                .zip(arguments)
                .map(|(((converted, found), mut expected), original)| {
                    if expected.specialise(&found, analysis.types) {
                        Ok(converted)
                    } else {
                        convert_expr(original, &mut expected, analysis)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            let found = builtin.result.clone();
            conform_types(
                &found,
                required,
                TypedExpr::BuiltinCall {
                    builtin: index,
                    arguments,
                    span: *span,
                },
                *span,
                analysis,
            )
        }
        unsupported => Err(type_error(
            "this expression form is not yet in the typed pipeline".into(),
            unsupported.span(),
        )),
    }
}

/// One registered builtin overload.
pub struct Builtin {
    pub name: &'static str,
    pub arg_type: Type,
    pub result: Type,
    pub run: fn(Vec<Value>, SourceSpan) -> Result<Value, Control>,
}

fn int_type() -> Type {
    Type::Primitive(Prim::Int)
}

fn int_pair() -> Type {
    Type::tuple(vec![int_type(), int_type()])
}

fn expect_ints(mut arguments: Vec<Value>) -> (BigInt, BigInt) {
    let second = arguments.pop();
    let first = arguments.pop();
    match (first, second) {
        (Some(Value::Integer(first)), Some(Value::Integer(second))) => (first, second),
        other => panic!("int builtin saw {other:?}"),
    }
}

/// The startup builtin registry (growing toward the traced inventory).
pub fn builtin_registry() -> &'static Vec<Builtin> {
    static REGISTRY: OnceLock<Vec<Builtin>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        vec![
            Builtin {
                name: "+",
                arg_type: int_pair(),
                result: int_type(),
                run: |arguments, _span| {
                    let (first, second) = expect_ints(arguments);
                    Ok(Value::Integer(first + second))
                },
            },
            Builtin {
                name: "-",
                arg_type: int_pair(),
                result: int_type(),
                run: |arguments, _span| {
                    let (first, second) = expect_ints(arguments);
                    Ok(Value::Integer(first - second))
                },
            },
            Builtin {
                name: "*",
                arg_type: int_pair(),
                result: int_type(),
                run: |arguments, _span| {
                    let (first, second) = expect_ints(arguments);
                    Ok(Value::Integer(first * second))
                },
            },
            Builtin {
                name: "-",
                arg_type: int_type(),
                result: int_type(),
                run: |mut arguments, _span| match arguments.pop() {
                    Some(Value::Integer(value)) => Ok(Value::Integer(-value)),
                    other => panic!("unary minus saw {other:?}"),
                },
            },
            Builtin {
                name: "/",
                arg_type: int_pair(),
                result: Type::Primitive(Prim::Rat),
                run: |arguments, span| {
                    let (first, second) = expect_ints(arguments);
                    if second == 0 {
                        return Err(runtime("Division by zero", span));
                    }
                    Ok(Value::Rational(BigRational::from_integers(first, second)))
                },
            },
        ]
    })
}

/// Variant indices for `name`, in registration order.
fn overload_variants(name: &str) -> &'static [usize] {
    static INDEX: OnceLock<BTreeMap<&'static str, Vec<usize>>> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            let mut index: BTreeMap<&'static str, Vec<usize>> = BTreeMap::new();
            for (position, builtin) in builtin_registry().iter().enumerate() {
                index.entry(builtin.name).or_default().push(position);
            }
            index
        })
        .get(name)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

impl TypedExpr {
    /// Evaluate at the demanded level. `NoValue` returns `None`.
    pub fn evaluate(
        &self,
        context: &mut EvaluationContext,
        level: Level,
    ) -> Result<Option<Value>, Control> {
        let _ = context;
        match self {
            Self::Denotation(value) => Ok(at_level(level, || value.clone())),
            Self::TupleDisplay(elements) => {
                let values = elements
                    .iter()
                    .map(|element| force(element, context))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(at_level(level, || Value::Tuple(values.clone())))
            }
            Self::ListDisplay(elements) => {
                let values = elements
                    .iter()
                    .map(|element| force(element, context))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(at_level(level, || Value::List(values.clone())))
            }
            Self::Conversion { tag, inner, span } => {
                let value = force(inner, context)?;
                let converted = apply_conversion(tag, value, *span)?;
                Ok(at_level(level, || converted.clone()))
            }
            Self::GlobalIdent { name, cell, span } => {
                let value = cell.borrow().clone();
                match value {
                    Some(value) => Ok(at_level(level, || value.as_ref().clone())),
                    None => Err(runtime(
                        format!("Taking value of uninitialized variable '{name}'"),
                        *span,
                    )),
                }
            }
            Self::BuiltinCall {
                builtin,
                arguments,
                span,
            } => {
                let values = arguments
                    .iter()
                    .map(|argument| force(argument, context))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = (builtin_registry()[*builtin].run)(values, *span)?;
                Ok(at_level(level, || result.clone()))
            }
        }
    }
}

fn at_level(level: Level, value: impl Fn() -> Value) -> Option<Value> {
    match level {
        Level::NoValue => None,
        Level::SingleValue => Some(value()),
    }
}

fn force(expression: &TypedExpr, context: &mut EvaluationContext) -> Result<Value, Control> {
    expression
        .evaluate(context, Level::SingleValue)
        .map(|value| value.expect("single-value evaluation yields a value"))
}

fn runtime(message: impl Into<String>, span: SourceSpan) -> Control {
    Control::Runtime(Diagnostic::new(ErrorKind::Runtime, message, Some(span)))
}

/// The upstream narrowing, exact message included (bigint.cpp:142-162).
fn narrow_i32(value: &BigInt, span: SourceSpan) -> Result<i32, Control> {
    i32::try_from(value).map_err(|_| runtime("Integer value to big for conversion", span))
}

fn narrow_i64(value: &BigInt, span: SourceSpan) -> Result<i64, Control> {
    i64::try_from(value).map_err(|_| runtime("Integer value to big for conversion", span))
}

fn expect_list(value: Value) -> Vec<Value> {
    match value {
        Value::List(values) => values,
        other => panic!("conversion applied to non-list value {other}"),
    }
}

fn list_to_vec32(values: Vec<Value>, span: SourceSpan) -> Result<Vec32, Control> {
    let entries = values
        .into_iter()
        .map(|value| match value {
            Value::Integer(value) => narrow_i32(&value, span),
            other => panic!("vec conversion saw non-integer {other}"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Vec32(entries))
}

fn rationals_to_ratvec(values: Vec<Value>, span: SourceSpan) -> Result<RatVec, Control> {
    // Bring all entries over a common denominator, then normalise.
    let rationals: Vec<BigRational> = values
        .into_iter()
        .map(|value| match value {
            Value::Rational(value) => value,
            Value::Integer(value) => BigRational::from(value),
            other => panic!("ratvec conversion saw non-rational {other}"),
        })
        .collect();
    let mut denominator = BigInt::from(1);
    for rational in &rationals {
        let entry_denominator = BigInt::from(rational.denominator_ref().clone());
        let gcd = gcd_big(denominator.clone(), entry_denominator.clone());
        denominator = denominator * entry_denominator / gcd;
    }
    let numerators = rationals
        .iter()
        .map(|rational| {
            let scaled = BigRational::from(denominator.clone()) * rational.clone();
            let numerator = BigInt::try_from(scaled)
                .expect("scaling by the common denominator yields an integer");
            narrow_i64(&numerator, span)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let denominator = u64::try_from(&denominator)
        .map_err(|_| runtime("Integer value to big for conversion", span))?;
    RatVec::new(numerators, denominator)
        .ok_or_else(|| runtime("ratvec denominator must be nonzero", span))
}

fn gcd_big(mut a: BigInt, mut b: BigInt) -> BigInt {
    while b != 0 {
        let remainder = a % b.clone();
        a = b;
        b = remainder;
    }
    if a < 0 {
        -a
    } else {
        a
    }
}

fn columns_to_matrix(columns: Vec<Vec32>, span: SourceSpan) -> Result<Matrix, Control> {
    let rows = columns.first().map_or(0, |column| column.0.len());
    if columns.iter().any(|column| column.0.len() != rows) {
        return Err(runtime("matrix columns must have equal lengths", span));
    }
    let cols = columns.len();
    let data = columns.into_iter().flat_map(|column| column.0).collect();
    Matrix::from_columns(rows, cols, data)
        .ok_or_else(|| runtime("inconsistent matrix dimensions", span))
}

/// Apply a registered conversion by tag. Only the tags reachable from the
/// currently converted forms are implemented; an unknown tag is a bug.
fn apply_conversion(tag: &str, value: Value, span: SourceSpan) -> Result<Value, Control> {
    match tag {
        "QI" => match value {
            Value::Integer(value) => Ok(Value::Rational(BigRational::from(value))),
            other => panic!("QI conversion saw {other}"),
        },
        "V[I]" => Ok(Value::Vector(list_to_vec32(expect_list(value), span)?)),
        "Qv[Q]" | "Qv[I]" => Ok(Value::RatVector(rationals_to_ratvec(
            expect_list(value),
            span,
        )?)),
        "QvV" => match value {
            Value::Vector(vector) => {
                let numerators = vector.0.iter().map(|&entry| i64::from(entry)).collect();
                RatVec::new(numerators, 1)
                    .map(Value::RatVector)
                    .ok_or_else(|| runtime("ratvec denominator must be nonzero", span))
            }
            other => panic!("QvV conversion saw {other}"),
        },
        "M[V]" => {
            let columns = expect_list(value)
                .into_iter()
                .map(|column| match column {
                    Value::Vector(vector) => vector,
                    other => panic!("M[V] conversion saw column {other}"),
                })
                .collect();
            Ok(Value::Matrix(columns_to_matrix(columns, span)?))
        }
        "M[[I]]" => {
            let columns = expect_list(value)
                .into_iter()
                .map(|column| list_to_vec32(expect_list(column), span))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::Matrix(columns_to_matrix(columns, span)?))
        }
        "[I]V" => match value {
            Value::Vector(vector) => Ok(Value::List(
                vector
                    .0
                    .into_iter()
                    .map(|entry| Value::Integer(BigInt::from(entry)))
                    .collect(),
            )),
            other => panic!("[I]V conversion saw {other}"),
        },
        "[Q][I]" => Ok(Value::List(
            expect_list(value)
                .into_iter()
                .map(|entry| match entry {
                    Value::Integer(value) => Value::Rational(BigRational::from(value)),
                    other => panic!("[Q][I] conversion saw {other}"),
                })
                .collect(),
        )),
        other => Err(runtime(
            format!("conversion '{other}' is not yet implemented"),
            span,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceText;
    use crate::syntax::parse;
    use std::rc::Rc;

    fn convert_and_run_with(source: &str, globals: &IdTable) -> Result<(Type, Value), Diagnostic> {
        let source = SourceText::new(source);
        let program = parse(&source).expect("test source parses");
        assert_eq!(program.expressions.len(), 1);
        let table = TypeTable::new();
        let analysis = Analysis {
            types: &table,
            globals,
        };
        let mut required = Type::Undetermined;
        let typed = convert_expr(&program.expressions[0], &mut required, &analysis)?;
        let mut context = EvaluationContext::new();
        let value = typed
            .evaluate(&mut context, Level::SingleValue)
            .map_err(|control| match control {
                Control::Runtime(diagnostic) => diagnostic,
                other => panic!("unexpected control flow {other:?}"),
            })?
            .expect("single value");
        Ok((required, value))
    }

    fn convert_and_run(source: &str) -> Result<(Type, Value), Diagnostic> {
        convert_and_run_with(source, &IdTable::new())
    }

    #[test]
    fn globals_read_through_captured_cells_and_report_unset() {
        let mut globals = IdTable::new();
        globals.define(
            "x",
            Type::Primitive(Prim::Int),
            crate::frames::global_with(Rc::new(Value::Integer(7.into()))),
        );
        globals.define(
            "y",
            Type::Primitive(Prim::Int),
            crate::frames::unset_global(),
        );
        let (type_, value) = convert_and_run_with("x", &globals).expect("defined global");
        assert_eq!(type_, Type::Primitive(Prim::Int));
        assert_eq!(value, Value::Integer(7.into()));
        let error = convert_and_run_with("y", &globals).expect_err("unset global");
        assert!(error.message.contains("uninitialized variable 'y'"));
        let error = convert_and_run_with("z", &globals).expect_err("unknown name");
        assert!(error.message.contains("undefined identifier"));
    }

    #[test]
    fn operator_calls_resolve_through_the_overload_registry() {
        let (type_, value) = convert_and_run("1 + 2 * 3").expect("int arithmetic");
        assert_eq!(type_, Type::Primitive(Prim::Int));
        assert_eq!(value, Value::Integer(7.into()));
        let (type_, value) = convert_and_run("1 / 2").expect("int division is rational");
        assert_eq!(type_, Type::Primitive(Prim::Rat));
        assert_eq!(value.to_string(), "1/2");
        let (_, value) = convert_and_run("- 3").expect("unary minus");
        assert_eq!(value, Value::Integer((-3).into()));
        let error = convert_and_run("1 + true").expect_err("no such overload");
        assert!(error.message.contains("no instance of operator '+'"));
        let error = convert_and_run("1 / 0").expect_err("division by zero");
        assert_eq!(error.message, "Division by zero");
    }

    #[test]
    fn casts_produce_real_linear_values_through_conversions() {
        let (type_, value) = convert_and_run("vec: [1,22]").expect("vec cast");
        assert_eq!(type_, Type::Primitive(Prim::Vec));
        assert_eq!(value.to_string(), "[  1, 22 ]");

        let (type_, value) = convert_and_run("mat: [[1,2],[3,4]]").expect("mat cast");
        assert_eq!(type_, Type::Primitive(Prim::Mat));
        // Columns fill the matrix; per-column widths in the print.
        assert_eq!(value.to_string(), "\n| 1, 3 |\n| 2, 4 |\n");

        let (_, value) = convert_and_run("ratvec: [1,2]").expect("ratvec cast");
        assert_eq!(value.to_string(), "[ 1, 2 ]/1");

        let (_, value) = convert_and_run("rat: 2").expect("int promotes");
        assert_eq!(value.to_string(), "2/1");

        let (_, value) = convert_and_run("[rat]: [1,2]").expect("rat row");
        assert_eq!(value.to_string(), "[1/1,2/1]");
    }

    #[test]
    fn narrowing_reports_the_exact_upstream_message() {
        let error = convert_and_run("vec: [123456789123456789]").expect_err("narrows");
        assert_eq!(error.message, "Integer value to big for conversion");
    }

    #[test]
    fn tuple_and_mismatch_behaviour() {
        let (type_, value) = convert_and_run("(1,true)").expect("tuple");
        assert_eq!(
            type_,
            Type::tuple(vec![
                Type::Primitive(Prim::Int),
                Type::Primitive(Prim::Bool)
            ])
        );
        assert_eq!(value.to_string(), "(1,true)");

        let error = convert_and_run("int: \"x\"").expect_err("mismatch");
        assert!(error.message.contains("does not match required pattern"));

        let error = convert_and_run("bool: (1,2)").expect_err("no tuple coercion");
        assert!(error.message.contains("does not match"));
    }
}
