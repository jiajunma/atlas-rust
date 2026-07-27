//! Ordered evaluation of the scalar Atlas expression and command slice.

use std::{cmp::Ordering, collections::BTreeMap};

use num_bigint::{BigInt, Sign};
use num_rational::BigRational;

use crate::{
    diagnostic::{Diagnostic, ErrorKind, SourceSpan},
    formula::FormulaOperator,
    syntax::{BinaryOp, Command, Expr, PrimitiveType, Program},
    value::Value,
};

/// A value produced by one top-level program expression.
///
/// Spans are retained so a CLI, editor, or differential harness can render
/// the same ordered result stream without the evaluator writing to stdout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvalEvent {
    Value { value: Value, span: SourceSpan },
    Output { text: String, span: SourceSpan },
}

/// Mutable bindings for one explicit Atlas interpreter session.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvalContext {
    names: BTreeMap<String, BindingId>,
    bindings: Vec<Binding>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct BindingId(usize);

#[derive(Clone, Debug, Eq, PartialEq)]
struct Binding {
    value: Option<Value>,
    value_type: ScalarType,
}

impl EvalContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bindings(bindings: impl IntoIterator<Item = (String, Value)>) -> Self {
        let mut context = Self::new();
        for (name, value) in bindings {
            context.insert(name, value);
        }
        context
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        let id = self.names.get(name)?;
        self.bindings.get(id.0)?.value.as_ref()
    }

    fn binding_type(&self, name: &str) -> Option<ScalarType> {
        let id = self.names.get(name)?;
        self.bindings.get(id.0).map(|binding| binding.value_type)
    }

    pub fn insert(&mut self, name: impl Into<String>, value: Value) -> Option<Value> {
        let name = name.into();
        let previous = self.get(&name).cloned();
        let id = BindingId(self.bindings.len());
        let value_type = value_type(&value);
        self.bindings.push(Binding {
            value: Some(value),
            value_type,
        });
        self.names.insert(name, id);
        previous
    }

    fn declare(&mut self, name: impl Into<String>, value_type: ScalarType) {
        let id = BindingId(self.bindings.len());
        self.bindings.push(Binding {
            value: None,
            value_type,
        });
        self.names.insert(name.into(), id);
    }

    fn assign(&mut self, name: &str, value: Value) -> bool {
        let Some(id) = self.names.get(name).copied() else {
            return false;
        };
        let Some(binding) = self.bindings.get_mut(id.0) else {
            return false;
        };
        binding.value = Some(value);
        true
    }

    #[cfg(test)]
    fn binding_id(&self, name: &str) -> Option<BindingId> {
        self.names.get(name).copied()
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
            let value = evaluate_expression_with_context(expression, context)?;
            Ok(EvalEvent::Value {
                value,
                span: expression.span(),
            })
        })
        .collect()
}

/// Execute one parsed top-level command against persistent interpreter state.
pub fn execute_command(
    command: &Command,
    context: &mut EvalContext,
) -> Result<Vec<EvalEvent>, Diagnostic> {
    match command {
        Command::Expression(expression) => {
            let value = evaluate_expression_with_context(expression, context)?;
            Ok(vec![EvalEvent::Value {
                value,
                span: expression.span(),
            }])
        }
        Command::Define {
            name, value, span, ..
        } => {
            let value = evaluate_expression_with_context(value, context)?;
            let new_type = value_type(&value);
            let previous_type = context.binding_type(name);
            context.insert(name.clone(), value);

            let mut text = format!("Variable {name}: {}", atlas_type_name(new_type));
            if let Some(previous_type) = previous_type {
                text.push_str(&format!(
                    " (overriding previous instance, which had type {})",
                    atlas_type_name(previous_type)
                ));
            }
            text.push('\n');
            Ok(vec![EvalEvent::Output { text, span: *span }])
        }
        Command::Declare {
            name,
            value_type,
            span,
            ..
        } => {
            let value_type = scalar_type(*value_type);
            context.declare(name.clone(), value_type);
            Ok(vec![EvalEvent::Output {
                text: format!(
                    "Declaring identifier '{name}': {}\n",
                    atlas_type_name(value_type)
                ),
                span: *span,
            }])
        }
    }
}

fn evaluate_expression_with_context(
    expression: &Expr,
    context: &mut EvalContext,
) -> Result<Value, Diagnostic> {
    validate_names(expression, context)?;
    eval_expr(expression, context)
}

fn scalar_type(value_type: PrimitiveType) -> ScalarType {
    match value_type {
        PrimitiveType::Integer => ScalarType::Integer,
        PrimitiveType::Rational => ScalarType::Rational,
        PrimitiveType::String => ScalarType::String,
        PrimitiveType::Boolean => ScalarType::Boolean,
    }
}

/// Resolve and type-check every subexpression before evaluation. Atlas lowers
/// boolean operators to short-circuit conditionals only after this analysis,
/// so an unvisited branch cannot hide an unknown name or incompatible type.
pub fn validate_names(expression: &Expr, context: &EvalContext) -> Result<(), Diagnostic> {
    infer_scalar_type(expression, context).map(|_| ())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScalarType {
    Integer,
    Rational,
    Boolean,
    String,
}

fn infer_scalar_type(expression: &Expr, context: &EvalContext) -> Result<ScalarType, Diagnostic> {
    match expression {
        Expr::Identifier { name, span } => context.binding_type(name).ok_or_else(|| {
            Diagnostic::new(
                ErrorKind::Name,
                format!("undefined identifier `{name}`"),
                Some(*span),
            )
        }),
        Expr::Integer { .. } => Ok(ScalarType::Integer),
        Expr::Boolean { .. } => Ok(ScalarType::Boolean),
        Expr::String { .. } => Ok(ScalarType::String),
        Expr::Assignment {
            name,
            target_span,
            value,
            span,
        } => {
            let target_type = context.binding_type(name).ok_or_else(|| {
                Diagnostic::new(
                    ErrorKind::Name,
                    format!("undefined identifier `{name}` in assignment"),
                    Some(*target_span),
                )
            })?;
            let rhs_type = infer_scalar_type(value, context)?;
            if assignment_compatible(target_type, rhs_type) {
                Ok(target_type)
            } else {
                Err(Diagnostic::new(
                    ErrorKind::Type,
                    format!(
                        "cannot assign {} to `{name}` of type {}",
                        scalar_type_name(rhs_type),
                        scalar_type_name(target_type)
                    ),
                    Some(*span),
                ))
            }
        }
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

fn assignment_compatible(target: ScalarType, value: ScalarType) -> bool {
    target == value || (target == ScalarType::Rational && value == ScalarType::Integer)
}

fn atlas_type_name(value_type: ScalarType) -> &'static str {
    match value_type {
        ScalarType::Integer => "int",
        ScalarType::Rational => "rat",
        ScalarType::Boolean => "bool",
        ScalarType::String => "string",
    }
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
fn eval_expr(expression: &Expr, context: &mut EvalContext) -> Result<Value, Diagnostic> {
    match expression {
        Expr::Integer { value, .. } => Ok(Value::Integer(value.clone())),
        Expr::Boolean { value, .. } => Ok(Value::Boolean(*value)),
        Expr::String { value, .. } => Ok(Value::String(value.clone())),
        Expr::Identifier { name, span } => match context.binding_type(name) {
            None => Err(Diagnostic::new(
                ErrorKind::Name,
                format!("undefined identifier `{name}`"),
                Some(*span),
            )),
            Some(_) => context.get(name).cloned().ok_or_else(|| {
                Diagnostic::new(
                    ErrorKind::Runtime,
                    format!("Taking value of uninitialized variable '{name}'"),
                    Some(*span),
                )
            }),
        },
        Expr::Assignment {
            name, value, span, ..
        } => {
            let target_type = context.binding_type(name).ok_or_else(|| {
                Diagnostic::new(
                    ErrorKind::Name,
                    format!("undefined identifier `{name}` in assignment"),
                    Some(*span),
                )
            })?;
            let value = eval_expr(value, context)?;
            let value = coerce_assignment_value(target_type, value, *span)?;
            debug_assert!(context.assign(name, value.clone()));
            Ok(value)
        }
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

fn coerce_assignment_value(
    target_type: ScalarType,
    value: Value,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    match (target_type, value) {
        (ScalarType::Rational, Value::Integer(value)) => {
            Ok(Value::Rational(BigRational::from_integer(value)))
        }
        (ScalarType::Integer, Value::Integer(value)) => Ok(Value::Integer(value)),
        (ScalarType::Rational, Value::Rational(value)) => Ok(Value::Rational(value)),
        (ScalarType::Boolean, Value::Boolean(value)) => Ok(Value::Boolean(value)),
        (ScalarType::String, Value::String(value)) => Ok(Value::String(value)),
        (target_type, value) => Err(Diagnostic::new(
            ErrorKind::Type,
            format!(
                "cannot assign {} to a {}",
                type_name(&value),
                scalar_type_name(target_type)
            ),
            Some(span),
        )),
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
                EvalEvent::Output { .. } => unreachable!("program evaluation emits values only"),
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
    fn assignment_updates_a_binding_cell_but_definition_rebinds_the_name() {
        let mut context = EvalContext::with_bindings([(String::from("x"), integer(1))]);
        let original = context.binding_id("x").expect("initial binding");
        let assignment = parse(&SourceText::new("x := 2"))
            .expect("assignment parses")
            .expressions
            .remove(0);
        evaluate_with_context(
            &Program {
                expressions: vec![assignment],
            },
            &mut context,
        )
        .expect("assignment evaluates");
        assert_eq!(context.binding_id("x"), Some(original));
        assert_eq!(context.get("x"), Some(&integer(2)));

        let definition = parse(&SourceText::new("x"))
            .expect("identifier parses")
            .expressions
            .remove(0);
        let value = eval_expr(&definition, &mut context).expect("read evaluates");
        context.insert("x", value);
        assert_ne!(context.binding_id("x"), Some(original));
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
                EvalEvent::Output { .. } => unreachable!("program evaluation emits values only"),
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
                EvalEvent::Output { .. } => unreachable!("program evaluation emits values only"),
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
                EvalEvent::Output { .. } => unreachable!("program evaluation emits values only"),
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
                EvalEvent::Output { .. } => unreachable!("program evaluation emits values only"),
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
                EvalEvent::Output { .. } => unreachable!("program evaluation emits values only"),
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
