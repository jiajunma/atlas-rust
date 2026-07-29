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

use malachite::base::num::arithmetic::traits::{Floor, Mod, Pow};
use malachite::{Integer as BigInt, Rational as BigRational};

use crate::coercions::{coercion_between, row_coercion};
use crate::diagnostic::{Diagnostic, ErrorKind, SourceSpan};
use crate::domain_builtins;
use crate::frames::{EvaluationContext, GlobalCell};
use crate::linear_values::{Matrix, RatVec, Vec32};
use crate::syntax::{Command, Expr, LambdaParam};
use crate::types::{Prim, Type, TypeTable};
use crate::value::{Closure, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
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
    /// Evaluate `inner` for effects and replace its value with void.
    Void(Box<TypedExpr>),
    /// A global read through the cell captured at analysis time.
    GlobalIdent {
        name: String,
        cell: GlobalCell,
        span: SourceSpan,
    },
    LocalIdent {
        name: String,
        depth: usize,
        offset: usize,
        span: SourceSpan,
    },
    GlobalAssignment {
        cell: GlobalCell,
        value: Box<TypedExpr>,
    },
    LocalAssignment {
        depth: usize,
        offset: usize,
        value: Box<TypedExpr>,
    },
    Subscription {
        array: Box<TypedExpr>,
        index: Box<TypedExpr>,
        reversed: bool,
        span: SourceSpan,
    },
    Slice {
        array: Box<TypedExpr>,
        lower: Box<TypedExpr>,
        upper: Box<TypedExpr>,
        flags: crate::syntax::SliceFlags,
        span: SourceSpan,
    },
    LetGroup {
        initializers: Vec<TypedExpr>,
        body: Box<TypedExpr>,
    },
    /// `if c then t else e fi` after balancing.
    Conditional {
        condition: Box<TypedExpr>,
        then_branch: Box<TypedExpr>,
        else_branch: Box<TypedExpr>,
    },
    /// A statically resolved builtin call (index into the registry).
    BuiltinCall {
        builtin: usize,
        arguments: Vec<TypedExpr>,
        span: SourceSpan,
    },
    /// A non-recursive function literal; evaluation captures the current
    /// frame chain into a closure value (upstream `lambda_expression`).
    Closure {
        /// Number of argument slots a call binds; 0 pushes no frame.
        parameters: usize,
        body: Rc<TypedExpr>,
    },
    /// `return value`, unwound to the innermost call boundary.
    Return {
        value: Box<TypedExpr>,
    },
    /// A user-function call: the callee evaluates to a closure and the
    /// argument is passed as one value (a tuple for several parameters).
    FunctionCall {
        function: Box<TypedExpr>,
        argument: Box<TypedExpr>,
        span: SourceSpan,
    },
}

/// Conversion-time context (locals are let bindings and lambda parameters).
pub struct Analysis<'a> {
    pub types: &'a TypeTable,
    pub globals: &'a IdTable,
    locals: BTreeMap<String, (TypeCell, usize, usize)>,
}

impl<'a> Analysis<'a> {
    pub fn new(types: &'a TypeTable, globals: &'a IdTable) -> Self {
        Self {
            types,
            globals,
            locals: BTreeMap::new(),
        }
    }
}

/// The global identifier table: one binding per name, each definition
/// holding the FRESH cell it allocated (converted code keeps the cell it
/// captured; re-definition rebinds the name only).
#[derive(Default)]
pub struct IdTable {
    entries: BTreeMap<String, (TypeCell, GlobalCell)>,
}

pub type TypeCell = Rc<RefCell<Type>>;

impl IdTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(&mut self, name: impl Into<String>, type_: Type, cell: GlobalCell) {
        self.entries
            .insert(name.into(), (Rc::new(RefCell::new(type_)), cell));
    }

    pub fn lookup(&self, name: &str) -> Option<&(TypeCell, GlobalCell)> {
        self.entries.get(name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedCommandEvent {
    Value {
        value: Value,
        type_: Type,
        span: SourceSpan,
    },
    ReportLine {
        text: String,
        span: SourceSpan,
    },
}

/// Persistent state for command-at-a-time typed execution.
#[derive(Default)]
pub struct TypedContext {
    types: TypeTable,
    globals: IdTable,
    evaluation: EvaluationContext,
}

impl TypedContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn globals(&self) -> &IdTable {
        &self.globals
    }

    pub fn execute(&mut self, command: &Command) -> Result<Vec<TypedCommandEvent>, Diagnostic> {
        match command {
            Command::Expression(expression) => {
                let mut type_ = Type::Undetermined;
                let typed = convert_expr(
                    expression,
                    &mut type_,
                    &Analysis::new(&self.types, &self.globals),
                )?;
                let value = evaluate_command_expr(&typed, &mut self.evaluation)?;
                Ok(vec![TypedCommandEvent::Value {
                    value,
                    type_,
                    span: expression.span(),
                }])
            }
            Command::Define {
                name, value, span, ..
            } => {
                let mut type_ = Type::Undetermined;
                let typed = convert_expr(
                    value,
                    &mut type_,
                    &Analysis::new(&self.types, &self.globals),
                )?;
                let value = evaluate_command_expr(&typed, &mut self.evaluation)?;
                let previous = self
                    .globals
                    .lookup(name)
                    .map(|(type_, _)| type_.borrow().display(&self.types).to_string());
                self.globals.define(
                    name.clone(),
                    type_.clone(),
                    crate::frames::global_with(Rc::new(value)),
                );

                let mut text = format!("Variable {name}: {}", type_.display(&self.types));
                if let Some(previous) = previous {
                    text.push_str(&format!(
                        " (overriding previous instance, which had type {previous})"
                    ));
                }
                text.push('\n');
                Ok(vec![TypedCommandEvent::ReportLine { text, span: *span }])
            }
            Command::Declare {
                name,
                value_type,
                span,
                ..
            } => {
                let type_ = Type::Primitive(*value_type);
                self.globals
                    .define(name.clone(), type_.clone(), crate::frames::unset_global());
                Ok(vec![TypedCommandEvent::ReportLine {
                    text: format!(
                        "Declaring identifier '{name}': {}\n",
                        type_.display(&self.types)
                    ),
                    span: *span,
                }])
            }
        }
    }
}

fn evaluate_command_expr(
    expression: &TypedExpr,
    context: &mut EvaluationContext,
) -> Result<Value, Diagnostic> {
    expression
        .evaluate(context, Level::SingleValue)
        .map_err(|control| match control {
            Control::Runtime(diagnostic) => diagnostic,
            Control::Break(_) | Control::Return(_) => Diagnostic::new(
                ErrorKind::Runtime,
                "illegal control flow at top level",
                None,
            ),
        })
        .map(|value| value.expect("single-value command evaluation yields a value"))
}

fn type_error(message: String, span: SourceSpan) -> Diagnostic {
    Diagnostic::new(ErrorKind::Type, message, Some(span))
}

/// A balance failure is kept structured until the enclosing balance has had
/// an opportunity to choose a broader common type.  Flattening this into a
/// diagnostic at the inner list would make a later `void` branch unable to
/// salvage the expression (the upstream `balance_error` behaviour).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BalanceContainer {
    Unknown,
    List,
    Conditional,
}

#[derive(Clone, Debug)]
struct BalanceFailure {
    span: SourceSpan,
    variants: Vec<Type>,
    container: BalanceContainer,
}

#[derive(Debug)]
enum BalanceConversionError {
    Diagnostic(Diagnostic),
    Balance(BalanceFailure),
}

impl BalanceConversionError {
    fn into_diagnostic(self, analysis: &Analysis<'_>) -> Diagnostic {
        match self {
            Self::Diagnostic(diagnostic) => diagnostic,
            Self::Balance(failure) => {
                let mut displays = failure
                    .variants
                    .iter()
                    .map(|type_| type_.display(analysis.types).to_string())
                    .collect::<Vec<_>>();
                if displays.is_empty() {
                    displays.push(Type::Undetermined.display(analysis.types).to_string());
                }
                type_error(
                    format!(
                        "branches have incompatible types {}",
                        displays.join(" and ")
                    ),
                    failure.span,
                )
            }
        }
    }
}

fn mark_balance_failure(
    error: BalanceConversionError,
    owner_span: SourceSpan,
    container: BalanceContainer,
) -> BalanceConversionError {
    match error {
        BalanceConversionError::Balance(mut failure) if failure.span == owner_span => {
            failure.container = container;
            BalanceConversionError::Balance(failure)
        }
        other => other,
    }
}

/// Convert a list display while retaining a balance failure for the caller
/// that is itself balancing branches.  The public conversion path maps that
/// failure to a diagnostic; only this narrowly scoped path preserves it.
fn convert_list_expression(
    elements: &[Expr],
    span: SourceSpan,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, BalanceConversionError> {
    // In row context (or undetermined), elements share the component pattern;
    // in a non-row context the first row coercion for that target decides the
    // component type (mat context -> vec).
    if required.is_void() {
        // Upstream still converts and evaluates a list in a void context,
        // while discarding its resulting row value.  Keep an undetermined
        // component pattern so nested balance errors can be resolved before
        // the enclosing void conversion is inserted.
        let mut component = Type::Undetermined;
        let branches = elements.iter().collect::<Vec<_>>();
        let display = TypedExpr::ListDisplay(balance(&branches, &mut component, span, analysis)?);
        return conform_types(&Type::row(component), required, display, span, analysis)
            .map_err(BalanceConversionError::Diagnostic);
    }
    let (mut component, coercion_tag) = match &*required {
        Type::Undetermined => (Type::Undetermined, None),
        Type::Row(component) => (component.as_ref().clone(), None),
        other => match row_coercion(other, analysis.types) {
            Some((coercion, component)) => (component.clone(), Some(coercion.tag)),
            None => {
                return Err(BalanceConversionError::Diagnostic(type_error(
                    format!(
                        "list display does not match required pattern {}",
                        other.display(analysis.types),
                    ),
                    span,
                )))
            }
        },
    };
    let branches = elements.iter().collect::<Vec<_>>();
    let converted = balance(&branches, &mut component, span, analysis)?;
    let display = TypedExpr::ListDisplay(converted);
    match coercion_tag {
        Some(tag) => Ok(TypedExpr::Conversion {
            tag,
            inner: Box::new(display),
            span,
        }),
        None => conform_types(&Type::row(component), required, display, span, analysis)
            .map_err(BalanceConversionError::Diagnostic),
    }
}

fn convert_conditional_expression(
    condition: &Expr,
    then_branch: &Expr,
    else_branch: &Expr,
    span: SourceSpan,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, BalanceConversionError> {
    let mut bool_type = Type::Primitive(Prim::Bool);
    let condition = convert_expr(condition, &mut bool_type, analysis)
        .map_err(BalanceConversionError::Diagnostic)?;
    let branches = balance(&[then_branch, else_branch], required, span, analysis)?;
    let mut branches = branches.into_iter();
    Ok(TypedExpr::Conditional {
        condition: Box::new(condition),
        then_branch: Box::new(branches.next().expect("two branches")),
        else_branch: Box::new(branches.next().expect("two branches")),
    })
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
        return Ok(if found.is_void() {
            converted
        } else {
            TypedExpr::Void(Box::new(converted))
        });
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
        Expr::Lambda {
            parameters,
            body,
            span,
        } => convert_lambda_expression(parameters, body, *span, required, analysis),
        Expr::Return { value, .. } => {
            // Inside a function body the enclosing context is the
            // function's result type (the axis layer's return_type);
            // evaluation unwinds to the innermost call boundary.
            let converted = convert_expr(value, required, analysis)?;
            Ok(TypedExpr::Return {
                value: Box::new(converted),
            })
        }
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
            convert_list_expression(elements, *span, required, analysis)
                .map_err(|error| error.into_diagnostic(analysis))
        }
        Expr::Identifier { name, span } => {
            if let Some((type_, depth, offset)) = analysis.locals.get(name) {
                let found = type_.borrow().clone();
                return conform_types(
                    &found,
                    required,
                    TypedExpr::LocalIdent {
                        name: name.clone(),
                        depth: *depth,
                        offset: *offset,
                        span: *span,
                    },
                    *span,
                    analysis,
                );
            }
            let Some((type_, cell)) = analysis.globals.lookup(name) else {
                return Err(Diagnostic::new(
                    ErrorKind::Name,
                    format!("undefined identifier `{name}`"),
                    Some(*span),
                ));
            };
            let found = type_.borrow().clone();
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
        Expr::Assignment {
            name,
            target_span,
            value,
            span,
        } => {
            if let Some((target, depth, offset)) = analysis.locals.get(name) {
                let mut required_value = target.borrow().clone();
                let converted = convert_expr(value, &mut required_value, analysis)?;
                *target.borrow_mut() = required_value.clone();
                return conform_types(
                    &required_value,
                    required,
                    TypedExpr::LocalAssignment {
                        depth: *depth,
                        offset: *offset,
                        value: Box::new(converted),
                    },
                    *span,
                    analysis,
                );
            }
            let Some((target, cell)) = analysis.globals.lookup(name) else {
                return Err(Diagnostic::new(
                    ErrorKind::Name,
                    format!("undefined identifier `{name}` in assignment"),
                    Some(*target_span),
                ));
            };
            let mut required_value = target.borrow().clone();
            let converted = convert_expr(value, &mut required_value, analysis)?;
            *target.borrow_mut() = required_value.clone();
            conform_types(
                &required_value,
                required,
                TypedExpr::GlobalAssignment {
                    cell: cell.clone(),
                    value: Box::new(converted),
                },
                *span,
                analysis,
            )
        }
        Expr::Subscription {
            array,
            index,
            reversed,
            span,
        } => {
            let mut array_type = Type::Undetermined;
            let converted_array = convert_expr(array, &mut array_type, analysis)?;
            let mut index_type = Type::Primitive(Prim::Int);
            let converted_index = convert_expr(index, &mut index_type, analysis)?;
            let Type::Row(component) = array_type else {
                return Err(type_error(
                    format!(
                        "subscription requires a row, found {}",
                        array_type.display(analysis.types)
                    ),
                    *span,
                ));
            };
            let found = (*component).clone();
            conform_types(
                &found,
                required,
                TypedExpr::Subscription {
                    array: Box::new(converted_array),
                    index: Box::new(converted_index),
                    reversed: *reversed,
                    span: *span,
                },
                *span,
                analysis,
            )
        }
        Expr::Slice {
            array,
            lower,
            upper,
            flags,
            span,
        } => {
            let mut array_type = Type::Undetermined;
            let converted_array = convert_expr(array, &mut array_type, analysis)?;
            let Type::Row(component) = array_type else {
                return Err(type_error(
                    format!(
                        "slice requires a row, found {}",
                        array_type.display(analysis.types)
                    ),
                    *span,
                ));
            };
            let mut bound_type = Type::Primitive(Prim::Int);
            let converted_lower = convert_expr(lower, &mut bound_type, analysis)?;
            let mut bound_type = Type::Primitive(Prim::Int);
            let converted_upper = convert_expr(upper, &mut bound_type, analysis)?;
            let found = Type::row((*component).clone());
            conform_types(
                &found,
                required,
                TypedExpr::Slice {
                    array: Box::new(converted_array),
                    lower: Box::new(converted_lower),
                    upper: Box::new(converted_upper),
                    flags: *flags,
                    span: *span,
                },
                *span,
                analysis,
            )
        }
        Expr::Let {
            binding_groups,
            body,
            span: _,
        } => {
            let mut locals = analysis.locals.clone();
            let mut groups = Vec::with_capacity(binding_groups.len());
            for bindings in binding_groups {
                let mut pending = Vec::with_capacity(bindings.len());
                for binding in bindings {
                    let mut binding_type = Type::Undetermined;
                    let converted = convert_expr(
                        &binding.initializer,
                        &mut binding_type,
                        &Analysis {
                            types: analysis.types,
                            globals: analysis.globals,
                            locals: locals.clone(),
                        },
                    )?;
                    pending.push((binding.name.clone(), binding_type, converted));
                }
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
                for (_, depth, _) in locals.values_mut() {
                    *depth += 1;
                }
                for (offset, (name, binding_type, _)) in pending.iter().enumerate() {
                    locals.insert(
                        name.clone(),
                        (Rc::new(RefCell::new(binding_type.clone())), 0, offset),
                    );
                }
                groups.push(
                    pending
                        .into_iter()
                        .map(|(_, _, converted)| converted)
                        .collect::<Vec<_>>(),
                );
            }
            let mut converted = convert_expr(
                body,
                required,
                &Analysis {
                    types: analysis.types,
                    globals: analysis.globals,
                    locals,
                },
            )?;
            for initializers in groups.into_iter().rev() {
                converted = TypedExpr::LetGroup {
                    initializers,
                    body: Box::new(converted),
                };
            }
            Ok(converted)
        }
        Expr::OperatorCall {
            operator,
            arguments,
            span,
        } => {
            // Atlas installs a special `(int->rat)` inverse overload whose
            // parser-side rewrite takes precedence for literal `1 / x`.
            // This is observable at zero (`Inverse of zero` vs fraction's
            // denominator diagnostic), so preserve it before normal lookup.
            if operator.symbol == "/"
                && arguments.len() == 2
                && matches!(&arguments[0], Expr::Integer { value, .. } if value == &BigInt::from(1))
            {
                let inverse = builtin_registry()
                    .iter()
                    .position(|builtin| {
                        builtin.name == "/"
                            && builtin.arg_type == int_type()
                            && builtin.result == rat_type()
                            && matches!(
                                builtin.implementation,
                                BuiltinImpl::Scalar(ScalarOp::IntInverse)
                            )
                    })
                    .expect("integer inverse overload is registered");
                let mut argument_type = int_type();
                let argument = convert_expr(&arguments[1], &mut argument_type, analysis)?;
                return conform_types(
                    &rat_type(),
                    required,
                    TypedExpr::BuiltinCall {
                        builtin: inverse,
                        arguments: vec![argument],
                        span: *span,
                    },
                    *span,
                    analysis,
                );
            }
            convert_builtin_application(
                &operator.symbol,
                arguments,
                required,
                *span,
                analysis,
                false,
            )
        }
        Expr::Call {
            callee,
            arguments,
            span,
        } => {
            // Call-head dispatch (axis-types.w:2396-2439): an applied
            // identifier resolves through the overload table UNLESS a local
            // function binding shadows every overload; a global function
            // value is used only when the overload table has no variants.
            if let Expr::Identifier { name, .. } = callee.as_ref() {
                let local = analysis.locals.get(name);
                let local_function = local
                    .is_some_and(|(type_, _, _)| matches!(&*type_.borrow(), Type::Function(_)));
                let use_overloads = !local_function
                    && (!overload_variants(name).is_empty()
                        || (local.is_none() && analysis.globals.lookup(name).is_none()));
                if use_overloads {
                    return convert_builtin_application(
                        name, arguments, required, *span, analysis, true,
                    );
                }
            }
            // Fallback path: the callee converts against the generic
            // function pattern (*->*), the argument against its parameter
            // type (axis-types.w:2403-2410).
            let mut function_type = Type::function(Type::Undetermined, Type::Undetermined);
            let function = convert_expr(callee, &mut function_type, analysis)?;
            let Type::Function(parts) = function_type else {
                unreachable!("converting against (*->*) yields a function type")
            };
            let (argument_type, result_type) = *parts;
            // Closures take their argument as ONE value (axis.w:3222): a
            // bare expression for a single argument, otherwise the tuple.
            let argument_source = if arguments.len() == 1 {
                arguments[0].clone()
            } else {
                Expr::Tuple {
                    elements: arguments.clone(),
                    span: *span,
                }
            };
            let mut argument_required = argument_type;
            let argument = convert_expr(&argument_source, &mut argument_required, analysis)?;
            conform_types(
                &result_type,
                required,
                TypedExpr::FunctionCall {
                    function: Box::new(function),
                    argument: Box::new(argument),
                    span: *span,
                },
                *span,
                analysis,
            )
        }
        Expr::Conditional {
            condition,
            then_branch,
            else_branch,
            span,
        } => convert_conditional_expression(
            condition,
            then_branch,
            else_branch,
            *span,
            required,
            analysis,
        )
        .map_err(|error| error.into_diagnostic(analysis)),
        // Upstream desugars the boolean connectives into conditionals at
        // parse time (parser.y:280-294); this port does it at conversion.
        Expr::Binary { op, lhs, rhs, span } => {
            let (condition, then_branch, else_branch) = match op {
                crate::syntax::BinaryOp::And => (
                    lhs.as_ref().clone(),
                    rhs.as_ref().clone(),
                    Expr::Boolean {
                        value: false,
                        span: *span,
                    },
                ),
                crate::syntax::BinaryOp::Or => (
                    lhs.as_ref().clone(),
                    Expr::Boolean {
                        value: true,
                        span: *span,
                    },
                    rhs.as_ref().clone(),
                ),
            };
            convert_expr(
                &Expr::Conditional {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                    span: *span,
                },
                required,
                analysis,
            )
        }
        Expr::Unary { op, operand, span } => match op {
            crate::syntax::UnaryOp::Not => convert_expr(
                &Expr::Conditional {
                    condition: operand.clone(),
                    then_branch: Box::new(Expr::Boolean {
                        value: false,
                        span: *span,
                    }),
                    else_branch: Box::new(Expr::Boolean {
                        value: true,
                        span: *span,
                    }),
                    span: *span,
                },
                required,
                analysis,
            ),
        },
    }
}

/// Convert a non-recursive function literal (axis.w:3093-3115): bind the
/// parameters as a new local layer, then convert the body against the
/// required pattern's result hole so a context type reaches the body (and
/// any `return`) directly. A void context converts the body against a
/// dummy result and discards the closure.
fn convert_lambda_expression(
    parameters: &[LambdaParam],
    body: &Expr,
    span: SourceSpan,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let mut names = BTreeSet::new();
    for parameter in parameters {
        if !names.insert(parameter.name.as_str()) {
            return Err(Diagnostic::new(
                ErrorKind::Name,
                format!("Multiple binding of '{}' in same scope", parameter.name),
                Some(parameter.name_span),
            ));
        }
    }
    let mut locals = analysis.locals.clone();
    // Parameterless functions push no frame at call time, so depths only
    // shift when the new layer is non-empty (the empty-layer rule).
    if !parameters.is_empty() {
        for (_, depth, _) in locals.values_mut() {
            *depth += 1;
        }
    }
    let mut parameter_types = Vec::with_capacity(parameters.len());
    for (offset, parameter) in parameters.iter().enumerate() {
        let parameter_type = parameter.type_expr.resolve();
        locals.insert(
            parameter.name.clone(),
            (Rc::new(RefCell::new(parameter_type.clone())), 0, offset),
        );
        parameter_types.push(parameter_type);
    }
    let body_analysis = Analysis {
        types: analysis.types,
        globals: analysis.globals,
        locals,
    };
    let closure = |body: TypedExpr| TypedExpr::Closure {
        parameters: parameters.len(),
        body: Rc::new(body),
    };
    if required.is_void() {
        let mut dummy = Type::Undetermined;
        let converted = convert_expr(body, &mut dummy, &body_analysis)?;
        return Ok(TypedExpr::Void(Box::new(closure(converted))));
    }
    let function_pattern = Type::function(Type::tuple(parameter_types), Type::Undetermined);
    if !required.specialise(&function_pattern, analysis.types) {
        return Err(type_error(
            format!(
                "type {} does not match required pattern {}",
                function_pattern.display(analysis.types),
                required.display(analysis.types)
            ),
            span,
        ));
    }
    let Type::Function(parts) = required else {
        unreachable!("specialising to a function pattern yields a function type")
    };
    let converted = convert_expr(body, &mut parts.1, &body_analysis)?;
    Ok(closure(converted))
}

fn convert_builtin_application(
    name: &str,
    expressions: &[Expr],
    required: &mut Type,
    span: SourceSpan,
    analysis: &Analysis<'_>,
    resolve_name_first: bool,
) -> Result<TypedExpr, Diagnostic> {
    let variants = overload_variants(name);
    if resolve_name_first && variants.is_empty() {
        // Atlas resolves the callee before analysing its arguments.  This is
        // observable for `foo(missing)`: the undefined function wins over an
        // error in an argument that would never be evaluated.
        return Err(Diagnostic::new(
            ErrorKind::Name,
            format!("Undefined identifier '{name}'"),
            Some(span),
        ));
    }
    // The a-priori-type design (axis.w:1552-1599): convert each argument
    // once in undetermined context, then choose the first exact or coercible
    // overload and re-convert divergent arguments against its signature.
    let mut converted = Vec::new();
    let mut a_priori = Vec::new();
    for expression in expressions {
        let mut slot = Type::Undetermined;
        converted.push(convert_expr(expression, &mut slot, analysis)?);
        a_priori.push(slot);
    }
    let a_priori_type = Type::tuple(a_priori.clone());
    let mut chosen = None;
    for &index in variants {
        let builtin = &builtin_registry()[index];
        if builtin.arg_type == a_priori_type {
            chosen = Some(index);
            break;
        }
        if crate::coercions::is_close(&a_priori_type, &builtin.arg_type, analysis.types) & 0x1 != 0
        {
            chosen = Some(index);
            break;
        }
    }
    let index = chosen.ok_or_else(|| {
        let message = if variants.len() == 1 {
            format!(
                "found {} while {} was needed.",
                a_priori_type.display(analysis.types),
                builtin_registry()[variants[0]]
                    .arg_type
                    .display(analysis.types),
            )
        } else {
            format!(
                "Failed to match '{}' with argument type {}",
                name,
                a_priori_type.display(analysis.types),
            )
        };
        type_error(message, span)
    })?;
    let builtin = &builtin_registry()[index];
    let expected: Vec<Type> = match &builtin.arg_type {
        Type::Tuple(components) => components.clone(),
        single => vec![single.clone()],
    };
    let arguments = converted
        .into_iter()
        .zip(a_priori)
        .zip(expected)
        .zip(expressions)
        .map(|(((converted, found), mut expected), original)| {
            if expected.specialise(&found, analysis.types) {
                Ok(converted)
            } else {
                convert_expr(original, &mut expected, analysis)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    conform_types(
        &builtin.result,
        required,
        TypedExpr::BuiltinCall {
            builtin: index,
            arguments,
            span,
        },
        span,
        analysis,
    )
}

/// Balance branch expressions to a common type (upstream `balance`,
/// axis.w:1022-1122): convert each against a copy of the target, keep the
/// broadest comparable type, defer incomparable types, then prune those
/// conflicts if a later branch supplies a type broad enough to absorb them.
fn balance(
    branches: &[&Expr],
    target: &mut Type,
    span: SourceSpan,
    analysis: &Analysis<'_>,
) -> Result<Vec<TypedExpr>, BalanceConversionError> {
    let mut converted = Vec::with_capacity(branches.len());
    let mut types = Vec::new();
    let mut common = Type::Undetermined;
    let mut conflicts = Vec::new();
    for branch in branches {
        let mut slot = target.clone();
        match convert_balanced_branch(branch, &mut slot, analysis) {
            Ok(expression) => {
                converted.push(Some(expression));
                if !crate::coercions::broader_eq(&common, &slot, analysis.types) {
                    if crate::coercions::broader_eq(&slot, &common, analysis.types) {
                        common = slot.clone();
                    } else {
                        conflicts.push(slot.clone());
                    }
                }
            }
            Err(BalanceConversionError::Diagnostic(diagnostic)) => {
                return Err(BalanceConversionError::Diagnostic(diagnostic));
            }
            Err(BalanceConversionError::Balance(failure)) => {
                if failure.span != branch.span() {
                    return Err(BalanceConversionError::Balance(failure));
                }
                // A balance error from the branch itself is salvageable at
                // this level.  A list adds one `row` layer to each variant,
                // exactly as axis.w does before pruning.
                converted.push(None);
                let variants = failure.variants.into_iter().map(|type_| {
                    if failure.container == BalanceContainer::List {
                        Type::row(type_)
                    } else {
                        type_
                    }
                });
                conflicts.extend(variants);
            }
        }
        // Failed branch conversions retain the original target as their
        // component type.  If pruning later chooses a broader common type,
        // the final reconversion below fills their previously empty slot.
        types.push(slot);
    }
    if types.is_empty() {
        return Ok(Vec::new());
    }
    conflicts.retain(|type_| !crate::coercions::broader_eq(&common, type_, analysis.types));
    if let Some(conflict) = conflicts.first().cloned() {
        let mut variants = Vec::with_capacity(conflicts.len() + 1);
        if common != Type::Undetermined {
            variants.push(common.clone());
        }
        variants.extend(conflicts);
        debug_assert!(variants.iter().any(|type_| type_ == &conflict));
        return Err(BalanceConversionError::Balance(BalanceFailure {
            span,
            variants,
            container: BalanceContainer::Unknown,
        }));
    }
    if !target.specialise(&common, analysis.types) {
        return Err(BalanceConversionError::Diagnostic(type_error(
            format!(
                "balanced type {} does not match required pattern {}",
                common.display(analysis.types),
                target.display(analysis.types),
            ),
            span,
        )));
    }
    for (index, type_) in types.iter().enumerate() {
        if type_ != &common {
            let mut slot = common.clone();
            converted[index] = Some(
                convert_expr(branches[index], &mut slot, analysis)
                    .map_err(BalanceConversionError::Diagnostic)?,
            );
        }
    }
    Ok(converted
        .into_iter()
        .map(|expression| expression.expect("balanced branch must be converted"))
        .collect())
}

/// Preserve only a balance failure owned by the branch expression itself.
/// Other conversion diagnostics are already final and must propagate without
/// being folded into the enclosing conflict set.
fn convert_balanced_branch(
    branch: &Expr,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, BalanceConversionError> {
    match branch {
        Expr::List { elements, span } => {
            convert_list_expression(elements, *span, required, analysis)
                .map_err(|error| mark_balance_failure(error, *span, BalanceContainer::List))
        }
        Expr::Conditional {
            condition,
            then_branch,
            else_branch,
            span,
        } => convert_conditional_expression(
            condition,
            then_branch,
            else_branch,
            *span,
            required,
            analysis,
        )
        .map_err(|error| mark_balance_failure(error, *span, BalanceContainer::Conditional)),
        _ => convert_expr(branch, required, analysis).map_err(BalanceConversionError::Diagnostic),
    }
}

/// One registered builtin overload.
pub struct Builtin {
    pub name: &'static str,
    pub arg_type: Type,
    pub result: Type,
    pub hunger: u8,
    implementation: BuiltinImpl,
}

#[derive(Clone, Copy)]
enum BuiltinImpl {
    Scalar(ScalarOp),
    Domain {
        name: &'static str,
        no_value: DomainNoValue,
    },
    DomainRelation(Relation),
}

#[derive(Clone, Copy)]
enum DomainNoValue {
    Skip,
    Validate,
    BuildAndDrop,
}

#[derive(Clone, Copy)]
enum ScalarOp {
    IntNegate,
    IntAdd,
    IntSubtract,
    IntMultiply,
    IntComplement,
    IntQuotient,
    IntModulo,
    IntDivMod,
    IntPower,
    IntInverse,
    IntFraction,
    RatUnfraction,
    RatAddInt,
    RatSubtractInt,
    RatMultiplyInt,
    RatDivideInt,
    RatQuotientInt,
    RatModuloInt,
    RatAdd,
    RatSubtract,
    RatMultiply,
    RatDivide,
    RatModulo,
    RatNegate,
    RatInverse,
    RatPower,
    UnaryRelation(Relation),
    BinaryRelation(Relation),
    StringConcat,
}

#[derive(Clone, Copy)]
enum Relation {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl Builtin {
    fn run(
        &self,
        arguments: Vec<Value>,
        span: SourceSpan,
        level: Level,
    ) -> Result<Option<Value>, Control> {
        match self.implementation {
            BuiltinImpl::Scalar(operation) => run_scalar(operation, arguments, span, level),
            BuiltinImpl::Domain { name, no_value } => {
                if level == Level::NoValue {
                    match no_value {
                        DomainNoValue::Skip => return Ok(None),
                        DomainNoValue::Validate => {
                            domain_builtins::validate(name, &arguments, span)
                                .map_err(Control::Runtime)?;
                            return Ok(None);
                        }
                        DomainNoValue::BuildAndDrop => {}
                    }
                }
                domain_builtins::call(name, &arguments, span)
                    .map(|value| at_builtin_level(level, || value))
                    .map_err(Control::Runtime)
            }
            BuiltinImpl::DomainRelation(relation) => {
                let (first, second) = expect_pair(arguments);
                let (Value::Domain(first), Value::Domain(second)) = (first, second) else {
                    panic!("domain relation saw non-domain arguments")
                };
                Ok(at_builtin_level(level, || {
                    Value::Boolean(match relation {
                        Relation::Equal => first == second,
                        Relation::NotEqual => first != second,
                        _ => panic!("ordered domain relation is not registered"),
                    })
                }))
            }
        }
    }
}

fn int_type() -> Type {
    Type::Primitive(Prim::Int)
}

fn int_pair() -> Type {
    Type::tuple(vec![int_type(), int_type()])
}

fn rat_type() -> Type {
    Type::Primitive(Prim::Rat)
}

fn bool_type() -> Type {
    Type::Primitive(Prim::Bool)
}

fn string_type() -> Type {
    Type::Primitive(Prim::String)
}

fn primitive_type(primitive: Prim) -> Type {
    Type::Primitive(primitive)
}

fn pair(type_: Type) -> Type {
    Type::tuple(vec![type_.clone(), type_])
}

fn scalar_builtin(
    name: &'static str,
    arg_type: Type,
    result: Type,
    hunger: u8,
    op: ScalarOp,
) -> Builtin {
    Builtin {
        name,
        arg_type,
        result,
        hunger,
        implementation: BuiltinImpl::Scalar(op),
    }
}

fn domain_builtin(name: &'static str, arg_type: Type, result: Type, hunger: u8) -> Builtin {
    domain_builtin_with_level(name, arg_type, result, hunger, DomainNoValue::BuildAndDrop)
}

fn domain_builtin_with_level(
    name: &'static str,
    arg_type: Type,
    result: Type,
    hunger: u8,
    no_value: DomainNoValue,
) -> Builtin {
    Builtin {
        name,
        arg_type,
        result,
        hunger,
        implementation: BuiltinImpl::Domain { name, no_value },
    }
}

fn domain_builtin_skip(name: &'static str, arg_type: Type, result: Type, hunger: u8) -> Builtin {
    domain_builtin_with_level(name, arg_type, result, hunger, DomainNoValue::Skip)
}

fn domain_builtin_validate(
    name: &'static str,
    arg_type: Type,
    result: Type,
    hunger: u8,
) -> Builtin {
    domain_builtin_with_level(name, arg_type, result, hunger, DomainNoValue::Validate)
}

fn domain_relation_builtin(name: &'static str, arg_type: Type, relation: Relation) -> Builtin {
    Builtin {
        name,
        arg_type,
        result: bool_type(),
        hunger: 0,
        implementation: BuiltinImpl::DomainRelation(relation),
    }
}

fn at_builtin_level(level: Level, value: impl FnOnce() -> Value) -> Option<Value> {
    match level {
        Level::NoValue => None,
        Level::SingleValue => Some(value()),
    }
}

fn expect_unary(mut arguments: Vec<Value>) -> Value {
    let value = arguments.pop().expect("unary builtin has one argument");
    assert!(arguments.is_empty(), "unary builtin saw extra arguments");
    value
}

fn expect_ints(mut arguments: Vec<Value>) -> (BigInt, BigInt) {
    let second = arguments.pop();
    let first = arguments.pop();
    match (first, second) {
        (Some(Value::Integer(first)), Some(Value::Integer(second))) => (first, second),
        other => panic!("int builtin saw {other:?}"),
    }
}

fn expect_rationals(mut arguments: Vec<Value>) -> (BigRational, BigRational) {
    let second = arguments.pop();
    let first = arguments.pop();
    match (first, second) {
        (Some(Value::Rational(first)), Some(Value::Rational(second))) => (first, second),
        other => panic!("rat builtin saw {other:?}"),
    }
}

fn expect_rat_int(mut arguments: Vec<Value>) -> (BigRational, BigInt) {
    let second = arguments.pop();
    let first = arguments.pop();
    match (first, second) {
        (Some(Value::Rational(first)), Some(Value::Integer(second))) => (first, second),
        other => panic!("rat-int builtin saw {other:?}"),
    }
}

fn expect_pair(mut arguments: Vec<Value>) -> (Value, Value) {
    let second = arguments.pop();
    let first = arguments.pop();
    match (first, second) {
        (Some(first), Some(second)) => (first, second),
        other => panic!("binary builtin saw {other:?}"),
    }
}

fn euclidean_divmod(left: &BigInt, right: &BigInt) -> (BigInt, BigInt) {
    let mut quotient = left / right;
    let mut remainder = left % right;
    if remainder != 0 && ((remainder < 0) != (*right < 0)) {
        quotient -= BigInt::from(1);
        remainder += right;
    }
    (quotient, remainder)
}

fn run_scalar(
    operation: ScalarOp,
    arguments: Vec<Value>,
    span: SourceSpan,
    level: Level,
) -> Result<Option<Value>, Control> {
    match operation {
        ScalarOp::IntNegate => match expect_unary(arguments) {
            Value::Integer(value) => Ok(at_builtin_level(level, || Value::Integer(-value))),
            other => panic!("integer negation saw {other:?}"),
        },
        ScalarOp::IntAdd | ScalarOp::IntSubtract | ScalarOp::IntMultiply => {
            let (first, second) = expect_ints(arguments);
            Ok(at_builtin_level(level, || {
                Value::Integer(match operation {
                    ScalarOp::IntAdd => first + second,
                    ScalarOp::IntSubtract => first - second,
                    ScalarOp::IntMultiply => first * second,
                    _ => unreachable!(),
                })
            }))
        }
        ScalarOp::IntComplement => match expect_unary(arguments) {
            Value::Integer(value) => Ok(at_builtin_level(level, || Value::Integer(!value))),
            other => panic!("integer complement saw {other:?}"),
        },
        ScalarOp::IntQuotient | ScalarOp::IntModulo | ScalarOp::IntDivMod => {
            let (first, second) = expect_ints(arguments);
            if second == 0 {
                let message = match operation {
                    ScalarOp::IntQuotient => "Division by zero",
                    ScalarOp::IntModulo => "Modulo zero",
                    ScalarOp::IntDivMod => "DivMod by zero",
                    _ => unreachable!(),
                };
                return Err(runtime(message, span));
            }
            Ok(at_builtin_level(level, || {
                let (quotient, remainder) = euclidean_divmod(&first, &second);
                match operation {
                    ScalarOp::IntQuotient => Value::Integer(quotient),
                    ScalarOp::IntModulo => Value::Integer(remainder),
                    ScalarOp::IntDivMod => {
                        Value::Tuple(vec![Value::Integer(quotient), Value::Integer(remainder)])
                    }
                    _ => unreachable!(),
                }
            }))
        }
        ScalarOp::IntPower => {
            let (base, exponent) = expect_ints(arguments);
            let unit_base = base == 1 || base == -1;
            if !unit_base && exponent < 0 {
                return Err(runtime("Negative power of integer", span));
            }
            if !unit_base && base != 0 && i32::try_from(&exponent).is_err() {
                return Err(runtime("Exponent too large in power of integer", span));
            }
            Ok(at_builtin_level(level, || {
                if unit_base {
                    if &exponent % BigInt::from(2) != 0 {
                        Value::Integer(base)
                    } else {
                        Value::Integer(BigInt::from(1))
                    }
                } else if base == 0 {
                    Value::Integer(if exponent == 0 { BigInt::from(1) } else { base })
                } else {
                    Value::Integer(
                        base.pow(u64::from(
                            u32::try_from(i32::try_from(&exponent).expect("validated exponent"))
                                .expect("validated exponent is nonnegative"),
                        )),
                    )
                }
            }))
        }
        ScalarOp::IntInverse => match expect_unary(arguments) {
            Value::Integer(value) => {
                if value == 0 {
                    return Err(runtime("Inverse of zero", span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Rational(BigRational::from_integers(BigInt::from(1), value))
                }))
            }
            other => panic!("integer inverse saw {other:?}"),
        },
        ScalarOp::IntFraction => {
            let (numerator, denominator) = expect_ints(arguments);
            if denominator == 0 {
                return Err(runtime("fraction with zero denominator", span));
            }
            Ok(at_builtin_level(level, || {
                Value::Rational(BigRational::from_integers(numerator, denominator))
            }))
        }
        ScalarOp::RatUnfraction => match expect_unary(arguments) {
            Value::Rational(value) => Ok(at_builtin_level(level, || {
                let negative = value < 0;
                let (numerator, denominator) = value.into_numerator_and_denominator();
                let numerator = BigInt::from(numerator);
                Value::Tuple(vec![
                    Value::Integer(if negative { -numerator } else { numerator }),
                    Value::Integer(BigInt::from(denominator)),
                ])
            })),
            other => panic!("rational unfraction saw {other:?}"),
        },
        ScalarOp::RatAddInt
        | ScalarOp::RatSubtractInt
        | ScalarOp::RatMultiplyInt
        | ScalarOp::RatDivideInt
        | ScalarOp::RatQuotientInt
        | ScalarOp::RatModuloInt => {
            let (rational, integer) = expect_rat_int(arguments);
            if integer == 0 {
                match operation {
                    ScalarOp::RatQuotientInt => {
                        return Err(runtime("Rational quotient by zero", span));
                    }
                    ScalarOp::RatDivideInt if level == Level::SingleValue => {
                        return Err(runtime("Division of rational by integer zero", span));
                    }
                    ScalarOp::RatModuloInt if level == Level::SingleValue => {
                        return Err(runtime("Division by zero", span));
                    }
                    _ => {}
                }
            }
            Ok(at_builtin_level(level, || {
                let integer_as_rational = BigRational::from(integer);
                match operation {
                    ScalarOp::RatAddInt => Value::Rational(rational + integer_as_rational),
                    ScalarOp::RatSubtractInt => Value::Rational(rational - integer_as_rational),
                    ScalarOp::RatMultiplyInt => Value::Rational(rational * integer_as_rational),
                    ScalarOp::RatDivideInt => Value::Rational(rational / integer_as_rational),
                    ScalarOp::RatQuotientInt => {
                        Value::Integer((rational / integer_as_rational).floor())
                    }
                    ScalarOp::RatModuloInt => Value::Rational(rational.mod_op(integer_as_rational)),
                    _ => unreachable!(),
                }
            }))
        }
        ScalarOp::RatAdd
        | ScalarOp::RatSubtract
        | ScalarOp::RatMultiply
        | ScalarOp::RatDivide
        | ScalarOp::RatModulo => {
            let (first, second) = expect_rationals(arguments);
            if second == 0 {
                let message = match operation {
                    ScalarOp::RatDivide => "Rational division by zero",
                    ScalarOp::RatModulo => "Rational modulo zero",
                    _ => "",
                };
                if !message.is_empty() {
                    return Err(runtime(message, span));
                }
            }
            Ok(at_builtin_level(level, || {
                Value::Rational(match operation {
                    ScalarOp::RatAdd => first + second,
                    ScalarOp::RatSubtract => first - second,
                    ScalarOp::RatMultiply => first * second,
                    ScalarOp::RatDivide => first / second,
                    ScalarOp::RatModulo => first.mod_op(second),
                    _ => unreachable!(),
                })
            }))
        }
        ScalarOp::RatNegate => match expect_unary(arguments) {
            Value::Rational(value) => Ok(at_builtin_level(level, || Value::Rational(-value))),
            other => panic!("rational negation saw {other:?}"),
        },
        ScalarOp::RatInverse => match expect_unary(arguments) {
            Value::Rational(value) => {
                if value == 0 {
                    return Err(runtime("Inverse of zero", span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Rational(BigRational::from(1) / value)
                }))
            }
            other => panic!("rational inverse saw {other:?}"),
        },
        ScalarOp::RatPower => {
            let (base, exponent) = expect_rat_int(arguments);
            let unit_base = base == 1 || base == -1;
            if base == 0 && exponent < 0 {
                return Err(runtime("Negative power of rational zero", span));
            }
            if base != 0 && !unit_base && i32::try_from(&exponent).is_err() {
                return Err(runtime(
                    "Exponent too large in power of rational number",
                    span,
                ));
            }
            if level == Level::NoValue {
                return Ok(None);
            }
            if base != 0 && !unit_base && exponent < 0 {
                return Err(runtime("Negative integer where unsigned is required", span));
            }
            Ok(Some({
                if unit_base {
                    if &exponent % BigInt::from(2) != 0 {
                        Value::Rational(base)
                    } else {
                        Value::Rational(BigRational::from(1))
                    }
                } else if base == 0 {
                    Value::Rational(if exponent == 0 {
                        BigRational::from(1)
                    } else {
                        base
                    })
                } else {
                    Value::Rational(base.pow(i64::from(
                        i32::try_from(&exponent).expect("validated exponent"),
                    )))
                }
            }))
        }
        ScalarOp::UnaryRelation(relation) => {
            let value = expect_unary(arguments);
            Ok(at_builtin_level(level, || {
                let ordering = match value {
                    Value::Integer(value) => value.cmp(&BigInt::from(0)),
                    Value::Rational(value) => value.cmp(&BigRational::from(0)),
                    Value::String(value) => value.as_str().cmp(""),
                    other => panic!("unary relation saw {other:?}"),
                };
                Value::Boolean(relation_matches(relation, ordering))
            }))
        }
        ScalarOp::BinaryRelation(relation) => {
            let (first, second) = expect_pair(arguments);
            Ok(at_builtin_level(level, || {
                let ordering = match (first, second) {
                    (Value::Integer(first), Value::Integer(second)) => first.cmp(&second),
                    (Value::Rational(first), Value::Rational(second)) => first.cmp(&second),
                    (Value::Boolean(first), Value::Boolean(second)) => first.cmp(&second),
                    (Value::String(first), Value::String(second)) => first.cmp(&second),
                    other => panic!("binary relation saw {other:?}"),
                };
                Value::Boolean(relation_matches(relation, ordering))
            }))
        }
        ScalarOp::StringConcat => {
            let (first, second) = expect_pair(arguments);
            match (first, second) {
                (Value::String(first), Value::String(second)) => {
                    Ok(at_builtin_level(level, || {
                        Value::String(format!("{first}{second}"))
                    }))
                }
                other => panic!("string concatenation saw {other:?}"),
            }
        }
    }
}

fn relation_matches(relation: Relation, ordering: Ordering) -> bool {
    match relation {
        Relation::Equal => ordering == Ordering::Equal,
        Relation::NotEqual => ordering != Ordering::Equal,
        Relation::Less => ordering == Ordering::Less,
        Relation::LessEqual => ordering != Ordering::Greater,
        Relation::Greater => ordering == Ordering::Greater,
        Relation::GreaterEqual => ordering != Ordering::Less,
    }
}

/// The startup builtin registry (growing toward the traced inventory).
pub fn builtin_registry() -> &'static Vec<Builtin> {
    static REGISTRY: OnceLock<Vec<Builtin>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        vec![
            scalar_builtin("-", int_type(), int_type(), 3, ScalarOp::IntNegate),
            scalar_builtin("+", int_pair(), int_type(), 1, ScalarOp::IntAdd),
            scalar_builtin("-", int_pair(), int_type(), 1, ScalarOp::IntSubtract),
            scalar_builtin("~", int_type(), int_type(), 3, ScalarOp::IntComplement),
            scalar_builtin("*", int_pair(), int_type(), 0, ScalarOp::IntMultiply),
            scalar_builtin("\\", int_pair(), int_type(), 1, ScalarOp::IntQuotient),
            scalar_builtin("%", int_pair(), int_type(), 1, ScalarOp::IntModulo),
            scalar_builtin("\\%", int_pair(), int_pair(), 1, ScalarOp::IntDivMod),
            scalar_builtin("^", int_pair(), int_type(), 0, ScalarOp::IntPower),
            scalar_builtin("/", int_type(), rat_type(), 0, ScalarOp::IntInverse),
            scalar_builtin("/", int_pair(), rat_type(), 0, ScalarOp::IntFraction),
            scalar_builtin("%", rat_type(), int_pair(), 0, ScalarOp::RatUnfraction),
            scalar_builtin(
                "+",
                Type::tuple(vec![rat_type(), int_type()]),
                rat_type(),
                1,
                ScalarOp::RatAddInt,
            ),
            scalar_builtin(
                "-",
                Type::tuple(vec![rat_type(), int_type()]),
                rat_type(),
                1,
                ScalarOp::RatSubtractInt,
            ),
            scalar_builtin(
                "*",
                Type::tuple(vec![rat_type(), int_type()]),
                rat_type(),
                1,
                ScalarOp::RatMultiplyInt,
            ),
            scalar_builtin(
                "/",
                Type::tuple(vec![rat_type(), int_type()]),
                rat_type(),
                0,
                ScalarOp::RatDivideInt,
            ),
            scalar_builtin(
                "\\",
                Type::tuple(vec![rat_type(), int_type()]),
                int_type(),
                0,
                ScalarOp::RatQuotientInt,
            ),
            scalar_builtin(
                "%",
                Type::tuple(vec![rat_type(), int_type()]),
                rat_type(),
                1,
                ScalarOp::RatModuloInt,
            ),
            scalar_builtin("+", pair(rat_type()), rat_type(), 1, ScalarOp::RatAdd),
            scalar_builtin("-", pair(rat_type()), rat_type(), 1, ScalarOp::RatSubtract),
            scalar_builtin("*", pair(rat_type()), rat_type(), 1, ScalarOp::RatMultiply),
            scalar_builtin("/", pair(rat_type()), rat_type(), 1, ScalarOp::RatDivide),
            scalar_builtin("%", pair(rat_type()), rat_type(), 1, ScalarOp::RatModulo),
            scalar_builtin("-", rat_type(), rat_type(), 3, ScalarOp::RatNegate),
            scalar_builtin("/", rat_type(), rat_type(), 3, ScalarOp::RatInverse),
            scalar_builtin(
                "^",
                Type::tuple(vec![rat_type(), int_type()]),
                rat_type(),
                1,
                ScalarOp::RatPower,
            ),
            scalar_builtin(
                "=",
                int_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                int_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                ">=",
                int_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::GreaterEqual),
            ),
            scalar_builtin(
                ">",
                int_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Greater),
            ),
            scalar_builtin(
                "<=",
                int_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::LessEqual),
            ),
            scalar_builtin(
                "<",
                int_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Less),
            ),
            scalar_builtin(
                "=",
                int_pair(),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                int_pair(),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                "<",
                int_pair(),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Less),
            ),
            scalar_builtin(
                "<=",
                int_pair(),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::LessEqual),
            ),
            scalar_builtin(
                ">",
                int_pair(),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Greater),
            ),
            scalar_builtin(
                ">=",
                int_pair(),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::GreaterEqual),
            ),
            scalar_builtin(
                "=",
                rat_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                rat_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                ">=",
                rat_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::GreaterEqual),
            ),
            scalar_builtin(
                ">",
                rat_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Greater),
            ),
            scalar_builtin(
                "<=",
                rat_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::LessEqual),
            ),
            scalar_builtin(
                "<",
                rat_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Less),
            ),
            scalar_builtin(
                "=",
                pair(rat_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                pair(rat_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                "<",
                pair(rat_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Less),
            ),
            scalar_builtin(
                "<=",
                pair(rat_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::LessEqual),
            ),
            scalar_builtin(
                ">",
                pair(rat_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Greater),
            ),
            scalar_builtin(
                ">=",
                pair(rat_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::GreaterEqual),
            ),
            scalar_builtin(
                "=",
                pair(bool_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                pair(bool_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                "=",
                string_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                string_type(),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                "=",
                pair(string_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                pair(string_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                "<",
                pair(string_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Less),
            ),
            scalar_builtin(
                "<=",
                pair(string_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::LessEqual),
            ),
            scalar_builtin(
                ">",
                pair(string_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Greater),
            ),
            scalar_builtin(
                ">=",
                pair(string_type()),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::GreaterEqual),
            ),
            scalar_builtin(
                "##",
                pair(string_type()),
                string_type(),
                1,
                ScalarOp::StringConcat,
            ),
            domain_builtin("Lie_type", string_type(), primitive_type(Prim::LieType), 0),
            domain_builtin(
                "Lie_type",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::LieType),
                0,
            ),
            domain_builtin_skip(
                "prefers_coroots",
                primitive_type(Prim::RootDatum),
                bool_type(),
                0,
            ),
            domain_builtin_skip(
                "simply_connected",
                Type::tuple(vec![primitive_type(Prim::LieType), bool_type()]),
                primitive_type(Prim::RootDatum),
                0,
            ),
            domain_builtin_skip(
                "adjoint",
                Type::tuple(vec![primitive_type(Prim::LieType), bool_type()]),
                primitive_type(Prim::RootDatum),
                0,
            ),
            domain_builtin(
                "root_datum",
                Type::tuple(vec![
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::Mat),
                    bool_type(),
                ]),
                primitive_type(Prim::RootDatum),
                0,
            ),
            domain_builtin(
                "root_datum",
                Type::tuple(vec![
                    primitive_type(Prim::LieType),
                    primitive_type(Prim::Mat),
                    bool_type(),
                ]),
                primitive_type(Prim::RootDatum),
                0,
            ),
            domain_builtin(
                "root_datum",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::Mat),
                ]),
                primitive_type(Prim::RootDatum),
                0,
            ),
            domain_builtin(
                "root_datum",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::RootDatum),
                0,
            ),
            domain_builtin_skip(
                "Cartan_matrix",
                primitive_type(Prim::LieType),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin_skip(
                "Cartan_matrix",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "inner_class",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::Mat),
                ]),
                primitive_type(Prim::InnerClass),
                0,
            ),
            domain_builtin_skip(
                "inner_class",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::InnerClass),
                0,
            ),
            domain_builtin_skip(
                "nr_of_real_forms",
                primitive_type(Prim::InnerClass),
                int_type(),
                0,
            ),
            domain_builtin_validate(
                "real_form",
                Type::tuple(vec![primitive_type(Prim::InnerClass), int_type()]),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin_skip(
                "quasisplit_form",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin_skip("form_number", primitive_type(Prim::RealForm), int_type(), 0),
            domain_builtin_skip("KGB_size", primitive_type(Prim::RealForm), int_type(), 0),
            domain_builtin_validate(
                "KGB",
                Type::tuple(vec![primitive_type(Prim::RealForm), int_type()]),
                primitive_type(Prim::KgbElt),
                0,
            ),
            domain_builtin_validate(
                "cross",
                Type::tuple(vec![int_type(), primitive_type(Prim::KgbElt)]),
                primitive_type(Prim::KgbElt),
                2,
            ),
            domain_builtin_validate(
                "Cayley",
                Type::tuple(vec![int_type(), primitive_type(Prim::KgbElt)]),
                primitive_type(Prim::KgbElt),
                2,
            ),
            domain_builtin_validate(
                "status",
                Type::tuple(vec![int_type(), primitive_type(Prim::KgbElt)]),
                int_type(),
                0,
            ),
            domain_builtin_skip("length", primitive_type(Prim::KgbElt), int_type(), 0),
            domain_builtin_skip(
                "involution",
                primitive_type(Prim::KgbElt),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin_skip(
                "torus_factor",
                primitive_type(Prim::KgbElt),
                primitive_type(Prim::RatVec),
                0,
            ),
            domain_relation_builtin("=", pair(primitive_type(Prim::LieType)), Relation::Equal),
            domain_relation_builtin(
                "!=",
                pair(primitive_type(Prim::LieType)),
                Relation::NotEqual,
            ),
            domain_relation_builtin("=", pair(primitive_type(Prim::RootDatum)), Relation::Equal),
            domain_relation_builtin(
                "!=",
                pair(primitive_type(Prim::RootDatum)),
                Relation::NotEqual,
            ),
            domain_relation_builtin("=", pair(primitive_type(Prim::InnerClass)), Relation::Equal),
            domain_relation_builtin(
                "!=",
                pair(primitive_type(Prim::InnerClass)),
                Relation::NotEqual,
            ),
            domain_relation_builtin("=", pair(primitive_type(Prim::RealForm)), Relation::Equal),
            domain_relation_builtin(
                "!=",
                pair(primitive_type(Prim::RealForm)),
                Relation::NotEqual,
            ),
            domain_relation_builtin("=", pair(primitive_type(Prim::KgbElt)), Relation::Equal),
            domain_relation_builtin("!=", pair(primitive_type(Prim::KgbElt)), Relation::NotEqual),
        ]
    })
}

/// Variant indices for `name`, most-specific first as in `locate_overload`.
fn overload_variants(name: &str) -> &'static [usize] {
    static INDEX: OnceLock<BTreeMap<&'static str, Vec<usize>>> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            let mut index: BTreeMap<&'static str, Vec<usize>> = BTreeMap::new();
            let table = TypeTable::new();
            for (position, builtin) in builtin_registry().iter().enumerate() {
                let variants = index.entry(builtin.name).or_default();
                let mut lower = 0;
                let mut upper = variants.len();
                for (slot, &existing) in variants.iter().enumerate() {
                    let existing = &builtin_registry()[existing];
                    match crate::coercions::is_close(&builtin.arg_type, &existing.arg_type, &table)
                    {
                        0x6 => lower = slot + 1,
                        0x5 => upper = upper.min(slot),
                        0x7 if builtin.arg_type == existing.arg_type => {
                            panic!("duplicate startup overload for {}", builtin.name)
                        }
                        0x4 | 0x7 => panic!("ambiguous startup overload for {}", builtin.name),
                        _ => {}
                    }
                }
                assert!(lower <= upper, "conflicting startup overload order");
                variants.insert(upper, position);
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
            Self::Void(inner) => {
                inner.evaluate(context, Level::NoValue)?;
                Ok(at_level(level, || Value::Tuple(Vec::new())))
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
            Self::LocalIdent {
                name,
                depth,
                offset,
                span,
            } => match context.local(*depth, *offset) {
                Some(value) => Ok(at_level(level, || value.as_ref().clone())),
                None => Err(runtime(
                    format!("Taking value of uninitialized variable '{name}'"),
                    *span,
                )),
            },
            Self::GlobalAssignment { cell, value } => {
                let value = force(value, context)?;
                *cell.borrow_mut() = Some(std::rc::Rc::new(value.clone()));
                Ok(at_level(level, || value.clone()))
            }
            Self::LocalAssignment {
                depth,
                offset,
                value,
            } => {
                let value = force(value, context)?;
                let updated = context.set_local(*depth, *offset, std::rc::Rc::new(value.clone()));
                assert!(
                    updated,
                    "analysis emitted an invalid local assignment address"
                );
                Ok(at_level(level, || value.clone()))
            }
            Self::Subscription {
                array,
                index,
                reversed,
                span,
            } => {
                let index = expect_integer(force(index, context)?, *span, "subscription index")?;
                let values = expect_typed_list(force(array, context)?, *span, "subscription")?;
                let position = checked_index(&index, values.len(), *reversed, *span)?;
                Ok(at_level(level, || values[position].clone()))
            }
            Self::Slice {
                array,
                lower,
                upper,
                flags,
                span,
            } => {
                let upper = expect_integer(force(upper, context)?, *span, "slice upper bound")?;
                let lower = expect_integer(force(lower, context)?, *span, "slice lower bound")?;
                let values = expect_typed_list(force(array, context)?, *span, "slice")?;
                let sliced = evaluate_slice(values, lower, upper, *flags, *span)?;
                Ok(at_level(level, || Value::List(sliced.clone())))
            }
            Self::LetGroup { initializers, body } => {
                let values = initializers
                    .iter()
                    .map(|initializer| force(initializer, context).map(std::rc::Rc::new))
                    .collect::<Result<Vec<_>, _>>()?;
                context.with_frame(values, |context| body.evaluate(context, level))
            }
            Self::Conditional {
                condition,
                then_branch,
                else_branch,
            } => match force(condition, context)? {
                Value::Boolean(true) => then_branch.evaluate(context, level),
                Value::Boolean(false) => else_branch.evaluate(context, level),
                other => panic!("analysis let a non-boolean condition through: {other}"),
            },
            Self::BuiltinCall {
                builtin,
                arguments,
                span,
            } => {
                let values = arguments
                    .iter()
                    .map(|argument| force(argument, context))
                    .collect::<Result<Vec<_>, _>>()?;
                builtin_registry()[*builtin].run(values, *span, level)
            }
            Self::Closure { parameters, body } => Ok(at_level(level, || {
                Value::Closure(Rc::new(Closure {
                    parameters: *parameters,
                    body: Rc::clone(body),
                    frame: context.capture(),
                }))
            })),
            Self::Return { value } => {
                let value = force(value, context)?;
                Err(Control::Return(value))
            }
            Self::FunctionCall {
                function, argument, ..
            } => {
                let closure = force(function, context)?;
                let Value::Closure(closure) = closure else {
                    panic!("analysis let a non-function callee through: {closure}")
                };
                let argument = force(argument, context)?;
                // The argument is one value: a tuple destructures into the
                // frame slots, anything else binds the single parameter; a
                // parameterless call pushes no frame (empty-layer rule).
                let slots = match closure.parameters {
                    0 => None,
                    1 => Some(vec![Rc::new(argument)]),
                    _ => match argument {
                        Value::Tuple(values) => Some(values.into_iter().map(Rc::new).collect()),
                        other => panic!("multi-parameter call saw non-tuple argument {other}"),
                    },
                };
                context.with_context(closure.frame.clone(), |context| {
                    let result = match slots {
                        Some(slots) => context
                            .with_frame(slots, |context| closure.body.evaluate(context, level)),
                        None => closure.body.evaluate(context, level),
                    };
                    match result {
                        // An explicit `return` ends the call and supplies
                        // its value (upstream function_return caught in
                        // apply, axis.w:3569-3571).
                        Err(Control::Return(value)) => Ok(at_level(level, move || value.clone())),
                        other => other,
                    }
                })
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

fn expect_typed_list(
    value: Value,
    span: SourceSpan,
    operation: &str,
) -> Result<Vec<Value>, Control> {
    match value {
        Value::List(values) => Ok(values),
        _ => Err(runtime(format!("{operation} requires a list"), span)),
    }
}

fn expect_integer(value: Value, span: SourceSpan, operation: &str) -> Result<BigInt, Control> {
    match value {
        Value::Integer(value) => Ok(value),
        _ => Err(Control::Runtime(Diagnostic::new(
            ErrorKind::Type,
            format!("{operation} must be an integer"),
            Some(span),
        ))),
    }
}

fn checked_index(
    index: &BigInt,
    length: usize,
    reversed: bool,
    span: SourceSpan,
) -> Result<usize, Control> {
    let original = index.clone();
    let index = usize::try_from(index).map_err(|_| {
        runtime(
            format!("index {original} out of range (0<= . <{length}) in subscription"),
            span,
        )
    })?;
    if index >= length {
        return Err(runtime(
            format!("index {original} out of range (0<= . <{length}) in subscription"),
            span,
        ));
    }
    Ok(if reversed { length - 1 - index } else { index })
}

fn evaluate_slice(
    values: Vec<Value>,
    lower: BigInt,
    upper: BigInt,
    flags: crate::syntax::SliceFlags,
    span: SourceSpan,
) -> Result<Vec<Value>, Control> {
    let length = BigInt::from(values.len());
    let lower = if flags.lower_from_end {
        &length - lower
    } else {
        lower
    };
    let upper = if flags.upper_from_end {
        &length - upper
    } else {
        upper
    };
    let lower_out_of_range = lower < 0;
    let upper_out_of_range = upper > length;
    if lower_out_of_range || upper_out_of_range {
        let message = match (lower_out_of_range, upper_out_of_range) {
            (true, true) => format!(
                "both bounds {lower}:{upper} out of range (should be >=0 respectively <= {}) in slice",
                values.len()
            ),
            (true, false) => {
                format!("lower bound {lower} out of range (should be >=0) in slice")
            }
            (false, true) => format!(
                "upper bound {upper} out of range (should be <= {}) in slice",
                values.len()
            ),
            (false, false) => unreachable!(),
        };
        return Err(runtime(message, span));
    }
    // Atlas treats a reversed/empty interval as an empty result before it
    // narrows either bound to a machine index.  This matters for e.g.
    // `[0:-1]`, whose negative upper bound is valid only because the range is
    // already empty.
    if lower >= upper {
        return Ok(Vec::new());
    }
    let lower_index = usize::try_from(&lower)
        .map_err(|_| runtime("slice lower bound is not a machine index", span))?;
    let upper_index = usize::try_from(&upper)
        .map_err(|_| runtime("slice upper bound is not a machine index", span))?;
    let mut result = values[lower_index..upper_index].to_vec();
    if flags.reverse_output {
        result.reverse();
    }
    Ok(result)
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
    if columns.is_empty() {
        return Err(runtime(
            "Implicit conversion to matrix for an empty set of vectors",
            span,
        ));
    }
    let rows = columns.first().map_or(0, |column| column.0.len());
    if columns.iter().any(|column| column.0.len() != rows) {
        return Err(runtime("matrix columns must have equal lengths", span));
    }
    let cols = columns.len();
    let data = columns.into_iter().flat_map(|column| column.0).collect();
    Matrix::from_columns(rows, cols, data)
        .ok_or_else(|| runtime("inconsistent matrix dimensions", span))
}

fn matrix_columns(matrix: Matrix) -> Vec<Vec<i32>> {
    (0..matrix.cols())
        .map(|column| {
            (0..matrix.rows())
                .map(|row| {
                    matrix
                        .entry(row, column)
                        .expect("matrix dimensions guarantee in-range entries")
                })
                .collect()
        })
        .collect()
}

fn rational_value(numerator: impl Into<BigInt>, denominator: impl Into<BigInt>) -> Value {
    Value::Rational(BigRational::from_integers(
        numerator.into(),
        denominator.into(),
    ))
}

fn matrix_to_vectors(matrix: Matrix) -> Value {
    Value::List(
        matrix_columns(matrix)
            .into_iter()
            .map(|column| Value::Vector(Vec32(column)))
            .collect(),
    )
}

fn matrix_to_integer_rows(matrix: Matrix) -> Value {
    Value::List(
        matrix_columns(matrix)
            .into_iter()
            .map(|column| {
                Value::List(
                    column
                        .into_iter()
                        .map(|entry| Value::Integer(BigInt::from(entry)))
                        .collect(),
                )
            })
            .collect(),
    )
}

fn matrix_to_ratvectors(matrix: Matrix) -> Value {
    Value::List(
        matrix_columns(matrix)
            .into_iter()
            .map(|column| {
                Value::RatVector(
                    RatVec::new(column.into_iter().map(i64::from).collect(), 1)
                        .expect("unit denominator is nonzero"),
                )
            })
            .collect(),
    )
}

fn matrix_to_rational_rows(matrix: Matrix) -> Value {
    Value::List(
        matrix_columns(matrix)
            .into_iter()
            .map(|column| {
                Value::List(
                    column
                        .into_iter()
                        .map(|entry| rational_value(entry, 1))
                        .collect(),
                )
            })
            .collect(),
    )
}

fn vectors_to_ratvectors(value: Value) -> Value {
    Value::List(
        expect_list(value)
            .into_iter()
            .map(|entry| match entry {
                Value::Vector(vector) => Value::RatVector(
                    RatVec::new(vector.0.into_iter().map(i64::from).collect(), 1)
                        .expect("unit denominator is nonzero"),
                ),
                other => panic!("[Qv][V] conversion saw {other}"),
            })
            .collect(),
    )
}

fn vectors_to_rational_rows(value: Value) -> Value {
    Value::List(
        expect_list(value)
            .into_iter()
            .map(|entry| match entry {
                Value::Vector(vector) => Value::List(
                    vector
                        .0
                        .into_iter()
                        .map(|item| rational_value(item, 1))
                        .collect(),
                ),
                other => panic!("[[Q]][V] conversion saw {other}"),
            })
            .collect(),
    )
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
        "[Q]Qv" => match value {
            Value::RatVector(vector) => Ok(Value::List(
                vector
                    .numerators()
                    .iter()
                    .map(|&numerator| rational_value(numerator, BigInt::from(vector.denominator())))
                    .collect(),
            )),
            other => panic!("[Q]Qv conversion saw {other}"),
        },
        "[Q]V" => match value {
            Value::Vector(vector) => Ok(Value::List(
                vector
                    .0
                    .into_iter()
                    .map(|entry| rational_value(entry, 1))
                    .collect(),
            )),
            other => panic!("[Q]V conversion saw {other}"),
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
        "[V]M" => match value {
            Value::Matrix(matrix) => Ok(matrix_to_vectors(matrix)),
            other => panic!("[V]M conversion saw {other}"),
        },
        "[[I]]M" => match value {
            Value::Matrix(matrix) => Ok(matrix_to_integer_rows(matrix)),
            other => panic!("[[I]]M conversion saw {other}"),
        },
        "[Qv]M" => match value {
            Value::Matrix(matrix) => Ok(matrix_to_ratvectors(matrix)),
            other => panic!("[Qv]M conversion saw {other}"),
        },
        "[[Q]]M" => match value {
            Value::Matrix(matrix) => Ok(matrix_to_rational_rows(matrix)),
            other => panic!("[[Q]]M conversion saw {other}"),
        },
        "[V][[I]]" => Ok(Value::List(
            expect_list(value)
                .into_iter()
                .map(|entry| Ok(Value::Vector(list_to_vec32(expect_list(entry), span)?)))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        "[[I]][V]" => Ok(Value::List(
            expect_list(value)
                .into_iter()
                .map(|entry| match entry {
                    Value::Vector(vector) => Ok(Value::List(
                        vector
                            .0
                            .into_iter()
                            .map(|item| Value::Integer(BigInt::from(item)))
                            .collect(),
                    )),
                    other => panic!("[[I]][V] conversion saw {other}"),
                })
                .collect::<Result<Vec<_>, Control>>()?,
        )),
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
        "[Qv][V]" => Ok(vectors_to_ratvectors(value)),
        "[[Q]][V]" => Ok(vectors_to_rational_rows(value)),
        "[Qv][[I]]" => Ok(Value::List(
            expect_list(value)
                .into_iter()
                .map(|entry| {
                    let vector = list_to_vec32(expect_list(entry), span)?;
                    Ok(Value::RatVector(
                        RatVec::new(vector.0.into_iter().map(i64::from).collect(), 1)
                            .expect("unit denominator is nonzero"),
                    ))
                })
                .collect::<Result<Vec<_>, Control>>()?,
        )),
        "[[Q]][[I]]" => Ok(Value::List(
            expect_list(value)
                .into_iter()
                .map(|entry| {
                    Ok(Value::List(
                        list_to_vec32(expect_list(entry), span)?
                            .0
                            .into_iter()
                            .map(|item| rational_value(item, 1))
                            .collect(),
                    ))
                })
                .collect::<Result<Vec<_>, Control>>()?,
        )),
        "LT" | "RdIc" | "IcRf" | "RdRf" => {
            domain_builtins::coerce(tag, value, span).map_err(Control::Runtime)
        }
        other => Err(runtime(
            format!("conversion '{other}' is not yet implemented"),
            span,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::{tokenize, TokenKind};
    use crate::source::SourceText;
    use crate::syntax::{parse, parse_command};
    use std::rc::Rc;

    fn convert_and_run_with(source: &str, globals: &IdTable) -> Result<(Type, Value), Diagnostic> {
        let source = SourceText::new(source);
        let program = parse(&source)
            .unwrap_or_else(|error| panic!("test source {source:?} parses: {error:?}"));
        assert_eq!(program.expressions.len(), 1);
        let table = TypeTable::new();
        let analysis = Analysis::new(&table, globals);
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

    fn command(source: &str) -> Command {
        let source = SourceText::new(source);
        let tokens = tokenize(&source)
            .expect("command tokenizes")
            .into_iter()
            .filter(|token| !matches!(token.kind, TokenKind::Newline | TokenKind::Eof))
            .collect::<Vec<_>>();
        parse_command(&tokens, &source).expect("command parses")
    }

    #[test]
    fn typed_context_executes_definitions_declarations_and_expressions() {
        let mut context = TypedContext::new();
        assert_eq!(
            context.execute(&command("x: 1")).expect("definition"),
            vec![TypedCommandEvent::ReportLine {
                text: "Variable x: int\n".into(),
                span: command("x: 1").span(),
            }]
        );

        let events = context.execute(&command("x")).expect("global read");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::Value {
                value: Value::Integer(value),
                type_: Type::Primitive(Prim::Int),
                ..
            }] if value == &BigInt::from(1)
        ));

        let events = context.execute(&command("x: \"new\"")).expect("override");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::ReportLine { text, .. }]
                if text == "Variable x: string (overriding previous instance, which had type int)\n"
        ));

        context.execute(&command("r: rat")).expect("declaration");
        let events = context.execute(&command("r := 2")).expect("assignment");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::Value {
                value: Value::Rational(value),
                type_: Type::Primitive(Prim::Rat),
                ..
            }] if value == &BigRational::from(2)
        ));
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
    fn assignment_writes_through_the_captured_cell() {
        let cell = crate::frames::global_with(Rc::new(Value::Integer(1.into())));
        let mut globals = IdTable::new();
        globals.define("x", Type::Primitive(Prim::Rat), cell.clone());

        let (type_, value) = convert_and_run_with("x := 2", &globals).expect("assignment");
        assert_eq!(type_, Type::Primitive(Prim::Rat));
        assert_eq!(value.to_string(), "2/1");
        assert_eq!(
            cell.borrow().as_ref().map(|value| value.to_string()),
            Some("2/1".into())
        );

        let error = convert_and_run("missing := 2").expect_err("unknown assignment target");
        assert!(error
            .message
            .contains("undefined identifier `missing` in assignment"));
    }

    #[test]
    fn assignment_voids_values_and_specialises_binding_types() {
        let void_cell = crate::frames::global_with(Rc::new(Value::Tuple(Vec::new())));
        let mut globals = IdTable::new();
        globals.define("sink", Type::void(), void_cell.clone());
        let (type_, value) = convert_and_run_with("sink := 7", &globals).expect("void assignment");
        assert_eq!(type_, Type::void());
        assert_eq!(value, Value::Tuple(Vec::new()));
        assert_eq!(
            void_cell
                .borrow()
                .as_ref()
                .map(|value| value.as_ref().clone()),
            Some(Value::Tuple(Vec::new()))
        );

        let row_cell = crate::frames::unset_global();
        globals.define("row", Type::row(Type::Undetermined), row_cell.clone());
        convert_and_run_with("row := [1,2]", &globals).expect("specialising assignment");
        let (type_cell, _) = globals.lookup("row").expect("row remains bound");
        assert_eq!(*type_cell.borrow(), Type::row(Type::Primitive(Prim::Int)));
    }

    #[test]
    fn row_subscription_and_slices_preserve_direction_flags() {
        let (_, value) = convert_and_run("[10,20,30]~[0]").expect("reverse subscription");
        assert_eq!(value, Value::Integer(30.into()));

        let (_, value) = convert_and_run("[10,20,30,40][1:3]").expect("forward slice");
        assert_eq!(
            value,
            Value::List(vec![Value::Integer(20.into()), Value::Integer(30.into())])
        );

        let (_, value) = convert_and_run("[10,20,30,40][2:]").expect("open upper slice");
        assert_eq!(
            value,
            Value::List(vec![Value::Integer(30.into()), Value::Integer(40.into())])
        );

        let (_, value) = convert_and_run("[10,20,30,40]~[1:3]").expect("reverse subject");
        assert_eq!(
            value,
            Value::List(vec![Value::Integer(30.into()), Value::Integer(20.into())])
        );
        let (_, value) = convert_and_run("[10,20,30,40][3~:4]").expect("reverse lower");
        assert_eq!(
            value,
            Value::List(vec![
                Value::Integer(20.into()),
                Value::Integer(30.into()),
                Value::Integer(40.into())
            ])
        );
        let (_, value) = convert_and_run("[10,20,30,40][0:1~]").expect("reverse upper");
        assert_eq!(
            value,
            Value::List(vec![
                Value::Integer(10.into()),
                Value::Integer(20.into()),
                Value::Integer(30.into())
            ])
        );
    }

    #[test]
    fn patternless_let_uses_parallel_groups_and_supports_assignment() {
        let (_, value) = convert_and_run("let x = 1 then x = x + 1 in x").expect("groups");
        assert_eq!(value, Value::Integer(2.into()));

        let (_, value) = convert_and_run("let x = 3 in x := x + 1").expect("local assignment");
        assert_eq!(value, Value::Integer(4.into()));

        let error = convert_and_run("let x = 1, y = x in y").expect_err("parallel group");
        assert!(error.message.contains("undefined identifier `x`"));
        let error = convert_and_run("let x = 1, x = 2 in x").expect_err("duplicate binding");
        assert!(error.message.contains("Multiple binding of 'x'"));

        let (_, value) = convert_and_run("let x = 1 then y = 2 then z = 3 in x := y + z")
            .expect("assignment reaches a depth-two local");
        assert_eq!(value, Value::Integer(5.into()));
    }

    #[test]
    fn subscription_and_slice_follow_upstream_evaluation_order() {
        let cell = crate::frames::global_with(Rc::new(Value::Integer(9.into())));
        let mut globals = IdTable::new();
        globals.define("i", Type::Primitive(Prim::Int), cell.clone());

        let (_, value) = convert_and_run_with("[(i := 1),(i := 2)][i := 0]", &globals)
            .expect("subscription order");
        assert_eq!(value, Value::Integer(1.into()));
        assert_eq!(
            cell.borrow().as_ref().map(|value| value.as_ref().clone()),
            Some(Value::Integer(2.into()))
        );

        *cell.borrow_mut() = Some(Rc::new(Value::Integer(0.into())));
        let (_, value) = convert_and_run_with(
            "[(i := i * 10 + 3),(i := i * 10 + 4)][(i := i * 10):(i := i * 10 + 2)]",
            &globals,
        )
        .expect("slice order");
        assert_eq!(value, Value::List(Vec::new()));
        assert_eq!(
            cell.borrow().as_ref().map(|value| value.as_ref().clone()),
            Some(Value::Integer(2034.into()))
        );
    }

    #[test]
    fn begin_end_groups_like_parentheses() {
        let (type_, value) = convert_and_run("begin 1 + 2 end").expect("begin/end group");
        assert_eq!(type_, Type::Primitive(Prim::Int));
        assert_eq!(value, Value::Integer(3.into()));
    }

    #[test]
    fn conditionals_balance_their_branches() {
        let (type_, value) = convert_and_run("if true then 1 else 2 fi").expect("simple if");
        assert_eq!(type_, Type::Primitive(Prim::Int));
        assert_eq!(value, Value::Integer(1.into()));
        // Balancing: the int branch is re-converted to rat.
        let (type_, value) = convert_and_run("if false then 1 else 1/2 fi").expect("balanced");
        assert_eq!(type_, Type::Primitive(Prim::Rat));
        assert_eq!(value.to_string(), "1/2");
        // A missing else branch makes the whole conditional void (the
        // void branch is broadest under balancing).
        let (type_, _) = convert_and_run("if false then 1 fi").expect("void conditional");
        assert_eq!(type_, Type::void());
        // elif nests; the inverted form parses.
        let (_, value) =
            convert_and_run("if false then 1 elif true then 2 else 3 fi").expect("elif");
        assert_eq!(value, Value::Integer(2.into()));
        let (_, value) = convert_and_run("if false else 9 then 8 fi").expect("inverted");
        assert_eq!(value, Value::Integer(9.into()));
        // Boolean connectives desugar to conditionals.
        let (_, value) = convert_and_run("true and false").expect("and");
        assert_eq!(value, Value::Boolean(false));
        let (_, value) = convert_and_run("false or true").expect("or");
        assert_eq!(value, Value::Boolean(true));
        let (_, value) = convert_and_run("not false").expect("not");
        assert_eq!(value, Value::Boolean(true));
        // Mismatched branches fail balancing.
        let error = convert_and_run("if true then 1 else \"x\" fi").expect_err("mismatch");
        assert!(error.message.contains("incompatible types"));
        // Non-boolean condition is a type error.
        let error = convert_and_run("if 1 then 2 else 3 fi").expect_err("condition type");
        assert!(error.message.contains("does not match"));
    }

    #[test]
    fn list_balancing_prunes_earlier_conflicts_after_a_broader_branch() {
        for source in [
            "[1,true,if true then 2 fi]",
            "[if true then 2 fi,1,true]",
            "[true,if true then 2 fi,1]",
        ] {
            let (type_, value) = convert_and_run(source).expect("void absorbs conflicts");
            assert_eq!(type_, Type::row(Type::void()), "source: {source}");
            assert_eq!(value.to_string(), "[(),(),()]", "source: {source}");
        }
    }

    #[test]
    fn nested_list_balance_failure_is_salvaged_by_outer_void_row() {
        for source in [
            "[[1,true],[if true then 2 fi]]",
            "[[if true then 2 fi],[1,true]]",
            "[[true,1],[if true then 2 fi]]",
        ] {
            let (type_, value) = convert_and_run(source).expect("nested void balance");
            assert_eq!(
                type_,
                Type::row(Type::row(Type::void())),
                "source: {source}"
            );
            assert_eq!(
                value.to_string(),
                if source.starts_with("[[if") {
                    "[[()],[(),()]]"
                } else {
                    "[[(),()],[()]]"
                },
                "source: {source}"
            );
        }
    }

    #[test]
    fn deeper_balance_failure_is_not_absorbed_by_outer_balance() {
        let error = convert_and_run("[begin [1,true] end,if true then 2 fi]")
            .expect_err("a wrapped inner balance failure must propagate");
        assert!(error.message.contains("incompatible types int and bool"));

        let error = convert_and_run("[[if true then 1 else true fi],if true then 2 fi]")
            .expect_err("a conditional failure nested in a list must propagate");
        assert!(error.message.contains("incompatible types int and bool"));
    }

    #[test]
    fn list_displays_balance_before_voiding_their_result() {
        let (type_, value) =
            convert_and_run("if false then [if true then 1 fi] fi").expect("void list");
        assert_eq!(type_, Type::void());
        assert_eq!(value, Value::Tuple(Vec::new()));
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
        assert_eq!(
            error.message,
            "Failed to match '+' with argument type (int,bool)"
        );
        let error = convert_and_run("1 / 0").expect_err("division by zero");
        assert_eq!(error.message, "Inverse of zero");
    }

    #[test]
    fn scalar_registry_matches_the_upstream_operator_surface() {
        let cases = [
            ("/2", "1/2"),
            ("%(5/2)", "(5,2)"),
            ("1 + 1/2", "3/2"),
            ("1/2 * 2", "1/1"),
            ("(5/2) / 2", "5/4"),
            ("(5/2) \\ 2", "1"),
            ("(5/2) % 2", "1/2"),
            ("(5/2) % (2/3)", "1/2"),
            ("(2/3) ^ 2", "4/9"),
            ("(-7) \\ 3", "-3"),
            ("(-7) % 3", "2"),
            ("7 \\% 3", "(2,1)"),
            ("2 ^ 10", "1024"),
            ("(-1) ^ -2147483649", "-1"),
            ("0 ^ 2147483648", "0"),
            ("1 = 1/1", "true"),
            ("1 < 3/2", "true"),
            ("=0", "true"),
            ("!=(1/2)", "true"),
            ("(>=(-1/2))", "false"),
            ("\"a\" < \"b\"", "true"),
            ("true != false", "true"),
            ("~1", "-2"),
            ("\"a\" ## \"b\"", "\"ab\""),
        ];
        for (source, expected) in cases {
            let (_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(value.to_string(), expected, "source: {source}");
        }

        let error = convert_and_run("2 ^ -1").expect_err("negative exponent");
        assert_eq!(error.message, "Negative power of integer");
        assert_eq!(
            convert_and_run("2 ^ 2147483648")
                .expect_err("large exponent")
                .message,
            "Exponent too large in power of integer"
        );
        assert_eq!(
            convert_and_run("1 % 0").expect_err("mod zero").message,
            "Modulo zero"
        );
        for (source, expected) in [
            ("(1/2) / 0", "Division of rational by integer zero"),
            ("(1/2) \\ 0", "Rational quotient by zero"),
            ("(1/2) % 0", "Division by zero"),
        ] {
            match convert_and_run(source) {
                Err(error) => assert_eq!(error.message, expected, "source: {source}"),
                Ok(value) => panic!("{source} unexpectedly succeeded with {value:?}"),
            }
        }
        assert_eq!(
            convert_and_run("+ 1")
                .expect_err("unary plus is not installed")
                .message,
            "Failed to match '+' with argument type int"
        );
        assert_eq!(
            convert_and_run("6 & 3")
                .expect_err("symbolic bit-and is not installed")
                .message,
            "Failed to match '&' with argument type (int,int)"
        );
    }

    #[test]
    fn named_domain_calls_resolve_through_typed_registry() {
        let (type_, value) =
            convert_and_run("prefers_coroots(simply_connected(Lie_type(\"A1\"), true))")
                .expect("formal root-datum constructor");
        assert_eq!(type_, bool_type());
        assert_eq!(value, Value::Boolean(true));

        let (type_, value) = convert_and_run(
            "nr_of_real_forms(inner_class(simply_connected(Lie_type(\"A1\"), true), mat: [[1]]))",
        )
        .expect("formal inner-class constructor");
        assert_eq!(type_, int_type());
        assert_eq!(value, Value::Integer(2.into()));

        let (_, value) = convert_and_run(
            "inner_class(simply_connected(Lie_type(\"A1\"), true), mat: [[1]]) = inner_class(simply_connected(Lie_type(\"A1\"), true), mat: [[1]])",
        )
        .expect("domain equality overload");
        assert_eq!(value, Value::Boolean(true));

        let error = convert_and_run("inner_class(\"A1\", [[1]])")
            .expect_err("all inner_class overloads reject string input");
        assert_eq!(
            error.message,
            "Failed to match 'inner_class' with argument type (string,[[int]])"
        );

        let (_, value) =
            convert_and_run("simply_connected(\"A1\", true)").expect("string-to-LieType coercion");
        assert_eq!(
            value.to_string(),
            "simply connected root datum of Lie type 'A1'"
        );

        let (_, value) =
            convert_and_run("Lie_type(inner_class(simply_connected(\"A1\", true), mat: [[1]]))")
                .expect("InnerClass-to-RootDatum-to-LieType coercions");
        assert_eq!(value.to_string(), "Lie type 'A1'");

        let (_, value) = convert_and_run(
            "Lie_type(real_form(inner_class(simply_connected(\"A1\", true), mat: [[1]]), 1))",
        )
        .expect("RealForm-to-RootDatum-to-LieType coercions");
        assert_eq!(value.to_string(), "Lie type 'A1'");

        let (_, value) = convert_and_run("Cartan_matrix(Lie_type(\"B2\"))")
            .expect("LieType Cartan_matrix overload");
        assert_eq!(value.to_string(), "\n|  2, -2 |\n| -1,  2 |\n");

        let (_, value) =
            convert_and_run("simply_connected(Lie_type(\"E8\"), true)").expect("E8 root datum");
        assert_eq!(
            value.to_string(),
            "simply connected adjoint root datum of Lie type 'E8'"
        );

        let (_, value) = convert_and_run("simply_connected(Lie_type(\"\"), true)")
            .expect("empty simply connected datum");
        assert_eq!(
            value.to_string(),
            "simply connected adjoint root datum of empty Lie type"
        );

        let error = convert_and_run("adjoint(Lie_type(\"A1.T1\"), false)")
            .expect_err("Atlas rejects adjoint construction with a torus factor");
        assert_eq!(error.message, "Sub-lattice matrix should have size 2x2");

        for (source, expected) in [
            (
                "Lie_type(root_datum([[2,-1],[-2,2]], [[1,0],[0,1]], true))",
                "Lie type 'C2'",
            ),
            (
                "Lie_type(root_datum([[2,-2],[-1,2]], [[1,0],[0,1]], true))",
                "Lie type 'B2'",
            ),
        ] {
            let (_, value) = convert_and_run(source).expect("oriented Cartan overload");
            assert_eq!(value.to_string(), expected, "source: {source}");
        }
    }

    #[test]
    fn unknown_named_calls_report_name_before_argument_errors() {
        for source in ["foo(1)", "foo(missing)"] {
            let error = convert_and_run(source).expect_err("unknown builtin");
            assert_eq!(error.kind, ErrorKind::Name, "source: {source}");
            assert_eq!(
                error.message, "Undefined identifier 'foo'",
                "source: {source}"
            );
        }
    }

    #[test]
    fn overloads_are_sorted_most_specific_first_and_carry_hunger() {
        let plus = overload_variants("+");
        let argument_types = plus
            .iter()
            .map(|&index| builtin_registry()[index].arg_type.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            argument_types,
            vec![
                int_pair(),
                Type::tuple(vec![rat_type(), int_type()]),
                pair(rat_type()),
            ]
        );
        assert_eq!(
            plus.iter()
                .map(|&index| builtin_registry()[index].hunger)
                .collect::<Vec<_>>(),
            vec![1, 1, 1]
        );
    }

    #[test]
    fn no_value_builtins_validate_without_constructing_results() {
        let source = SourceText::new("0 ^ 999999999999999999999999999999999999");
        let program = parse(&source).expect("power parses");
        let table = TypeTable::new();
        let globals = IdTable::new();
        let analysis = Analysis::new(&table, &globals);
        let mut required = Type::Undetermined;
        let typed = convert_expr(&program.expressions[0], &mut required, &analysis)
            .expect("power converts");
        assert_eq!(
            typed
                .evaluate(&mut EvaluationContext::new(), Level::NoValue)
                .expect("zero power validates"),
            None
        );

        for source_text in ["(1/2) / 0", "(1/2) % 0"] {
            let source = SourceText::new(source_text);
            let program = parse(&source).expect("rational zero operation parses");
            let mut required = Type::Undetermined;
            let typed = convert_expr(&program.expressions[0], &mut required, &analysis)
                .expect("rational zero operation converts");
            assert_eq!(
                typed
                    .evaluate(&mut EvaluationContext::new(), Level::NoValue)
                    .expect("no-value skips the underlying rational operation"),
                None,
                "source: {source_text}"
            );
        }

        let source = SourceText::new("(2/3) ^ -2");
        let program = parse(&source).expect("negative rational power parses");
        let mut required = Type::Undetermined;
        let typed = convert_expr(&program.expressions[0], &mut required, &analysis)
            .expect("negative rational power converts");
        assert_eq!(
            typed
                .evaluate(&mut EvaluationContext::new(), Level::NoValue)
                .expect("no-value skips the unsigned exponent conversion"),
            None
        );

        let source = SourceText::new("1 / 0");
        let program = parse(&source).expect("fraction parses");
        let mut required = Type::Undetermined;
        let typed = convert_expr(&program.expressions[0], &mut required, &analysis)
            .expect("fraction converts");
        let error = typed
            .evaluate(&mut EvaluationContext::new(), Level::NoValue)
            .expect_err("no-value still checks the denominator");
        assert!(matches!(
            error,
            Control::Runtime(Diagnostic { message, .. })
                if message == "Inverse of zero"
        ));

        let source = SourceText::new(
            "real_form(inner_class(simply_connected(Lie_type(\"A1\"), true), mat: [[1]]), 99)",
        );
        let program = parse(&source).expect("real form parses");
        let mut required = Type::Undetermined;
        let typed = convert_expr(&program.expressions[0], &mut required, &analysis)
            .expect("real form converts");
        let error = typed
            .evaluate(&mut EvaluationContext::new(), Level::NoValue)
            .expect_err("no-value still validates real-form indices");
        assert!(matches!(
            error,
            Control::Runtime(Diagnostic { message, .. })
                if message == "Illegal real form number"
        ));
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

        let error = convert_and_run("mat: []").expect_err("empty matrix conversion");
        assert_eq!(
            error.message,
            "Implicit conversion to matrix for an empty set of vectors"
        );

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

    #[test]
    fn lambdas_convert_to_function_typed_closures() {
        let (type_, value) = convert_and_run("(int n): n + 1").expect("lambda literal");
        assert_eq!(type_, Type::function(int_type(), int_type()));
        assert!(matches!(value, Value::Closure(_)));

        // A parameterless lambda has the void argument type.
        let (type_, value) = convert_and_run("@: 42").expect("parameterless lambda");
        assert_eq!(type_, Type::function(Type::void(), int_type()));
        assert!(matches!(value, Value::Closure(_)));

        let error = convert_and_run("(int x, rat x): x").expect_err("duplicate parameter");
        assert!(error.message.contains("Multiple binding of 'x'"));
    }

    #[test]
    fn closure_calls_bind_arguments_and_catch_return() {
        // The five B3a fixture shapes (sanity expectations only; the oracle
        // capture is still pending).
        for (source, expected) in [
            ("let f = (int n): n + 1 in f(2)", 3),
            ("let x = 41 in let f = @: x + 1 in f()", 42),
            ("let make = (int x): @: x in let f = make(7) in f()", 7),
            ("let f = (int n): return n + 1 in f(2)", 3),
            ("let f = (int n): n + 1 in 2.f", 3),
        ] {
            let (type_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(type_, Type::Primitive(Prim::Int), "source: {source}");
            assert_eq!(value, Value::Integer(expected.into()), "source: {source}");
        }

        // A parameter body sees both its own frame and the enclosing one.
        let (_, value) =
            convert_and_run("let a = 10 in let f = (int n): a + n in f(2)").expect("depth shift");
        assert_eq!(value, Value::Integer(12.into()));
    }

    #[test]
    fn a_return_reaching_top_level_is_an_error() {
        let mut context = TypedContext::new();
        let error = context
            .execute(&command("return 3"))
            .expect_err("top-level return");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(error.message, "illegal control flow at top level");
    }

    #[test]
    fn globals_hold_closures_and_report_function_types() {
        let mut context = TypedContext::new();
        let events = context
            .execute(&command("f: (int n): n + 1"))
            .expect("define function");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::ReportLine { text, .. }] if text == "Variable f: (int->int)\n"
        ));

        let events = context
            .execute(&command("f(2)"))
            .expect("global closure call");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::Value {
                value: Value::Integer(value),
                ..
            }] if value == &BigInt::from(3)
        ));

        // A non-function value is not callable.
        context.execute(&command("x: 3")).expect("define int");
        let error = context
            .execute(&command("x(1)"))
            .expect_err("non-function call");
        assert_eq!(error.kind, ErrorKind::Type);
        assert!(error.message.contains("does not match required pattern"));
    }
}
