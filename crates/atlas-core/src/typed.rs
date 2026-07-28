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
use crate::frames::{EvaluationContext, GlobalCell};
use crate::linear_values::{Matrix, RatVec, Vec32};
use crate::syntax::{Command, Expr};
use crate::types::{Prim, Type, TypeTable};
use crate::value::Value;
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
}

/// Conversion-time context (locals arrive with lambdas in B3).
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
            let mut chosen = None;
            for &index in variants {
                let builtin = &builtin_registry()[index];
                if builtin.arg_type == a_priori_type {
                    chosen = Some(index);
                    break;
                }
                if crate::coercions::is_close(&a_priori_type, &builtin.arg_type, analysis.types)
                    & 0x1
                    != 0
                {
                    chosen = Some(index);
                    break;
                }
            }
            let index = match chosen {
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
        Expr::Conditional {
            condition,
            then_branch,
            else_branch,
            span,
        } => {
            let mut bool_type = Type::Primitive(Prim::Bool);
            let condition = convert_expr(condition, &mut bool_type, analysis)?;
            let branches = balance(
                &[then_branch.as_ref(), else_branch.as_ref()],
                required,
                *span,
                analysis,
            )?;
            let mut branches = branches.into_iter();
            Ok(TypedExpr::Conditional {
                condition: Box::new(condition),
                then_branch: Box::new(branches.next().expect("two branches")),
                else_branch: Box::new(branches.next().expect("two branches")),
            })
        }
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
        unsupported => Err(type_error(
            "this expression form is not yet in the typed pipeline".into(),
            unsupported.span(),
        )),
    }
}

/// Balance branch expressions to a common type (upstream `balance`,
/// axis.w:1022-1122, two-branch simplification): convert each against a
/// copy of the target, keep the broadest type under `broader_eq`, then
/// re-convert every branch whose type diverges from the winner.
fn balance(
    branches: &[&Expr],
    target: &mut Type,
    span: SourceSpan,
    analysis: &Analysis<'_>,
) -> Result<Vec<TypedExpr>, Diagnostic> {
    let mut converted = Vec::new();
    let mut types = Vec::new();
    for branch in branches {
        let mut slot = target.clone();
        converted.push(convert_expr(branch, &mut slot, analysis)?);
        types.push(slot);
    }
    let mut common = types.first().cloned().unwrap_or(Type::Undetermined);
    for type_ in &types[1..] {
        if crate::coercions::broader_eq(&common, type_, analysis.types) {
            continue;
        }
        if crate::coercions::broader_eq(type_, &common, analysis.types) {
            common = type_.clone();
            continue;
        }
        return Err(type_error(
            format!(
                "branches have incompatible types {} and {}",
                common.display(analysis.types),
                type_.display(analysis.types),
            ),
            span,
        ));
    }
    if !target.specialise(&common, analysis.types) {
        return Err(type_error(
            format!(
                "balanced type {} does not match required pattern {}",
                common.display(analysis.types),
                target.display(analysis.types),
            ),
            span,
        ));
    }
    for (index, type_) in types.iter().enumerate() {
        if type_ != &common {
            let mut slot = common.clone();
            converted[index] = convert_expr(branches[index], &mut slot, analysis)?;
        }
    }
    Ok(converted)
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
    let lower_index = usize::try_from(&lower)
        .map_err(|_| runtime("slice lower bound is not a machine index", span))?;
    let upper_index = usize::try_from(&upper)
        .map_err(|_| runtime("slice upper bound is not a machine index", span))?;
    if lower_index >= upper_index {
        return Ok(Vec::new());
    }
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
        assert_eq!(error.message, "fraction with zero denominator");
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
        assert!(convert_and_run("+ 1")
            .expect_err("unary plus is not installed")
            .message
            .contains("no instance of operator '+'"));
        assert!(convert_and_run("6 & 3")
            .expect_err("symbolic bit-and is not installed")
            .message
            .contains("no instance of operator '&'"));
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
                if message == "fraction with zero denominator"
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
