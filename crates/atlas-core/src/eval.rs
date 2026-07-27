//! Ordered, side-effect-free evaluation of the scalar Atlas expression slice.

use std::{cmp::Ordering, collections::BTreeMap};

use num_bigint::{BigInt, Sign};
use num_rational::BigRational;

use crate::{
    diagnostic::{Diagnostic, ErrorKind, SourceSpan},
    formula::FormulaOperator,
    syntax::{BinaryOp, Expr, Program},
    value::Value,
};

/// A value produced by one top-level program expression.
///
/// Spans are retained so a CLI, editor, or differential harness can render
/// the same ordered result stream without the evaluator writing to stdout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvalEvent {
    Value { value: Value, span: SourceSpan },
}

/// Mutable bindings for a future command/evaluator layer.
///
/// The current expression grammar has no assignment form, but keeping
/// bindings explicit now avoids introducing global mutable state later.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvalContext {
    bindings: BTreeMap<String, Value>,
}

impl EvalContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bindings(bindings: impl IntoIterator<Item = (String, Value)>) -> Self {
        Self {
            bindings: bindings.into_iter().collect(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    pub fn insert(&mut self, name: impl Into<String>, value: Value) -> Option<Value> {
        self.bindings.insert(name.into(), value)
    }
}

/// Evaluate every top-level expression in source order.
pub fn evaluate(program: &Program) -> Result<Vec<EvalEvent>, Diagnostic> {
    let mut context = EvalContext::new();
    evaluate_with_context(program, &mut context)
}

/// Evaluate a program using explicit bindings and preserve top-level order.
pub fn evaluate_with_context(
    program: &Program,
    context: &mut EvalContext,
) -> Result<Vec<EvalEvent>, Diagnostic> {
    program
        .expressions
        .iter()
        .map(|expression| {
            validate_names(expression, context)?;
            let value = eval_expr(expression, context)?;
            Ok(EvalEvent::Value {
                value,
                span: expression.span(),
            })
        })
        .collect()
}

/// Resolve and type-check every subexpression before evaluation. Atlas lowers
/// boolean operators to short-circuit conditionals only after this analysis,
/// so an unvisited branch cannot hide an unknown name or incompatible type.
pub fn validate_names(expression: &Expr, context: &EvalContext) -> Result<(), Diagnostic> {
    infer_scalar_type(expression, context).map(|_| ())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScalarType {
    Integer,
    Rational,
    Boolean,
    String,
}

fn infer_scalar_type(expression: &Expr, context: &EvalContext) -> Result<ScalarType, Diagnostic> {
    match expression {
        Expr::Identifier { name, span } => context.get(name).map(value_type).ok_or_else(|| {
            Diagnostic::new(
                ErrorKind::Name,
                format!("undefined identifier `{name}`"),
                Some(*span),
            )
        }),
        Expr::Integer { .. } => Ok(ScalarType::Integer),
        Expr::Boolean { .. } => Ok(ScalarType::Boolean),
        Expr::String { .. } => Ok(ScalarType::String),
        Expr::Group { inner, .. } => infer_scalar_type(inner, context),
        Expr::Unary { operand, span, .. } => {
            let operand_type = infer_scalar_type(operand, context)?;
            if operand_type == ScalarType::Boolean {
                Ok(ScalarType::Boolean)
            } else {
                Err(static_type_error("not", "boolean", operand_type, *span))
            }
        }
        Expr::Binary {
            op: op @ (BinaryOp::And | BinaryOp::Or),
            lhs,
            rhs,
            span,
        } => {
            let left = infer_scalar_type(lhs, context)?;
            if left != ScalarType::Boolean {
                return Err(static_type_error(binary_name(*op), "boolean", left, *span));
            }
            let right = infer_scalar_type(rhs, context)?;
            if right != ScalarType::Boolean {
                return Err(static_type_error(binary_name(*op), "boolean", right, *span));
            }
            Ok(ScalarType::Boolean)
        }
        Expr::OperatorCall {
            operator,
            arguments,
            span,
        } => {
            let mut argument_types = Vec::with_capacity(arguments.len());
            for argument in arguments {
                argument_types.push(infer_scalar_type(argument, context)?);
            }
            infer_operator_type(operator, &argument_types, *span)
        }
    }
}

fn infer_operator_type(
    operator: &FormulaOperator,
    arguments: &[ScalarType],
    span: SourceSpan,
) -> Result<ScalarType, Diagnostic> {
    let symbol = operator.symbol.as_str();
    match (symbol, arguments) {
        ("-" | "+", [value]) if is_numeric(*value) => Ok(*value),
        ("-" | "+", [value]) => Err(static_type_error(symbol, "numeric", *value, span)),
        ("+" | "-" | "*", [left, right]) if is_numeric(*left) && is_numeric(*right) => {
            Ok(promoted_numeric_type(*left, *right))
        }
        ("/", [left, right]) if is_numeric(*left) && is_numeric(*right) => Ok(ScalarType::Rational),
        ("%" | "\\", [ScalarType::Integer, ScalarType::Integer]) => Ok(ScalarType::Integer),
        ("^", [ScalarType::Integer, ScalarType::Integer]) => Ok(ScalarType::Integer),
        ("&", [ScalarType::Integer, ScalarType::Integer]) => Ok(ScalarType::Integer),
        ("=", [left, right]) | ("!=", [left, right])
            if left == right || (is_numeric(*left) && is_numeric(*right)) =>
        {
            Ok(ScalarType::Boolean)
        }
        ("<" | "<=" | ">" | ">=", [left, right])
            if (is_numeric(*left) && is_numeric(*right))
                || (*left == ScalarType::String && *right == ScalarType::String) =>
        {
            Ok(ScalarType::Boolean)
        }
        ("##", [ScalarType::String, ScalarType::String]) => Ok(ScalarType::String),
        _ => Err(operator_type_error(operator, arguments, span)),
    }
}

fn promoted_numeric_type(left: ScalarType, right: ScalarType) -> ScalarType {
    if left == ScalarType::Rational || right == ScalarType::Rational {
        ScalarType::Rational
    } else {
        ScalarType::Integer
    }
}

fn is_numeric(value_type: ScalarType) -> bool {
    matches!(value_type, ScalarType::Integer | ScalarType::Rational)
}

fn value_type(value: &Value) -> ScalarType {
    match value {
        Value::Integer(_) => ScalarType::Integer,
        Value::Rational(_) => ScalarType::Rational,
        Value::Boolean(_) => ScalarType::Boolean,
        Value::String(_) => ScalarType::String,
    }
}

fn static_type_error(
    operator: &str,
    expected: &str,
    actual: ScalarType,
    span: SourceSpan,
) -> Diagnostic {
    Diagnostic::new(
        ErrorKind::Type,
        format!(
            "operator `{operator}` expects {expected}, got {}",
            scalar_type_name(actual)
        ),
        Some(span),
    )
}

fn operator_type_error(
    operator: &FormulaOperator,
    arguments: &[ScalarType],
    span: SourceSpan,
) -> Diagnostic {
    let actual = arguments
        .iter()
        .map(|argument| scalar_type_name(*argument))
        .collect::<Vec<_>>()
        .join(", ");
    Diagnostic::new(
        ErrorKind::Type,
        format!(
            "no scalar overload for operator `{}` with ({actual})",
            operator.symbol
        ),
        Some(span),
    )
}

fn scalar_type_name(value_type: ScalarType) -> &'static str {
    match value_type {
        ScalarType::Integer => "integer",
        ScalarType::Rational => "rational",
        ScalarType::Boolean => "boolean",
        ScalarType::String => "string",
    }
}

/// Evaluate one expression against an explicit context.
fn eval_expr(expression: &Expr, context: &EvalContext) -> Result<Value, Diagnostic> {
    match expression {
        Expr::Integer { value, .. } => Ok(Value::Integer(value.clone())),
        Expr::Boolean { value, .. } => Ok(Value::Boolean(*value)),
        Expr::String { value, .. } => Ok(Value::String(value.clone())),
        Expr::Identifier { name, span } => context.get(name).cloned().ok_or_else(|| {
            Diagnostic::new(
                ErrorKind::Name,
                format!("undefined identifier `{name}`"),
                Some(*span),
            )
        }),
        Expr::Group { inner, .. } => eval_expr(inner, context),
        Expr::Unary { operand, span, .. } => {
            let value = eval_expr(operand, context)?;
            match value {
                Value::Boolean(value) => Ok(Value::Boolean(!value)),
                other => Err(type_error("not", &["boolean"], &other, *span)),
            }
        }
        Expr::Binary { op, lhs, rhs, span } => {
            // Keep short-circuit behavior explicit: the right operand is not
            // evaluated for a decisive boolean `and`/`or` result.
            let left = eval_expr(lhs, context)?;
            match op {
                BinaryOp::And => match left {
                    Value::Boolean(false) => Ok(Value::Boolean(false)),
                    Value::Boolean(true) => match eval_expr(rhs, context)? {
                        Value::Boolean(value) => Ok(Value::Boolean(value)),
                        other => Err(type_error("and", &["boolean"], &other, *span)),
                    },
                    other => Err(type_error("and", &["boolean"], &other, *span)),
                },
                BinaryOp::Or => match left {
                    Value::Boolean(true) => Ok(Value::Boolean(true)),
                    Value::Boolean(false) => match eval_expr(rhs, context)? {
                        Value::Boolean(value) => Ok(Value::Boolean(value)),
                        other => Err(type_error("or", &["boolean"], &other, *span)),
                    },
                    other => Err(type_error("or", &["boolean"], &other, *span)),
                },
            }
        }
        Expr::OperatorCall {
            operator,
            arguments,
            span,
        } => {
            let mut values = Vec::with_capacity(arguments.len());
            for argument in arguments {
                values.push(eval_expr(argument, context)?);
            }
            eval_operator_call(operator, values, *span)
        }
    }
}

fn eval_operator_call(
    operator: &FormulaOperator,
    arguments: Vec<Value>,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let symbol = operator.symbol.as_str();
    match (symbol, arguments.as_slice()) {
        ("-", [Value::Integer(value)]) => Ok(Value::Integer(-value.clone())),
        ("-", [Value::Rational(value)]) => Ok(Value::Rational(-value.clone())),
        ("+", [value]) => Ok(value.clone()),
        ("+" | "-" | "*", [left, right]) => {
            eval_exact_arithmetic(symbol, left.clone(), right.clone(), span)
        }
        ("/", [left, right]) => eval_exact_division(left.clone(), right.clone(), span),
        ("=", [left, right]) => Ok(Value::Boolean(
            numeric_cmp(left, right)
                .map(|ordering| ordering == Ordering::Equal)
                .unwrap_or(left == right),
        )),
        ("!=", [left, right]) => Ok(Value::Boolean(
            !numeric_cmp(left, right)
                .map(|ordering| ordering == Ordering::Equal)
                .unwrap_or(left == right),
        )),
        ("<" | "<=" | ">" | ">=", [left, right]) => {
            let ordering = numeric_cmp(left, right).or_else(|| match (left, right) {
                (Value::String(left), Value::String(right)) => Some(left.cmp(right)),
                _ => None,
            });
            let Some(ordering) = ordering else {
                return Err(operator_type_error(
                    operator,
                    &[value_type(left), value_type(right)],
                    span,
                ));
            };
            let result = match symbol {
                "<" => ordering == Ordering::Less,
                "<=" => ordering != Ordering::Greater,
                ">" => ordering == Ordering::Greater,
                ">=" => ordering != Ordering::Less,
                _ => unreachable!(),
            };
            Ok(Value::Boolean(result))
        }
        ("##", [Value::String(left), Value::String(right)]) => {
            Ok(Value::String(format!("{left}{right}")))
        }
        ("%", [Value::Integer(left), Value::Integer(right)]) => {
            if right == &BigInt::from(0) {
                Err(Diagnostic::new(
                    ErrorKind::Runtime,
                    "division by zero",
                    Some(span),
                ))
            } else {
                Ok(Value::Integer(euclidean_divmod(left, right).1))
            }
        }
        ("\\", [Value::Integer(left), Value::Integer(right)]) => {
            if right == &BigInt::from(0) {
                Err(Diagnostic::new(
                    ErrorKind::Runtime,
                    "division by zero",
                    Some(span),
                ))
            } else {
                Ok(Value::Integer(euclidean_divmod(left, right).0))
            }
        }
        ("&", [Value::Integer(left), Value::Integer(right)]) => Ok(Value::Integer(left & right)),
        ("^", [Value::Integer(left), Value::Integer(right)]) => {
            let Ok(exponent) = right.clone().try_into() else {
                return Err(Diagnostic::new(
                    ErrorKind::Type,
                    "exponent must be a non-negative machine-sized integer",
                    Some(span),
                ));
            };
            Ok(Value::Integer(left.pow(exponent)))
        }
        _ => {
            let types = arguments.iter().map(value_type).collect::<Vec<_>>();
            Err(operator_type_error(operator, &types, span))
        }
    }
}

/// Match Atlas's sign-aware Euclidean quotient and remainder rather than
/// Rust's truncating integer division.
fn euclidean_divmod(left: &BigInt, right: &BigInt) -> (BigInt, BigInt) {
    let mut quotient = left / right;
    let mut remainder = left % right;
    let opposite_signs = matches!(
        (remainder.sign(), right.sign()),
        (Sign::Minus, Sign::Plus) | (Sign::Plus, Sign::Minus)
    );
    if remainder != BigInt::from(0) && opposite_signs {
        quotient -= 1;
        remainder += right;
    }
    (quotient, remainder)
}

fn eval_exact_arithmetic(
    op: &str,
    left: Value,
    right: Value,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let value = match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => Value::Integer(match op {
            "+" => left + right,
            "-" => left - right,
            "*" => left * right,
            _ => unreachable!("only exact arithmetic operators reach this helper"),
        }),
        (Value::Integer(left), Value::Rational(right)) => {
            let left = BigRational::from_integer(left);
            Value::Rational(match op {
                "+" => left + right,
                "-" => left - right,
                "*" => left * right,
                _ => unreachable!("only exact arithmetic operators reach this helper"),
            })
        }
        (Value::Rational(left), Value::Integer(right)) => {
            let right = BigRational::from_integer(right);
            Value::Rational(match op {
                "+" => left + right,
                "-" => left - right,
                "*" => left * right,
                _ => unreachable!("only exact arithmetic operators reach this helper"),
            })
        }
        (Value::Rational(left), Value::Rational(right)) => Value::Rational(match op {
            "+" => left + right,
            "-" => left - right,
            "*" => left * right,
            _ => unreachable!("only exact arithmetic operators reach this helper"),
        }),
        (left, right) => return Err(numeric_type_error(op, &left, &right, span)),
    };
    Ok(value)
}

fn eval_exact_division(left: Value, right: Value, span: SourceSpan) -> Result<Value, Diagnostic> {
    let (left, right) = match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => (
            BigRational::from_integer(left),
            BigRational::from_integer(right),
        ),
        (Value::Integer(left), Value::Rational(right)) => (BigRational::from_integer(left), right),
        (Value::Rational(left), Value::Integer(right)) => (left, BigRational::from_integer(right)),
        (Value::Rational(left), Value::Rational(right)) => (left, right),
        (left, right) => {
            return Err(numeric_type_error("/", &left, &right, span));
        }
    };

    if right.numer() == &BigInt::from(0) {
        return Err(Diagnostic::new(
            ErrorKind::Runtime,
            "division by zero",
            Some(span),
        ));
    }
    Ok(Value::Rational(left / right))
}

fn numeric_cmp(left: &Value, right: &Value) -> Option<Ordering> {
    match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => Some(left.cmp(right)),
        (Value::Integer(left), Value::Rational(right)) => {
            Some(BigRational::from_integer(left.clone()).cmp(right))
        }
        (Value::Rational(left), Value::Integer(right)) => {
            Some(left.cmp(&BigRational::from_integer(right.clone())))
        }
        (Value::Rational(left), Value::Rational(right)) => Some(left.cmp(right)),
        _ => None,
    }
}

fn numeric_type_error(op: &str, left: &Value, right: &Value, span: SourceSpan) -> Diagnostic {
    Diagnostic::new(
        ErrorKind::Type,
        format!(
            "operator `{}` expects two numeric values (integer or rational), got {} and {}",
            op,
            type_name(left),
            type_name(right)
        ),
        Some(span),
    )
}

fn type_error(operator: &str, expected: &[&str], actual: &Value, span: SourceSpan) -> Diagnostic {
    Diagnostic::new(
        ErrorKind::Type,
        format!(
            "operator `{operator}` expects {}, got {}",
            expected.join(" or "),
            type_name(actual)
        ),
        Some(span),
    )
}

fn binary_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
    }
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Integer(_) => "integer",
        Value::Rational(_) => "rational",
        Value::Boolean(_) => "boolean",
        Value::String(_) => "string",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{source::SourceText, syntax::parse};
    use num_bigint::BigInt;
    use num_rational::BigRational;

    fn integer(value: impl Into<BigInt>) -> Value {
        Value::Integer(value.into())
    }

    fn rational(numerator: impl Into<BigInt>, denominator: impl Into<BigInt>) -> Value {
        Value::Rational(BigRational::new(numerator.into(), denominator.into()))
    }

    fn run(source: &str) -> Result<Vec<EvalEvent>, Diagnostic> {
        let source = SourceText::new(source);
        let program = parse(&source).expect("test source parses");
        evaluate(&program)
    }

    #[test]
    fn evaluates_scalar_fixture_in_source_order() {
        let source = include_str!("../../../tests/fixtures/eval/scalars.atlas");
        let events = run(source).expect("scalar fixture evaluates");
        let values: Vec<_> = events
            .into_iter()
            .map(|event| match event {
                EvalEvent::Value { value, .. } => value,
            })
            .collect();
        assert_eq!(
            values,
            vec![
                integer(42),
                Value::Boolean(true),
                Value::String("atlas".into()),
                rational(7, 1),
                Value::Boolean(true),
                Value::Boolean(true),
            ]
        );
    }

    #[test]
    fn evaluates_explicit_bindings_without_stdout_side_effects() {
        let source = SourceText::new(include_str!("../../../tests/fixtures/eval/context.atlas"));
        let program = parse(&source).expect("context fixture parses");
        let mut context = EvalContext::with_bindings([(String::from("answer"), integer(41))]);
        let events = evaluate_with_context(&program, &mut context).expect("context evaluates");
        assert_eq!(
            events,
            vec![
                EvalEvent::Value {
                    value: integer(41),
                    span: program.expressions[0].span()
                },
                EvalEvent::Value {
                    value: integer(42),
                    span: program.expressions[1].span()
                },
            ]
        );
    }

    #[test]
    fn reports_undefined_identifier_with_name_diagnostic() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/negative_undefined.atlas"
        ));
        let program = parse(&source).expect("negative fixture parses");
        let error = evaluate(&program).expect_err("missing identifier is rejected");
        assert_eq!(error.kind, ErrorKind::Name);
        assert!(error.message.contains("missing"));
        assert_eq!(error.span, Some(program.expressions[0].span()));
    }

    #[test]
    fn reports_type_error_with_type_diagnostic() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/negative_type.atlas"
        ));
        let program = parse(&source).expect("negative fixture parses");
        let error = evaluate(&program).expect_err("mixed arithmetic is rejected");
        assert_eq!(error.kind, ErrorKind::Type);
        assert!(error.message.contains("integer"));
    }

    #[test]
    fn resolves_names_before_boolean_short_circuiting() {
        let source = SourceText::new("false and missing");
        let program = parse(&source).expect("short-circuit source parses");
        let error = evaluate(&program).expect_err("unvisited names are still resolved");
        assert_eq!(error.kind, ErrorKind::Name);
    }

    #[test]
    fn short_circuits_runtime_evaluation_after_validation() {
        let source = SourceText::new("false and ((1 / 0) = 0)\ntrue or ((1 / 0) = 0)");
        let program = parse(&source).expect("short-circuit source parses");
        let events = evaluate(&program).expect("unvisited runtime errors are short-circuited");
        assert_eq!(
            events,
            vec![
                EvalEvent::Value {
                    value: Value::Boolean(false),
                    span: program.expressions[0].span()
                },
                EvalEvent::Value {
                    value: Value::Boolean(true),
                    span: program.expressions[1].span()
                },
            ]
        );
    }

    #[test]
    fn type_checks_unvisited_boolean_branches() {
        let source = SourceText::new("false and 1");
        let program = parse(&source).expect("source parses");
        let error = evaluate(&program).expect_err("non-boolean branch is rejected");
        assert_eq!(error.kind, ErrorKind::Type);
    }

    #[test]
    fn reports_left_boolean_type_before_resolving_right_name() {
        let source = SourceText::new("1 and missing");
        let program = parse(&source).expect("source parses");
        let error = evaluate(&program).expect_err("left condition is rejected first");
        assert_eq!(error.kind, ErrorKind::Type);
    }

    #[test]
    fn rejects_equality_without_a_compatible_overload() {
        let source = SourceText::new("true = 1");
        let program = parse(&source).expect("source parses");
        let error = evaluate(&program).expect_err("mixed equality is rejected");
        assert_eq!(error.kind, ErrorKind::Type);
    }

    #[test]
    fn reports_division_by_zero_as_runtime_error() {
        let source = SourceText::new("1 / 0");
        let program = parse(&source).expect("division source parses");
        let error = evaluate(&program).expect_err("division by zero is rejected");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert!(error.message.contains("division by zero"));
    }

    #[test]
    fn evaluates_exact_integer_and_rational_fixture() {
        let source = include_str!("../../../tests/fixtures/eval/exact_numerics.atlas");
        let events = run(source).expect("exact numeric fixture evaluates");
        let values: Vec<_> = events
            .into_iter()
            .map(|event| match event {
                EvalEvent::Value { value, .. } => value,
            })
            .collect();

        assert_eq!(
            values,
            vec![
                integer(
                    "1234567890123456789012345678901234567890"
                        .parse::<BigInt>()
                        .expect("valid expected integer"),
                ),
                integer(
                    "-1234567890123456789012345678901234567890"
                        .parse::<BigInt>()
                        .expect("valid expected integer"),
                ),
                rational(-1, 2),
                integer(
                    "1000000000000000000000000000000000000000000000000000000000000"
                        .parse::<BigInt>()
                        .expect("valid expected integer"),
                ),
                integer(1),
                rational(1, 3),
                rational(1, 1),
                Value::Boolean(true),
                Value::Boolean(true),
                rational(5, 6),
                rational(2, 1),
            ]
        );
    }

    #[test]
    fn evaluates_symbolic_operator_calls_by_name_and_arity() {
        let source = SourceText::new("2 ^ 3\n4 % 3\n4 \\ 3\n1 <= 1\n1 != 2\n\"a\" ## \"b\"");
        let program = parse(&source).expect("symbolic operators parse");
        let events = evaluate(&program).expect("symbolic operators evaluate");
        let values: Vec<_> = events
            .into_iter()
            .map(|event| match event {
                EvalEvent::Value { value, .. } => value,
            })
            .collect();
        assert_eq!(
            values,
            vec![
                integer(8),
                integer(1),
                integer(1),
                Value::Boolean(true),
                Value::Boolean(true),
                Value::String("ab".into()),
            ]
        );
    }

    #[test]
    fn rejects_divmod_until_tuple_values_are_available() {
        let source = SourceText::new("1 \\% 2");
        let program = parse(&source).expect("divmod parses");
        let error = evaluate(&program).expect_err("tuple-valued divmod is not scalar");
        assert_eq!(error.kind, ErrorKind::Type);
    }

    #[test]
    fn uses_atlas_euclidean_integer_quotients_and_remainders() {
        let events = run("(-7) \\ 3\n(-7) % 3\n7 \\ -3\n7 % -3")
            .expect("integer quotient and remainder evaluate");
        let values: Vec<_> = events
            .into_iter()
            .map(|event| match event {
                EvalEvent::Value { value, .. } => value,
            })
            .collect();
        assert_eq!(
            values,
            vec![integer(-3), integer(2), integer(-3), integer(-2)]
        );
    }

    #[test]
    fn displays_rationals_with_an_explicit_denominator() {
        assert_eq!(rational(2, 1).to_string(), "2/1");
        assert_eq!(rational(-2, -4).to_string(), "1/2");
    }

    #[test]
    fn promotes_mixed_integer_and_rational_operands() {
        let events = run("1 + 1 / 2\n\
             1 / 2 + 1\n\
             1 - 1 / 2\n\
             1 / 2 - 1\n\
             2 * (1 / 3)\n\
             (1 / 3) * 2\n\
             1 / (1 / 2)\n\
             (1 / 2) / 1\n\
             1 / 2 = 2 / 4\n\
             3 / 2 > 1")
        .expect("mixed exact numerics evaluate");
        let values: Vec<_> = events
            .into_iter()
            .map(|event| match event {
                EvalEvent::Value { value, .. } => value,
            })
            .collect();
        assert_eq!(
            values,
            vec![
                rational(3, 2),
                rational(3, 2),
                rational(1, 2),
                rational(-1, 2),
                rational(2, 3),
                rational(2, 3),
                rational(2, 1),
                rational(1, 2),
                Value::Boolean(true),
                Value::Boolean(true),
            ]
        );
    }

    #[test]
    fn rejects_division_by_a_rational_zero() {
        let error = run("1 / (0 / 1)").expect_err("rational zero divisor is rejected");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert!(error.message.contains("division by zero"));
    }

    #[test]
    fn evaluates_unary_minus_for_integer_and_rational_values() {
        let events = run("-123456789012345678901234567890\n-(1 / 2)\n-1 / 2")
            .expect("unary numeric negation evaluates");
        let values: Vec<_> = events
            .into_iter()
            .map(|event| match event {
                EvalEvent::Value { value, .. } => value,
            })
            .collect();
        assert_eq!(
            values,
            vec![
                integer(
                    "-123456789012345678901234567890"
                        .parse::<BigInt>()
                        .expect("valid expected integer"),
                ),
                rational(-1, 2),
                rational(-1, 2),
            ]
        );
    }
}
