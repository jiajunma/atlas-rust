//! Ordered evaluation of the scalar Atlas expression and command slice.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use malachite::base::num::arithmetic::traits::Pow;
use malachite::base::num::conversion::traits::{ConvertibleFrom, WrappingFrom};
use malachite::{Integer as BigInt, Rational as BigRational};

use crate::{
    diagnostic::{Diagnostic, ErrorKind, SourceSpan},
    formula::FormulaOperator,
    syntax::{BinaryOp, Command, Expr, LetBinding, Prim, Program, SliceFlags},
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
    scopes: Vec<LocalScope>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct BindingId(usize);

#[derive(Clone, Debug, Eq, PartialEq)]
struct Binding {
    value: Option<Value>,
    value_type: ScalarType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalScope {
    names: BTreeMap<String, BindingId>,
    binding_start: usize,
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
        let id = self.resolve_binding_id(name)?;
        self.bindings.get(id.0)?.value.as_ref()
    }

    fn binding_type(&self, name: &str) -> Option<ScalarType> {
        let id = self.resolve_binding_id(name)?;
        self.bindings
            .get(id.0)
            .map(|binding| binding.value_type.clone())
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
        let Some(id) = self.resolve_binding_id(name) else {
            return false;
        };
        let Some(binding) = self.bindings.get_mut(id.0) else {
            return false;
        };
        binding.value = Some(value);
        true
    }

    fn push_scope(&mut self) {
        self.scopes.push(LocalScope {
            names: BTreeMap::new(),
            binding_start: self.bindings.len(),
        });
    }

    fn pop_scope(&mut self) {
        let scope = self.scopes.pop().expect("local scope stack is balanced");
        self.bindings.truncate(scope.binding_start);
    }

    fn bind_local(
        &mut self,
        name: impl Into<String>,
        value_type: ScalarType,
        value: Option<Value>,
    ) {
        let id = BindingId(self.bindings.len());
        self.bindings.push(Binding { value, value_type });
        self.scopes
            .last_mut()
            .expect("a local binding requires an active scope")
            .names
            .insert(name.into(), id);
    }

    fn resolve_binding_id(&self, name: &str) -> Option<BindingId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.names.get(name).copied())
            .or_else(|| self.names.get(name).copied())
    }

    #[cfg(test)]
    fn binding_id(&self, name: &str) -> Option<BindingId> {
        self.resolve_binding_id(name)
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

            let mut text = format!("Variable {name}: {}", atlas_type_name(&new_type));
            if let Some(previous_type) = previous_type {
                text.push_str(&format!(
                    " (overriding previous instance, which had type {})",
                    atlas_type_name(&previous_type)
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
            let Some(value_type) = scalar_type(*value_type) else {
                return Err(Diagnostic::new(
                    ErrorKind::Type,
                    format!(
                        "declared type '{}' is not yet supported by this evaluator",
                        value_type.name()
                    ),
                    Some(*span),
                ));
            };
            context.declare(name.clone(), value_type.clone());
            Ok(vec![EvalEvent::Output {
                text: format!(
                    "Declaring identifier '{name}': {}\n",
                    atlas_type_name(&value_type)
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

/// The dynamic-evaluator view of a declared primitive. Only the scalar four
/// are checkable here; the rest wait for the typed pipeline (phase B stage
/// B2), and a declaration using them reports a clear diagnostic instead.
fn scalar_type(value_type: Prim) -> Option<ScalarType> {
    match value_type {
        Prim::Int => Some(ScalarType::Integer),
        Prim::Rat => Some(ScalarType::Rational),
        Prim::String => Some(ScalarType::String),
        Prim::Bool => Some(ScalarType::Boolean),
        _ => None,
    }
}

/// Resolve and type-check every subexpression before evaluation. Atlas lowers
/// boolean operators to short-circuit conditionals only after this analysis,
/// so an unvisited branch cannot hide an unknown name or incompatible type.
pub fn validate_names(expression: &Expr, context: &EvalContext) -> Result<(), Diagnostic> {
    infer_scalar_type(expression, context).map(|_| ())
}

/// This evaluator's view of a structural type — a lossy bridge that retires
/// with the typed pipeline (phase B stage B2).
fn scalar_view_of_type(target: &crate::types::Type) -> ScalarType {
    use crate::types::Type;
    match target {
        Type::Primitive(Prim::Int) => ScalarType::Integer,
        Type::Primitive(Prim::Rat) => ScalarType::Rational,
        Type::Primitive(Prim::String) => ScalarType::String,
        Type::Primitive(Prim::Bool) => ScalarType::Boolean,
        Type::Row(component) => {
            let inner = scalar_view_of_type(component);
            if inner == ScalarType::Unknown {
                ScalarType::List(None)
            } else {
                ScalarType::List(Some(Box::new(inner)))
            }
        }
        Type::Tuple(components) => {
            ScalarType::Tuple(components.iter().map(scalar_view_of_type).collect())
        }
        _ => ScalarType::Unknown,
    }
}

/// Dynamic cast semantics until the typed pipeline lands: check the value
/// against the target structurally, applying the int-to-rat promotion that
/// upstream reaches through its coercion table.
fn cast_value(
    value: Value,
    target: &crate::types::Type,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    use crate::types::Type;
    let mismatch = |value: &Value| {
        Diagnostic::new(
            ErrorKind::Type,
            format!("cannot cast value {value} to the requested type"),
            Some(span),
        )
    };
    match (target, value) {
        (Type::Undetermined, value) => Ok(value),
        (Type::Primitive(Prim::Int), Value::Integer(value)) => Ok(Value::Integer(value)),
        (Type::Primitive(Prim::Rat), Value::Integer(value)) => {
            Ok(Value::Rational(BigRational::from(value)))
        }
        (Type::Primitive(Prim::Rat), Value::Rational(value)) => Ok(Value::Rational(value)),
        (Type::Primitive(Prim::String), Value::String(value)) => Ok(Value::String(value)),
        (Type::Primitive(Prim::Bool), Value::Boolean(value)) => Ok(Value::Boolean(value)),
        (Type::Row(component), Value::List(elements)) => Ok(Value::List(
            elements
                .into_iter()
                .map(|element| cast_value(element, component, span))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        (Type::Tuple(components), Value::Tuple(elements)) if components.len() == elements.len() => {
            Ok(Value::Tuple(
                elements
                    .into_iter()
                    .zip(components)
                    .map(|(element, component)| cast_value(element, component, span))
                    .collect::<Result<Vec<_>, _>>()?,
            ))
        }
        (Type::Primitive(_) | Type::Row(_) | Type::Tuple(_), value) => Err(mismatch(&value)),
        (_, value) => Err(Diagnostic::new(
            ErrorKind::Type,
            format!("cast target is not yet supported by this evaluator (value {value})"),
            Some(span),
        )),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ScalarType {
    Unknown,
    UnknownNumeric,
    Integer,
    Rational,
    Boolean,
    String,
    Tuple(Vec<ScalarType>),
    List(Option<Box<ScalarType>>),
    /// An opaque domain handle, named by its language-facing kind.
    Domain(&'static str),
}

fn infer_scalar_type(expression: &Expr, context: &EvalContext) -> Result<ScalarType, Diagnostic> {
    match expression {
        Expr::Cast { target, body, .. } => {
            // Validate the body, then report the cast's own type as far as
            // this evaluator can express it.
            infer_scalar_type(body, context)?;
            Ok(scalar_view_of_type(&target.resolve()))
        }
        Expr::Call { callee, span, .. } => match callee.as_ref() {
            Expr::Identifier { .. } => Ok(ScalarType::Unknown),
            _ => Err(Diagnostic::new(
                ErrorKind::Type,
                "only named functions can be applied".to_string(),
                Some(*span),
            )),
        },
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
        Expr::Tuple { elements, .. } => elements
            .iter()
            .map(|element| infer_scalar_type(element, context))
            .collect::<Result<Vec<_>, _>>()
            .map(ScalarType::Tuple),
        Expr::List { elements, span } => {
            let element_types = elements
                .iter()
                .map(|element| infer_scalar_type(element, context))
                .collect::<Result<Vec<_>, _>>()?;
            let element_type = common_list_element_type(&element_types).ok_or_else(|| {
                Diagnostic::new(
                    ErrorKind::Type,
                    "list elements must have a common type",
                    Some(*span),
                )
            })?;
            Ok(ScalarType::List(element_type.map(Box::new)))
        }
        Expr::Subscription {
            array, index, span, ..
        } => {
            let array_type = infer_scalar_type(array, context)?;
            let index_type = infer_scalar_type(index, context)?;
            match (&array_type, &index_type) {
                (ScalarType::List(Some(element_type)), ScalarType::Integer) => {
                    Ok((**element_type).clone())
                }
                (ScalarType::List(None), ScalarType::Integer) => Ok(ScalarType::Unknown),
                _ => Err(subscription_type_error(&array_type, &index_type, *span)),
            }
        }
        Expr::Slice {
            array,
            lower,
            upper,
            span,
            ..
        } => {
            let array_type = infer_scalar_type(array, context)?;
            let lower_type = infer_scalar_type(lower, context)?;
            let upper_type = infer_scalar_type(upper, context)?;
            if !is_integer_or_unknown(&lower_type) || !is_integer_or_unknown(&upper_type) {
                return Err(slice_type_error(
                    &array_type,
                    &lower_type,
                    &upper_type,
                    *span,
                ));
            }
            if matches!(&array_type, ScalarType::List(_)) {
                Ok(array_type)
            } else {
                Err(slice_type_error(
                    &array_type,
                    &lower_type,
                    &upper_type,
                    *span,
                ))
            }
        }
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
            if assignment_compatible(&target_type, &rhs_type) {
                Ok(target_type)
            } else {
                Err(Diagnostic::new(
                    ErrorKind::Type,
                    format!(
                        "cannot assign {} to `{name}` of type {}",
                        scalar_type_name(&rhs_type),
                        scalar_type_name(&target_type)
                    ),
                    Some(*span),
                ))
            }
        }
        Expr::Let {
            binding_groups,
            body,
            ..
        } => infer_let_type(binding_groups, body, context),
        Expr::Group { inner, .. } => infer_scalar_type(inner, context),
        Expr::Unary { operand, span, .. } => {
            let operand_type = infer_scalar_type(operand, context)?;
            if operand_type == ScalarType::Boolean || operand_type == ScalarType::Unknown {
                Ok(ScalarType::Boolean)
            } else {
                Err(static_type_error("not", "boolean", &operand_type, *span))
            }
        }
        Expr::Binary {
            op: op @ (BinaryOp::And | BinaryOp::Or),
            lhs,
            rhs,
            span,
        } => {
            let left = infer_scalar_type(lhs, context)?;
            if left != ScalarType::Boolean && left != ScalarType::Unknown {
                return Err(static_type_error(binary_name(*op), "boolean", &left, *span));
            }
            let right = infer_scalar_type(rhs, context)?;
            if right != ScalarType::Boolean && right != ScalarType::Unknown {
                return Err(static_type_error(
                    binary_name(*op),
                    "boolean",
                    &right,
                    *span,
                ));
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

fn infer_let_type(
    binding_groups: &[Vec<LetBinding>],
    body: &Expr,
    context: &EvalContext,
) -> Result<ScalarType, Diagnostic> {
    let Some((bindings, remaining)) = binding_groups.split_first() else {
        return infer_scalar_type(body, context);
    };
    // Every initializer in one comma group sees only the enclosing scopes.
    let binding_types = bindings
        .iter()
        .map(|binding| infer_scalar_type(&binding.initializer, context))
        .collect::<Result<Vec<_>, _>>()?;
    reject_duplicate_let_bindings(bindings)?;
    let mut local_context = context.clone();
    local_context.push_scope();
    for (binding, value_type) in bindings.iter().zip(binding_types) {
        local_context.bind_local(binding.name.clone(), value_type, None);
    }
    infer_let_type(remaining, body, &local_context)
}

fn reject_duplicate_let_bindings(bindings: &[LetBinding]) -> Result<(), Diagnostic> {
    let mut names = BTreeSet::new();
    for binding in bindings {
        if !names.insert(binding.name.as_str()) {
            return Err(Diagnostic::new(
                ErrorKind::Name,
                format!("Multiple binding of '{}' in same scope", binding.name),
                Some(binding.name_span),
            ));
        }
    }
    Ok(())
}

fn infer_operator_type(
    operator: &FormulaOperator,
    arguments: &[ScalarType],
    span: SourceSpan,
) -> Result<ScalarType, Diagnostic> {
    let symbol = operator.symbol.as_str();
    match (symbol, arguments) {
        ("-" | "+", [value]) if is_numeric_or_unknown(value) => {
            Ok(result_with_unknown(arguments, value.clone()))
        }
        ("-" | "+", [value]) => Err(static_type_error(symbol, "numeric", value, span)),
        ("+" | "-" | "*", [left, right])
            if is_numeric_or_unknown(left) && is_numeric_or_unknown(right) =>
        {
            Ok(promoted_numeric_type(left, right))
        }
        ("/", [left, right]) if is_numeric_or_unknown(left) && is_numeric_or_unknown(right) => {
            Ok(ScalarType::Rational)
        }
        ("%" | "\\", [left, right])
            if is_integer_or_unknown(left) && is_integer_or_unknown(right) =>
        {
            Ok(ScalarType::Integer)
        }
        ("\\%", [left, right]) if is_integer_or_unknown(left) && is_integer_or_unknown(right) => {
            Ok(ScalarType::Tuple(vec![
                ScalarType::Integer,
                ScalarType::Integer,
            ]))
        }
        ("^" | "&", [left, right])
            if is_integer_or_unknown(left) && is_integer_or_unknown(right) =>
        {
            Ok(ScalarType::Integer)
        }
        ("=", [left, right]) | ("!=", [left, right])
            if (is_numeric_or_unknown(left) && is_numeric_or_unknown(right))
                || (is_boolean_or_unknown(left) && is_boolean_or_unknown(right))
                || (is_string_or_unknown(left) && is_string_or_unknown(right)) =>
        {
            Ok(ScalarType::Boolean)
        }
        ("<" | "<=" | ">" | ">=", [left, right])
            if (is_numeric_or_unknown(left) && is_numeric_or_unknown(right))
                || (is_string_or_unknown(left) && is_string_or_unknown(right)) =>
        {
            Ok(ScalarType::Boolean)
        }
        ("##", [left, right]) if is_string_or_unknown(left) && is_string_or_unknown(right) => {
            Ok(ScalarType::String)
        }
        _ => Err(operator_type_error(operator, arguments, span)),
    }
}

fn has_unknown(arguments: &[ScalarType]) -> bool {
    arguments.contains(&ScalarType::Unknown)
}

fn result_with_unknown(arguments: &[ScalarType], known_result: ScalarType) -> ScalarType {
    if has_unknown(arguments) {
        ScalarType::UnknownNumeric
    } else {
        known_result
    }
}

fn is_numeric_or_unknown(value_type: &ScalarType) -> bool {
    *value_type == ScalarType::Unknown
        || *value_type == ScalarType::UnknownNumeric
        || is_numeric(value_type)
}

fn is_integer_or_unknown(value_type: &ScalarType) -> bool {
    *value_type == ScalarType::Unknown
        || *value_type == ScalarType::UnknownNumeric
        || *value_type == ScalarType::Integer
}

fn is_boolean_or_unknown(value_type: &ScalarType) -> bool {
    *value_type == ScalarType::Unknown || *value_type == ScalarType::Boolean
}

fn is_string_or_unknown(value_type: &ScalarType) -> bool {
    *value_type == ScalarType::Unknown || *value_type == ScalarType::String
}

fn promoted_numeric_type(left: &ScalarType, right: &ScalarType) -> ScalarType {
    if (*left == ScalarType::Unknown || *left == ScalarType::UnknownNumeric)
        && (*right == ScalarType::Unknown || *right == ScalarType::UnknownNumeric)
    {
        return ScalarType::UnknownNumeric;
    }
    if *left == ScalarType::UnknownNumeric || *right == ScalarType::UnknownNumeric {
        if *left == ScalarType::Rational || *right == ScalarType::Rational {
            return ScalarType::Rational;
        }
        return ScalarType::UnknownNumeric;
    }
    if *left == ScalarType::Rational || *right == ScalarType::Rational {
        ScalarType::Rational
    } else {
        ScalarType::Integer
    }
}

fn is_numeric(value_type: &ScalarType) -> bool {
    matches!(value_type, ScalarType::Integer | ScalarType::Rational)
}

fn common_list_element_type(types: &[ScalarType]) -> Option<Option<ScalarType>> {
    let Some((first, rest)) = types.split_first() else {
        return Some(None);
    };
    let common = rest.iter().try_fold(first.clone(), |common, value_type| {
        common_type(&common, value_type)
    })?;
    Some(Some(common))
}

fn common_type(left: &ScalarType, right: &ScalarType) -> Option<ScalarType> {
    if *left == ScalarType::Unknown {
        return Some(right.clone());
    }
    if *right == ScalarType::Unknown {
        return Some(left.clone());
    }
    if *left == ScalarType::UnknownNumeric {
        return match right {
            ScalarType::Integer | ScalarType::Rational => Some(right.clone()),
            ScalarType::UnknownNumeric => Some(ScalarType::UnknownNumeric),
            _ => None,
        };
    }
    if *right == ScalarType::UnknownNumeric {
        return match left {
            ScalarType::Integer | ScalarType::Rational => Some(left.clone()),
            _ => None,
        };
    }
    if left == right {
        return Some(left.clone());
    }
    if is_numeric(left) && is_numeric(right) {
        return Some(ScalarType::Rational);
    }
    match (left, right) {
        (ScalarType::Tuple(left), ScalarType::Tuple(right)) if left.len() == right.len() => left
            .iter()
            .zip(right)
            .map(|(left, right)| common_type(left, right))
            .collect::<Option<Vec<_>>>()
            .map(ScalarType::Tuple),
        (ScalarType::List(None), ScalarType::List(element))
        | (ScalarType::List(element), ScalarType::List(None)) => {
            Some(ScalarType::List(element.clone()))
        }
        (ScalarType::List(Some(left)), ScalarType::List(Some(right))) => {
            common_type(left, right).map(|element| ScalarType::List(Some(Box::new(element))))
        }
        _ => None,
    }
}

fn assignment_compatible(target: &ScalarType, value: &ScalarType) -> bool {
    *target == ScalarType::Unknown
        || *value == ScalarType::Unknown
        || (*target == ScalarType::UnknownNumeric && is_numeric(value))
        || target == value
        || (*value == ScalarType::UnknownNumeric
            && matches!(target, ScalarType::Integer | ScalarType::Rational))
        || matches!((target, value), (ScalarType::Rational, ScalarType::Integer))
        || match (target, value) {
            (ScalarType::Tuple(target), ScalarType::Tuple(value))
                if target.len() == value.len() =>
            {
                target
                    .iter()
                    .zip(value)
                    .all(|(target, value)| assignment_compatible(target, value))
            }
            (ScalarType::List(None), ScalarType::List(_))
            | (ScalarType::List(Some(_)), ScalarType::List(None)) => true,
            (ScalarType::List(Some(target)), ScalarType::List(Some(value))) => {
                assignment_compatible(target, value)
            }
            _ => false,
        }
}

fn atlas_type_name(value_type: &ScalarType) -> String {
    match value_type {
        ScalarType::Unknown => "*".into(),
        ScalarType::UnknownNumeric => "*numeric*".into(),
        ScalarType::Integer => "int".into(),
        ScalarType::Rational => "rat".into(),
        ScalarType::Boolean => "bool".into(),
        ScalarType::String => "string".into(),
        ScalarType::Tuple(elements) => format!(
            "({})",
            elements
                .iter()
                .map(atlas_type_name)
                .collect::<Vec<_>>()
                .join(",")
        ),
        ScalarType::List(Some(element)) => format!("[{}]", atlas_type_name(element)),
        ScalarType::List(None) => "[*]".into(),
        ScalarType::Domain(kind) => (*kind).into(),
    }
}

fn value_type(value: &Value) -> ScalarType {
    match value {
        Value::Integer(_) => ScalarType::Integer,
        Value::Rational(_) => ScalarType::Rational,
        Value::Boolean(_) => ScalarType::Boolean,
        Value::String(_) => ScalarType::String,
        Value::Tuple(values) => ScalarType::Tuple(values.iter().map(value_type).collect()),
        Value::List(values) => {
            let types = values.iter().map(value_type).collect::<Vec<_>>();
            let element_type = common_list_element_type(&types)
                .and_then(|element_type| element_type.map(Box::new));
            ScalarType::List(element_type)
        }
        Value::Domain(value) => ScalarType::Domain(crate::domain_builtins::kind_name(value)),
    }
}

fn static_type_error(
    operator: &str,
    expected: &str,
    actual: &ScalarType,
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
        .map(scalar_type_name)
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

fn scalar_type_name(value_type: &ScalarType) -> String {
    match value_type {
        ScalarType::Unknown => "unknown".into(),
        ScalarType::UnknownNumeric => "unknown numeric".into(),
        ScalarType::Integer => "integer".into(),
        ScalarType::Rational => "rational".into(),
        ScalarType::Boolean => "boolean".into(),
        ScalarType::String => "string".into(),
        ScalarType::Tuple(elements) => format!(
            "({})",
            elements
                .iter()
                .map(scalar_type_name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ScalarType::List(Some(element)) => format!("list of {}", scalar_type_name(element)),
        ScalarType::List(None) => "empty list".into(),
        ScalarType::Domain(kind) => (*kind).into(),
    }
}

/// Evaluate a named-function application. Only identifier heads are
/// callable; the name set is the domain-builtin registry.
fn eval_call(
    callee: &Expr,
    arguments: &[Expr],
    span: crate::diagnostic::SourceSpan,
    context: &mut EvalContext,
) -> Result<Value, Diagnostic> {
    let Expr::Identifier { name, .. } = callee else {
        return Err(Diagnostic::new(
            ErrorKind::Type,
            "only named functions can be applied".to_string(),
            Some(span),
        ));
    };
    let mut values = Vec::with_capacity(arguments.len());
    for argument in arguments {
        values.push(eval_expr(argument, context)?);
    }
    crate::domain_builtins::call(name, &values, span)
}

/// Evaluate one expression against an explicit context.
fn eval_expr(expression: &Expr, context: &mut EvalContext) -> Result<Value, Diagnostic> {
    match expression {
        Expr::Cast { target, body, span } => {
            let value = eval_expr(body, context)?;
            cast_value(value, &target.resolve(), *span)
        }
        Expr::Call {
            callee,
            arguments,
            span,
        } => eval_call(callee, arguments, *span, context),
        Expr::Integer { value, .. } => Ok(Value::Integer(value.clone())),
        Expr::Boolean { value, .. } => Ok(Value::Boolean(*value)),
        Expr::String { value, .. } => Ok(Value::String(value.clone())),
        Expr::Tuple { elements, .. } => elements
            .iter()
            .map(|element| eval_expr(element, context))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Tuple),
        Expr::List { elements, span } => eval_list(elements, *span, context),
        Expr::Subscription {
            array,
            index,
            reversed,
            span,
        } => eval_subscription(array, index, *reversed, *span, context),
        Expr::Slice {
            array,
            lower,
            upper,
            flags,
            span,
        } => eval_slice(array, lower, upper, *flags, *span, context),
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
        Expr::Let {
            binding_groups,
            body,
            ..
        } => eval_let(binding_groups, body, context),
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

fn eval_list(
    elements: &[Expr],
    span: SourceSpan,
    context: &mut EvalContext,
) -> Result<Value, Diagnostic> {
    let values = elements
        .iter()
        .map(|element| eval_expr(element, context))
        .collect::<Result<Vec<_>, _>>()?;
    let types = values.iter().map(value_type).collect::<Vec<_>>();
    let Some(element_type) = common_list_element_type(&types) else {
        return Err(Diagnostic::new(
            ErrorKind::Type,
            "list elements must have a common type",
            Some(span),
        ));
    };
    let Some(element_type) = element_type else {
        return Ok(Value::List(values));
    };
    values
        .into_iter()
        .map(|value| coerce_value_to_type(&element_type, value, span))
        .collect::<Result<Vec<_>, _>>()
        .map(Value::List)
}

fn eval_subscription(
    array: &Expr,
    index: &Expr,
    reversed: bool,
    span: SourceSpan,
    context: &mut EvalContext,
) -> Result<Value, Diagnostic> {
    let index = eval_expr(index, context)?;
    let Value::Integer(index) = index else {
        return Err(Diagnostic::new(
            ErrorKind::Type,
            "subscription index must be an integer",
            Some(span),
        ));
    };
    let array = eval_expr(array, context)?;
    let Value::List(values) = array else {
        return Err(Diagnostic::new(
            ErrorKind::Type,
            "subscription requires a list",
            Some(span),
        ));
    };

    if index < 0 || !usize::convertible_from(&index) {
        return Err(subscription_range_error(&index, values.len(), span));
    }
    let index = usize::wrapping_from(&index);
    if index >= values.len() {
        return Err(subscription_range_error(
            &BigInt::from(index),
            values.len(),
            span,
        ));
    }
    let position = if reversed {
        values.len() - 1 - index
    } else {
        index
    };
    Ok(values[position].clone())
}

fn eval_slice(
    array: &Expr,
    lower: &Expr,
    upper: &Expr,
    flags: SliceFlags,
    span: SourceSpan,
    context: &mut EvalContext,
) -> Result<Value, Diagnostic> {
    let upper = eval_slice_bound(upper, "upper", span, context)?;
    let lower = eval_slice_bound(lower, "lower", span, context)?;
    let array = eval_expr(array, context)?;
    let Value::List(values) = array else {
        return Err(Diagnostic::new(
            ErrorKind::Type,
            "slice requires a list",
            Some(span),
        ));
    };

    let length = BigInt::from(values.len());
    let lower = adjust_slice_bound(lower, flags.lower_from_end, &length);
    let upper = adjust_slice_bound(upper, flags.upper_from_end, &length);
    let lower_out_of_range = lower < 0;
    let upper_out_of_range = upper > length;
    if lower_out_of_range || upper_out_of_range {
        return Err(slice_range_error(
            &lower,
            &upper,
            values.len(),
            lower_out_of_range,
            upper_out_of_range,
            span,
        ));
    }
    if lower >= upper {
        return Ok(Value::List(Vec::new()));
    }

    debug_assert!(usize::convertible_from(&lower));
    debug_assert!(usize::convertible_from(&upper));
    let lower = usize::wrapping_from(&lower);
    let upper = usize::wrapping_from(&upper);
    let mut result = values[lower..upper].to_vec();
    if flags.reverse_output {
        result.reverse();
    }
    Ok(Value::List(result))
}

fn eval_slice_bound(
    expression: &Expr,
    name: &str,
    span: SourceSpan,
    context: &mut EvalContext,
) -> Result<BigInt, Diagnostic> {
    match eval_expr(expression, context)? {
        Value::Integer(value) => Ok(value),
        _ => Err(Diagnostic::new(
            ErrorKind::Type,
            format!("slice {name} bound must be an integer"),
            Some(span),
        )),
    }
}

fn adjust_slice_bound(bound: BigInt, from_end: bool, length: &BigInt) -> BigInt {
    if from_end {
        length - bound
    } else {
        bound
    }
}

fn eval_let(
    binding_groups: &[Vec<LetBinding>],
    body: &Expr,
    context: &mut EvalContext,
) -> Result<Value, Diagnostic> {
    let Some((bindings, remaining)) = binding_groups.split_first() else {
        return eval_expr(body, context);
    };

    let values = bindings
        .iter()
        .map(|binding| eval_expr(&binding.initializer, context))
        .collect::<Result<Vec<_>, _>>()?;
    context.push_scope();
    for (binding, value) in bindings.iter().zip(values) {
        context.bind_local(binding.name.clone(), value_type(&value), Some(value));
    }
    let result = eval_let(remaining, body, context);
    context.pop_scope();
    result
}

fn coerce_assignment_value(
    target_type: ScalarType,
    value: Value,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    coerce_value_to_type(&target_type, value, span)
}

fn coerce_value_to_type(
    target_type: &ScalarType,
    value: Value,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    match (target_type, value) {
        (ScalarType::Rational, Value::Integer(value)) => {
            Ok(Value::Rational(BigRational::from(value)))
        }
        (ScalarType::Integer, Value::Integer(value)) => Ok(Value::Integer(value)),
        (ScalarType::Rational, Value::Rational(value)) => Ok(Value::Rational(value)),
        (ScalarType::Boolean, Value::Boolean(value)) => Ok(Value::Boolean(value)),
        (ScalarType::String, Value::String(value)) => Ok(Value::String(value)),
        (ScalarType::Tuple(types), Value::Tuple(values)) if types.len() == values.len() => types
            .iter()
            .zip(values)
            .map(|(value_type, value)| coerce_value_to_type(value_type, value, span))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Tuple),
        (ScalarType::List(None), Value::List(values)) => Ok(Value::List(values)),
        (ScalarType::List(Some(element_type)), Value::List(values)) => values
            .into_iter()
            .map(|value| coerce_value_to_type(element_type, value, span))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::List),
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
                    "Modulo zero",
                    Some(span),
                ))
            } else {
                Ok(Value::Integer(euclidean_divmod(left, right).1))
            }
        }
        ("\\%", [Value::Integer(left), Value::Integer(right)]) => {
            if right == &BigInt::from(0) {
                Err(Diagnostic::new(
                    ErrorKind::Runtime,
                    "DivMod by zero",
                    Some(span),
                ))
            } else {
                let (quotient, remainder) = euclidean_divmod(left, right);
                Ok(Value::Tuple(vec![
                    Value::Integer(quotient),
                    Value::Integer(remainder),
                ]))
            }
        }
        ("\\", [Value::Integer(left), Value::Integer(right)]) => {
            if right == &BigInt::from(0) {
                Err(Diagnostic::new(
                    ErrorKind::Runtime,
                    "Division by zero",
                    Some(span),
                ))
            } else {
                Ok(Value::Integer(euclidean_divmod(left, right).0))
            }
        }
        ("&", [Value::Integer(left), Value::Integer(right)]) => Ok(Value::Integer(left & right)),
        ("^", [Value::Integer(left), Value::Integer(right)]) => {
            if !u64::convertible_from(right) {
                return Err(Diagnostic::new(
                    ErrorKind::Type,
                    "exponent must be a non-negative machine-sized integer",
                    Some(span),
                ));
            }
            let exponent = u64::wrapping_from(right);
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
    let opposite_signs = (remainder < 0) != (*right < 0);
    if remainder != 0 && opposite_signs {
        quotient -= BigInt::from(1);
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
            let left = BigRational::from(left);
            Value::Rational(match op {
                "+" => left + right,
                "-" => left - right,
                "*" => left * right,
                _ => unreachable!("only exact arithmetic operators reach this helper"),
            })
        }
        (Value::Rational(left), Value::Integer(right)) => {
            let right = BigRational::from(right);
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
        (Value::Integer(left), Value::Integer(right)) => {
            (BigRational::from(left), BigRational::from(right))
        }
        (Value::Integer(left), Value::Rational(right)) => (BigRational::from(left), right),
        (Value::Rational(left), Value::Integer(right)) => (left, BigRational::from(right)),
        (Value::Rational(left), Value::Rational(right)) => (left, right),
        (left, right) => {
            return Err(numeric_type_error("/", &left, &right, span));
        }
    };

    if right == 0 {
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
        (Value::Integer(left), Value::Rational(right)) => Some(BigRational::from(left).cmp(right)),
        (Value::Rational(left), Value::Integer(right)) => Some(left.cmp(&BigRational::from(right))),
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

fn subscription_type_error(
    array_type: &ScalarType,
    index_type: &ScalarType,
    span: SourceSpan,
) -> Diagnostic {
    Diagnostic::new(
        ErrorKind::Type,
        format!(
            "Cannot subscript value of type {} with index of type {}",
            atlas_type_name(array_type),
            atlas_type_name(index_type)
        ),
        Some(span),
    )
}

fn subscription_range_error(index: &BigInt, length: usize, span: SourceSpan) -> Diagnostic {
    Diagnostic::new(
        ErrorKind::Runtime,
        format!("index {index} out of range (0<= . <{length}) in subscription"),
        Some(span),
    )
}

fn slice_type_error(
    array_type: &ScalarType,
    lower_type: &ScalarType,
    upper_type: &ScalarType,
    span: SourceSpan,
) -> Diagnostic {
    Diagnostic::new(
        ErrorKind::Type,
        format!(
            "Cannot slice value of type {} with bounds of type {} and {}",
            atlas_type_name(array_type),
            atlas_type_name(lower_type),
            atlas_type_name(upper_type)
        ),
        Some(span),
    )
}

fn slice_range_error(
    lower: &BigInt,
    upper: &BigInt,
    length: usize,
    lower_out_of_range: bool,
    upper_out_of_range: bool,
    span: SourceSpan,
) -> Diagnostic {
    let message = match (lower_out_of_range, upper_out_of_range) {
        (true, true) => format!(
            "both bounds {lower}:{upper} out of range (should be >=0 respectively <= {length}) in slice"
        ),
        (true, false) => {
            format!("lower bound {lower} out of range (should be >=0) in slice")
        }
        (false, true) => {
            format!("upper bound {upper} out of range (should be <= {length}) in slice")
        }
        (false, false) => unreachable!("slice range error requires an invalid bound"),
    };
    Diagnostic::new(ErrorKind::Runtime, message, Some(span))
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
        Value::Tuple(_) => "tuple",
        Value::List(_) => "list",
        Value::Domain(value) => crate::domain_builtins::kind_name(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{source::SourceText, syntax::parse};
    use malachite::{Integer as BigInt, Rational as BigRational};

    fn integer(value: impl Into<BigInt>) -> Value {
        Value::Integer(value.into())
    }

    fn rational(numerator: impl Into<BigInt>, denominator: impl Into<BigInt>) -> Value {
        Value::Rational(BigRational::from_integers(
            numerator.into(),
            denominator.into(),
        ))
    }

    fn run(source: &str) -> Result<Vec<EvalEvent>, Diagnostic> {
        let source = SourceText::new(source);
        let program = parse(&source).expect("test source parses");
        evaluate(&program)
    }

    #[test]
    fn casts_check_and_promote_dynamically() {
        let events = run("rat: 2").expect("int promotes to rat");
        assert!(matches!(
            &events[0],
            EvalEvent::Value { value, .. } if *value == rational(2, 1)
        ));
        let empty = run("[int]: []").expect("empty list casts to any row");
        assert!(matches!(
            &empty[0],
            EvalEvent::Value { value, .. } if *value == Value::List(Vec::new())
        ));
        let wild = run("[*]: [1,2]").expect("wild row accepts any components");
        assert!(matches!(&wild[0], EvalEvent::Value { .. }));
        let mismatch = run("int: \"x\"").expect_err("string is not an int");
        assert_eq!(mismatch.kind, ErrorKind::Type);
        let unsupported = run("(int->int): 3").expect_err("function casts wait for B3");
        assert_eq!(unsupported.kind, ErrorKind::Type);
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
    fn evaluates_tuple_valued_divmod() {
        let events = run("1 \\% 2").expect("divmod evaluates");
        let span = match &events[0] {
            EvalEvent::Value { span, .. } => *span,
            EvalEvent::Output { .. } => unreachable!("program evaluation emits values only"),
        };
        assert_eq!(
            events,
            vec![EvalEvent::Value {
                value: Value::Tuple(vec![integer(0), integer(1)]),
                span,
            }]
        );
    }

    #[test]
    fn distinguishes_integer_zero_division_diagnostics() {
        let cases = [
            (r"1 \ 0", "Division by zero"),
            ("1 % 0", "Modulo zero"),
            (r"1 \% 0", "DivMod by zero"),
        ];

        for (source, expected_message) in cases {
            let source = SourceText::new(source);
            let program = parse(&source).expect("integer division source parses");
            let error = evaluate(&program).expect_err("zero divisor is rejected");
            assert_eq!(error.kind, ErrorKind::Runtime);
            assert_eq!(error.message, expected_message);
        }
    }

    #[test]
    fn evaluates_tuple_and_list_fixture_in_source_order() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/containers.atlas"
        ));
        let program = parse(&source).expect("container fixture parses");
        let events = evaluate(&program).expect("container fixture evaluates");
        assert_eq!(events.len(), 7);
        assert!(matches!(
            &events[0],
            EvalEvent::Value {
                value: Value::Tuple(values), ..
            } if values == &vec![integer(1), Value::String("a".into()), Value::Boolean(true)]
        ));
        assert!(matches!(
            &events[1],
            EvalEvent::Value {
                value: Value::List(values), ..
            } if values == &vec![integer(1), integer(2), integer(3)]
        ));
        assert!(matches!(
            &events[2],
            EvalEvent::Value {
                value: Value::List(values), ..
            } if values.is_empty()
        ));
        assert!(matches!(
            &events[3],
            EvalEvent::Value {
                value: Value::List(values), ..
            } if values == &vec![rational(1, 1), rational(1, 2)]
        ));
        assert!(matches!(
            &events[4],
            EvalEvent::Value {
                value: Value::List(values), ..
            } if values == &vec![
                Value::List(vec![rational(1, 1)]),
                Value::List(vec![rational(1, 2)])
            ]
        ));
        assert!(matches!(
            &events[5],
            EvalEvent::Value {
                value: Value::List(values), ..
            } if values == &vec![
                Value::Tuple(vec![integer(1), rational(2, 1)]),
                Value::Tuple(vec![integer(3), rational(1, 2)])
            ]
        ));
        assert!(matches!(
            &events[6],
            EvalEvent::Value {
                value: Value::Tuple(values), ..
            } if values == &vec![integer(0), integer(1)]
        ));
    }

    #[test]
    fn evaluates_row_subscription_fixture_in_source_order() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/subscriptions.atlas"
        ));
        let program = parse(&source).expect("subscription fixture parses");
        let events = evaluate(&program).expect("subscription fixture evaluates");
        let values = events
            .into_iter()
            .map(|event| match event {
                EvalEvent::Value { value, .. } => value,
                EvalEvent::Output { .. } => unreachable!("program evaluation emits values only"),
            })
            .collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![
                integer(10),
                integer(30),
                integer(30),
                integer(10),
                integer(3),
                rational(1, 1),
            ]
        );
    }

    #[test]
    fn evaluates_forward_row_slice_fixture() {
        let source = SourceText::new(include_str!("../../../tests/fixtures/eval/slices.atlas"));
        let program = parse(&source).expect("slice fixture parses");
        let events = evaluate(&program).expect("slice fixture evaluates");
        let values = events
            .into_iter()
            .map(|event| match event {
                EvalEvent::Value { value, .. } => value,
                EvalEvent::Output { .. } => unreachable!("program evaluation emits values only"),
            })
            .collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![
                Value::List(vec![integer(20), integer(30)]),
                Value::List(vec![integer(10), integer(20)]),
                Value::List(vec![integer(30), integer(40)]),
                Value::List(vec![integer(10), integer(20), integer(30), integer(40)]),
                Value::List(Vec::new()),
                Value::List(Vec::new()),
                Value::List(Vec::new()),
                Value::List(Vec::new()),
            ]
        );
    }

    #[test]
    fn empty_row_subscription_flows_through_enclosing_types() {
        for source in ["[][0] + 1", "[[][0], 1]"] {
            let source = SourceText::new(source);
            let program = parse(&source).expect("contextual empty-row source parses");
            let error = evaluate(&program).expect_err("empty-row subscription reaches runtime");
            assert_eq!(error.kind, ErrorKind::Runtime);
        }
    }

    #[test]
    fn unknown_subscription_does_not_mask_incompatible_operator_operands() {
        for source in ["[][0] + true", "[][0] % (1 / 2)"] {
            let source = SourceText::new(source);
            let program = parse(&source).expect("operator source parses");
            let error = evaluate(&program).expect_err("incompatible overload is rejected");
            assert_eq!(error.kind, ErrorKind::Type);
        }
    }

    #[test]
    fn boolean_results_do_not_remain_unknown_after_subscription_input() {
        for source in ["(not [][0]) + 1", "([][0] and true) + 1"] {
            let source = SourceText::new(source);
            let program = parse(&source).expect("boolean source parses");
            let error = evaluate(&program).expect_err("boolean result rejects numeric addition");
            assert_eq!(error.kind, ErrorKind::Type);
        }
    }

    #[test]
    fn fixed_operator_results_remain_type_checked_after_subscription_input() {
        let source = SourceText::new("([][0] % 1) + true");
        let program = parse(&source).expect("fixed-result operator source parses");
        let error = evaluate(&program).expect_err("integer result rejects boolean addition");
        assert_eq!(error.kind, ErrorKind::Type);
    }

    #[test]
    fn rejects_heterogeneous_lists_before_runtime_evaluation() {
        let source = SourceText::new("[1, \"bad\"]");
        let program = parse(&source).expect("heterogeneous list parses");
        let error = evaluate(&program).expect_err("heterogeneous list is not a row");
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
