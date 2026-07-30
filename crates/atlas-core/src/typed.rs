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
use crate::syntax::{
    compact_expression, Command, Expr, ForLoop, LambdaParam, LetBinding, Pattern, TypeSpec,
};
use crate::types::{Prim, Type, TypeBinding, TypeNumber, TypeTable};
use crate::value::{Closure, SlotShape, Value};
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
    Dont,
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
        /// Compact rendering of the source expression, quoted by the
        /// out-of-range diagnostic exactly like the oracle's `range_mess`
        /// prints the subscription node (axis.w:4188-4194).
        source: String,
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
        /// One (shape, initializer) pair per binding in the group; each
        /// value distributes into the frame slots its shape describes.
        initializers: Vec<(SlotShape, TypedExpr)>,
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
    /// A function literal; evaluation captures the current frame chain
    /// into a closure value (upstream `lambda_expression`).
    Closure {
        /// Number of argument slots a call binds; 0 pushes no frame.
        parameters: usize,
        /// How each argument value distributes into frame slots.
        shapes: Rc<[SlotShape]>,
        /// A recursive closure additionally binds itself at slot 0.
        recursive: bool,
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
    /// `first; second`: the first half evaluates for effects at `NoValue`;
    /// the sequence yields the second half's value.
    Sequence {
        first: Box<TypedExpr>,
        second: Box<TypedExpr>,
    },
    /// A while loop collecting each completed iteration's body value into
    /// a row; a missing condition is the constant true.
    While {
        condition: Option<Box<TypedExpr>>,
        body: Box<TypedExpr>,
    },
    /// A for loop over a row value; each iteration distributes the element
    /// per `shape`, then pushes the 0-based index slot when `index` is set
    /// (the upstream (pattern, index) pair wrap).
    For {
        shape: SlotShape,
        index: bool,
        iterable: Box<TypedExpr>,
        body: Box<TypedExpr>,
    },
    /// `break`, unwound to the innermost loop boundary.
    Break,
    /// `dont` terminates the current while iteration without an error.
    Dont,
    /// `die` (upstream `shell`, axis.w:621-630): analysing it succeeds
    /// against any required type; evaluating it throws `I die`.
    Die {
        span: SourceSpan,
    },
    /// The body of an injector closure: wraps the payload in a union value
    /// carrying the variant tag and the injector's name.
    UnionInject {
        tag: u16,
        injector_name: String,
        payload: Box<TypedExpr>,
    },
    /// The body of a projector closure: extracts one component of a tuple.
    TupleProject {
        index: usize,
        inner: Box<TypedExpr>,
    },
    /// Case discrimination on a tabled union value: the first branch whose
    /// tag matches distributes the payload per `shape` and evaluates its
    /// body; `fallback` is the `else` branch.
    Case {
        subject: Box<TypedExpr>,
        branches: Vec<(u16, SlotShape, TypedExpr)>,
        fallback: Option<Box<TypedExpr>>,
        span: SourceSpan,
    },
    /// `first next second`: yields the FIRST value; the second still
    /// evaluates for effects (upstream `next_expression`).
    Next {
        first: Box<TypedExpr>,
        second: Box<TypedExpr>,
    },
    /// Integer case selection (upstream `int_case_expression` and its
    /// else/then-else variants): the selector picks a branch by 0-based
    /// index, `then_branch` catches negative selectors, `else_branch` any
    /// out-of-range one; without either the index wraps modulo the branch
    /// count.
    IntCase {
        condition: Box<TypedExpr>,
        branches: Vec<TypedExpr>,
        then_branch: Option<Box<TypedExpr>>,
        else_branch: Option<Box<TypedExpr>>,
        span: SourceSpan,
    },
    /// Positional union case (upstream `union_case_expression`): each
    /// branch evaluates to a function applied to the subject union's
    /// payload, selected by variant position.
    UnionCase {
        subject: Box<TypedExpr>,
        branches: Vec<TypedExpr>,
        span: SourceSpan,
    },
    /// A counted for loop (upstream `counted_for_expression`): `count`
    /// iterations collecting each body value; with `has_name` the counter
    /// is bound (as a constant) in a per-iteration frame, increasing from
    /// the bound (default 0), or decreasing to it inclusive when
    /// `decreasing`.
    CountedFor {
        has_name: bool,
        decreasing: bool,
        count: Box<TypedExpr>,
        bound: Option<Box<TypedExpr>>,
        body: Box<TypedExpr>,
        span: SourceSpan,
    },
}

/// Conversion-time context (locals are let bindings and lambda parameters).
pub struct Analysis<'a> {
    pub types: &'a TypeTable,
    pub globals: &'a IdTable,
    /// The context's overload state: `forget`-removed startup overloads
    /// and user `set` definitions, merged into resolution.
    pub overloads: &'a OverloadState,
    locals: BTreeMap<String, (TypeCell, usize, usize)>,
    /// Names bound by a const `!x` pattern; assignment to them is an
    /// analysis error. Entries shadow outward like `locals` does: a
    /// non-const rebinding removes the name.
    constant_locals: BTreeSet<String>,
    /// Set while converting a function body: `return` is legal only there
    /// (the axis layer's return_type marker).
    in_function: bool,
    /// Number of enclosing loops: `break` is legal only when nonzero
    /// (mirrors `in_function`; upstream rejects a stray `break` during
    /// analysis, before anything evaluates).
    loop_depth: usize,
}

impl<'a> Analysis<'a> {
    pub fn new(types: &'a TypeTable, globals: &'a IdTable, overloads: &'a OverloadState) -> Self {
        Self {
            types,
            globals,
            overloads,
            locals: BTreeMap::new(),
            constant_locals: BTreeSet::new(),
            in_function: false,
            loop_depth: 0,
        }
    }

    /// The context for a loop body: same bindings, one more loop level.
    fn in_loop(&self) -> Self {
        Self {
            types: self.types,
            globals: self.globals,
            overloads: self.overloads,
            locals: self.locals.clone(),
            constant_locals: self.constant_locals.clone(),
            in_function: self.in_function,
            loop_depth: self.loop_depth + 1,
        }
    }
}

/// The global identifier table: one binding per name, each definition
/// holding the FRESH cell it allocated (converted code keeps the cell it
/// captured; re-definition rebinds the name only). Names bound by a const
/// `!x` pattern in `set` reject assignment until a plain rebinding clears
/// the mark.
#[derive(Default)]
pub struct IdTable {
    entries: BTreeMap<String, (TypeCell, GlobalCell)>,
    const_names: BTreeSet<String>,
}

pub type TypeCell = Rc<RefCell<Type>>;

impl IdTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(&mut self, name: impl Into<String>, type_: Type, cell: GlobalCell) {
        let name = name.into();
        // A fresh definition is never constant, even when it replaces a
        // const binding (upstream rebinds the identifier outright).
        self.const_names.remove(&name);
        self.entries
            .insert(name, (Rc::new(RefCell::new(type_)), cell));
    }

    pub fn lookup(&self, name: &str) -> Option<&(TypeCell, GlobalCell)> {
        self.entries.get(name)
    }

    /// Mark a binding constant (`set !x = …`); assignment to it becomes an
    /// analysis error.
    pub fn mark_const(&mut self, name: &str) {
        self.const_names.insert(name.to_owned());
    }

    pub fn is_const(&self, name: &str) -> bool {
        self.const_names.contains(name)
    }

    /// Remove a binding (`forget name`); `false` when it was not known.
    pub fn remove(&mut self, name: &str) -> bool {
        self.const_names.remove(name);
        self.entries.remove(name).is_some()
    }
}

/// A user `set`-defined overload: the full function type (always
/// `Type::Function`) and the closure value a call applies.
#[derive(Clone, Debug, PartialEq)]
pub struct UserOverload {
    function_type: Type,
    value: Value,
}

/// Per-context overload state (the upstream overload table,
/// global.w:446-560): the startup registry is static, so
/// `forget name @ type` records removals here and `set` records user
/// variants; resolution merges both views into one ordered list.
#[derive(Default)]
pub struct OverloadState {
    forgotten: Vec<(String, Type)>,
    user: BTreeMap<String, Vec<UserOverload>>,
}

impl OverloadState {
    /// Whether `forget name @ type` hid the startup overload at `arg_type`.
    fn is_forgotten(&self, name: &str, arg_type: &Type) -> bool {
        self.forgotten
            .iter()
            .any(|(forgotten_name, signature)| forgotten_name == name && signature == arg_type)
    }

    /// The user variants of `name`, in insertion order.
    fn user_variants(&self, name: &str) -> &[UserOverload] {
        self.user.get(name).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Add a `set` function definition, replaying the upstream
    /// single-table `add` over the merged view: an exact argument-type
    /// twin is replaced in place (a startup twin is shadowed through the
    /// forgotten list), a too-close neighbour is the upstream ambiguity
    /// error, and anything else inserts at its ordered position. Returns
    /// the variant counts before and after, which select the report
    /// wording (`add_overload`, global.w:1004-1023).
    fn add_user(
        &mut self,
        name: &str,
        function_type: Type,
        value: Value,
        types: &TypeTable,
        span: SourceSpan,
    ) -> Result<(usize, usize), Diagnostic> {
        let Type::Function(parts) = &function_type else {
            unreachable!("only function-typed values enter the overload table")
        };
        let arg_type = parts.0.clone();
        let merged = merged_variants(name, self, types);
        let old_n = merged.len();
        let mut lower = 0;
        let mut upper = old_n;
        let mut replacement = None;
        for (slot, existing) in merged.iter().enumerate() {
            match crate::coercions::is_close(&arg_type, &existing.arg_type, types) {
                0x6 => lower = slot + 1,
                0x5 => upper = upper.min(slot),
                0x7 if arg_type == existing.arg_type => {
                    replacement = Some(slot);
                    break;
                }
                0x4 | 0x7 => {
                    return Err(type_error(
                        format!(
                            "Cannot overload `{name}':\nalready overloaded type '{}' is too close to new argument type '{}',\nwhich would make overloading ambiguous for certain arguments. Simultaneous\noverloading for these types is not possible, forget the other one first.",
                            existing.arg_type.display(types),
                            arg_type.display(types),
                        ),
                        span,
                    ));
                }
                _ => {}
            }
        }
        let entry = UserOverload {
            function_type,
            value,
        };
        let (slot, n) = match replacement {
            Some(slot) => match merged[slot].origin {
                OverloadOrigin::User(user_index) => {
                    self.user
                        .get_mut(name)
                        .expect("a merged user variant has a slot")[user_index] = entry;
                    return Ok((old_n, old_n));
                }
                // Replacing a startup overload keeps the count: hide the
                // builtin and take its position with the user value.
                OverloadOrigin::Builtin(_) => {
                    self.forgotten.push((name.to_owned(), arg_type));
                    (slot, old_n)
                }
            },
            None => (upper.max(lower), old_n + 1),
        };
        let user_position = merged[..slot]
            .iter()
            .filter(|variant| matches!(variant.origin, OverloadOrigin::User(_)))
            .count();
        self.user
            .entry(name.to_owned())
            .or_default()
            .insert(user_position, entry);
        Ok((old_n, n))
    }

    /// Remove ONE active overload at exactly `arg_type`
    /// (`forget name @ type`, global.w:1253-1261): a user `set` variant
    /// drops out; a startup overload is hidden by recording it (a variant
    /// shadowed by `set` stays hidden, so forgetting the user replacement
    /// never resurrects the builtin). `false` when nothing matched.
    fn remove(&mut self, name: &str, arg_type: &Type) -> bool {
        if let Some(users) = self.user.get_mut(name) {
            let position = users.iter().position(
                |user| matches!(&user.function_type, Type::Function(parts) if parts.0 == *arg_type),
            );
            if let Some(position) = position {
                users.remove(position);
                if users.is_empty() {
                    self.user.remove(name);
                }
                return true;
            }
        }
        let active_builtin = overload_variants(name)
            .iter()
            .map(|&index| &builtin_registry()[index])
            .any(|builtin| builtin.arg_type == *arg_type);
        if active_builtin && !self.is_forgotten(name, arg_type) {
            self.forgotten.push((name.to_owned(), arg_type.clone()));
            return true;
        }
        false
    }
}

/// One candidate in the merged overload view: a startup builtin or a user
/// `set` definition, most specific first (the upstream single-table
/// order).
#[derive(Clone, Copy)]
enum OverloadOrigin {
    Builtin(usize),
    User(usize),
}

struct MergedVariant {
    arg_type: Type,
    result_type: Type,
    origin: OverloadOrigin,
}

/// The active variants for `name`: startup overloads not hidden by
/// `forget`, with user variants inserted at the position the upstream
/// single-table ordering gives them.
fn merged_variants(name: &str, overloads: &OverloadState, types: &TypeTable) -> Vec<MergedVariant> {
    let mut merged: Vec<MergedVariant> = overload_variants(name)
        .iter()
        .copied()
        .filter(|&index| !overloads.is_forgotten(name, &builtin_registry()[index].arg_type))
        .map(|index| {
            let builtin = &builtin_registry()[index];
            MergedVariant {
                arg_type: builtin.arg_type.clone(),
                result_type: builtin.result.clone(),
                origin: OverloadOrigin::Builtin(index),
            }
        })
        .collect();
    for (user_index, user) in overloads.user_variants(name).iter().enumerate() {
        let Type::Function(parts) = &user.function_type else {
            unreachable!("user overloads always hold a function type")
        };
        let variant = MergedVariant {
            arg_type: parts.0.clone(),
            result_type: parts.1.clone(),
            origin: OverloadOrigin::User(user_index),
        };
        let position = insert_position(&variant.arg_type, &merged, types);
        merged.insert(position, variant);
    }
    merged
}

/// The slot the upstream single-table ordering gives a new argument type:
/// after every strictly more specific entry, before the less specific
/// ones (`overload_table::add` without the conflict cases — `add_user`
/// already rejected or resolved those).
fn insert_position(arg_type: &Type, merged: &[MergedVariant], types: &TypeTable) -> usize {
    let mut lower = 0;
    let mut upper = merged.len();
    for (slot, existing) in merged.iter().enumerate() {
        match crate::coercions::is_close(arg_type, &existing.arg_type, types) {
            0x6 => lower = slot + 1,
            0x5 => upper = upper.min(slot),
            // Exact or ambiguous neighbours cannot occur here; keep them
            // ahead of the new variant defensively.
            0x4 | 0x7 => lower = slot + 1,
            _ => {}
        }
    }
    upper.max(lower)
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
    /// The overload table: startup overloads hidden by
    /// `forget name @ type` plus user `set` definitions.
    overloads: OverloadState,
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
                    &Analysis::new(&self.types, &self.globals, &self.overloads),
                )?;
                let value = match evaluate_command_expr(&typed, &mut self.evaluation) {
                    Ok(value) => value,
                    Err(diagnostic) => {
                        // Upstream keeps text printed before the failure;
                        // this port has no channel for events alongside a
                        // diagnostic, so the buffer is dropped instead of
                        // leaking into the next command.
                        self.evaluation.take_printed();
                        return Err(diagnostic);
                    }
                };
                let mut events = self.drain_printed(expression.span());
                events.push(TypedCommandEvent::Value {
                    value,
                    type_,
                    span: expression.span(),
                });
                Ok(events)
            }
            Command::Define {
                name, value, span, ..
            } => {
                let mut type_ = Type::Undetermined;
                let typed = convert_expr(
                    value,
                    &mut type_,
                    &Analysis::new(&self.types, &self.globals, &self.overloads),
                )?;
                let value = match evaluate_command_expr(&typed, &mut self.evaluation) {
                    Ok(value) => value,
                    Err(diagnostic) => {
                        self.evaluation.take_printed();
                        return Err(diagnostic);
                    }
                };
                let mut events = self.drain_printed(*span);
                events.push(self.define_variable(name, type_, value, false, *span));
                Ok(events)
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
            Command::SetType {
                definitions,
                tabled,
                span,
            } => self.execute_set_type(definitions, *tabled, *span),
            Command::Whattype { target, span } => self.execute_whattype(target, *span),
            Command::Forget { name, span } => {
                // global_forget_identifier (global.w:1241-1248): the report
                // goes to standard output and never fails the command. The
                // upstream type-identifier cleanup has no counterpart yet:
                // the tabled type map supports no removal.
                let was_known = self.globals.remove(&name.value);
                let state = if was_known { "forgotten" } else { "not known" };
                Ok(vec![TypedCommandEvent::ReportLine {
                    text: format!("Identifier '{}' {state}\n", name.value),
                    span: *span,
                }])
            }
            Command::ForgetOverload {
                name,
                signature,
                span,
            } => {
                // global_forget_overload (global.w:1253-1261): removes ONE
                // overload, reporting either way; the printed type is the
                // resolved signature, exactly as upstream prints `type`.
                let resolved = signature.resolve_in(&self.types).map_err(|unknown| {
                    Diagnostic::new(
                        ErrorKind::Name,
                        format!("undefined type name '{}'", unknown.value),
                        Some(unknown.span),
                    )
                })?;
                let removed = self.overloads.remove(&name.value, &resolved);
                let state = if removed { "forgotten" } else { "not known" };
                Ok(vec![TypedCommandEvent::ReportLine {
                    text: format!(
                        "Definition of '{}@{}' {state}\n",
                        name.value,
                        resolved.display(&self.types)
                    ),
                    span: *span,
                }])
            }
            Command::Set { bindings, span } => self.execute_set(bindings, *span),
            Command::ShowOverloads { name, span } => {
                // show_overloads (global.w:1790-1799): one line per active
                // variant, argument and result types printed independently.
                let variants = merged_variants(&name.value, &self.overloads, &self.types);
                let mut text = if variants.is_empty() {
                    format!("No overloads for '{}'\n", name.value)
                } else {
                    format!("Overloaded instances of '{}'\n", name.value)
                };
                for variant in &variants {
                    text.push_str(&format!(
                        "  {}->{}\n",
                        variant.arg_type.display(&self.types),
                        variant.result_type.display(&self.types)
                    ));
                }
                Ok(vec![TypedCommandEvent::ReportLine { text, span: *span }])
            }
            Command::ShowAll { span } => Ok(vec![TypedCommandEvent::ReportLine {
                text: self.show_all_text(),
                span: *span,
            }]),
        }
    }

    fn show_all_text(&self) -> String {
        let mut text = String::from("Overloaded operators and functions:\n");
        let mut names = Vec::new();
        for builtin in builtin_registry() {
            if !names.iter().any(|name| name == &builtin.name) {
                names.push(builtin.name);
            }
        }
        for name in self.overloads.user.keys() {
            if !names.iter().any(|known| known == name) {
                names.push(name.as_str());
            }
        }
        for name in names {
            for variant in merged_variants(name, &self.overloads, &self.types) {
                let source = match variant.origin {
                    OverloadOrigin::Builtin(_index) => {
                        format!("{{{}@{}}}", name, variant.arg_type.display(&self.types))
                    }
                    OverloadOrigin::User(index) => self
                        .overloads
                        .user_variants(name)
                        .get(index)
                        .map(|entry| entry.value.to_string())
                        .unwrap_or_else(|| "*".to_owned()),
                };
                text.push_str(&format!(
                    "{}: {}: {}\n",
                    name,
                    Self::format_function_signature(
                        &variant.arg_type,
                        &variant.result_type,
                        &self.types,
                    ),
                    source
                ));
            }
        }
        text.push_str("Global variables:\n");
        for (name, (type_cell, cell)) in &self.globals.entries {
            let value = cell
                .borrow()
                .as_ref()
                .map_or_else(|| "*".to_owned(), |value| value.to_string());
            text.push_str(&format!(
                "{}: {}: {}\n",
                name,
                type_cell.borrow().display(&self.types),
                value
            ));
        }
        text
    }

    fn format_function_signature(arg_type: &Type, result_type: &Type, types: &TypeTable) -> String {
        Type::function(arg_type.clone(), result_type.clone())
            .display(types)
            .to_string()
    }

    /// Drain printer-builtin output produced by the last evaluation into
    /// report events (upstream writes to `*output_stream` mid-evaluation,
    /// so the report precedes the command's own value or binding report).
    fn drain_printed(&mut self, span: SourceSpan) -> Vec<TypedCommandEvent> {
        self.evaluation
            .take_printed()
            .into_iter()
            .map(|text| TypedCommandEvent::ReportLine { text, span })
            .collect()
    }

    /// Bind a global value and report `(Constant|Variable) name: type`
    /// (global_define_identifier and the identifier-table branch of
    /// `do_global_set`, global.w:911-994). A redefinition notes the
    /// overridden type.
    fn define_variable(
        &mut self,
        name: &str,
        type_: Type,
        value: Value,
        constant: bool,
        span: SourceSpan,
    ) -> TypedCommandEvent {
        let previous = self
            .globals
            .lookup(name)
            .map(|(type_, _)| type_.borrow().display(&self.types).to_string());
        self.globals.define(
            name.to_owned(),
            type_.clone(),
            crate::frames::global_with(Rc::new(value)),
        );
        if constant {
            self.globals.mark_const(name);
        }
        let role = if constant { "Constant" } else { "Variable" };
        let mut text = format!("{role} {name}: {}", type_.display(&self.types));
        if let Some(previous) = previous {
            text.push_str(&format!(
                " (overriding previous instance, which had type {previous})"
            ));
        }
        text.push('\n');
        TypedCommandEvent::ReportLine { text, span }
    }

    /// Add a user function definition to the overload table and report it
    /// (`add_overload`, global.w:1004-1023): replacing keeps the variant
    /// count (`Redefined`), the first variant is `Defined`, otherwise
    /// `Added definition [n] of`.
    fn add_overload(
        &mut self,
        name: &str,
        function_type: Type,
        value: Value,
        span: SourceSpan,
    ) -> Result<TypedCommandEvent, Diagnostic> {
        let (old_n, n) =
            self.overloads
                .add_user(name, function_type.clone(), value, &self.types, span)?;
        let prefix = if n == old_n {
            "Redefined ".to_string()
        } else if n == 1 {
            "Defined ".to_string()
        } else {
            format!("Added definition [{n}] of ")
        };
        Ok(TypedCommandEvent::ReportLine {
            text: format!("{prefix}{name}: {}\n", function_type.display(&self.types)),
            span,
        })
    }

    /// `set declarations` (parser.y:140, `do_global_set` global.w:911-994):
    /// every right-hand side converts against the CURRENT tables (parallel
    /// semantics — no binding sees another), then evaluates, then binds
    /// leaf by leaf: function-typed leaves join the overload table, the
    /// rest the identifier table, each reporting as it lands.
    fn execute_set(
        &mut self,
        bindings: &[LetBinding],
        span: SourceSpan,
    ) -> Result<Vec<TypedCommandEvent>, Diagnostic> {
        struct Pending {
            shape: SlotShape,
            leaves: Vec<PatternLeaf>,
            typed: TypedExpr,
        }
        // Phase 0: analyse all right-hand sides and patterns before
        // anything evaluates or binds.
        let mut pending = Vec::with_capacity(bindings.len());
        {
            let analysis = Analysis::new(&self.types, &self.globals, &self.overloads);
            for binding in bindings {
                let mut found = Type::Undetermined;
                let typed = convert_expr(&binding.initializer, &mut found, &analysis)?;
                let leaves = bind_pattern_leaves(&binding.pattern, &found, &self.types)?;
                pending.push(Pending {
                    shape: pattern_slot_shape(&binding.pattern),
                    leaves,
                    typed,
                });
            }
        }
        // Phase 1: evaluate every right-hand side before any binding
        // happens, so a failing initializer leaves the tables untouched.
        let mut printed = Vec::new();
        let mut evaluated = Vec::with_capacity(pending.len());
        for pending in pending {
            let value = match evaluate_command_expr(&pending.typed, &mut self.evaluation) {
                Ok(value) => value,
                Err(diagnostic) => {
                    self.evaluation.take_printed();
                    return Err(diagnostic);
                }
            };
            printed.extend(self.drain_printed(span));
            evaluated.push((pending.shape, pending.leaves, value));
        }
        // Phase 2: distribute each value over its pattern leaves and bind
        // them in declaration order, reporting as each lands.
        let mut events = printed;
        for (shape, leaves, value) in evaluated {
            let mut slots = Vec::new();
            distribute(value, &shape, &mut slots);
            debug_assert_eq!(slots.len(), leaves.len());
            for ((name, name_span, constant, leaf_type), slot) in leaves.into_iter().zip(slots) {
                let value = Rc::try_unwrap(slot).unwrap_or_else(|rc| (*rc).clone());
                let event = if matches!(leaf_type, Type::Function(_)) {
                    self.add_overload(&name, leaf_type, value, name_span)?
                } else {
                    self.define_variable(&name, leaf_type, value, constant, name_span)
                };
                events.push(event);
            }
        }
        Ok(events)
    }

    /// `set_type` (axis.w:5092-5168): the bracketed form registers every
    /// name of the group as a placeholder first so right-hand sides can be
    /// recursive, then resolves each spec and installs the projector or
    /// injector globals; the single-name form is a plain alias that never
    /// enters the tabled map.
    fn execute_set_type(
        &mut self,
        definitions: &[crate::syntax::TypeDefinition],
        tabled: bool,
        span: SourceSpan,
    ) -> Result<Vec<TypedCommandEvent>, Diagnostic> {
        let mut targets = Vec::with_capacity(definitions.len());
        if tabled {
            // Pass 1: placeholders, so every name of the group resolves.
            let numbers: Vec<TypeNumber> = definitions
                .iter()
                .map(|definition| {
                    self.types.add(TypeBinding {
                        name: definition.name.value.clone(),
                        definition: Type::Undetermined,
                        fields: Vec::new(),
                    })
                })
                .collect();
            // Pass 2: resolve each spec with every group name visible.
            for (definition, number) in definitions.iter().zip(numbers) {
                let (expansion, fields) = resolve_type_spec(&definition.spec, &self.types)?;
                self.types.update(number, expansion, fields);
                targets.push(Type::Tabled(number));
            }
        } else {
            let definition = definitions
                .first()
                .expect("the single-name set_type form holds one equation");
            // The alias never enters the tabled map, so its field names
            // live only in the syntax tree (define_type_members reads
            // them from the spec).
            let (expansion, _fields) = resolve_type_spec(&definition.spec, &self.types)?;
            self.types
                .add_alias(definition.name.value.clone(), expansion.clone());
            targets.push(expansion);
        }
        let mut events = Vec::with_capacity(definitions.len());
        for (definition, target) in definitions.iter().zip(&targets) {
            let text = self.define_type_members(definition, target);
            events.push(TypedCommandEvent::ReportLine { text, span });
        }
        Ok(events)
    }

    /// Install the projector (struct) or injector (union) globals of one
    /// definition as one-argument closures, and render its report line.
    fn define_type_members(
        &mut self,
        definition: &crate::syntax::TypeDefinition,
        target: &Type,
    ) -> String {
        let expansion = match target {
            Type::Tabled(number) => self.types.expansion(*number).clone(),
            other => other.clone(),
        };
        let heading = format!(
            "Type name '{}' defined as {}\n",
            definition.name.value,
            expansion.display(&self.types)
        );
        let fields = match &definition.spec {
            TypeSpec::Alias(_) => return heading,
            TypeSpec::Struct(fields) | TypeSpec::Union(fields) => fields,
        };
        let components: &[Type] = match &expansion {
            Type::Tuple(components) | Type::Union(components) => components,
            _ => &[],
        };
        let union = matches!(definition.spec, TypeSpec::Union(_));
        let mut names = Vec::new();
        for (index, field) in fields.iter().enumerate() {
            let Some(field_name) = &field.name else {
                continue;
            };
            let component = components.get(index).cloned().unwrap_or(Type::Undetermined);
            let (function_type, body) = if union {
                (
                    Type::function(component, target.clone()),
                    TypedExpr::UnionInject {
                        tag: index as u16,
                        injector_name: field_name.value.clone(),
                        payload: Box::new(parameter_body(&field_name.value, field.span)),
                    },
                )
            } else {
                (
                    Type::function(target.clone(), component),
                    TypedExpr::TupleProject {
                        index,
                        inner: Box::new(parameter_body(&field_name.value, field.span)),
                    },
                )
            };
            self.globals.define(
                field_name.value.clone(),
                function_type,
                crate::frames::global_with(Rc::new(member_closure(body))),
            );
            names.push(field_name.value.clone());
        }
        if names.is_empty() {
            return heading;
        }
        let role = if union { "injectors" } else { "projectors" };
        format!("{heading}  with {role}: {}.\n", names.join(", "))
    }

    /// `whattype` (parser.y:169-171): a defined type name prints its
    /// definition (a tabled type as an equation naming its tags), any
    /// other target converts unevaluated and prints its type.
    fn execute_whattype(
        &mut self,
        target: &Expr,
        span: SourceSpan,
    ) -> Result<Vec<TypedCommandEvent>, Diagnostic> {
        if let Expr::Identifier { name, .. } = target {
            if let Some(resolved) = self.types.resolve_name(name) {
                let text = match resolved {
                    Type::Tabled(number) => {
                        let binding = self.types.binding(number);
                        let body = match &binding.definition {
                            Type::Union(variants) => {
                                type_equation(variants, &binding.fields, " | ", &self.types)
                            }
                            Type::Tuple(components) if !components.is_empty() => {
                                type_equation(components, &binding.fields, ", ", &self.types)
                            }
                            other => {
                                return Ok(vec![TypedCommandEvent::ReportLine {
                                    text: format!("Defined type: {}\n", other.display(&self.types)),
                                    span,
                                }])
                            }
                        };
                        format!("Defined type: ( {body} )\n")
                    }
                    other => format!("Defined type: {}\n", other.display(&self.types)),
                };
                return Ok(vec![TypedCommandEvent::ReportLine { text, span }]);
            }
        }
        let mut type_ = Type::Undetermined;
        convert_expr(
            target,
            &mut type_,
            &Analysis::new(&self.types, &self.globals, &self.overloads),
        )?;
        Ok(vec![TypedCommandEvent::ReportLine {
            text: format!("Type: {}\n", type_.display(&self.types)),
            span,
        }])
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
            Control::Break(_) | Control::Dont | Control::Return(_) => Diagnostic::new(
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

/// The single parameter of an injector/projector closure body.
fn parameter_body(name: &str, span: SourceSpan) -> TypedExpr {
    TypedExpr::LocalIdent {
        name: name.to_string(),
        depth: 0,
        offset: 0,
        span,
    }
}

/// A one-argument closure value with no captured frame, used for the
/// projector and injector globals a `set_type` definition installs.
fn member_closure(body: TypedExpr) -> Value {
    Value::Closure(Rc::new(Closure {
        parameters: 1,
        shapes: Rc::from(vec![SlotShape::Leaf]),
        recursive: false,
        body: Rc::new(body),
        frame: None,
    }))
}

/// Resolve a `set_type` right-hand side against the table: the expansion
/// type plus the per-field names (projectors or injectors), positionally.
fn resolve_type_spec(
    spec: &TypeSpec,
    types: &TypeTable,
) -> Result<(Type, Vec<Option<String>>), Diagnostic> {
    fn field_type(field: &crate::syntax::TypeField, types: &TypeTable) -> Result<Type, Diagnostic> {
        field.type_expr.resolve_in(types).map_err(|unknown| {
            Diagnostic::new(
                ErrorKind::Name,
                format!("undefined type name '{}'", unknown.value),
                Some(unknown.span),
            )
        })
    }
    match spec {
        TypeSpec::Alias(type_expr) => Ok((
            type_expr.resolve_in(types).map_err(|unknown| {
                Diagnostic::new(
                    ErrorKind::Name,
                    format!("undefined type name '{}'", unknown.value),
                    Some(unknown.span),
                )
            })?,
            Vec::new(),
        )),
        TypeSpec::Struct(fields) => {
            let mut components = Vec::with_capacity(fields.len());
            let mut names = Vec::with_capacity(fields.len());
            for field in fields {
                components.push(field_type(field, types)?);
                names.push(field.name.as_ref().map(|name| name.value.clone()));
            }
            Ok((Type::tuple(components), names))
        }
        TypeSpec::Union(fields) => {
            let mut variants = Vec::with_capacity(fields.len());
            let mut names = Vec::with_capacity(fields.len());
            for field in fields {
                variants.push(field_type(field, types)?);
                names.push(field.name.as_ref().map(|name| name.value.clone()));
            }
            Ok((Type::union_of(variants), names))
        }
    }
}

/// The inside of a tabled type's equation print: each component followed
/// by its tag name, joined per kind (`( void nil | (int,IntList) cons )`).
fn type_equation(
    components: &[Type],
    fields: &[Option<String>],
    joiner: &str,
    types: &TypeTable,
) -> String {
    components
        .iter()
        .zip(fields)
        .map(|(component, field)| match field {
            Some(name) => format!("{} {}", component.display(types), name),
            None => component.display(types).to_string(),
        })
        .collect::<Vec<_>>()
        .join(joiner)
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
/// converted expression. The error wording is the oracle's uniform
/// type_error rendering (global.w:655-663).
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
            "found {} while {} was needed.",
            found.display(analysis.types),
            required.display(analysis.types)
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
        Expr::RecLambda { .. } => convert_rec_lambda_expression(expression, required, analysis),
        Expr::Return { value, span } => {
            // `return` is legal only lexically inside a function body (the
            // axis layer's return_type marker); upstream rejects it during
            // analysis, before anything evaluates.
            if !analysis.in_function {
                return Err(type_error(
                    "One can only use 'return' within a function body".into(),
                    *span,
                ));
            }
            // The enclosing context is the function's result type;
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
                    format!("Undefined identifier '{name}'"),
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
        Expr::MultiAssignment(assignment) => {
            // parser.y:264 accepts `set pattern := value`; the evaluation
            // slice has not landed yet, so analysis rejects it explicitly
            // rather than silently mis-binding.
            Err(Diagnostic::new(
                ErrorKind::Type,
                "multi-assignment 'set <pattern> := <value>' is not yet implemented",
                Some(assignment.span),
            ))
        }
        Expr::Assignment {
            name,
            target_span,
            value,
            span,
        } => {
            if let Some((target, depth, offset)) = analysis.locals.get(name) {
                // A const `!x` binding rejects assignment during analysis,
                // before anything evaluates (upstream `is_constant`).
                if analysis.constant_locals.contains(name.as_str()) {
                    return Err(Diagnostic::new(
                        ErrorKind::Name,
                        format!(
                            "Name '{name}' is constant in assignment {name}:={}",
                            compact_expression(value)
                        ),
                        Some(*target_span),
                    ));
                }
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
                    format!("Undefined identifier '{name}' in assignment"),
                    Some(*target_span),
                ));
            };
            // A const global (`set !x = …`) rejects assignment during
            // analysis, exactly like a const local does.
            if analysis.globals.is_const(name) {
                return Err(Diagnostic::new(
                    ErrorKind::Name,
                    format!(
                        "Name '{name}' is constant in assignment {name}:={}",
                        compact_expression(value)
                    ),
                    Some(*target_span),
                ));
            }
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
            // Upstream `subscr_base::index_kind` (axis.w:3941-3973): a row
            // subscripts to its component type, a string to a one-character
            // string; anything unsubscriptable is the analysis-time `not_so`
            // error (axis.w:4101-4105).
            let found = match &array_type {
                Type::Row(component) => (**component).clone(),
                Type::Primitive(Prim::String) => Type::Primitive(Prim::String),
                Type::Primitive(Prim::Vec | Prim::RatVec | Prim::Mat) => {
                    // vec/ratvec/mat subscription is upstream-legal but not
                    // yet implemented; keep the not-a-row diagnostic there.
                    return Err(type_error(
                        format!(
                            "subscription requires a row, found {}",
                            array_type.display(analysis.types)
                        ),
                        *span,
                    ));
                }
                _ => {
                    return Err(type_error(
                        format!(
                            "Cannot subscript value of type {} with index of type {}",
                            array_type.display(analysis.types),
                            index_type.display(analysis.types)
                        ),
                        *span,
                    ));
                }
            };
            conform_types(
                &found,
                required,
                TypedExpr::Subscription {
                    array: Box::new(converted_array),
                    index: Box::new(converted_index),
                    reversed: *reversed,
                    source: compact_expression(expression),
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
            let mut constant_locals = analysis.constant_locals.clone();
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
                            overloads: analysis.overloads,
                            locals: locals.clone(),
                            constant_locals: constant_locals.clone(),
                            in_function: analysis.in_function,
                            loop_depth: analysis.loop_depth,
                        },
                    )?;
                    // The initializer converts freely; the pattern then
                    // claims the resulting type (upstream `bind_pattern`).
                    let leaves =
                        bind_pattern_leaves(&binding.pattern, &binding_type, analysis.types)?;
                    pending.push((pattern_slot_shape(&binding.pattern), leaves, converted));
                }
                let mut names = BTreeSet::new();
                for (_, leaves, _) in &pending {
                    for (name, name_span, _, _) in leaves {
                        if !names.insert(name.as_str()) {
                            return Err(Diagnostic::new(
                                ErrorKind::Name,
                                format!("Multiple binding of '{name}' in same scope"),
                                Some(*name_span),
                            ));
                        }
                    }
                }
                // A group of pure discards claims no frame (empty-layer
                // rule), so depths shift only when it binds a slot.
                let group_slots: usize = pending.iter().map(|(_, leaves, _)| leaves.len()).sum();
                if group_slots > 0 {
                    for (_, depth, _) in locals.values_mut() {
                        *depth += 1;
                    }
                }
                let mut offset = 0;
                for (_, leaves, _) in &pending {
                    for (name, _, constant, binding_type) in leaves {
                        locals.insert(
                            name.clone(),
                            (Rc::new(RefCell::new(binding_type.clone())), 0, offset),
                        );
                        if *constant {
                            constant_locals.insert(name.clone());
                        } else {
                            constant_locals.remove(name);
                        }
                        offset += 1;
                    }
                }
                groups.push(
                    pending
                        .into_iter()
                        .map(|(shape, _, converted)| (shape, converted))
                        .collect::<Vec<_>>(),
                );
            }
            let mut converted = convert_expr(
                body,
                required,
                &Analysis {
                    types: analysis.types,
                    globals: analysis.globals,
                    overloads: analysis.overloads,
                    locals,
                    constant_locals,
                    in_function: analysis.in_function,
                    loop_depth: analysis.loop_depth,
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
            convert_overload_application(
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
                    && (!merged_variants(name, analysis.overloads, analysis.types).is_empty()
                        || (local.is_none() && analysis.globals.lookup(name).is_none()));
                if use_overloads {
                    return convert_overload_application(
                        name, arguments, required, *span, analysis, true,
                    );
                }
            }
            // Fallback path: the callee must have function type and the
            // argument converts against its parameter type; mismatches
            // carry the upstream wording (axis-types.w:2403-2410).
            let mut callee_type = Type::Undetermined;
            let function = convert_expr(callee, &mut callee_type, analysis)?;
            let mut function_pattern = Type::function(Type::Undetermined, Type::Undetermined);
            if !function_pattern.specialise(&callee_type, analysis.types) {
                return Err(type_error(
                    format!(
                        "found {} while {} was needed.",
                        callee_type.display(analysis.types),
                        function_pattern.display(analysis.types)
                    ),
                    callee.span(),
                ));
            }
            let Type::Function(parts) = function_pattern else {
                unreachable!("a specialised (*->*) pattern stays a function type")
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
            // A-priori conversion, as for overloads: convert once in
            // undetermined context, then re-convert only when the parameter
            // pattern needs a coercion; a genuine mismatch reports the
            // a-priori type against the pattern.
            let mut a_priori = Type::Undetermined;
            let converted_argument = convert_expr(&argument_source, &mut a_priori, analysis)?;
            let mut expected = argument_type.clone();
            let argument = if expected.specialise(&a_priori, analysis.types) {
                converted_argument
            } else {
                if crate::coercions::is_close(&a_priori, &argument_type, analysis.types) & 0x1 == 0
                {
                    return Err(type_error(
                        format!(
                            "found {} while {} was needed.",
                            a_priori.display(analysis.types),
                            argument_type.display(analysis.types)
                        ),
                        argument_source.span(),
                    ));
                }
                expected = argument_type;
                convert_expr(&argument_source, &mut expected, analysis)?
            };
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
        Expr::Sequence { first, second, .. } => {
            // The first half evaluates for effects against a void context
            // (upstream make_sequence); the sequence yields the second half.
            let mut void = Type::void();
            let first = convert_expr(first, &mut void, analysis)?;
            let second = convert_expr(second, required, analysis)?;
            Ok(TypedExpr::Sequence {
                first: Box::new(first),
                second: Box::new(second),
            })
        }
        Expr::While {
            condition,
            body,
            span,
        } => {
            let condition = condition
                .as_ref()
                .map(|condition| {
                    // A-priori conversion, then the bool check with the
                    // upstream `found … while … was needed.` wording.
                    let mut found = Type::Undetermined;
                    let converted = convert_expr(condition, &mut found, analysis)?;
                    if !Type::Primitive(Prim::Bool).specialise(&found, analysis.types) {
                        return Err(type_error(
                            format!(
                                "found {} while bool was needed.",
                                found.display(analysis.types)
                            ),
                            condition.span(),
                        ));
                    }
                    Ok(converted)
                })
                .transpose()?;
            // The body converts against a fresh component pattern with the
            // loop depth raised; the loop's type is a row of that pattern.
            let mut component = Type::Undetermined;
            let body = convert_expr(body, &mut component, &analysis.in_loop())?;
            conform_types(
                &Type::row(component),
                required,
                TypedExpr::While {
                    condition: condition.map(Box::new),
                    body: Box::new(body),
                },
                *span,
                analysis,
            )
        }
        Expr::For(loop_) => {
            let ForLoop {
                pattern,
                index,
                iterable,
                body,
                span,
            } = loop_.as_ref();
            let mut found = Type::Undetermined;
            let iterable = convert_expr(iterable, &mut found, analysis)?;
            let Type::Row(component) = found else {
                return Err(type_error(
                    format!(
                        "Cannot iterate over value of type {}",
                        found.display(analysis.types)
                    ),
                    loop_.iterable.span(),
                ));
            };
            // The pattern claims the row's component; the `@` name binds
            // the 0-based index as int (the upstream (pattern, index)
            // pair wrap, in that slot order).
            let leaves = match pattern {
                Some(pattern) => bind_pattern_leaves(pattern, &component, analysis.types)?,
                None => Vec::new(),
            };
            let mut names = BTreeSet::new();
            for (name, name_span, _, _) in &leaves {
                if !names.insert(name.as_str()) {
                    return Err(Diagnostic::new(
                        ErrorKind::Name,
                        format!("Multiple binding of '{name}' in same scope"),
                        Some(*name_span),
                    ));
                }
            }
            if let Some(index) = index {
                if !names.insert(index.value.as_str()) {
                    return Err(Diagnostic::new(
                        ErrorKind::Name,
                        format!("Multiple binding of '{}' in same scope", index.value),
                        Some(index.span),
                    ));
                }
            }
            let mut locals = analysis.locals.clone();
            let mut constant_locals = analysis.constant_locals.clone();
            // Like a let group: a layer binding no slot claims no frame,
            // so depths shift only when a slot exists (empty-layer rule).
            let layer_slots = leaves.len() + usize::from(index.is_some());
            if layer_slots > 0 {
                for (_, depth, _) in locals.values_mut() {
                    *depth += 1;
                }
            }
            let mut offset = 0;
            for (name, _, constant, leaf_type) in &leaves {
                locals.insert(
                    name.clone(),
                    (Rc::new(RefCell::new(leaf_type.clone())), 0, offset),
                );
                if *constant {
                    constant_locals.insert(name.clone());
                } else {
                    constant_locals.remove(name);
                }
                offset += 1;
            }
            if let Some(index) = index {
                locals.insert(
                    index.value.clone(),
                    (Rc::new(RefCell::new(Type::Primitive(Prim::Int))), 0, offset),
                );
                constant_locals.remove(&index.value);
            }
            let shape = pattern
                .as_ref()
                .map(pattern_slot_shape)
                .unwrap_or(SlotShape::Discard);
            let mut body_type = Type::Undetermined;
            let body = convert_expr(
                body,
                &mut body_type,
                &Analysis {
                    types: analysis.types,
                    globals: analysis.globals,
                    overloads: analysis.overloads,
                    locals,
                    constant_locals,
                    in_function: analysis.in_function,
                    loop_depth: analysis.loop_depth + 1,
                },
            )?;
            conform_types(
                &Type::row(body_type),
                required,
                TypedExpr::For {
                    shape,
                    index: index.is_some(),
                    iterable: Box::new(iterable),
                    body: Box::new(body),
                },
                *span,
                analysis,
            )
        }
        Expr::Case(case) => {
            let crate::syntax::CaseExpr {
                subject,
                branches,
                span,
            } = case.as_ref();
            let mut subject_type = Type::Undetermined;
            let converted_subject = convert_expr(subject, &mut subject_type, analysis)?;
            // Discrimination needs a tabled union with named injectors
            // (axis-types.w:2890-2910); a structural union from the
            // single-name form is rejected with the upstream wording.
            let tabled_union = match &subject_type {
                Type::Tabled(number) => match &analysis.types.binding(*number).definition {
                    Type::Union(variants)
                        if analysis.types.binding(*number).fields.len() == variants.len()
                            && analysis
                                .types
                                .binding(*number)
                                .fields
                                .iter()
                                .all(Option::is_some) =>
                    {
                        Some((
                            variants.clone(),
                            analysis.types.binding(*number).fields.clone(),
                        ))
                    }
                    _ => None,
                },
                _ => None,
            };
            let Some((variants, injector_names)) = tabled_union else {
                return Err(type_error(
                    format!(
                        "Discrimination on expression of type {} requires using 'set_type' for \
                         this type, and naming injectors for it",
                        subject_type.display(analysis.types)
                    ),
                    subject.span(),
                ));
            };
            // Branch bodies share one type pattern, converted in source
            // order: the first body fixes it and a later mismatch reports
            // against what the earlier branches needed.
            let mut common = required.clone();
            let mut converted_branches = Vec::new();
            let mut fallback = None;
            for branch in branches {
                let Some(tag) = &branch.tag else {
                    let mut found = Type::Undetermined;
                    let body = convert_expr(&branch.body, &mut found, analysis)?;
                    if !common.specialise(&found, analysis.types) {
                        return Err(type_error(
                            format!(
                                "found {} while {} was needed.",
                                found.display(analysis.types),
                                common.display(analysis.types)
                            ),
                            branch.body.span(),
                        ));
                    }
                    fallback = Some(Box::new(body));
                    continue;
                };
                let Some(index) = injector_names
                    .iter()
                    .position(|field| field.as_deref() == Some(tag.value.as_str()))
                else {
                    return Err(Diagnostic::new(
                        ErrorKind::Name,
                        format!(
                            "Injector '{}' does not belong to type {}",
                            tag.value,
                            subject_type.display(analysis.types)
                        ),
                        Some(tag.span),
                    ));
                };
                let payload = variants[index].clone();
                let (shape, leaves) = match &branch.pattern {
                    Some(pattern) => {
                        let leaves = bind_pattern_leaves(pattern, &payload, analysis.types)?;
                        (pattern_slot_shape(pattern), leaves)
                    }
                    None => (SlotShape::Discard, Vec::new()),
                };
                let mut names = BTreeSet::new();
                for (name, name_span, _, _) in &leaves {
                    if !names.insert(name.as_str()) {
                        return Err(Diagnostic::new(
                            ErrorKind::Name,
                            format!("Multiple binding of '{name}' in same scope"),
                            Some(*name_span),
                        ));
                    }
                }
                let mut locals = analysis.locals.clone();
                let mut constant_locals = analysis.constant_locals.clone();
                // Same layer rule as a loop pattern: a branch binding no
                // slot claims no frame, so depths shift only when a slot
                // exists (empty-layer rule).
                if !leaves.is_empty() {
                    for (_, depth, _) in locals.values_mut() {
                        *depth += 1;
                    }
                }
                for (offset, (name, _, constant, leaf_type)) in leaves.iter().enumerate() {
                    locals.insert(
                        name.clone(),
                        (Rc::new(RefCell::new(leaf_type.clone())), 0, offset),
                    );
                    if *constant {
                        constant_locals.insert(name.clone());
                    } else {
                        constant_locals.remove(name);
                    }
                }
                let mut found = Type::Undetermined;
                let body = convert_expr(
                    &branch.body,
                    &mut found,
                    &Analysis {
                        types: analysis.types,
                        globals: analysis.globals,
                        overloads: analysis.overloads,
                        locals,
                        constant_locals,
                        in_function: analysis.in_function,
                        loop_depth: analysis.loop_depth,
                    },
                )?;
                if !common.specialise(&found, analysis.types) {
                    return Err(type_error(
                        format!(
                            "found {} while {} was needed.",
                            found.display(analysis.types),
                            common.display(analysis.types)
                        ),
                        branch.body.span(),
                    ));
                }
                converted_branches.push((index as u16, shape, body));
            }
            conform_types(
                &common,
                required,
                TypedExpr::Case {
                    subject: Box::new(converted_subject),
                    branches: converted_branches,
                    fallback,
                    span: *span,
                },
                *span,
                analysis,
            )
        }
        Expr::Next { first, second, .. } => {
            // next_expr (axis.w:3697-3704): the first half converts
            // against the required pattern, the second against void.
            let first = convert_expr(first, required, analysis)?;
            let mut void = Type::void();
            let second = convert_expr(second, &mut void, analysis)?;
            Ok(TypedExpr::Next {
                first: Box::new(first),
                second: Box::new(second),
            })
        }
        Expr::IntCase(case) => {
            let crate::syntax::IntCaseExpr {
                condition,
                branches,
                then_branch,
                else_branch,
                span,
            } = case.as_ref();
            let mut int_type = Type::Primitive(Prim::Int);
            let condition = convert_expr(condition, &mut int_type, analysis)?;
            // Balance all sub-branches against the shared pattern, with
            // the then/else branches ahead of the in-list ones in the
            // upstream node's storage order (axis.w:4926-4946).
            let mut ordered = Vec::with_capacity(branches.len() + 2);
            if let Some(then_branch) = then_branch {
                ordered.push(then_branch);
            }
            if let Some(else_branch) = else_branch {
                ordered.push(else_branch);
            }
            ordered.extend(branches.iter());
            let mut converted = balance(&ordered, required, *span, analysis)
                .map_err(|error| error.into_diagnostic(analysis))?
                .into_iter();
            let then_branch = then_branch
                .is_some()
                .then(|| Box::new(converted.next().expect("then branch balanced")));
            let else_branch = else_branch
                .is_some()
                .then(|| Box::new(converted.next().expect("else branch balanced")));
            Ok(TypedExpr::IntCase {
                condition: Box::new(condition),
                branches: converted.collect(),
                then_branch,
                else_branch,
                span: *span,
            })
        }
        Expr::UnionCase(case) => {
            let crate::syntax::UnionCaseExpr {
                condition,
                branches,
                span,
            } = case.as_ref();
            let mut subject_type = Type::Undetermined;
            let subject = convert_expr(condition, &mut subject_type, analysis)?;
            // kind() untables transparently (axis-types.w:376-382).
            let expansion = match &subject_type {
                Type::Tabled(number) => analysis.types.expansion(*number).clone(),
                other => other.clone(),
            };
            let Type::Union(variants) = &expansion else {
                let pattern = Type::union_of(vec![Type::Undetermined; branches.len().max(2)]);
                return Err(type_error(
                    format!(
                        "found {} while {} was needed.",
                        subject_type.display(analysis.types),
                        pattern.display(analysis.types)
                    ),
                    condition.span(),
                ));
            };
            if variants.len() != branches.len() {
                return Err(type_error(
                    format!(
                        "Union case expression has {} branches,\nwhile the union type {} has {} \
                         variants",
                        branches.len(),
                        subject_type.display(analysis.types),
                        variants.len()
                    ),
                    *span,
                ));
            }
            // Each branch converts against (variant_i -> shared hole);
            // the shared pattern specialises left-to-right
            // (axis.w:5098-5109).
            let mut common = required.clone();
            let mut converted = Vec::with_capacity(branches.len());
            for (branch, variant) in branches.iter().zip(variants) {
                let mut function_type = Type::function(variant.clone(), common.clone());
                converted.push(convert_expr(branch, &mut function_type, analysis)?);
                let Type::Function(parts) = &function_type else {
                    unreachable!("a function pattern stays a function type")
                };
                common.specialise(&parts.1, analysis.types);
            }
            conform_types(
                &common,
                required,
                TypedExpr::UnionCase {
                    subject: Box::new(subject),
                    branches: converted,
                    span: *span,
                },
                *span,
                analysis,
            )
        }
        Expr::CountedFor(loop_) => {
            let crate::syntax::CountedForLoop {
                name,
                count,
                bound,
                decreasing,
                body,
                span,
            } = loop_.as_ref();
            let mut count_type = Type::Primitive(Prim::Int);
            let count = convert_expr(count, &mut count_type, analysis)?;
            let bound = match bound {
                Some(bound) => {
                    let mut bound_type = Type::Primitive(Prim::Int);
                    Some(Box::new(convert_expr(bound, &mut bound_type, analysis)?))
                }
                None => None,
            };
            // The loop variable is bound as a CONSTANT int (axis.w:6484).
            let mut locals = analysis.locals.clone();
            let mut constant_locals = analysis.constant_locals.clone();
            if let Some(name) = name {
                for (_, depth, _) in locals.values_mut() {
                    *depth += 1;
                }
                locals.insert(
                    name.value.clone(),
                    (Rc::new(RefCell::new(Type::Primitive(Prim::Int))), 0, 0),
                );
                constant_locals.insert(name.value.clone());
            }
            let mut body_type = Type::Undetermined;
            let body = convert_expr(
                body,
                &mut body_type,
                &Analysis {
                    types: analysis.types,
                    globals: analysis.globals,
                    overloads: analysis.overloads,
                    locals,
                    constant_locals,
                    in_function: analysis.in_function,
                    loop_depth: analysis.loop_depth + 1,
                },
            )?;
            conform_types(
                &Type::row(body_type),
                required,
                TypedExpr::CountedFor {
                    has_name: name.is_some(),
                    decreasing: *decreasing,
                    count: Box::new(count),
                    bound,
                    body: Box::new(body),
                    span: *span,
                },
                *span,
                analysis,
            )
        }
        Expr::Break { span } => {
            // `break` is legal only lexically inside a loop; upstream
            // rejects it during analysis, before anything evaluates
            // (mirroring the `return` check above).
            if analysis.loop_depth == 0 {
                return Err(type_error(
                    "Using 'break' not in the reach of any loop".into(),
                    *span,
                ));
            }
            // A break yields no value and converts as void, so a
            // `… then break fi` branch balances against the implicit
            // void else branch.
            conform_types(&Type::void(), required, TypedExpr::Break, *span, analysis)
        }
        Expr::Dont { span } => {
            if analysis.loop_depth == 0 {
                return Err(type_error(
                    "Using 'dont' not in the reach of any loop".into(),
                    *span,
                ));
            }
            conform_types(&Type::void(), required, TypedExpr::Dont, *span, analysis)
        }
        Expr::Die { span } => {
            // `die` passes analysis trivially in ANY context, leaving the
            // required type untouched (upstream die_expr, axis.w:634-638);
            // only evaluation throws.
            Ok(TypedExpr::Die { span: *span })
        }
    }
}

/// Convert a non-recursive function literal (axis.w:3093-3115): bind the
/// parameters as a new local layer, then convert the body against the
/// required pattern's result hole so a context type reaches the body (and
/// any `return`) directly. A void context converts the body against a
/// dummy result and discards the closure.
/// The frame-slot layout one bound value distributes into (upstream
/// `bind_pattern`'s variable list): the whole-value name of a `(a, b): t`
/// pattern takes the first slot, then elements left-to-right.
fn pattern_slot_shape(pattern: &Pattern) -> SlotShape {
    match pattern {
        Pattern::Discard { .. } => SlotShape::Discard,
        Pattern::Name { .. } => SlotShape::Leaf,
        Pattern::Tuple {
            elements, whole, ..
        } => SlotShape::Tuple {
            elements: elements.iter().map(pattern_slot_shape).collect(),
            whole: whole.is_some(),
        },
    }
}

/// The undetermined structure a pattern requires of a value's type,
/// rendered in mismatch messages (`(*,*)` for a 2-tuple pattern).
fn pattern_type(pattern: &Pattern) -> Type {
    match pattern {
        Pattern::Discard { .. } | Pattern::Name { .. } => Type::Undetermined,
        Pattern::Tuple { elements, .. } => Type::tuple(elements.iter().map(pattern_type).collect()),
    }
}

/// One bound name of a pattern, in slot order: the name, its span for
/// duplicate diagnostics, constness, and the claimed component type.
type PatternLeaf = (String, SourceSpan, bool, Type);

/// The names a pattern binds, in slot order (whole-value name first), each
/// with the component type claimed from `found`. A structural mismatch is
/// the upstream `found … while … was needed.` error (`bind_pattern`).
fn bind_pattern_leaves(
    pattern: &Pattern,
    found: &Type,
    types: &TypeTable,
) -> Result<Vec<PatternLeaf>, Diagnostic> {
    match pattern {
        Pattern::Discard { .. } => Ok(Vec::new()),
        Pattern::Name {
            name,
            name_span,
            constant,
            ..
        } => Ok(vec![(name.clone(), *name_span, *constant, found.clone())]),
        Pattern::Tuple {
            elements,
            whole,
            span,
        } => {
            let mismatch = || {
                type_error(
                    format!(
                        "found {} while {} was needed.",
                        found.display(types),
                        pattern_type(pattern).display(types)
                    ),
                    *span,
                )
            };
            let Type::Tuple(components) = found else {
                return Err(mismatch());
            };
            if components.len() != elements.len() {
                return Err(mismatch());
            }
            let mut leaves = Vec::new();
            if let Some(whole) = whole {
                leaves.extend(bind_pattern_leaves(whole, found, types)?);
            }
            for (element, component) in elements.iter().zip(components) {
                leaves.extend(bind_pattern_leaves(element, component, types)?);
            }
            Ok(leaves)
        }
    }
}

/// One lambda parameter (parser.y `id_spec`): the declared argument type,
/// the slot shape, and the bound leaves in slot order. A tuple parameter
/// composes its specs; `type pattern` claims the pattern's leaves from the
/// declared type.
fn convert_parameter(
    parameter: &LambdaParam,
    types: &TypeTable,
) -> Result<(Type, SlotShape, Vec<PatternLeaf>), Diagnostic> {
    match parameter {
        LambdaParam::Typed(typed) => {
            let declared = typed.type_expr.resolve();
            let leaves = bind_pattern_leaves(&typed.pattern, &declared, types)?;
            Ok((declared, pattern_slot_shape(&typed.pattern), leaves))
        }
        LambdaParam::Tuple { elements, .. } => {
            let mut element_types = Vec::with_capacity(elements.len());
            let mut shapes = Vec::with_capacity(elements.len());
            let mut leaves = Vec::new();
            for element in elements {
                let (element_type, shape, element_leaves) = convert_parameter(element, types)?;
                element_types.push(element_type);
                shapes.push(shape);
                leaves.extend(element_leaves);
            }
            Ok((
                Type::tuple(element_types),
                SlotShape::Tuple {
                    elements: shapes,
                    whole: false,
                },
                leaves,
            ))
        }
    }
}

fn convert_lambda_expression(
    parameters: &[LambdaParam],
    body: &Expr,
    span: SourceSpan,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let mut converted_parameters = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        converted_parameters.push(convert_parameter(parameter, analysis.types)?);
    }
    let mut names = BTreeSet::new();
    for (_, _, leaves) in &converted_parameters {
        for (name, name_span, _, _) in leaves {
            if !names.insert(name.as_str()) {
                return Err(Diagnostic::new(
                    ErrorKind::Name,
                    format!("Multiple binding of '{name}' in same scope"),
                    Some(*name_span),
                ));
            }
        }
    }
    let mut locals = analysis.locals.clone();
    let mut constant_locals = analysis.constant_locals.clone();
    // Functions whose parameters bind nothing push no frame at call time,
    // so depths only shift when the new layer holds a slot (the
    // empty-layer rule).
    let layer_slots: usize = converted_parameters
        .iter()
        .map(|(_, _, leaves)| leaves.len())
        .sum();
    if layer_slots > 0 {
        for (_, depth, _) in locals.values_mut() {
            *depth += 1;
        }
    }
    let mut parameter_types = Vec::with_capacity(parameters.len());
    let mut shapes = Vec::with_capacity(parameters.len());
    let mut offset = 0;
    for (parameter_type, shape, leaves) in converted_parameters {
        parameter_types.push(parameter_type);
        shapes.push(shape);
        for (name, _, constant, leaf_type) in leaves {
            locals.insert(name.clone(), (Rc::new(RefCell::new(leaf_type)), 0, offset));
            if constant {
                constant_locals.insert(name.clone());
            } else {
                constant_locals.remove(&name);
            }
            offset += 1;
        }
    }
    let body_analysis = Analysis {
        types: analysis.types,
        globals: analysis.globals,
        overloads: analysis.overloads,
        locals,
        constant_locals,
        in_function: true,
        // A closure evaluates in its captured context, not the defining
        // loop's; `break` legality starts over at the function boundary.
        loop_depth: 0,
    };
    let closure = |body: TypedExpr| TypedExpr::Closure {
        parameters: parameters.len(),
        shapes: Rc::from(shapes),
        recursive: false,
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

/// Convert a recursive function literal (axis.w:3137-3158): the declared
/// result type makes the function type fully determined, the self name is
/// bound to that type ahead of the parameters (one shared layer, so a call
/// frame holds self at slot 0), and the body converts against the declared
/// result type.
fn convert_rec_lambda_expression(
    expression: &Expr,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let Expr::RecLambda {
        self_name,
        parameters,
        result_type,
        body,
        span,
        ..
    } = expression
    else {
        unreachable!("only called for recursive lambdas")
    };
    let mut names = BTreeSet::new();
    names.insert(self_name.as_str());
    let mut converted_parameters = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        converted_parameters.push(convert_parameter(parameter, analysis.types)?);
    }
    for (_, _, leaves) in &converted_parameters {
        for (name, name_span, _, _) in leaves {
            if !names.insert(name.as_str()) {
                return Err(Diagnostic::new(
                    ErrorKind::Name,
                    format!("Multiple binding of '{name}' in same scope"),
                    Some(*name_span),
                ));
            }
        }
    }
    let mut parameter_types = Vec::with_capacity(parameters.len());
    let mut shapes = Vec::with_capacity(parameters.len());
    for (parameter_type, shape, _) in &converted_parameters {
        parameter_types.push(parameter_type.clone());
        shapes.push(shape.clone());
    }
    let function_type = Type::function(Type::tuple(parameter_types), result_type.resolve());
    let mut locals = analysis.locals.clone();
    let mut constant_locals = analysis.constant_locals.clone();
    // The call frame always holds the self binding, so depths shift by one
    // even for a parameterless recursive function.
    for (_, depth, _) in locals.values_mut() {
        *depth += 1;
    }
    locals.insert(
        self_name.clone(),
        (Rc::new(RefCell::new(function_type.clone())), 0, 0),
    );
    let mut offset = 1;
    for (_, _, leaves) in converted_parameters {
        for (name, _, constant, leaf_type) in leaves {
            locals.insert(name.clone(), (Rc::new(RefCell::new(leaf_type)), 0, offset));
            if constant {
                constant_locals.insert(name.clone());
            } else {
                constant_locals.remove(&name);
            }
            offset += 1;
        }
    }
    let body_analysis = Analysis {
        types: analysis.types,
        globals: analysis.globals,
        overloads: analysis.overloads,
        locals,
        constant_locals,
        in_function: true,
        // A closure evaluates in its captured context, not the defining
        // loop's; `break` legality starts over at the function boundary.
        loop_depth: 0,
    };
    let closure = |body: TypedExpr| TypedExpr::Closure {
        parameters: parameters.len(),
        shapes: Rc::from(shapes),
        recursive: true,
        body: Rc::new(body),
    };
    if required.is_void() {
        let mut dummy = result_type.resolve();
        let converted = convert_expr(body, &mut dummy, &body_analysis)?;
        return Ok(TypedExpr::Void(Box::new(closure(converted))));
    }
    if !required.specialise(&function_type, analysis.types) {
        return Err(type_error(
            format!(
                "type {} does not match required pattern {}",
                function_type.display(analysis.types),
                required.display(analysis.types)
            ),
            *span,
        ));
    }
    let mut result_required = result_type.resolve();
    let converted = convert_expr(body, &mut result_required, &body_analysis)?;
    Ok(closure(converted))
}

fn convert_overload_application(
    name: &str,
    expressions: &[Expr],
    required: &mut Type,
    span: SourceSpan,
    analysis: &Analysis<'_>,
    resolve_name_first: bool,
) -> Result<TypedExpr, Diagnostic> {
    let variants = merged_variants(name, analysis.overloads, analysis.types);
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
    for (position, variant) in variants.iter().enumerate() {
        if variant.arg_type == a_priori_type {
            chosen = Some(position);
            break;
        }
        if crate::coercions::is_close(&a_priori_type, &variant.arg_type, analysis.types) & 0x1 != 0
        {
            chosen = Some(position);
            break;
        }
    }
    let position = chosen.ok_or_else(|| {
        let message = if variants.len() == 1 {
            format!(
                "found {} while {} was needed.",
                a_priori_type.display(analysis.types),
                variants[0].arg_type.display(analysis.types),
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
    let variant = &variants[position];
    let expected: Vec<Type> = if expressions.len() == 1 {
        vec![variant.arg_type.clone()]
    } else {
        match &variant.arg_type {
            Type::Tuple(components) => components.clone(),
            single => vec![single.clone()],
        }
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
    match variant.origin {
        OverloadOrigin::Builtin(index) => conform_types(
            &variant.result_type,
            required,
            TypedExpr::BuiltinCall {
                builtin: index,
                arguments,
                span,
            },
            span,
            analysis,
        ),
        OverloadOrigin::User(user_index) => {
            let user = &analysis.overloads.user_variants(name)[user_index];
            // A user overload applies its closure: the argument is ONE
            // value, the tuple display for several parameters.
            let argument = if arguments.len() == 1 {
                arguments
                    .into_iter()
                    .next()
                    .expect("a single argument was converted")
            } else {
                TypedExpr::TupleDisplay(arguments)
            };
            conform_types(
                &variant.result_type,
                required,
                TypedExpr::FunctionCall {
                    function: Box::new(TypedExpr::Denotation(user.value.clone())),
                    argument: Box::new(argument),
                    span,
                },
                span,
                analysis,
            )
        }
    }
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
    /// A printer wrapper (atlas-types.w:8944-8957, 8850-8859): writes its
    /// report to the evaluation output at BOTH levels and yields the empty
    /// tuple at single_value; no diagnostics precede its no-value gate.
    DomainPrinter {
        name: &'static str,
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
    VecDivideInt,
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
        context: &mut EvaluationContext,
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
            BuiltinImpl::DomainPrinter { name } => {
                let text = domain_builtins::print_text(name, &arguments, span)
                    .map_err(Control::Runtime)?;
                context.print_text(text);
                Ok(at_builtin_level(level, || Value::Tuple(Vec::new())))
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

/// A printer wrapper entry: `void` result, the report produced at both
/// evaluation levels (upstream prints before its `wrap_tuple<0>()` gate).
fn domain_printer_builtin(name: &'static str, arg_type: Type) -> Builtin {
    Builtin {
        name,
        arg_type,
        result: Type::void(),
        hunger: 0,
        implementation: BuiltinImpl::DomainPrinter { name },
    }
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
        ScalarOp::VecDivideInt => match expect_pair(arguments) {
            (Value::Vector(vector), Value::Integer(denominator)) => {
                // Upstream constructs the rational vector inside its
                // no_value gate, so the diagnostics fire only when the
                // value is actually produced.
                if level == Level::NoValue {
                    return Ok(None);
                }
                if denominator == 0 {
                    return Err(runtime("Denominator 0 in rational vector", span));
                }
                let negative = denominator < 0;
                let magnitude = if negative {
                    -&denominator
                } else {
                    denominator.clone()
                };
                let magnitude = u64::try_from(&magnitude)
                    .map_err(|_| runtime("Integer value to big for conversion", span))?;
                Ok(at_builtin_level(level, || {
                    let numerators = vector
                        .0
                        .iter()
                        .map(|&entry| {
                            let entry = i64::from(entry);
                            if negative {
                                -entry
                            } else {
                                entry
                            }
                        })
                        .collect();
                    Value::RatVector(
                        RatVec::new(numerators, magnitude).expect("denominator checked nonzero"),
                    )
                }))
            }
            (first, second) => panic!("vector division saw {first:?} and {second:?}"),
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
            // vector_div_wrapper (global.w:4108): vec/int is the ratvec
            // literal surface (`[0]/1`); the zero-denominator diagnostic
            // fires only when the value is produced, as upstream gates it.
            scalar_builtin(
                "/",
                Type::tuple(vec![primitive_type(Prim::Vec), int_type()]),
                primitive_type(Prim::RatVec),
                0,
                ScalarOp::VecDivideInt,
            ),
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
            // The language surface exposes these startup overloads even
            // before their domain implementations land.  Keeping them in
            // the registry makes `whattype * ?` and user replacement obey
            // the reference overload table; calls still fail through the
            // domain bridge until the owning math slice is implemented.
            domain_builtin_skip(
                "*",
                Type::tuple(vec![primitive_type(Prim::Vec), int_type()]),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![primitive_type(Prim::RatVec), int_type()]),
                primitive_type(Prim::RatVec),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![primitive_type(Prim::RatVec), rat_type()]),
                primitive_type(Prim::RatVec),
                0,
            ),
            domain_builtin_skip("*", pair(primitive_type(Prim::Vec)), int_type(), 0),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![primitive_type(Prim::Mat), primitive_type(Prim::Vec)]),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::RatVec),
                ]),
                primitive_type(Prim::RatVec),
                0,
            ),
            domain_builtin_skip(
                "*",
                pair(primitive_type(Prim::Mat)),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![primitive_type(Prim::Vec), primitive_type(Prim::Mat)]),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::RatVec),
                    primitive_type(Prim::Mat),
                ]),
                primitive_type(Prim::RatVec),
                0,
            ),
            domain_builtin_skip(
                "*",
                pair(primitive_type(Prim::LieType)),
                primitive_type(Prim::LieType),
                0,
            ),
            domain_builtin(
                "*",
                pair(primitive_type(Prim::WeylElt)),
                primitive_type(Prim::WeylElt),
                1,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::WeylElt),
                    primitive_type(Prim::Vec),
                ]),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::WeylElt),
                ]),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin_skip(
                "*",
                pair(primitive_type(Prim::Split)),
                primitive_type(Prim::Split),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![int_type(), primitive_type(Prim::KTypePol)]),
                primitive_type(Prim::KTypePol),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::Split),
                    primitive_type(Prim::KTypePol),
                ]),
                primitive_type(Prim::KTypePol),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![primitive_type(Prim::Param), rat_type()]),
                primitive_type(Prim::Param),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![int_type(), primitive_type(Prim::ParamPol)]),
                primitive_type(Prim::ParamPol),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::Split),
                    primitive_type(Prim::ParamPol),
                ]),
                primitive_type(Prim::ParamPol),
                0,
            ),
            domain_builtin_skip(
                "*",
                Type::tuple(vec![primitive_type(Prim::ParamPol), rat_type()]),
                primitive_type(Prim::ParamPol),
                0,
            ),
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
            domain_builtin_skip(
                "Smith_Cartan",
                primitive_type(Prim::LieType),
                Type::tuple(vec![primitive_type(Prim::Mat), primitive_type(Prim::Vec)]),
                0,
            ),
            domain_builtin_validate(
                "filter_units",
                Type::tuple(vec![primitive_type(Prim::Mat), primitive_type(Prim::Vec)]),
                Type::tuple(vec![primitive_type(Prim::Mat), primitive_type(Prim::Vec)]),
                0,
            ),
            domain_builtin_validate(
                "ann_mod",
                Type::tuple(vec![primitive_type(Prim::Mat), int_type()]),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "replace_gen",
                Type::tuple(vec![
                    Type::tuple(vec![primitive_type(Prim::Mat), primitive_type(Prim::Vec)]),
                    primitive_type(Prim::Mat),
                ]),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "quotient_basis",
                Type::tuple(vec![
                    primitive_type(Prim::LieType),
                    Type::row(primitive_type(Prim::RatVec)),
                ]),
                primitive_type(Prim::Mat),
                0,
            ),
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
            domain_builtin_skip(
                "nr_of_posroots",
                primitive_type(Prim::RootDatum),
                int_type(),
                0,
            ),
            domain_builtin_skip("rank", primitive_type(Prim::RootDatum), int_type(), 0),
            domain_builtin(
                "root",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin(
                "coroot",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin(
                "is_long_root",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                bool_type(),
                0,
            ),
            domain_builtin(
                "is_long_coroot",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                bool_type(),
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
            domain_builtin_skip(
                "nr_of_dual_real_forms",
                primitive_type(Prim::InnerClass),
                int_type(),
                0,
            ),
            domain_builtin_skip(
                "form_names",
                primitive_type(Prim::InnerClass),
                Type::row(string_type()),
                0,
            ),
            domain_builtin_skip(
                "dual_form_names",
                primitive_type(Prim::InnerClass),
                Type::row(string_type()),
                0,
            ),
            domain_builtin_validate(
                "real_form",
                Type::tuple(vec![primitive_type(Prim::InnerClass), int_type()]),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin_skip(
                "dual_real_form",
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
            domain_builtin_skip(
                "dual_quasisplit_form",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin_skip("form_number", primitive_type(Prim::RealForm), int_type(), 0),
            domain_builtin_skip("KGB_size", primitive_type(Prim::RealForm), int_type(), 0),
            // central_fiber_wrapper (atlas-types.w:3915-3929): only the type
            // layer's conform error precedes its no-value gate, so skip.
            domain_builtin_skip(
                "central_fiber",
                primitive_type(Prim::RealForm),
                Type::row(primitive_type(Prim::Vec)),
                0,
            ),
            // print_KGB_wrapper (atlas-types.w:8944-8957): prints
            // `kgbsize: N` then kgb_io::var_print_KGB, unconditionally —
            // the report fires at both evaluation levels. The selection
            // overload (atlas-types.w:8958-8973) shares the plumbing; its
            // real-form mismatch diagnostic fires inside the print.
            domain_printer_builtin("print_KGB", primitive_type(Prim::RealForm)),
            domain_printer_builtin(
                "print_KGB",
                Type::tuple(vec![
                    primitive_type(Prim::RealForm),
                    Type::row(primitive_type(Prim::KgbElt)),
                ]),
            ),
            domain_builtin_validate(
                "KGB",
                Type::tuple(vec![primitive_type(Prim::RealForm), int_type()]),
                primitive_type(Prim::KgbElt),
                0,
            ),
            // build_KGB_element_wrapper (atlas-types.w:4580-4607): the
            // synthetic RealForm+mat+ratvec constructor; every diagnostic
            // fires before its no_value gate, so it validates.
            domain_builtin_validate(
                "KGB_elt",
                Type::tuple(vec![
                    primitive_type(Prim::RealForm),
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::RatVec),
                ]),
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
            domain_builtin_skip(
                "base_grading_vector",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::RatVec),
                0,
            ),
            domain_builtin_skip(
                "initial_torus_bits",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin_skip(
                "torus_bits",
                primitive_type(Prim::KgbElt),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin_skip(
                "%",
                primitive_type(Prim::KgbElt),
                Type::tuple(vec![primitive_type(Prim::RealForm), int_type()]),
                0,
            ),
            domain_builtin_skip(
                "twist",
                primitive_type(Prim::KgbElt),
                primitive_type(Prim::KgbElt),
                3,
            ),
            domain_builtin_validate(
                "twist",
                Type::tuple(vec![
                    primitive_type(Prim::KgbElt),
                    primitive_type(Prim::Mat),
                ]),
                primitive_type(Prim::KgbElt),
                1,
            ),
            // WeylElt surface (atlas-types.w:2629-2639): constructor,
            // attributes, unary relations, and the product/inverse/
            // generator-product operators. Binary =/!= are domain
            // relations registered in the relation block below.
            domain_builtin(
                "W_elt",
                Type::tuple(vec![primitive_type(Prim::RootDatum), Type::row(int_type())]),
                primitive_type(Prim::WeylElt),
                0,
            ),
            domain_builtin(
                "word",
                primitive_type(Prim::WeylElt),
                Type::row(int_type()),
                0,
            ),
            domain_builtin(
                "root_datum",
                primitive_type(Prim::WeylElt),
                primitive_type(Prim::RootDatum),
                0,
            ),
            domain_builtin("length", primitive_type(Prim::WeylElt), int_type(), 0),
            domain_builtin("=", primitive_type(Prim::WeylElt), bool_type(), 0),
            domain_builtin("!=", primitive_type(Prim::WeylElt), bool_type(), 0),
            domain_builtin(
                "/",
                primitive_type(Prim::WeylElt),
                primitive_type(Prim::WeylElt),
                3,
            ),
            domain_builtin(
                "#",
                Type::tuple(vec![primitive_type(Prim::WeylElt), int_type()]),
                primitive_type(Prim::WeylElt),
                1,
            ),
            // CartanClass surface (atlas-types.w:4347-4363): the two
            // constructors, counts, most-split, involution, the (dual)
            // real-form sweeps, square classes, and the per-form fiber
            // partition. The constructors bounds-check and fiber_partition
            // guards before their upstream no-value gates, so they validate.
            domain_builtin_validate(
                "Cartan_class",
                Type::tuple(vec![primitive_type(Prim::InnerClass), int_type()]),
                primitive_type(Prim::CartanClass),
                0,
            ),
            domain_builtin_validate(
                "Cartan_class",
                Type::tuple(vec![primitive_type(Prim::RealForm), int_type()]),
                primitive_type(Prim::CartanClass),
                0,
            ),
            domain_builtin_skip(
                "nr_of_Cartan_classes",
                primitive_type(Prim::InnerClass),
                int_type(),
                0,
            ),
            domain_builtin_skip(
                "nr_of_Cartan_classes",
                primitive_type(Prim::RealForm),
                int_type(),
                0,
            ),
            domain_builtin_skip(
                "most_split_Cartan",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::CartanClass),
                0,
            ),
            domain_builtin_skip(
                "involution",
                primitive_type(Prim::CartanClass),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin_skip(
                "real_forms",
                primitive_type(Prim::CartanClass),
                Type::row(primitive_type(Prim::RealForm)),
                0,
            ),
            domain_builtin_skip(
                "dual_real_forms",
                primitive_type(Prim::CartanClass),
                Type::row(primitive_type(Prim::RealForm)),
                0,
            ),
            domain_builtin_skip(
                "square_classes",
                primitive_type(Prim::CartanClass),
                Type::row(Type::row(int_type())),
                0,
            ),
            domain_builtin_validate(
                "fiber_partition",
                Type::tuple(vec![
                    primitive_type(Prim::CartanClass),
                    primitive_type(Prim::RealForm),
                ]),
                Type::row(int_type()),
                0,
            ),
            // Real-form label matrices (atlas-types.w:3323-3400, 3709-3724):
            // occurrence/block_sizes/Cartan_order only read and print values
            // behind their upstream no-value gates, so they skip; block_size
            // bounds-checks before its gate, so it validates.
            domain_builtin_skip(
                "occurrence_matrix",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin_skip(
                "dual_occurrence_matrix",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin_skip(
                "block_sizes",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin_validate(
                "block_size",
                Type::tuple(vec![
                    primitive_type(Prim::InnerClass),
                    int_type(),
                    int_type(),
                ]),
                int_type(),
                0,
            ),
            domain_builtin_skip(
                "Cartan_order",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::Mat),
                0,
            ),
            // print_strongreal_wrapper (atlas-types.w:8850-8859):
            // output::printStrongReal, unconditional like print_KGB.
            domain_printer_builtin("print_strong_real", primitive_type(Prim::CartanClass)),
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
            domain_relation_builtin("=", pair(primitive_type(Prim::WeylElt)), Relation::Equal),
            domain_relation_builtin(
                "!=",
                pair(primitive_type(Prim::WeylElt)),
                Relation::NotEqual,
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
                source,
                span,
            } => {
                let index = expect_integer(force(index, context)?, *span, "subscription index")?;
                match force(array, context)? {
                    Value::List(values) => {
                        let position =
                            checked_index(&index, values.len(), *reversed, source, *span)?;
                        Ok(at_level(level, || values[position].clone()))
                    }
                    // Upstream `string_subscription` (axis.w:4229-4239): the
                    // result is the one-character string at the position.
                    Value::String(text) => {
                        let bytes = text.as_bytes();
                        let position =
                            checked_index(&index, bytes.len(), *reversed, source, *span)?;
                        Ok(at_level(level, || {
                            Value::String(
                                String::from_utf8_lossy(&bytes[position..position + 1])
                                    .into_owned(),
                            )
                        }))
                    }
                    other => panic!("analysis let a non-subscriptable value through: {other}"),
                }
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
                let mut slots = Vec::new();
                for (shape, initializer) in initializers {
                    let value = force(initializer, context)?;
                    distribute(value, shape, &mut slots);
                }
                // A group of pure discards claims no frame (empty-layer
                // rule), exactly as analysis counted no layer for it.
                if slots.is_empty() {
                    body.evaluate(context, level)
                } else {
                    context.with_frame(slots, |context| body.evaluate(context, level))
                }
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
                let mut values = arguments
                    .iter()
                    .map(|argument| force(argument, context))
                    .collect::<Result<Vec<_>, _>>()?;
                if values.len() == 1
                    && matches!(builtin_registry()[*builtin].arg_type, Type::Tuple(_))
                    && matches!(values.first(), Some(Value::Tuple(_)))
                {
                    let Value::Tuple(components) = values.pop().expect("one tuple argument") else {
                        unreachable!("tuple shape checked above")
                    };
                    values = components;
                }
                builtin_registry()[*builtin].run(values, *span, level, context)
            }
            Self::Closure {
                parameters,
                shapes,
                recursive,
                body,
            } => Ok(at_level(level, || {
                Value::Closure(Rc::new(Closure {
                    parameters: *parameters,
                    shapes: shapes.clone(),
                    recursive: *recursive,
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
                apply_closure(&closure, argument, context, level)
            }
            Self::Sequence { first, second } => {
                first.evaluate(context, Level::NoValue)?;
                second.evaluate(context, level)
            }
            Self::While { condition, body } => {
                let mut collected = Vec::new();
                loop {
                    if let Some(condition) = condition {
                        match force(condition, context)? {
                            Value::Boolean(true) => {}
                            Value::Boolean(false) => break,
                            other => {
                                panic!("analysis let a non-boolean loop condition through: {other}")
                            }
                        }
                    }
                    match body.evaluate(context, Level::SingleValue) {
                        Ok(Some(value)) => collected.push(value),
                        Ok(None) => unreachable!("single-value loop body yields a value"),
                        // The breaking iteration contributes no value.
                        Err(Control::Break(0)) => break,
                        Err(Control::Dont) => break,
                        Err(Control::Break(levels)) => {
                            return Err(Control::Break(levels - 1));
                        }
                        Err(control) => return Err(control),
                    }
                }
                Ok(at_level(level, || Value::List(collected.clone())))
            }
            Self::For {
                shape,
                index,
                iterable,
                body,
            } => {
                let values = match force(iterable, context)? {
                    Value::List(values) => values,
                    other => panic!("analysis let a non-row iterable through: {other}"),
                };
                let mut collected = Vec::new();
                for (position, element) in values.into_iter().enumerate() {
                    let mut slots = Vec::new();
                    distribute(element, shape, &mut slots);
                    if *index {
                        slots.push(Rc::new(Value::Integer(BigInt::from(position))));
                    }
                    let result = if slots.is_empty() {
                        // A pure-discard layer pushes no frame, matching the
                        // analysis-time empty-layer rule.
                        body.evaluate(context, Level::SingleValue)
                    } else {
                        context
                            .with_frame(slots, |context| body.evaluate(context, Level::SingleValue))
                    };
                    match result {
                        Ok(Some(value)) => collected.push(value),
                        Ok(None) => unreachable!("single-value loop body yields a value"),
                        Err(Control::Break(0)) => break,
                        Err(Control::Break(levels)) => {
                            return Err(Control::Break(levels - 1));
                        }
                        Err(control) => return Err(control),
                    }
                }
                Ok(at_level(level, || Value::List(collected.clone())))
            }
            Self::Break => Err(Control::Break(0)),
            Self::Dont => Err(Control::Dont),
            Self::Die { span } => Err(runtime("I die", *span)),
            Self::UnionInject {
                tag,
                injector_name,
                payload,
            } => {
                let value = force(payload, context)?;
                Ok(at_level(level, || Value::Union {
                    tag: *tag,
                    injector_name: injector_name.clone(),
                    value: Box::new(value.clone()),
                }))
            }
            Self::TupleProject { index, inner } => {
                let value = force(inner, context)?;
                let Value::Tuple(components) = value else {
                    panic!("analysis let a non-tuple projection through: {value}")
                };
                Ok(at_level(level, || components[*index].clone()))
            }
            Self::Case {
                subject,
                branches,
                fallback,
                span,
            } => {
                let subject = force(subject, context)?;
                let Value::Union { tag, value, .. } = subject else {
                    panic!("analysis let a non-union discrimination subject through: {subject}")
                };
                for (branch_tag, shape, body) in branches {
                    if *branch_tag != tag {
                        continue;
                    }
                    let mut slots = Vec::new();
                    distribute(value.as_ref().clone(), shape, &mut slots);
                    // A branch binding no slot claims no frame, exactly as
                    // analysis counted no layer for it (empty-layer rule).
                    return if slots.is_empty() {
                        body.evaluate(context, level)
                    } else {
                        context.with_frame(slots, |context| body.evaluate(context, level))
                    };
                }
                match fallback {
                    Some(body) => body.evaluate(context, level),
                    None => Err(runtime(
                        "Discrimination without else branch failed to match",
                        *span,
                    )),
                }
            }
            Self::Next { first, second } => {
                // The first value is retained while the second half still
                // evaluates for effects (upstream next_expression).
                let value = first.evaluate(context, level)?;
                second.evaluate(context, Level::NoValue)?;
                Ok(value)
            }
            Self::IntCase {
                condition,
                branches,
                then_branch,
                else_branch,
                span,
            } => {
                let selector = expect_integer(force(condition, context)?, *span, "case selector")?;
                let negative = selector < 0;
                let in_range = (!negative)
                    .then(|| usize::try_from(&selector).ok())
                    .flatten()
                    .filter(|index| *index < branches.len());
                if negative {
                    if let Some(then_branch) = then_branch {
                        return then_branch.evaluate(context, level);
                    }
                } else if let Some(index) = in_range {
                    return branches[index].evaluate(context, level);
                }
                if let Some(else_branch) = else_branch {
                    return else_branch.evaluate(context, level);
                }
                // No else branch: wrap modulo the branch count
                // (axis.w:4884-4897 arithmetic::remainder; floored so a
                // negative selector stays in range, which upstream leaves
                // undefined).
                let count = BigInt::from(branches.len());
                let (_, remainder) = euclidean_divmod(&selector, &count);
                let index = usize::try_from(&remainder).expect("wrapped index is in range");
                branches[index].evaluate(context, level)
            }
            Self::UnionCase {
                subject,
                branches,
                span: _,
            } => {
                let subject = force(subject, context)?;
                let Value::Union { tag, value, .. } = subject else {
                    panic!("analysis let a non-union union-case subject through: {subject}")
                };
                // The positional branch evaluates to a function, applied
                // to the payload (axis.w:5041-5049).
                let function = force(&branches[usize::from(tag)], context)?;
                let Value::Closure(closure) = function else {
                    panic!("analysis let a non-function union-case branch through: {function}")
                };
                apply_closure(&closure, value.as_ref().clone(), context, level)
            }
            Self::CountedFor {
                has_name,
                decreasing,
                count,
                bound,
                body,
                span,
            } => {
                let count = expect_integer(force(count, context)?, *span, "loop count")?;
                // A negative count yields an empty row (axis.w:6521).
                let count = if count < 0 { BigInt::from(0) } else { count };
                let lower = match bound {
                    Some(bound) => expect_integer(force(bound, context)?, *span, "loop bound")?,
                    None => BigInt::from(0),
                };
                // Increasing takes `count` steps from the bound;
                // decreasing runs from bound+count-1 down to the bound
                // inclusive (axis.w:6638-6670).
                let mut index = if *decreasing {
                    &lower + &count - BigInt::from(1)
                } else {
                    lower.clone()
                };
                let mut collected = Vec::new();
                loop {
                    let active = if *decreasing {
                        index >= lower
                    } else {
                        index < &lower + &count
                    };
                    if !active {
                        break;
                    }
                    let result = if *has_name {
                        context
                            .with_frame(vec![Rc::new(Value::Integer(index.clone()))], |context| {
                                body.evaluate(context, Level::SingleValue)
                            })
                    } else {
                        body.evaluate(context, Level::SingleValue)
                    };
                    match result {
                        Ok(Some(value)) => collected.push(value),
                        Ok(None) => unreachable!("single-value loop body yields a value"),
                        // The breaking iteration contributes no value.
                        Err(Control::Break(0)) => break,
                        Err(Control::Break(levels)) => return Err(Control::Break(levels - 1)),
                        Err(control) => return Err(control),
                    }
                    if *decreasing {
                        index -= BigInt::from(1);
                    } else {
                        index += BigInt::from(1);
                    }
                }
                Ok(at_level(level, || Value::List(collected.clone())))
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

/// Apply a closure value to one argument value (upstream `apply`,
/// axis.w:3222-3571): the argument distributes into the frame slots its
/// parameter shapes describe, a recursive closure additionally binds
/// itself at slot 0, and a `return` unwinds to this call boundary.
fn apply_closure(
    closure: &Rc<Closure>,
    argument: Value,
    context: &mut EvaluationContext,
    level: Level,
) -> Result<Option<Value>, Control> {
    // The argument is one value: several parameters split it as a tuple,
    // a single parameter takes it whole. A parameterless call pushes no
    // frame (empty-layer rule).
    let mut slots = match closure.parameters {
        0 => None,
        1 => {
            let mut slots = Vec::new();
            distribute(argument, &closure.shapes[0], &mut slots);
            Some(slots)
        }
        _ => match argument {
            Value::Tuple(values) => {
                assert_eq!(
                    values.len(),
                    closure.parameters,
                    "analysis let an argument arity mismatch through"
                );
                let mut slots = Vec::new();
                for (value, shape) in values.into_iter().zip(closure.shapes.iter()) {
                    distribute(value, shape, &mut slots);
                }
                Some(slots)
            }
            other => panic!("multi-parameter call saw non-tuple argument {other}"),
        },
    };
    // All-anonymous parameter lists claim no frame, matching the
    // analysis-time layer rule; a recursive closure still gains one for
    // its self slot below.
    if !closure.recursive {
        slots = slots.filter(|slots| !slots.is_empty());
    }
    // A recursive closure binds itself at slot 0, ahead of the argument
    // slots (upstream `maybe_push`, axis.w:3548-3560); the new frame is
    // not part of the captured chain, so the Rc structure stays acyclic.
    if closure.recursive {
        slots
            .get_or_insert_with(Vec::new)
            .insert(0, Rc::new(Value::Closure(closure.clone())));
    }
    context.with_context(closure.frame.clone(), |context| {
        let result = match slots {
            Some(slots) => {
                context.with_frame(slots, |context| closure.body.evaluate(context, level))
            }
            None => closure.body.evaluate(context, level),
        };
        match result {
            // An explicit `return` ends the call and supplies its value
            // (upstream function_return caught in apply, axis.w:3569-3571).
            Err(Control::Return(value)) => Ok(at_level(level, move || value.clone())),
            other => other,
        }
    })
}

/// Bind one value against a slot shape, pushing leaves left-to-right
/// (upstream `bind_pattern` at evaluation time).
fn distribute(value: Value, shape: &SlotShape, slots: &mut Vec<Rc<Value>>) {
    match shape {
        SlotShape::Leaf => slots.push(Rc::new(value)),
        SlotShape::Discard => {}
        SlotShape::Tuple { elements, whole } => {
            if *whole {
                slots.push(Rc::new(value.clone()));
            }
            let Value::Tuple(values) = value else {
                panic!("analysis let a non-tuple value reach a tuple pattern: {value}")
            };
            assert_eq!(
                values.len(),
                elements.len(),
                "analysis let a tuple arity mismatch through"
            );
            for (value, element) in values.into_iter().zip(elements) {
                distribute(value, element, slots);
            }
        }
    }
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
    source: &str,
    span: SourceSpan,
) -> Result<usize, Control> {
    let original = index.clone();
    let out_of_range = || {
        runtime(
            format!("index {original} out of range (0<= . <{length}) in subscription {source}"),
            span,
        )
    };
    let index = usize::try_from(index).map_err(|_| out_of_range())?;
    if index >= length {
        return Err(out_of_range());
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
        let overloads = OverloadState::default();
        let analysis = Analysis::new(&table, globals, &overloads);
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
        assert!(error.message.contains("Undefined identifier"));
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
            .contains("Undefined identifier 'missing' in assignment"));
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
    fn subscription_reports_quote_the_compact_source_like_the_oracle() {
        // B12 corpus (tests/fixtures/eval/runtime_errors_b12.atlas): the
        // oracle's `range_mess` appends the printed subscription node
        // (axis.w:4188-4194), and a tuple subscript is the analysis-time
        // `not_so` type error (axis.w:4101-4105).
        let error = convert_and_run("[1,2][5]").expect_err("row out of range");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(
            error.message,
            "index 5 out of range (0<= . <2) in subscription [1,2][5]"
        );

        let error = convert_and_run("(1,2)[5]").expect_err("tuple subscript");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(
            error.message,
            "Cannot subscript value of type (int,int) with index of type int"
        );

        let error = convert_and_run("\"abc\"[7]").expect_err("string out of range");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(
            error.message,
            "index 7 out of range (0<= . <3) in subscription \"abc\"[7]"
        );
    }

    #[test]
    fn string_subscription_yields_one_character_strings() {
        // Upstream `string_subscription` (axis.w:4229-4239), including the
        // reversed form.
        let (found, value) = convert_and_run("\"abc\"[1]").expect("forward");
        assert_eq!(found, Type::Primitive(Prim::String));
        assert_eq!(value, Value::String("b".to_owned()));

        let (_, value) = convert_and_run("\"abc\"~[0]").expect("reversed");
        assert_eq!(value, Value::String("c".to_owned()));
    }

    #[test]
    fn patternless_let_uses_parallel_groups_and_supports_assignment() {
        let (_, value) = convert_and_run("let x = 1 then x = x + 1 in x").expect("groups");
        assert_eq!(value, Value::Integer(2.into()));

        let (_, value) = convert_and_run("let x = 3 in x := x + 1").expect("local assignment");
        assert_eq!(value, Value::Integer(4.into()));

        let error = convert_and_run("let x = 1, y = x in y").expect_err("parallel group");
        assert!(error.message.contains("Undefined identifier 'x'"));
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
        // Non-boolean condition is a type error (the loops_b4_rejected
        // fixture pins this exact oracle wording).
        let error = convert_and_run("if 1 then 2 else 3 fi").expect_err("condition type");
        assert_eq!(error.message, "found int while bool was needed.");
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
        let overloads = OverloadState::default();
        let analysis = Analysis::new(&table, &globals, &overloads);
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
                if message == "Illegal real form number: 99"
        ));
    }

    #[test]
    fn ann_mod_no_value_narrows_the_modulus_without_running_the_lattice() {
        let error = convert_and_run("begin ann_mod([[1]],2147483648);7 end")
            .expect_err("the wrapper narrows before its no-value gate");
        assert_eq!(error.message, "Integer value to big for conversion");

        let (_, value) = convert_and_run("begin ann_mod([[1]],0);7 end")
            .expect("zero is not passed to the lattice in no-value context");
        assert_eq!(value, Value::Integer(BigInt::from(7)));
    }

    #[test]
    fn filter_units_no_value_skips_relation_conversion_and_reduction() {
        let tall_column = (0..65).map(|_| "0").collect::<Vec<_>>().join(",");
        let source = format!("begin filter_units(([[{tall_column}]],[1]));7 end");
        let (_, value) = convert_and_run(&source)
            .expect("no-value filter_units does not build a relation matrix");
        assert_eq!(value, Value::Integer(BigInt::from(7)));
    }

    #[test]
    fn filter_units_no_value_still_checks_the_factor_count() {
        let error = convert_and_run("begin filter_units(([[0]],[1,2]));7 end")
            .expect_err("factor count precedes the no-value gate");
        assert_eq!(error.message, "Too many factors: 2 for 1 columns");
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
        assert_eq!(error.message, "found string while int was needed.");

        let error = convert_and_run("bool: (1,2)").expect_err("no tuple coercion");
        assert!(error.message.contains("does not match"));
    }

    #[test]
    fn tuple_producing_expression_supplies_one_builtin_argument_pack() {
        let (type_, value) =
            convert_and_run("+(%(5/2))").expect("divmod tuple supplies the binary plus overload");
        assert_eq!(type_, int_type());
        assert_eq!(value, Value::Integer(7.into()));
    }

    #[test]
    fn nested_tuple_user_parameter_is_not_flattened() {
        let (type_, value) =
            convert_and_run("let f = (((int a, int b), int c)): a + b + c in f((1,2),3)")
                .expect("nested tuple parameter retains its inner tuple");
        assert_eq!(type_, int_type());
        assert_eq!(value, Value::Integer(6.into()));
    }

    #[test]
    fn ordinary_user_calls_and_set_overloads_keep_argument_shape() {
        let (_, value) = convert_and_run("let add(int a, int b) = a + b in add(20,22)")
            .expect("ordinary local function call");
        assert_eq!(value, Value::Integer(42.into()));

        let mut context = TypedContext::new();
        context
            .execute(&command("set add = (int a, int b): a + b"))
            .expect("install user overload");
        let events = context
            .execute(&command("add(20,22)"))
            .expect("ordinary user overload call");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::Value {
                value: Value::Integer(value),
                ..
            }] if value == &BigInt::from(42)
        ));
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
    fn a_return_outside_a_function_body_is_rejected_at_analysis() {
        let error = convert_and_run("return 3").expect_err("top-level return");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(
            error.message,
            "One can only use 'return' within a function body"
        );

        // Analysis rejects the whole expression before anything evaluates,
        // even when the return hides behind a non-function construct.
        let error = convert_and_run("if true then return 1 fi").expect_err("conditional return");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(
            error.message,
            "One can only use 'return' within a function body"
        );

        // ...while a return lexically inside a body converts and unwinds.
        let (_, value) =
            convert_and_run("let f = (int n): if n = 0 then return 1 else 2 fi in f(0)")
                .expect("return inside a body");
        assert_eq!(value, Value::Integer(1.into()));
    }

    #[test]
    fn user_call_mismatches_use_the_oracle_wording() {
        let error =
            convert_and_run("let f = (int n): n + 1 in f(\"x\")").expect_err("string argument");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "found string while int was needed.");

        // The empty argument list is void matched against the pattern.
        let error = convert_and_run("let f = (int n): n + 1 in f()").expect_err("missing argument");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "found void while int was needed.");

        let error = convert_and_run("let x = 1 in x(2)").expect_err("non-function callee");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "found int while (*->*) was needed.");

        // Coercible arguments still convert against the parameter type.
        let (type_, value) =
            convert_and_run("let f = (rat x): x + 1 in f(2)").expect("coerced argument");
        assert_eq!(type_, rat_type());
        assert_eq!(value.to_string(), "3/1");

        // An undefined selector keeps the name error.
        let error = convert_and_run("2.g").expect_err("undefined selector");
        assert_eq!(error.kind, ErrorKind::Name);
        assert_eq!(error.message, "Undefined identifier 'g'");
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
        assert_eq!(error.message, "found int while (*->*) was needed.");
    }

    #[test]
    fn let_function_sugar_and_recursion_evaluate() {
        // The B3b fixture shapes (sanity expectations only; the oracle
        // capture is still pending).
        for (source, expected) in [
            ("let f(int n) = n + 1 in f(2)", 3),
            ("let add(int a, int b) = a + b in add(20, 22)", 42),
            (
                "let rec_fun f(int n) = int: if n=0 then 1 else n*f(n-1) fi in f(5)",
                120,
            ),
            (
                "let f = rec_fun g(int n) int: if n=0 then 1 else n*g(n-1) fi in f(5)",
                120,
            ),
            ("let x = 41 in let f() = x + 1 in f()", 42),
            (
                "let x = 2 in let rec_fun f(int n) = int: if n=0 then 1 else x*f(n-1) fi in f(4)",
                16,
            ),
            // `return` is legal inside a recursive body.
            (
                "let rec_fun f(int n) = int: if n=0 then return 1 else n*f(n-1) fi in f(5)",
                120,
            ),
        ] {
            let (type_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(type_, Type::Primitive(Prim::Int), "source: {source}");
            assert_eq!(value, Value::Integer(expected.into()), "source: {source}");
        }

        // A recursive closure escaping its defining scope keeps both the
        // captured frame and its self binding.
        let (_, value) = convert_and_run(
            "let make = (int x): rec_fun f(int n) int: if n=0 then x else f(n-1) fi in make(7)(2)",
        )
        .expect("escaping recursive closure");
        assert_eq!(value, Value::Integer(7.into()));
    }

    #[test]
    fn recursive_and_sugar_rejections_are_analysis_errors() {
        let error = convert_and_run("let f(int n) = n + \"x\" in f(2)").expect_err("bad body");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(
            error.message,
            "Failed to match '+' with argument type (int,string)"
        );

        let error =
            convert_and_run("let rec_fun f(int n) = int: if n=0 then 1 else n*f(\"x\") fi in f(5)")
                .expect_err("bad recursive call argument");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "found string while int was needed.");

        let error = convert_and_run("rec_fun f(int f) int: f").expect_err("self shadows parameter");
        assert_eq!(error.kind, ErrorKind::Name);
        assert!(error.message.contains("Multiple binding of 'f'"));
    }

    #[test]
    fn binding_patterns_evaluate() {
        // The B3c fixture shapes against the frozen reference events.
        for (source, expected) in [
            ("let (a, b) = (1, 2) in a + b", 3),
            ("let !x = 41 in x + 1", 42),
            ("let (a, (b, c)) = (1, (2, 3)) in a + b + c", 6),
            ("let f = ((int a, int b)): a + b in f(3, 4)", 7),
            ("let (a, b): t = (20, 22) in a + b", 42),
            // An empty slot binds nothing; `type .` takes the argument
            // anonymously.
            ("let (a, , c) = (1, 2, 3) in a + c", 4),
            ("let f(int x, int .) = x in f(3, 4)", 3),
        ] {
            let (type_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(type_, Type::Primitive(Prim::Int), "source: {source}");
            assert_eq!(value, Value::Integer(expected.into()), "source: {source}");
        }

        // The whole-value name sees the undestructured tuple.
        let (_, value) = convert_and_run("let (a, b): t = (20, 22) in t").expect("whole binding");
        assert_eq!(
            value,
            Value::Tuple(vec![Value::Integer(20.into()), Value::Integer(22.into())])
        );
    }

    #[test]
    fn pattern_rejections_are_analysis_errors() {
        // The three frozen oracle diagnostics of patterns_b3c_rejected.
        let error = convert_and_run("let !x = 41 in x := 2").expect_err("const assignment");
        assert_eq!(error.kind, ErrorKind::Name);
        assert_eq!(error.message, "Name 'x' is constant in assignment x:=2");

        let error = convert_and_run("let (a, b) = (1, 2, 3) in a").expect_err("arity mismatch");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "found (int,int,int) while (*,*) was needed.");

        let error = convert_and_run("let (a, b) = 1 in a").expect_err("non-tuple destructure");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "found int while (*,*) was needed.");

        // Duplicate names inside one pattern reuse the scope error, and a
        // non-const rebinding shadows a const one outward.
        let error = convert_and_run("let (a, a) = (1, 2) in a").expect_err("duplicate");
        assert_eq!(error.kind, ErrorKind::Name);
        assert!(error.message.contains("Multiple binding of 'a'"));
        let (_, value) =
            convert_and_run("let !x = 1 in let x = 2 in x := 3").expect("non-const shadow rebinds");
        assert_eq!(value, Value::Integer(3.into()));
    }

    #[test]
    fn unit_and_operator_selectors_evaluate() {
        // The B3d fixture shapes against the frozen reference events.
        for (source, expected) in [
            ("let f() = 42 in ().f", 42),
            ("let f(int n) = n * 2 in let g(int n) = n + 1 in 2.f.g", 5),
            ("2.-", -2),
            // An operator selector resolves like the prefix form, and
            // selectors chain.
            ("2.-.-", 2),
        ] {
            let (type_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(type_, Type::Primitive(Prim::Int), "source: {source}");
            assert_eq!(value, Value::Integer(expected.into()), "source: {source}");
        }
    }

    #[test]
    fn selector_rejections_are_analysis_errors() {
        // The two frozen oracle diagnostics of selectors_b3d_rejected.
        let error = convert_and_run("2.+").expect_err("no unary plus for int");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "Failed to match '+' with argument type int");

        let error = convert_and_run("2.3").expect_err("literal is not callable");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "found int while (*->*) was needed.");
    }

    #[test]
    fn loops_evaluate_and_collect_rows() {
        // The eight B4 fixture shapes against the frozen reference events.
        for (source, expected) in [
            ("let x = 0 in while x < 5 do x := x + 1 od", "[1,2,3,4,5]"),
            ("for i in [1, 2, 3] do i * 2 od", "[2,4,6]"),
            ("for x@i in [10, 20, 30] do i od", "[0,1,2]"),
            (
                "let x = 0 in begin while x < 5 do x := x + 1 od; x end",
                "5",
            ),
            (
                "let x = 0 in while do x := x + 1; if x = 3 then break fi od",
                "[(),()]",
            ),
            (
                "let x = 0 in while x < 5 do if x = 3 then break fi; x := x + 1 od",
                "[1,2,3]",
            ),
            ("let x = 10 in while x > 0 do x := x - 2 od", "[8,6,4,2,0]"),
            (
                "for i in [1, 2, 3] do if i = 2 then break fi; i * 10 od",
                "[10]",
            ),
        ] {
            let (_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(value.to_string(), expected, "source: {source}");
        }

        // A break in an inner loop leaves the outer loop collecting, and
        // the loop pattern reuses the binding machinery (destructuring).
        let (_, value) =
            convert_and_run("let x = 0 in while x < 3 do x := x + 1; while do break od; x od")
                .expect("nested loops");
        assert_eq!(value.to_string(), "[1,2,3]");
        let (_, value) =
            convert_and_run("for (a, b) in [(1, 2), (3, 4)] do a + b od").expect("tuple pattern");
        assert_eq!(value.to_string(), "[3,7]");
    }

    #[test]
    fn loop_rejections_match_the_oracle() {
        // Three of the four frozen oracle diagnostics of loops_b4_rejected
        // (the fourth, `break x`, is a parse error covered in syntax.rs).
        let error = convert_and_run("break").expect_err("top-level break");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "Using 'break' not in the reach of any loop");

        // Analysis rejects the whole expression before anything evaluates,
        // even when the break hides inside a conditional.
        let error = convert_and_run("if true then break fi").expect_err("conditional break");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "Using 'break' not in the reach of any loop");

        let error = convert_and_run("for i in 5 do i od").expect_err("non-row iterable");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "Cannot iterate over value of type int");

        let error = convert_and_run("while 1 do 2 od").expect_err("non-boolean condition");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "found int while bool was needed.");
    }

    #[test]
    fn relation_lattice_builtins_flow_through_tuple_results() {
        for (source, expected) in [
            (
                "Smith_Cartan(Lie_type(\"A2\"))",
                "(\n|  1, 0 |\n| -2, 1 |\n,[ 1, 3 ])",
            ),
            (
                "filter_units(Smith_Cartan(Lie_type(\"A2\")))",
                "(\n| 0 |\n| 1 |\n,[ 3 ])",
            ),
            ("ann_mod([[2,4],[6,3]],2)", "\n| -1,  4 |\n|  0, -2 |\n"),
            (
                "replace_gen(Smith_Cartan(Lie_type(\"A2\")),[[0,3]])",
                "\n|  1, 0 |\n| -2, 3 |\n",
            ),
            (
                "quotient_basis(Lie_type(\"A2\"),[[1]/3])",
                "\n|  1, 0 |\n| -2, 3 |\n",
            ),
        ] {
            let (_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should run: {error:?}"));
            assert_eq!(value.to_string(), expected, "source: {source}");
        }
    }
}
