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

use malachite::base::num::arithmetic::traits::{Ceiling, Floor, Mod, Pow};
use malachite::base::num::logic::traits::SignificantBits;
use malachite::{Integer as BigInt, Rational as BigRational};

use crate::coercions::{coercion_between, row_coercion};
use crate::diagnostic::{Diagnostic, ErrorKind, SourceSpan};
use crate::domain_builtins;
use crate::frames::{EvaluationContext, GlobalCell};
use crate::linear_values::{Matrix, RatVec, Vec32};
use crate::syntax::{
    compact_expression, compact_pattern, Command, Expr, ForLoop, LambdaParam, LetBinding,
    MultiAssignmentExpr, Pattern, TypeSpec,
};
use crate::types::{Prim, Type, TypeBinding, TypeNumber, TypeTable};
use crate::value::{Closure, SlotShape, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Instant;

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

/// One resolved destination in a `set pattern := value` expression. Globals
/// capture their cell at analysis time; locals retain lexical coordinates,
/// exactly like the two ordinary assignment nodes.
#[derive(Clone, Debug, PartialEq)]
pub enum MultiAssignmentDestination {
    Global(GlobalCell),
    Local { depth: usize, offset: usize },
}

/// Runtime traversal plan for a multiple assignment. It deliberately keeps
/// the whole-value destination separate so evaluation can visit children
/// left-to-right and the whole destination last (axis.w `thread_assign`).
#[derive(Clone, Debug, PartialEq)]
pub enum MultiAssignmentPlan {
    Omitted,
    Destination(MultiAssignmentDestination),
    Tuple {
        elements: Vec<MultiAssignmentPlan>,
        whole: Option<Box<MultiAssignmentPlan>>,
    },
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
    /// Evaluate `value` completely, then distribute it through `plan` in
    /// post-order. The expression yields that same right-hand-side value.
    MultiAssignment {
        plan: MultiAssignmentPlan,
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
        /// Compact rendering of the source expression, quoted by the
        /// out-of-range diagnostic exactly like the oracle's
        /// `slice_range_error` prints the slice node (axis.w:4299).
        source: String,
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
    /// A top-level builtin RHS of a simple assignment whose hungry operand
    /// is exactly that assignment's destination. Evaluation moves the value
    /// out of the destination immediately before that operand is supplied.
    HungryBuiltinCall {
        builtin: usize,
        arguments: Vec<TypedExpr>,
        pilfer: PilferDestination,
        pilfer_index: usize,
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

#[derive(Clone, Debug, PartialEq)]
pub enum PilferDestination {
    Global {
        name: String,
        cell: GlobalCell,
        span: SourceSpan,
    },
    Local {
        name: String,
        depth: usize,
        offset: usize,
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

/// Compact rendering of a converted (typed) expression for the verbose
/// analysis trace (main.w:528-540 `Converted expression:`). Mirrors the
/// upstream `expression_base` prints for the node shapes the trace needs:
/// denotations print their value, identifiers their name, and calls print
/// `name(arg1,arg2)` (call_expression::print, axis.w:1912-1924). Other
/// shapes are rendered structurally where cheap; the fallback keeps the
/// trace parseable without claiming oracle fidelity for unverified nodes.
fn compact_typed_expression(expression: &TypedExpr) -> String {
    match expression {
        TypedExpr::Denotation(value) => value.to_string(),
        TypedExpr::GlobalIdent { name, .. } | TypedExpr::LocalIdent { name, .. } => name.clone(),
        TypedExpr::BuiltinCall {
            builtin, arguments, ..
        }
        | TypedExpr::HungryBuiltinCall {
            builtin, arguments, ..
        } => {
            let name = builtin_registry()[*builtin].name;
            let mut out = String::from(name);
            out.push('(');
            for (index, argument) in arguments.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&compact_typed_expression(argument));
            }
            out.push(')');
            out
        }
        TypedExpr::TupleDisplay(elements) => {
            let inner = elements
                .iter()
                .map(compact_typed_expression)
                .collect::<Vec<_>>()
                .join(",");
            format!("({inner})")
        }
        TypedExpr::ListDisplay(elements) => {
            let inner = elements
                .iter()
                .map(compact_typed_expression)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{inner}]")
        }
        TypedExpr::Conversion { inner, .. } => compact_typed_expression(inner),
        TypedExpr::FunctionCall {
            function, argument, ..
        } => format!(
            "{}({})",
            compact_typed_expression(function),
            compact_typed_expression(argument)
        ),
        TypedExpr::Sequence { first, second } => format!(
            "{};{}",
            compact_typed_expression(first),
            compact_typed_expression(second)
        ),
        _ => "<expression>".to_string(),
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
    Output {
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
    /// The session verbosity (main.w `verbosity`): `set quiet` = 0,
    /// `set verbose` = 1; the verbose analysis trace prints when this is
    /// nonzero.
    verbosity: u8,
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
                // The verbose analysis trace (main.w:495-516, 528-540):
                // `Expression before type analysis` right after parsing,
                // then `Type found` / `Converted expression` after
                // analysis and before evaluation.
                let mut events = Vec::new();
                if self.verbosity == 1 {
                    events.push(TypedCommandEvent::Output {
                        text: format!(
                            "Expression before type analysis: {}\n",
                            compact_expression(expression)
                        ),
                        span: expression.span(),
                    });
                }
                let mut type_ = Type::Undetermined;
                let typed = convert_expr(
                    expression,
                    &mut type_,
                    &Analysis::new(&self.types, &self.globals, &self.overloads),
                )?;
                if self.verbosity > 0 {
                    events.push(TypedCommandEvent::Output {
                        text: format!("Type found: {}\n", type_.display(&self.types)),
                        span: expression.span(),
                    });
                    events.push(TypedCommandEvent::Output {
                        text: format!(
                            "Converted expression: {}\n",
                            compact_typed_expression(&typed)
                        ),
                        span: expression.span(),
                    });
                }
                let value = match evaluate_command_expr(&typed, &mut self.evaluation) {
                    Ok(value) => value,
                    Err(diagnostic) => {
                        // Upstream keeps text printed before the failure
                        // (ext_kl.cpp:947); the buffer survives here and the
                        // session layer drains it ahead of the diagnostic.
                        return Err(diagnostic);
                    }
                };
                events.extend(self.drain_printed(expression.span()));
                events.push(TypedCommandEvent::Value {
                    value,
                    type_,
                    span: expression.span(),
                });
                Ok(events)
            }
            Command::SetOption { option, .. } => match option.as_str() {
                // The option identifiers are the first two entries of the
                // main hash table (parser.y:171-178): `quiet` gives 0 and
                // `verbose` gives 1; the flag persists across commands.
                "quiet" => {
                    self.verbosity = 0;
                    Ok(vec![])
                }
                "verbose" => {
                    self.verbosity = 1;
                    Ok(vec![])
                }
                other => Err(Diagnostic::new(
                    ErrorKind::Io,
                    format!("'{other}' is not something one can set"),
                    None,
                )),
            },
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
                        // Printed text survives the failure (see the
                        // expression-command branch above).
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
                // parser.y:162: any type expression may ascribe a declared
                // identifier (tuples, rows, function types included).
                let type_ = value_type.resolve_in(&self.types).map_err(|unknown| {
                    Diagnostic::new(
                        ErrorKind::Name,
                        format!("undefined type name '{}'", unknown.value),
                        Some(unknown.span),
                    )
                })?;
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
        for builtin in builtin_registry()
            .iter()
            .filter(|builtin| builtin.overload_visible)
        {
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

    /// Printer output buffered by an evaluation that then FAILED: upstream
    /// writes to `*output_stream` before the error is thrown
    /// (ext_kl.cpp:947), so the session layer drains these ahead of the
    /// diagnostic instead of dropping them.
    pub(crate) fn drain_failed_printed(
        &mut self,
        span: Option<SourceSpan>,
    ) -> Vec<TypedCommandEvent> {
        // Runtime diagnostics always carry the call span; the anonymous
        // fallback only guards against a spanless diagnostic.
        let fallback = || {
            SourceSpan::new(
                crate::diagnostic::SourceId::anonymous(),
                0,
                0,
                crate::diagnostic::SourcePosition { line: 1, column: 1 },
                crate::diagnostic::SourcePosition { line: 1, column: 1 },
            )
        };
        self.drain_printed(span.unwrap_or_else(fallback))
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
                let mut found = pattern_type(&binding.pattern);
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
                    // Printed text survives the failure (see the
                    // expression-command branch above).
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
    /// Flatten to a diagnostic once the enclosing expression kind is known;
    /// `items` is the upstream `balance_error` items name, and the variants
    /// print between braces exactly like global.w:665-671 reports them.
    fn into_diagnostic(self, analysis: &Analysis<'_>, items: &str) -> Diagnostic {
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
                        "No common type found between {items}: {{ {} }}",
                        displays.join(", ")
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
            // Prepare a tuple pattern of the right arity.  When it cannot
            // specialise to the required type the display is not yet
            // rejected: the a priori tuple may still COERCE into the
            // required type (axis.w:786-815 — the (int,int)->Split
            // conversion is the unique such entry), so components convert
            // against undetermined slots and the coercion applies directly;
            // failing that, the error is the standard found/needed wording.
            let mut pattern = Type::Tuple(vec![Type::Undetermined; elements.len()]);
            if !pattern.can_specialise(required, analysis.types) {
                let mut components = vec![Type::Undetermined; elements.len()];
                let converted = elements
                    .iter()
                    .zip(components.iter_mut())
                    .map(|(element, component)| convert_expr(element, component, analysis))
                    .collect::<Result<Vec<_>, _>>()?;
                let found = Type::tuple(components);
                if let Some(coercion) =
                    crate::coercions::coercion_between(&found, required, analysis.types)
                {
                    return Ok(TypedExpr::Conversion {
                        tag: coercion.tag,
                        inner: Box::new(TypedExpr::TupleDisplay(converted)),
                        span: *span,
                    });
                }
                return Err(type_error(
                    format!(
                        "found {} while {} was needed.",
                        found.display(analysis.types),
                        required.display(analysis.types)
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
                .map_err(|error| error.into_diagnostic(analysis, "components of list expression"))
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
            // Parenthesised single names collapse in the parser. Upstream
            // recognises their exact kind and sends them through ordinary
            // assignment, including its diagnostics and type refinement.
            if let Pattern::Name {
                name,
                name_span,
                constant: false,
                ..
            } = &assignment.pattern
            {
                return convert_simple_assignment(
                    name,
                    *name_span,
                    &assignment.value,
                    assignment.span,
                    required,
                    analysis,
                );
            }
            convert_multi_assignment(expression, assignment, required, analysis)
        }
        Expr::Assignment {
            name,
            target_span,
            value,
            span,
        } => convert_simple_assignment(name, *target_span, value, *span, required, analysis),
        Expr::Subscription {
            array,
            index,
            reversed,
            span,
        } => {
            let mut array_type = Type::Undetermined;
            let converted_array = convert_expr(array, &mut array_type, analysis)?;
            // The index converts with an undetermined a-priori type, exactly
            // like upstream (axis.w:4020-4026); only equality with `int` is
            // admitted, so a mistyped index falls to the dedicated `not_so`
            // error rather than a coercion failure.
            let mut index_type = Type::Undetermined;
            let converted_index = convert_expr(index, &mut index_type, analysis)?;
            // Upstream `subscr_base::index_kind` (axis.w:3941-3973): a row
            // subscripts to its component type, a string to a one-character
            // string; anything unsubscriptable is the analysis-time `not_so`
            // error (axis.w:4101-4105).
            let int_index = matches!(index_type, Type::Primitive(Prim::Int));
            let found = match &array_type {
                Type::Row(component) if int_index => (**component).clone(),
                Type::Primitive(Prim::String) if int_index => Type::Primitive(Prim::String),
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
            // Only row slicing is implemented; anything else is the
            // analysis-time error upstream raises from the `make_slice`
            // default case (axis.w:4171-4173).
            let Type::Row(component) = array_type else {
                return Err(type_error(
                    format!(
                        "Cannot slice value of type {}",
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
                    source: compact_expression(expression),
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
                    let mut binding_type = pattern_type(&binding.pattern);
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
                    // The pattern supplies the RHS context first. Omitted
                    // slots stay open, while an explicit `()` is void.
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
        .map_err(|error| error.into_diagnostic(analysis, "branches of conditional")),
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
                        // Upstream validates a discrimination pattern's
                        // structure against the selected variant before
                        // creating its binding layer. This has dedicated
                        // wording rather than `bind_pattern`'s generic type
                        // mismatch diagnostic (axis.w:5229-5235).
                        let mut required_pattern = pattern_type(pattern);
                        if !required_pattern.specialise(&payload, analysis.types) {
                            return Err(type_error(
                                format!(
                                    "Pattern {} does not match type {} for variant {}",
                                    compact_pattern(pattern),
                                    payload.display(analysis.types),
                                    tag.value
                                ),
                                pattern.span(),
                            ));
                        }
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
                .map_err(|error| error.into_diagnostic(analysis, "branches of case"))?
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

/// Convert the ordinary one-name assignment path. `set x := value` and
/// `set (x) := value` deliberately share this with bare `x := value`.
fn convert_simple_assignment(
    name: &str,
    target_span: SourceSpan,
    value: &Expr,
    span: SourceSpan,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    if let Some((target, depth, offset)) = analysis.locals.get(name) {
        if analysis.constant_locals.contains(name) {
            return Err(Diagnostic::new(
                ErrorKind::Name,
                format!(
                    "Name '{name}' is constant in assignment {name}:={}",
                    compact_expression(value)
                ),
                Some(target_span),
            ));
        }
        let mut required_value = target.borrow().clone();
        let converted = convert_expr(value, &mut required_value, analysis)?;
        *target.borrow_mut() = required_value.clone();
        let converted = prepare_hungry_assignment(converted, name, Some((*depth, *offset)), None);
        return conform_types(
            &required_value,
            required,
            TypedExpr::LocalAssignment {
                depth: *depth,
                offset: *offset,
                value: Box::new(converted),
            },
            span,
            analysis,
        );
    }
    let Some((target, cell)) = analysis.globals.lookup(name) else {
        return Err(Diagnostic::new(
            ErrorKind::Name,
            format!(
                "Undefined identifier '{name}' in assignment {name}:={}",
                compact_expression(value)
            ),
            Some(target_span),
        ));
    };
    if analysis.globals.is_const(name) {
        return Err(Diagnostic::new(
            ErrorKind::Name,
            format!(
                "Name '{name}' is constant in assignment {name}:={}",
                compact_expression(value)
            ),
            Some(target_span),
        ));
    }
    let mut required_value = target.borrow().clone();
    let converted = convert_expr(value, &mut required_value, analysis)?;
    *target.borrow_mut() = required_value.clone();
    let converted = prepare_hungry_assignment(converted, name, None, Some(cell));
    conform_types(
        &required_value,
        required,
        TypedExpr::GlobalAssignment {
            cell: cell.clone(),
            value: Box::new(converted),
        },
        span,
        analysis,
    )
}

/// Rebuild exactly the simple-assignment cases selected by upstream's
/// builtin hunger optimisation (axis.w:7165-7301). A nested call, a user
/// overload, a non-identifier operand, or an identifier other than the
/// destination is returned unchanged.
fn prepare_hungry_assignment(
    converted: TypedExpr,
    destination_name: &str,
    local: Option<(usize, usize)>,
    global: Option<&GlobalCell>,
) -> TypedExpr {
    let TypedExpr::BuiltinCall {
        builtin,
        arguments,
        span,
    } = converted
    else {
        return converted;
    };
    let hunger = builtin_registry()[builtin].hunger;
    let pilfer_index = match (hunger, arguments.len()) {
        (3, 1) => 0,
        (1, 2) => 0,
        (2, 2) => 1,
        _ => {
            return TypedExpr::BuiltinCall {
                builtin,
                arguments,
                span,
            };
        }
    };
    let pilfer = match &arguments[pilfer_index] {
        TypedExpr::LocalIdent {
            name,
            depth,
            offset,
            span,
        } if name == destination_name && local == Some((*depth, *offset)) => {
            PilferDestination::Local {
                name: name.clone(),
                depth: *depth,
                offset: *offset,
                span: *span,
            }
        }
        TypedExpr::GlobalIdent { name, cell, span }
            if name == destination_name
                && global.is_some_and(|destination| Rc::ptr_eq(destination, cell)) =>
        {
            PilferDestination::Global {
                name: name.clone(),
                cell: cell.clone(),
                span: *span,
            }
        }
        _ => {
            return TypedExpr::BuiltinCall {
                builtin,
                arguments,
                span,
            };
        }
    };
    TypedExpr::HungryBuiltinCall {
        builtin,
        arguments,
        pilfer,
        pilfer_index,
        span,
    }
}

#[derive(Clone)]
struct MultiAssignmentRefinement {
    target_type: TypeCell,
    path: Vec<usize>,
}

struct MultiAssignmentThreader<'a> {
    assignment: &'a Expr,
    span: SourceSpan,
    analysis: &'a Analysis<'a>,
    names: BTreeSet<String>,
    refinements: Vec<MultiAssignmentRefinement>,
}

impl<'a> MultiAssignmentThreader<'a> {
    fn new(assignment: &'a Expr, span: SourceSpan, analysis: &'a Analysis<'a>) -> Self {
        Self {
            assignment,
            span,
            analysis,
            names: BTreeSet::new(),
            refinements: Vec::new(),
        }
    }

    /// Analyse one pattern node in post-order. `type_` is the corresponding
    /// mutable component of the RHS type under construction; `path` records
    /// how to find that component again after RHS conversion specialises it.
    fn thread(
        &mut self,
        pattern: &Pattern,
        type_: &mut Type,
        path: &mut Vec<usize>,
    ) -> Result<MultiAssignmentPlan, Diagnostic> {
        // In the upstream bit-packed pattern a const-qualified whole name is
        // a flag on the tuple node itself, so it is rejected before visiting
        // any children. Preserve that precedence despite our split AST.
        let forbidden_constant = match pattern {
            Pattern::Name {
                name,
                constant: true,
                ..
            } => Some(name),
            Pattern::Tuple {
                whole: Some(whole), ..
            } => match whole.as_ref() {
                Pattern::Name {
                    name,
                    constant: true,
                    ..
                } => Some(name),
                _ => None,
            },
            _ => None,
        };
        if let Some(name) = forbidden_constant {
            return Err(type_error(
                format!("Cannot constant-qualify '!' identifier '{name}' in multi-assignment"),
                self.span,
            ));
        }

        match pattern {
            Pattern::Discard { .. } | Pattern::Omitted { .. } => Ok(MultiAssignmentPlan::Omitted),
            Pattern::Name { name, .. } => self.thread_name(name, type_, path),
            Pattern::Tuple {
                elements, whole, ..
            } => {
                let tuple_pattern = Type::Tuple(vec![Type::Undetermined; elements.len()]);
                let compatible = type_.specialise(&tuple_pattern, self.analysis.types);
                assert!(
                    compatible,
                    "a fresh multi-assignment tuple slot is compatible"
                );
                let element_plans = {
                    let Type::Tuple(components) = type_ else {
                        unreachable!("specialising a tuple pattern produces a tuple")
                    };
                    let mut plans = Vec::with_capacity(elements.len());
                    for (index, (element, component)) in elements.iter().zip(components).enumerate()
                    {
                        path.push(index);
                        plans.push(self.thread(element, component, path)?);
                        path.pop();
                    }
                    plans
                };
                // The whole-value name is intentionally threaded after all
                // children. Its known type therefore checks the tuple shape
                // already established by those children.
                let whole_plan = whole
                    .as_deref()
                    .map(|whole| self.thread(whole, type_, path).map(Box::new))
                    .transpose()?;
                Ok(MultiAssignmentPlan::Tuple {
                    elements: element_plans,
                    whole: whole_plan,
                })
            }
        }
    }

    fn thread_name(
        &mut self,
        name: &str,
        type_: &mut Type,
        path: &[usize],
    ) -> Result<MultiAssignmentPlan, Diagnostic> {
        if !self.names.insert(name.to_owned()) {
            return Err(type_error(
                format!("Multiple assignments to same identifier '{name}' in multi-assignment"),
                self.span,
            ));
        }

        let (target_type, type_cell, destination, is_const) =
            if let Some((target, depth, offset)) = self.analysis.locals.get(name) {
                (
                    target.borrow().clone(),
                    target.clone(),
                    MultiAssignmentDestination::Local {
                        depth: *depth,
                        offset: *offset,
                    },
                    self.analysis.constant_locals.contains(name),
                )
            } else if let Some((target, cell)) = self.analysis.globals.lookup(name) {
                (
                    target.borrow().clone(),
                    target.clone(),
                    MultiAssignmentDestination::Global(cell.clone()),
                    self.analysis.globals.is_const(name),
                )
            } else {
                return Err(Diagnostic::new(
                    ErrorKind::Name,
                    format!(
                        "Undefined identifier '{name}' in multiple assignment {}",
                        compact_expression(self.assignment)
                    ),
                    Some(self.span),
                ));
            };

        if is_const {
            return Err(Diagnostic::new(
                ErrorKind::Name,
                format!(
                    "Name '{name}' is constant in multiple assignment {}",
                    compact_expression(self.assignment)
                ),
                Some(self.span),
            ));
        }
        if !type_.specialise(&target_type, self.analysis.types) {
            return Err(type_error(
                format!(
                    "Incompatible type for '{name}' in multi-assignment: type {} does no match pattern {}",
                    target_type.display(self.analysis.types),
                    type_.display(self.analysis.types)
                ),
                self.span,
            ));
        }
        self.refinements.push(MultiAssignmentRefinement {
            target_type: type_cell,
            path: path.to_vec(),
        });
        Ok(MultiAssignmentPlan::Destination(destination))
    }
}

fn multi_assignment_component(type_: &Type, path: &[usize], types: &TypeTable) -> Type {
    let mut current = type_;
    for &index in path {
        while let Type::Tabled(number) = current {
            current = types.expansion(*number);
        }
        let Type::Tuple(components) = current else {
            unreachable!("multi-assignment type path only traverses tuple components")
        };
        current = &components[index];
    }
    current.clone()
}

fn convert_multi_assignment(
    expression: &Expr,
    assignment: &MultiAssignmentExpr,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let mut rhs_type = Type::Undetermined;
    let mut threader = MultiAssignmentThreader::new(expression, assignment.span, analysis);
    let plan = threader.thread(&assignment.pattern, &mut rhs_type, &mut Vec::new())?;
    let converted = convert_expr(&assignment.value, &mut rhs_type, analysis)?;

    // RHS conversion can fill holes left by omitted slots or polymorphic
    // targets. Only after that conversion succeeds do target TypeCells learn
    // their refined types.
    for refinement in threader.refinements {
        let component = multi_assignment_component(&rhs_type, &refinement.path, analysis.types);
        // The RHS can itself assign to the same target and specialise its
        // live TypeCell incompatibly before this refinement runs. Upstream
        // `threader::refine` deliberately ignores `specialise`'s result, so
        // retain the RHS side effect's type instead of treating that legal
        // source program as an internal invariant violation.
        let _ = refinement
            .target_type
            .borrow_mut()
            .specialise(&component, analysis.types);
    }

    conform_types(
        &rhs_type,
        required,
        TypedExpr::MultiAssignment {
            plan,
            value: Box::new(converted),
        },
        assignment.span,
        analysis,
    )
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
        Pattern::Discard { .. } | Pattern::Omitted { .. } => SlotShape::Discard,
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
        Pattern::Discard { .. } | Pattern::Omitted { .. } | Pattern::Name { .. } => {
            Type::Undetermined
        }
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
        Pattern::Discard { .. } | Pattern::Omitted { .. } => Ok(Vec::new()),
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
    let hidden_special = hidden_special_builtin(name);
    if resolve_name_first && variants.is_empty() && hidden_special.is_none() {
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
    // `resolve_overload` first honours an exact ordinary overload, then
    // recognises the hidden generic row `#`, and only afterwards considers
    // coercible ordinary overloads (axis.w:2458-2550).
    let exact = variants
        .iter()
        .position(|variant| variant.arg_type == a_priori_type);
    let use_hidden =
        exact.is_none() && hidden_special.is_some() && matches!(&a_priori_type, Type::Row(_));
    let inexact = if exact.is_none() && !use_hidden {
        variants.iter().position(|variant| {
            crate::coercions::is_close(&a_priori_type, &variant.arg_type, analysis.types) & 0x1 != 0
        })
    } else {
        None
    };
    let position = exact.or(inexact);
    if position.is_none() && !use_hidden {
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
        return Err(type_error(message, span));
    }
    let hidden_variant;
    let variant = if use_hidden {
        let index = hidden_special.expect("hidden special was checked above");
        let builtin = &builtin_registry()[index];
        hidden_variant = MergedVariant {
            arg_type: builtin.arg_type.clone(),
            result_type: builtin.result.clone(),
            origin: OverloadOrigin::Builtin(index),
        };
        &hidden_variant
    } else {
        &variants[position.expect("ordinary overload was found")]
    };
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
    overload_visible: bool,
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

#[derive(Clone, Copy, PartialEq, Eq)]
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
    VecAdd,
    VecNegate,
    VecDivideInt,
    RatAdd,
    RatSubtract,
    RatMultiply,
    RatDivide,
    RatModulo,
    RatNegate,
    RatInverse,
    RatPower,
    NullMatrix,
    RatFloor,
    RatCeil,
    RatFrac,
    StringListConcat,
    StringToAscii,
    AsciiChar,
    SizeOf,
    MatrixShape,
    MatrixRow,
    MatrixColumn,
    MatrixRows,
    MatrixColumns,
    UnaryRelation(Relation),
    BinaryRelation(Relation),
    ListCardinality,
    StringConcat,
    IntSuccessor,
    IntPredecessor,
    IntAnd,
    IntOr,
    IntXor,
    IntAndNot,
    IntBitwiseSubset,
    IntNthSetBit,
    IntBitLength,
    VecToBitset,
    VecJoin,
    VecRowJoin,
    VecSuffix,
    VecPrefix,
    VecSubtract,
    VecMultiplyInt,
    VecQuotientInt,
    VecModuloInt,
    RatvecUnfraction,
    RatvecAdd,
    RatvecSubtract,
    RatvecNegate,
    RatvecMultiplyInt,
    RatvecDivideInt,
    RatvecModuloInt,
    RatvecMultiplyRat,
    RatvecDivideRat,
    MatAddInt,
    MatSubtractInt,
    IntAddMat,
    IntSubtractMat,
    VecDot,
    FlexAdd,
    FlexSub,
    VecConvolve,
    MatAdd,
    MatSubtract,
    MatMulRatVec,
    MatMulVec,
    MatMulMat,
    VecMulMat,
    RatVecMulMat,
    NullVector,
    VecTranspose,
    MatTranspose,
    IdMat,
    Diagonal,
    StackRows,
    CombineColumns,
    CombineRows,
    VectorGcd,
    ElapsedMs,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
                domain_builtins::call_owned_with_printed(
                    name,
                    arguments,
                    span,
                    context.printed_buffer(),
                )
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
        overload_visible: true,
        implementation: BuiltinImpl::Scalar(op),
    }
}

fn hidden_scalar_builtin(
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
        overload_visible: false,
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
        overload_visible: true,
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
        overload_visible: true,
        implementation: BuiltinImpl::DomainPrinter { name },
    }
}

fn domain_relation_builtin(name: &'static str, arg_type: Type, relation: Relation) -> Builtin {
    Builtin {
        name,
        arg_type,
        result: bool_type(),
        hunger: 0,
        overload_visible: true,
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

fn unsigned_long(value: &BigInt, span: SourceSpan) -> Result<u64, Control> {
    if value < &BigInt::from(0) {
        return Err(runtime("Negative integer where unsigned is required", span));
    }
    u64::try_from(value).map_err(|_| runtime("Integer value to big for conversion", span))
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

/// `int_val()` narrowing (bigint.cpp:142-162): the machine-`int` payload or
/// the exact upstream diagnostic (typo included).
fn plain_int(value: &BigInt, span: SourceSpan) -> Result<i32, Control> {
    i32::try_from(value).map_err(|_| runtime("Integer value to big for conversion", span))
}

/// `long_val()` narrowing (bigint.cpp:142-162): the machine-`long` payload.
fn long_int(value: &BigInt, span: SourceSpan) -> Result<i64, Control> {
    i64::try_from(value).map_err(|_| runtime("Integer value to big for conversion", span))
}

/// The shared `check_size` diagnostic (global.w:3874-3880).
fn size_mismatch(left: usize, right: usize, span: SourceSpan) -> Control {
    runtime(format!("Size mismatch {left}:{right}"), span)
}

fn gcd_u64(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

/// Vector `\`/`%` entry arithmetic (global.w:3917-3937): unlike the int
/// operators, the remainder is always taken in `[0,|m|)` and the quotient
/// follows exactly (oracle: `[7] % -3 == [1]`, `[7] \ -3 == [-2]`).
fn vec_divmod_entry(entry: i32, divisor: i32) -> (i32, i32) {
    let entry = i64::from(entry);
    let divisor = i64::from(divisor);
    let remainder = entry.rem_euclid(divisor.abs());
    let quotient = ((entry - remainder) / divisor) as i32;
    (quotient, remainder as i32)
}

/// The `n`th set bit (0-based) of the two's-complement bit string of `value`
/// (bigint.cpp `index_of_set_bit`), or -1 when a non-negative `value` has
/// fewer than `n+1` set bits. Negative values have infinitely many set bits,
/// so their `n`th set bit always exists; it is found by walking the finitely
/// many set-bit positions of the complement (the cleared positions).
fn index_of_set_bit(value: &BigInt, mut n: u64) -> BigInt {
    if *value >= 0 {
        for (limb_index, &limb) in value.unsigned_abs_ref().to_limbs_asc().iter().enumerate() {
            let count = u64::from(limb.count_ones());
            if n >= count {
                n -= count;
                continue;
            }
            let mut rest = limb;
            loop {
                let position = u64::from(rest.trailing_zeros());
                if n == 0 {
                    return BigInt::from(limb_index as u64 * 64 + position);
                }
                rest &= rest - 1;
                n -= 1;
            }
        }
        return BigInt::from(-1);
    }
    let mut next = 0u64; // next candidate position, all below it resolved
    for (limb_index, &limb) in (!value)
        .unsigned_abs_ref()
        .to_limbs_asc()
        .iter()
        .enumerate()
    {
        let mut rest = limb;
        while rest != 0 {
            let cleared = limb_index as u64 * 64 + u64::from(rest.trailing_zeros());
            rest &= rest - 1;
            let gap = cleared - next; // set-bit positions of `value` in [next, cleared)
            if n < gap {
                return BigInt::from(next + n);
            }
            n -= gap;
            next = cleared + 1;
        }
    }
    BigInt::from(next + n)
}

/// `flex_add`/`flex_sub` (global.w:3953-4048): size-adaptive polynomial
/// arithmetic; trailing zeros are trimmed from both arguments, and from the
/// result only in the equal-trimmed-size case.
fn flex_add_sub(left: &[i32], right: &[i32], subtract: bool) -> Vec<i32> {
    let trimmed = |vector: &[i32]| {
        vector
            .iter()
            .rposition(|&entry| entry != 0)
            .map_or(0, |p| p + 1)
    };
    let (left_size, right_size) = (trimmed(left), trimmed(right));
    let size = left_size.max(right_size);
    let mut result = vec![0i32; size];
    for (index, &entry) in left[..left_size].iter().enumerate() {
        result[index] = result[index].wrapping_add(entry);
    }
    for (index, &entry) in right[..right_size].iter().enumerate() {
        result[index] = if subtract {
            result[index].wrapping_sub(entry)
        } else {
            result[index].wrapping_add(entry)
        };
    }
    if left_size == right_size {
        result.truncate(trimmed(&result));
    }
    result
}

/// `convolve` (global.w:4056-4085): polynomial multiplication of the
/// zero-trimmed arguments; empty when either trimmed argument is empty.
fn convolve(left: &[i32], right: &[i32]) -> Vec<i32> {
    let trimmed = |vector: &[i32]| {
        vector
            .iter()
            .rposition(|&entry| entry != 0)
            .map_or(0, |p| p + 1)
    };
    let (left, right) = (&left[..trimmed(left)], &right[..trimmed(right)]);
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }
    let mut result = vec![0i32; left.len() + right.len() - 1];
    for (index, &entry) in left.iter().enumerate() {
        result[index] = entry.wrapping_mul(right[0]);
    }
    for (shift, &entry) in right.iter().enumerate().skip(1) {
        result[left.len() - 1 + shift] = left[left.len() - 1].wrapping_mul(entry);
        for (index, &base) in left[..left.len() - 1].iter().enumerate() {
            result[index + shift] = result[index + shift].wrapping_add(base.wrapping_mul(entry));
        }
    }
    result
}

/// `ratvec + ratvec`/`ratvec - ratvec` (global.w:4127-4139): cross-multiply
/// over the least common denominator; `RatVec::new` normalises the sum.
fn ratvec_add_sub(left: &RatVec, right: &RatVec, subtract: bool) -> RatVec {
    let (left_den, right_den) = (left.denominator(), right.denominator());
    let divisor = gcd_u64(left_den, right_den);
    let (left_scale, right_scale) = (right_den / divisor, left_den / divisor);
    let numerators = left
        .numerators()
        .iter()
        .zip(right.numerators())
        .map(|(&a, &b)| {
            let b = if subtract {
                -i128::from(b)
            } else {
                i128::from(b)
            };
            (i128::from(a) * i128::from(left_scale) + b * i128::from(right_scale)) as i64
        })
        .collect();
    let denominator = (u128::from(left_den) * u128::from(left_scale)) as u64;
    RatVec::new(numerators, denominator).expect("ratvec sum keeps a nonzero denominator")
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
        ScalarOp::VecAdd => match expect_pair(arguments) {
            (Value::Vector(left), Value::Vector(right)) => {
                if left.0.len() != right.0.len() {
                    return Err(runtime(
                        format!("Size mismatch {}:{}", left.0.len(), right.0.len()),
                        span,
                    ));
                }
                Ok(at_builtin_level(level, || {
                    Value::Vector(Vec32(
                        left.0
                            .into_iter()
                            .zip(right.0)
                            .map(|(a, b)| a.wrapping_add(b))
                            .collect(),
                    ))
                }))
            }
            other => panic!("vector addition saw {other:?}"),
        },
        ScalarOp::VecNegate => match expect_unary(arguments) {
            Value::Vector(vector) => Ok(at_builtin_level(level, || {
                Value::Vector(Vec32(vector.0.into_iter().map(i32::wrapping_neg).collect()))
            })),
            other => panic!("vector negation saw {other:?}"),
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
        ScalarOp::NullMatrix => {
            let (row_value, column_value) = expect_ints(arguments);
            // Atlas pops/converts the column count first, then the row count.
            let columns = unsigned_long(&column_value, span)?;
            let rows = unsigned_long(&row_value, span)?;
            if rows > u64::from(u32::MAX) {
                return Err(runtime(
                    format!("Number of rows {rows} exceeds implementation limit"),
                    span,
                ));
            }
            if columns > u64::from(u32::MAX) {
                return Err(runtime(
                    format!("Number of columns {columns} exceeds implementation limit"),
                    span,
                ));
            }
            if level == Level::NoValue {
                return Ok(None);
            }
            let rows = usize::try_from(rows).expect("u32 matrix row count fits usize");
            let columns = usize::try_from(columns).expect("u32 matrix column count fits usize");
            let cells = rows
                .checked_mul(columns)
                .ok_or_else(|| runtime("Matrix dimensions exceed available memory", span))?;
            let mut data = Vec::new();
            data.try_reserve_exact(cells)
                .map_err(|_| runtime("Matrix dimensions exceed available memory", span))?;
            data.resize(cells, 0);
            Ok(Some(Value::Matrix(
                Matrix::from_columns(rows, columns, data)
                    .expect("allocated zero data matches matrix dimensions"),
            )))
        }
        ScalarOp::UnaryRelation(relation) => {
            let value = expect_unary(arguments);
            Ok(at_builtin_level(level, || {
                // Containers compare against an implicit zero of the same
                // kind: only =/!=/>=/> are registered for vec and ratvec,
                // only =/!= for mat (global.w:4405-4420).
                let result = match &value {
                    Value::Integer(value) => {
                        relation_matches(relation, value.cmp(&BigInt::from(0)))
                    }
                    Value::Rational(value) => {
                        relation_matches(relation, value.cmp(&BigRational::from(0)))
                    }
                    Value::String(value) => relation_matches(relation, value.as_str().cmp("")),
                    Value::Vector(vector) => match relation {
                        Relation::Equal => vector.0.iter().all(|&entry| entry == 0),
                        Relation::NotEqual => vector.0.iter().any(|&entry| entry != 0),
                        Relation::GreaterEqual => vector.0.iter().all(|&entry| entry >= 0),
                        Relation::Greater => vector.0.iter().all(|&entry| entry > 0),
                        _ => panic!("ordered vector relation is not registered"),
                    },
                    Value::RatVector(ratvec) => match relation {
                        Relation::Equal => ratvec.numerators().iter().all(|&entry| entry == 0),
                        Relation::NotEqual => ratvec.numerators().iter().any(|&entry| entry != 0),
                        Relation::GreaterEqual => {
                            ratvec.numerators().iter().all(|&entry| entry >= 0)
                        }
                        Relation::Greater => ratvec.numerators().iter().all(|&entry| entry > 0),
                        _ => panic!("ordered ratvec relation is not registered"),
                    },
                    Value::Matrix(matrix) => match relation {
                        Relation::Equal => matrix.is_zero(),
                        Relation::NotEqual => !matrix.is_zero(),
                        _ => panic!("ordered matrix relation is not registered"),
                    },
                    other => panic!("unary relation saw {other:?}"),
                };
                Value::Boolean(result)
            }))
        }
        ScalarOp::BinaryRelation(relation) => {
            let (first, second) = expect_pair(arguments);
            Ok(at_builtin_level(level, || {
                let result = match (first, second) {
                    (Value::Integer(first), Value::Integer(second)) => {
                        relation_matches(relation, first.cmp(&second))
                    }
                    (Value::Rational(first), Value::Rational(second)) => {
                        relation_matches(relation, first.cmp(&second))
                    }
                    (Value::Boolean(first), Value::Boolean(second)) => {
                        relation_matches(relation, first.cmp(&second))
                    }
                    (Value::String(first), Value::String(second)) => {
                        relation_matches(relation, first.cmp(&second))
                    }
                    // Only =/!= are registered for containers.
                    (Value::Vector(first), Value::Vector(second)) => {
                        container_equality(relation, first == second)
                    }
                    (Value::RatVector(first), Value::RatVector(second)) => {
                        container_equality(relation, first == second)
                    }
                    (Value::Matrix(first), Value::Matrix(second)) => {
                        container_equality(relation, first == second)
                    }
                    other => panic!("binary relation saw {other:?}"),
                };
                Value::Boolean(result)
            }))
        }
        ScalarOp::ListCardinality => match expect_unary(arguments) {
            Value::List(values) => Ok(at_builtin_level(level, || {
                Value::Integer(BigInt::from(values.len()))
            })),
            other => panic!("list cardinality saw {other:?}"),
        },
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
        // rat_floor/ceil/frac (global.w:3153-3167): floor division rounding,
        // the fractional part lying in [0,1).
        ScalarOp::RatFloor | ScalarOp::RatCeil | ScalarOp::RatFrac => {
            match expect_unary(arguments) {
                Value::Rational(value) => Ok(at_builtin_level(level, || match operation {
                    ScalarOp::RatFloor => Value::Integer(value.floor()),
                    ScalarOp::RatCeil => Value::Integer(value.ceiling()),
                    ScalarOp::RatFrac => {
                        let floor = (&value).floor();
                        Value::Rational(value - BigRational::from(floor))
                    }
                    _ => unreachable!(),
                })),
                other => panic!("rational decomposer saw {other:?}"),
            }
        }
        // concatenate_strings (global.w:3492-3508): fold a row of strings.
        ScalarOp::StringListConcat => match expect_unary(arguments) {
            Value::List(values) => Ok(at_builtin_level(level, || {
                let mut joined = String::new();
                for value in values {
                    match value {
                        Value::String(text) => joined.push_str(&text),
                        other => panic!("string list concatenation saw {other:?}"),
                    }
                }
                Value::String(joined)
            })),
            other => panic!("string list concatenation saw {other:?}"),
        },
        // string_to_ascii (global.w:3516-3521): first byte unsigned, -1 when
        // the string is empty.
        ScalarOp::StringToAscii => match expect_unary(arguments) {
            Value::String(value) => Ok(at_builtin_level(level, || {
                Value::Integer(match value.as_bytes().first() {
                    None => BigInt::from(-1),
                    Some(&byte) => BigInt::from(byte),
                })
            })),
            other => panic!("string to ascii saw {other:?}"),
        },
        // ascii_char (global.w:3523-3532): int_val narrowing, then the
        // printable-or-newline gate, both before the no-value gate.
        ScalarOp::AsciiChar => match expect_unary(arguments) {
            Value::Integer(value) => {
                let code = i32::try_from(&value)
                    .map_err(|_| runtime("Integer value to big for conversion", span))?;
                if (code < i32::from(b' ') && code != i32::from(b'\n')) || code > i32::from(b'~') {
                    return Err(runtime(
                        format!("Value {code} out of printable ASCII range"),
                        span,
                    ));
                }
                Ok(at_builtin_level(level, || {
                    Value::String(String::from(char::from(
                        u8::try_from(code).expect("printable ASCII fits u8"),
                    )))
                }))
            }
            other => panic!("ascii char saw {other:?}"),
        },
        // sizeof_string/vector/ratvec and matrix_ncols (global.w:3578-3601):
        // byte count, lengths, and the column count respectively.
        ScalarOp::SizeOf => {
            let size = match expect_unary(arguments) {
                Value::String(value) => value.len(),
                Value::Vector(vector) => vector.0.len(),
                Value::RatVector(ratvec) => ratvec.numerators().len(),
                Value::Matrix(matrix) => matrix.cols(),
                other => panic!("size-of saw {other:?}"),
            };
            Ok(at_builtin_level(level, || {
                Value::Integer(BigInt::from(size))
            }))
        }
        // matrix_shape (global.w:3610-3618): both bounds as an (int,int).
        ScalarOp::MatrixShape => match expect_unary(arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                Value::Tuple(vec![
                    Value::Integer(BigInt::from(matrix.rows())),
                    Value::Integer(BigInt::from(matrix.cols())),
                ])
            })),
            other => panic!("matrix shape saw {other:?}"),
        },
        // matrix_row/column (global.w:3626-3648): the index narrows through
        // ulong_val and bounds-checks before the no-value gate.
        ScalarOp::MatrixRow | ScalarOp::MatrixColumn => {
            let (matrix, index) = match expect_pair(arguments) {
                (Value::Matrix(matrix), Value::Integer(index)) => (matrix, index),
                other => panic!("matrix row/column saw {other:?}"),
            };
            let index = unsigned_long(&index, span)?;
            let (limit, what) = match operation {
                ScalarOp::MatrixRow => (matrix.rows(), "row"),
                ScalarOp::MatrixColumn => (matrix.cols(), "column"),
                _ => unreachable!(),
            };
            if index >= limit as u64 {
                return Err(runtime(
                    format!("{what} index {index} out of range (0<= . <{limit})"),
                    span,
                ));
            }
            let index = usize::try_from(index).expect("bounded by the matrix dimension");
            Ok(at_builtin_level(level, || {
                Value::Vector(match operation {
                    ScalarOp::MatrixRow => matrix.row(index),
                    ScalarOp::MatrixColumn => matrix.column(index),
                    _ => unreachable!(),
                })
            }))
        }
        // rows/columns (global.w:2432-2449): externalise to a row of vecs.
        ScalarOp::MatrixRows | ScalarOp::MatrixColumns => match expect_unary(arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                let count = match operation {
                    ScalarOp::MatrixRows => matrix.rows(),
                    ScalarOp::MatrixColumns => matrix.cols(),
                    _ => unreachable!(),
                };
                Value::List(
                    (0..count)
                        .map(|index| {
                            Value::Vector(match operation {
                                ScalarOp::MatrixRows => matrix.row(index),
                                ScalarOp::MatrixColumns => matrix.column(index),
                                _ => unreachable!(),
                            })
                        })
                        .collect(),
                )
            })),
            other => panic!("matrix rows/columns saw {other:?}"),
        },
        // succ/pred (global.w:2761-2773): upstream's parse-time rewrite
        // turns x+1/x-1 into these; as builtins they are plain increments.
        ScalarOp::IntSuccessor | ScalarOp::IntPredecessor => match expect_unary(arguments) {
            Value::Integer(value) => Ok(at_builtin_level(level, || {
                Value::Integer(match operation {
                    ScalarOp::IntSuccessor => value + BigInt::from(1),
                    ScalarOp::IntPredecessor => value - BigInt::from(1),
                    _ => unreachable!(),
                })
            })),
            other => panic!("integer successor/predecessor saw {other:?}"),
        },
        // AND/OR/XOR/AND_NOT (global.w:2817-2849): two's-complement bit
        // strings; call syntax only (AND(6,3)), they are not operators.
        ScalarOp::IntAnd | ScalarOp::IntOr | ScalarOp::IntXor | ScalarOp::IntAndNot => {
            let (first, second) = expect_ints(arguments);
            Ok(at_builtin_level(level, || {
                Value::Integer(match operation {
                    ScalarOp::IntAnd => first & second,
                    ScalarOp::IntOr => first | second,
                    ScalarOp::IntXor => first ^ second,
                    ScalarOp::IntAndNot => first & !second,
                    _ => unreachable!(),
                })
            }))
        }
        // bitwise_subset (global.w:2852-2857): the set bits of the left
        // operand all occur in the right operand.
        ScalarOp::IntBitwiseSubset => {
            let (first, second) = expect_ints(arguments);
            Ok(at_builtin_level(level, || {
                Value::Boolean(&first & &second == first)
            }))
        }
        // nth_set_bit (global.w:2859-2870): the index narrows through
        // long_val before the no-value gate; a negative index counts cleared
        // bits of the operand instead.
        ScalarOp::IntNthSetBit => {
            let (value, index) = expect_ints(arguments);
            let index = long_int(&index, span)?;
            Ok(at_builtin_level(level, || {
                Value::Integer(if index >= 0 {
                    index_of_set_bit(&value, index as u64)
                } else {
                    index_of_set_bit(&!&value, index.wrapping_neg().wrapping_sub(1) as u64)
                })
            }))
        }
        // bit_length (global.w:2872-2877): significant bits for n>=0; for
        // n<0 the negated two's-complement size, -(bits(~n)+1).
        ScalarOp::IntBitLength => match expect_unary(arguments) {
            Value::Integer(value) => Ok(at_builtin_level(level, || {
                Value::Integer(if value >= 0 {
                    BigInt::from((&value).significant_bits())
                } else {
                    -BigInt::from((&!&value).significant_bits() + 1)
                })
            })),
            other => panic!("bit_length saw {other:?}"),
        },
        // to_bitset (global.w:2887-2899): the negative-entry scan runs
        // before the no-value gate.
        ScalarOp::VecToBitset => match expect_unary(arguments) {
            Value::Vector(vector) => {
                for &entry in &vector.0 {
                    if entry < 0 {
                        return Err(runtime("Negative entry in conversion to bitset", span));
                    }
                }
                Ok(at_builtin_level(level, || {
                    let mut bits = BigInt::from(0);
                    for &entry in &vector.0 {
                        bits += BigInt::from(1) << (entry as u64);
                    }
                    Value::Integer(bits)
                }))
            }
            other => panic!("to_bitset saw {other:?}"),
        },
        // join_vectors / join_vector_row (global.w:3675-3706): plain
        // concatenation, pairwise or of a row of vecs.
        ScalarOp::VecJoin => match expect_pair(arguments) {
            (Value::Vector(left), Value::Vector(right)) => Ok(at_builtin_level(level, || {
                let mut joined = left.0;
                joined.extend(right.0);
                Value::Vector(Vec32(joined))
            })),
            other => panic!("vector join saw {other:?}"),
        },
        ScalarOp::VecRowJoin => match expect_unary(arguments) {
            Value::List(parts) => Ok(at_builtin_level(level, || {
                let mut joined = Vec::new();
                for part in parts {
                    match part {
                        Value::Vector(vector) => joined.extend(vector.0),
                        other => panic!("vector row join saw {other:?}"),
                    }
                }
                Value::Vector(Vec32(joined))
            })),
            other => panic!("vector row join saw {other:?}"),
        },
        // vector suffix/prefix (global.w:3657-3673): the element narrows
        // through int_val before the gate.
        ScalarOp::VecSuffix | ScalarOp::VecPrefix => {
            let (first, second) = expect_pair(arguments);
            let (vector, element) = match operation {
                ScalarOp::VecSuffix => match (first, second) {
                    (Value::Vector(vector), Value::Integer(element)) => (vector, element),
                    (first, second) => {
                        panic!("vector suffix saw {first:?} and {second:?}")
                    }
                },
                ScalarOp::VecPrefix => match (first, second) {
                    (Value::Integer(element), Value::Vector(vector)) => (vector, element),
                    (first, second) => {
                        panic!("vector prefix saw {first:?} and {second:?}")
                    }
                },
                _ => unreachable!(),
            };
            let element = plain_int(&element, span)?;
            Ok(at_builtin_level(level, || {
                let mut entries = vector.0;
                match operation {
                    ScalarOp::VecSuffix => entries.push(element),
                    ScalarOp::VecPrefix => entries.insert(0, element),
                    _ => unreachable!(),
                }
                Value::Vector(Vec32(entries))
            }))
        }
        // vec-vec subtraction (global.w:3891-3899); the check_size
        // diagnostic names left:right sizes and fires before the gate.
        ScalarOp::VecSubtract => match expect_pair(arguments) {
            (Value::Vector(left), Value::Vector(right)) => {
                if left.0.len() != right.0.len() {
                    return Err(size_mismatch(left.0.len(), right.0.len(), span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Vector(Vec32(
                        left.0
                            .into_iter()
                            .zip(right.0)
                            .map(|(a, b)| a.wrapping_sub(b))
                            .collect(),
                    ))
                }))
            }
            other => panic!("vector subtraction saw {other:?}"),
        },
        // vec*int (global.w:3909-3915): int_val narrowing before the gate.
        ScalarOp::VecMultiplyInt => match expect_pair(arguments) {
            (Value::Vector(vector), Value::Integer(factor)) => {
                let factor = plain_int(&factor, span)?;
                Ok(at_builtin_level(level, || {
                    Value::Vector(Vec32(
                        vector
                            .0
                            .into_iter()
                            .map(|entry| entry.wrapping_mul(factor))
                            .collect(),
                    ))
                }))
            }
            other => panic!("vector scaling saw {other:?}"),
        },
        // vec\int and vec%int (global.w:3917-3937): narrowing and the zero
        // divisor diagnostic fire before the gate; the remainder is always
        // non-negative (see vec_divmod_entry).
        ScalarOp::VecQuotientInt | ScalarOp::VecModuloInt => match expect_pair(arguments) {
            (Value::Vector(vector), Value::Integer(divisor)) => {
                let divisor = plain_int(&divisor, span)?;
                if divisor == 0 {
                    let message = match operation {
                        ScalarOp::VecQuotientInt => "Vector division by 0",
                        ScalarOp::VecModuloInt => "Vector modulo 0",
                        _ => unreachable!(),
                    };
                    return Err(runtime(message, span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Vector(Vec32(
                        vector
                            .0
                            .into_iter()
                            .map(|entry| {
                                let (quotient, remainder) = vec_divmod_entry(entry, divisor);
                                match operation {
                                    ScalarOp::VecQuotientInt => quotient,
                                    ScalarOp::VecModuloInt => remainder,
                                    _ => unreachable!(),
                                }
                            })
                            .collect(),
                    ))
                }))
            }
            other => panic!("vector division/modulo saw {other:?}"),
        },
        // ratvec unfraction (global.w:4119-4125): numerators narrow from
        // machine long to int, wrapping as upstream's iterator copy does.
        ScalarOp::RatvecUnfraction => match expect_unary(arguments) {
            Value::RatVector(ratvec) => Ok(at_builtin_level(level, || {
                Value::Tuple(vec![
                    Value::Vector(Vec32(
                        ratvec.numerators().iter().map(|&n| n as i32).collect(),
                    )),
                    Value::Integer(BigInt::from(ratvec.denominator())),
                ])
            })),
            other => panic!("ratvec unfraction saw {other:?}"),
        },
        // ratvec+ratvec/ratvec-ratvec (global.w:4127-4139): check_size
        // fires before the gate.
        ScalarOp::RatvecAdd | ScalarOp::RatvecSubtract => match expect_pair(arguments) {
            (Value::RatVector(left), Value::RatVector(right)) => {
                if left.numerators().len() != right.numerators().len() {
                    return Err(size_mismatch(
                        left.numerators().len(),
                        right.numerators().len(),
                        span,
                    ));
                }
                Ok(at_builtin_level(level, || {
                    Value::RatVector(ratvec_add_sub(
                        &left,
                        &right,
                        operation == ScalarOp::RatvecSubtract,
                    ))
                }))
            }
            other => panic!("ratvec addition/subtraction saw {other:?}"),
        },
        ScalarOp::RatvecNegate => match expect_unary(arguments) {
            Value::RatVector(ratvec) => Ok(at_builtin_level(level, || {
                Value::RatVector(
                    RatVec::new(
                        ratvec
                            .numerators()
                            .iter()
                            .map(|&n| n.wrapping_neg())
                            .collect(),
                        ratvec.denominator(),
                    )
                    .expect("negation keeps a nonzero denominator"),
                )
            })),
            other => panic!("ratvec negation saw {other:?}"),
        },
        // ratvec *int, /int, %int (global.w:4154-4180): long_val narrowing
        // and the zero-divisor diagnostics fire before the gate; every
        // operation re-normalises the result.
        ScalarOp::RatvecMultiplyInt | ScalarOp::RatvecDivideInt | ScalarOp::RatvecModuloInt => {
            match expect_pair(arguments) {
                (Value::RatVector(ratvec), Value::Integer(factor)) => {
                    let factor = long_int(&factor, span)?;
                    if factor == 0 {
                        match operation {
                            ScalarOp::RatvecDivideInt => {
                                return Err(runtime("Rational vector division by 0", span));
                            }
                            ScalarOp::RatvecModuloInt => {
                                return Err(runtime("Rational vector modulo 0", span));
                            }
                            _ => {}
                        }
                    }
                    Ok(at_builtin_level(level, || {
                        Value::RatVector(match operation {
                            ScalarOp::RatvecMultiplyInt => RatVec::new(
                                ratvec
                                    .numerators()
                                    .iter()
                                    .map(|&n| n.wrapping_mul(factor))
                                    .collect(),
                                ratvec.denominator(),
                            )
                            .expect("scaling keeps a nonzero denominator"),
                            ScalarOp::RatvecDivideInt => {
                                let negative = factor < 0;
                                RatVec::new(
                                    ratvec
                                        .numerators()
                                        .iter()
                                        .map(|&n| if negative { n.wrapping_neg() } else { n })
                                        .collect(),
                                    ratvec.denominator().wrapping_mul(factor.unsigned_abs()),
                                )
                                .expect("division keeps a nonzero denominator")
                            }
                            ScalarOp::RatvecModuloInt => {
                                let modulus = i128::from(ratvec.denominator())
                                    * i128::from(factor.unsigned_abs());
                                RatVec::new(
                                    ratvec
                                        .numerators()
                                        .iter()
                                        .map(|&n| (i128::from(n).rem_euclid(modulus)) as i64)
                                        .collect(),
                                    ratvec.denominator(),
                                )
                                .expect("modulo keeps a nonzero denominator")
                            }
                            _ => unreachable!(),
                        })
                    }))
                }
                other => panic!("ratvec int arithmetic saw {other:?}"),
            }
        }
        // ratvec *rat and /rat (global.w:4183-4197): the zero-divisor
        // diagnostic fires before the gate, but the computation (including
        // its narrowing) happens inside it, as upstream does.
        ScalarOp::RatvecMultiplyRat | ScalarOp::RatvecDivideRat => match expect_pair(arguments) {
            (Value::RatVector(ratvec), Value::Rational(factor)) => {
                if operation == ScalarOp::RatvecDivideRat && factor == 0 {
                    return Err(runtime("Rational vector division by 0", span));
                }
                if level == Level::NoValue {
                    return Ok(None);
                }
                // Malachite splits off the sign; the magnitude narrows
                // through machine long, as upstream's ratvec arithmetic.
                let negative = factor < 0;
                let (magnitude, denominator) = factor.into_numerator_and_denominator();
                let magnitude = i64::try_from(&BigInt::from(magnitude))
                    .map_err(|_| runtime("Integer value to big for conversion", span))?;
                let denominator = u64::try_from(&BigInt::from(denominator))
                    .map_err(|_| runtime("Integer value to big for conversion", span))?;
                // Dividing by p/q multiplies by q/p.
                let (scale_num, scale_den) = match operation {
                    ScalarOp::RatvecMultiplyRat => (magnitude, denominator),
                    ScalarOp::RatvecDivideRat => (
                        i64::try_from(denominator)
                            .map_err(|_| runtime("Integer value to big for conversion", span))?,
                        magnitude as u64,
                    ),
                    _ => unreachable!(),
                };
                Ok(Some(Value::RatVector(
                    RatVec::new(
                        ratvec
                            .numerators()
                            .iter()
                            .map(|&n| {
                                let n = if negative { n.wrapping_neg() } else { n };
                                n.wrapping_mul(scale_num)
                            })
                            .collect(),
                        ratvec.denominator().wrapping_mul(scale_den),
                    )
                    .expect("rational scaling keeps a nonzero denominator"),
                )))
            }
            other => panic!("ratvec rational scaling saw {other:?}"),
        },
        // mat±int and int±mat (global.w:4235-4248): int_val narrowing
        // before the gate; the integer is added to the main diagonal, and
        // upstream does not require a square matrix.
        ScalarOp::MatAddInt
        | ScalarOp::MatSubtractInt
        | ScalarOp::IntAddMat
        | ScalarOp::IntSubtractMat => {
            let (first, second) = expect_pair(arguments);
            let (matrix, value) = match (first, second) {
                (Value::Matrix(matrix), Value::Integer(value))
                    if matches!(operation, ScalarOp::MatAddInt | ScalarOp::MatSubtractInt) =>
                {
                    (matrix, value)
                }
                (Value::Integer(value), Value::Matrix(matrix))
                    if matches!(operation, ScalarOp::IntAddMat | ScalarOp::IntSubtractMat) =>
                {
                    (matrix, value)
                }
                (first, second) => {
                    panic!("matrix-int addition saw {first:?} and {second:?}")
                }
            };
            let value = plain_int(&value, span)?;
            Ok(at_builtin_level(level, || {
                Value::Matrix(match operation {
                    ScalarOp::MatAddInt | ScalarOp::IntAddMat => matrix.added_diagonal(value),
                    ScalarOp::MatSubtractInt => matrix.added_diagonal(value.wrapping_neg()),
                    ScalarOp::IntSubtractMat => matrix.negated().added_diagonal(value),
                    _ => unreachable!(),
                })
            }))
        }
        // vec*vec dot product (global.w:3938-3943): check_size first, then
        // machine-int wrapping accumulation.
        ScalarOp::VecDot => match expect_pair(arguments) {
            (Value::Vector(left), Value::Vector(right)) => {
                if left.0.len() != right.0.len() {
                    return Err(size_mismatch(left.0.len(), right.0.len(), span));
                }
                Ok(at_builtin_level(level, || {
                    let mut sum = 0i32;
                    for (&a, &b) in left.0.iter().zip(&right.0) {
                        sum = sum.wrapping_add(a.wrapping_mul(b));
                    }
                    Value::Integer(BigInt::from(sum))
                }))
            }
            other => panic!("vector dot product saw {other:?}"),
        },
        ScalarOp::FlexAdd | ScalarOp::FlexSub => match expect_pair(arguments) {
            (Value::Vector(left), Value::Vector(right)) => Ok(at_builtin_level(level, || {
                Value::Vector(Vec32(flex_add_sub(
                    &left.0,
                    &right.0,
                    operation == ScalarOp::FlexSub,
                )))
            })),
            other => panic!("flex add/sub saw {other:?}"),
        },
        ScalarOp::VecConvolve => match expect_pair(arguments) {
            (Value::Vector(left), Value::Vector(right)) => Ok(at_builtin_level(level, || {
                Value::Vector(Vec32(convolve(&left.0, &right.0)))
            })),
            other => panic!("convolve saw {other:?}"),
        },
        // mat±mat (global.w:4253-4275): the row check fires before the
        // column check, both before the gate.
        ScalarOp::MatAdd | ScalarOp::MatSubtract => match expect_pair(arguments) {
            (Value::Matrix(left), Value::Matrix(right)) => {
                if left.rows() != right.rows() {
                    return Err(size_mismatch(left.rows(), right.rows(), span));
                }
                if left.cols() != right.cols() {
                    return Err(size_mismatch(left.cols(), right.cols(), span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Matrix(match operation {
                        ScalarOp::MatAdd => left.added(&right),
                        ScalarOp::MatSubtract => left.subtracted(&right),
                        _ => unreachable!(),
                    })
                }))
            }
            other => panic!("matrix addition/subtraction saw {other:?}"),
        },
        // The matrix/vector products (global.w:4284-4342): each dimension
        // diagnostic fires before the gate, with the product's own wording
        // ("Size mismatch <inner left>:<inner right>").
        ScalarOp::MatMulVec => match expect_pair(arguments) {
            (Value::Matrix(matrix), Value::Vector(vector)) => {
                if matrix.cols() != vector.0.len() {
                    return Err(size_mismatch(matrix.cols(), vector.0.len(), span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Vector(matrix.multiplied_vec(&vector))
                }))
            }
            other => panic!("matrix-vector product saw {other:?}"),
        },
        ScalarOp::MatMulRatVec => match expect_pair(arguments) {
            (Value::Matrix(matrix), Value::RatVector(vector)) => {
                if matrix.cols() != vector.numerators().len() {
                    return Err(size_mismatch(
                        matrix.cols(),
                        vector.numerators().len(),
                        span,
                    ));
                }
                Ok(at_builtin_level(level, || {
                    Value::RatVector(matrix.multiplied_ratvec(&vector))
                }))
            }
            other => panic!("matrix-ratvec product saw {other:?}"),
        },
        ScalarOp::MatMulMat => match expect_pair(arguments) {
            (Value::Matrix(left), Value::Matrix(right)) => {
                if left.cols() != right.rows() {
                    return Err(size_mismatch(left.cols(), right.rows(), span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Matrix(left.multiplied(&right))
                }))
            }
            other => panic!("matrix product saw {other:?}"),
        },
        ScalarOp::VecMulMat => match expect_pair(arguments) {
            (Value::Vector(vector), Value::Matrix(matrix)) => {
                if vector.0.len() != matrix.rows() {
                    return Err(size_mismatch(vector.0.len(), matrix.rows(), span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Vector(matrix.left_multiplied_vec(&vector))
                }))
            }
            other => panic!("vector-matrix product saw {other:?}"),
        },
        ScalarOp::RatVecMulMat => match expect_pair(arguments) {
            (Value::RatVector(vector), Value::Matrix(matrix)) => {
                if vector.numerators().len() != matrix.rows() {
                    return Err(size_mismatch(
                        vector.numerators().len(),
                        matrix.rows(),
                        span,
                    ));
                }
                Ok(at_builtin_level(level, || {
                    Value::RatVector(matrix.left_multiplied_ratvec(&vector))
                }))
            }
            other => panic!("ratvec-matrix product saw {other:?}"),
        },
        // null(int->vec) (global.w:4471-4475): ulong_val narrowing, then
        // the gate; the allocation guard mirrors NullMatrix.
        ScalarOp::NullVector => match expect_unary(arguments) {
            Value::Integer(value) => {
                let size = unsigned_long(&value, span)?;
                if level == Level::NoValue {
                    return Ok(None);
                }
                let size = usize::try_from(size).expect("u64 vector size fits usize");
                let mut data = Vec::new();
                data.try_reserve_exact(size)
                    .map_err(|_| runtime("Vector size exceeds available memory", span))?;
                data.resize(size, 0);
                Ok(Some(Value::Vector(Vec32(data))))
            }
            other => panic!("null vector saw {other:?}"),
        },
        // ^(vec->mat) (global.w:4492-4506): a one-row matrix; the size
        // limit check fires before the gate.
        ScalarOp::VecTranspose => match expect_unary(arguments) {
            Value::Vector(vector) => {
                if vector.0.len() as u64 > u64::from(u32::MAX) {
                    return Err(runtime(
                        format!(
                            "Vector size {} exceeds matrix implementation limit",
                            vector.0.len()
                        ),
                        span,
                    ));
                }
                Ok(at_builtin_level(level, || {
                    let size = vector.0.len();
                    Value::Matrix(
                        Matrix::from_columns(1, size, vector.0)
                            .expect("one-row transpose data matches"),
                    )
                }))
            }
            other => panic!("vector transpose saw {other:?}"),
        },
        ScalarOp::MatTranspose => match expect_unary(arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                Value::Matrix(matrix.transposed())
            })),
            other => panic!("matrix transpose saw {other:?}"),
        },
        // id_mat (global.w:4518-4528): ulong_val narrowing and the size
        // limit fire before the gate.
        ScalarOp::IdMat => match expect_unary(arguments) {
            Value::Integer(value) => {
                let size = unsigned_long(&value, span)?;
                if size > u64::from(u32::MAX) {
                    return Err(runtime(
                        format!("Size {size} of identity matrix exceeds implementation limit"),
                        span,
                    ));
                }
                if level == Level::NoValue {
                    return Ok(None);
                }
                let size = usize::try_from(size).expect("u32 identity size fits usize");
                let cells = size
                    .checked_mul(size)
                    .ok_or_else(|| runtime("Matrix dimensions exceed available memory", span))?;
                let mut data = Vec::new();
                data.try_reserve_exact(cells)
                    .map_err(|_| runtime("Matrix dimensions exceed available memory", span))?;
                data.resize(cells, 0);
                for index in 0..size {
                    data[index * size + index] = 1;
                }
                Ok(Some(Value::Matrix(
                    Matrix::from_columns(size, size, data).expect("identity data matches"),
                )))
            }
            other => panic!("identity matrix saw {other:?}"),
        },
        // diagonal (global.w:4535-4548): the size limit fires before the
        // gate.
        ScalarOp::Diagonal => match expect_unary(arguments) {
            Value::Vector(vector) => {
                if vector.0.len() as u64 > u64::from(u32::MAX) {
                    return Err(runtime(
                        format!(
                            "Size {} of diagonal matrix exceeds implementation limit",
                            vector.0.len()
                        ),
                        span,
                    ));
                }
                Ok(at_builtin_level(level, || {
                    Value::Matrix(Matrix::diagonal(&vector))
                }))
            }
            other => panic!("diagonal matrix saw {other:?}"),
        },
        // stack_rows (global.w:4557-4584): a ragged row of vecs becomes a
        // zero-padded matrix; both limit checks fire before the gate.
        ScalarOp::StackRows => match expect_unary(arguments) {
            Value::List(rows) => {
                if rows.len() as u64 > u64::from(u32::MAX) {
                    return Err(runtime(
                        format!(
                            "Height {} of stacked matrix exceeds implementation limit",
                            rows.len()
                        ),
                        span,
                    ));
                }
                let mut vectors = Vec::with_capacity(rows.len());
                let mut width = 0usize;
                for row in rows {
                    match row {
                        Value::Vector(vector) => {
                            width = width.max(vector.0.len());
                            vectors.push(vector);
                        }
                        other => panic!("stack_rows saw non-vector {other:?}"),
                    }
                }
                if width as u64 > u64::from(u32::MAX) {
                    return Err(runtime(
                        format!("Width {width} of stacked matrix exceeds implementation limit"),
                        span,
                    ));
                }
                Ok(at_builtin_level(level, || {
                    let mut data = vec![0i32; vectors.len() * width];
                    for (row, vector) in vectors.iter().enumerate() {
                        for (col, &entry) in vector.0.iter().enumerate() {
                            data[col * vectors.len() + row] = entry;
                        }
                    }
                    Value::Matrix(
                        Matrix::from_columns(vectors.len(), width, data)
                            .expect("stacked data matches dimensions"),
                    )
                }))
            }
            other => panic!("stack_rows saw {other:?}"),
        },
        // combine_columns `#` and combine_rows `^` (global.w:4591-4638):
        // narrowing and all size diagnostics fire before the gate.
        ScalarOp::CombineColumns | ScalarOp::CombineRows => match expect_pair(arguments) {
            (Value::Integer(size), Value::List(parts)) => {
                let size = unsigned_long(&size, span)?;
                let (requested, supplied) = match operation {
                    ScalarOp::CombineColumns => ("rows", "columns"),
                    ScalarOp::CombineRows => ("columns", "rows"),
                    _ => unreachable!(),
                };
                if size > u64::from(u32::MAX) {
                    return Err(runtime(
                        format!(
                            "Number {size} of {requested} requested exceeds implementation limit"
                        ),
                        span,
                    ));
                }
                if parts.len() as u64 > u64::from(u32::MAX) {
                    return Err(runtime(
                        format!(
                            "Number {} of {supplied} exceeds implementation limit",
                            parts.len()
                        ),
                        span,
                    ));
                }
                let mut vectors = Vec::with_capacity(parts.len());
                for (index, part) in parts.into_iter().enumerate() {
                    match part {
                        Value::Vector(vector) => {
                            if vector.0.len() as u64 != size {
                                let kind = match operation {
                                    ScalarOp::CombineColumns => "Column",
                                    ScalarOp::CombineRows => "Row",
                                    _ => unreachable!(),
                                };
                                return Err(runtime(
                                    format!(
                                        "{kind} {index} size {} does not match specified size {size}",
                                        vector.0.len()
                                    ),
                                    span,
                                ));
                            }
                            vectors.push(vector);
                        }
                        other => panic!("matrix combiner saw non-vector {other:?}"),
                    }
                }
                let size = usize::try_from(size).expect("u32 dimension fits usize");
                Ok(at_builtin_level(level, || {
                    Value::Matrix(match operation {
                        // combine_columns: each vec is a column, so the
                        // column-major data is just the concatenation.
                        ScalarOp::CombineColumns => Matrix::from_columns(
                            size,
                            vectors.len(),
                            vectors.into_iter().flat_map(|vector| vector.0).collect(),
                        )
                        .expect("column data matches dimensions"),
                        ScalarOp::CombineRows => {
                            let count = vectors.len();
                            let mut data = vec![0i32; count * size];
                            for (row, vector) in vectors.iter().enumerate() {
                                for (col, &entry) in vector.0.iter().enumerate() {
                                    data[col * count + row] = entry;
                                }
                            }
                            Matrix::from_columns(count, size, data)
                                .expect("row data matches dimensions")
                        }
                        _ => unreachable!(),
                    })
                }))
            }
            other => panic!("matrix combiner saw {other:?}"),
        },
        // gcd(vec->int) (global.w:4820-4828): the non-negative gcd of the
        // entries, computed in machine int arithmetic — gcd([-2^31]) prints
        // -2147483648 upstream, so the fold runs in u32 and wraps back.
        ScalarOp::VectorGcd => match expect_unary(arguments) {
            Value::Vector(vector) => Ok(at_builtin_level(level, || {
                let mut divisor = 0u64;
                for &entry in &vector.0 {
                    divisor = gcd_u64(divisor, u64::from(entry.unsigned_abs()));
                }
                Value::Integer(BigInt::from(divisor as u32 as i32))
            })),
            other => panic!("vector gcd saw {other:?}"),
        },
        // elapsed_ms (global.w:5231-5245): a static stopwatch, started on
        // first call (upstream primes it at startup with no_value).
        ScalarOp::ElapsedMs => {
            static STOPWATCH: OnceLock<Instant> = OnceLock::new();
            assert!(arguments.is_empty(), "elapsed_ms takes no arguments");
            let stopwatch = STOPWATCH.get_or_init(Instant::now);
            Ok(at_builtin_level(level, || {
                Value::Integer(BigInt::from(stopwatch.elapsed().as_millis() as u64))
            }))
        }
    }
}

fn container_equality(relation: Relation, equal: bool) -> bool {
    match relation {
        Relation::Equal => equal,
        Relation::NotEqual => !equal,
        _ => panic!("ordered container relation is not registered"),
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
            // Integer bit utilities (global.w:2966-2994): succ/pred are the
            // rewrite targets of x+1/x-1; AND/OR/XOR/AND_NOT are
            // call-syntax bitwise ops on two's-complement bit strings.
            scalar_builtin("succ", int_type(), int_type(), 3, ScalarOp::IntSuccessor),
            scalar_builtin("pred", int_type(), int_type(), 3, ScalarOp::IntPredecessor),
            scalar_builtin("AND", int_pair(), int_type(), 1, ScalarOp::IntAnd),
            scalar_builtin("OR", int_pair(), int_type(), 1, ScalarOp::IntOr),
            scalar_builtin("XOR", int_pair(), int_type(), 1, ScalarOp::IntXor),
            scalar_builtin("AND_NOT", int_pair(), int_type(), 1, ScalarOp::IntAndNot),
            scalar_builtin(
                "bitwise_subset",
                int_pair(),
                bool_type(),
                0,
                ScalarOp::IntBitwiseSubset,
            ),
            scalar_builtin(
                "nth_set_bit",
                int_pair(),
                int_type(),
                0,
                ScalarOp::IntNthSetBit,
            ),
            scalar_builtin(
                "bit_length",
                int_type(),
                int_type(),
                0,
                ScalarOp::IntBitLength,
            ),
            scalar_builtin(
                "to_bitset",
                primitive_type(Prim::Vec),
                int_type(),
                0,
                ScalarOp::VecToBitset,
            ),
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
            scalar_builtin(
                "+",
                pair(primitive_type(Prim::Vec)),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::VecAdd,
            ),
            // Container arithmetic (global.w:4421-4451), in upstream order.
            scalar_builtin(
                "-",
                pair(primitive_type(Prim::Vec)),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::VecSubtract,
            ),
            scalar_builtin(
                "-",
                primitive_type(Prim::Vec),
                primitive_type(Prim::Vec),
                3,
                ScalarOp::VecNegate,
            ),
            scalar_builtin(
                "*",
                Type::tuple(vec![primitive_type(Prim::Vec), int_type()]),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::VecMultiplyInt,
            ),
            scalar_builtin(
                "\\",
                Type::tuple(vec![primitive_type(Prim::Vec), int_type()]),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::VecQuotientInt,
            ),
            scalar_builtin(
                "%",
                Type::tuple(vec![primitive_type(Prim::Vec), int_type()]),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::VecModuloInt,
            ),
            scalar_builtin(
                "%",
                primitive_type(Prim::RatVec),
                Type::tuple(vec![primitive_type(Prim::Vec), int_type()]),
                0,
                ScalarOp::RatvecUnfraction,
            ),
            scalar_builtin(
                "+",
                pair(primitive_type(Prim::RatVec)),
                primitive_type(Prim::RatVec),
                1,
                ScalarOp::RatvecAdd,
            ),
            scalar_builtin(
                "-",
                pair(primitive_type(Prim::RatVec)),
                primitive_type(Prim::RatVec),
                1,
                ScalarOp::RatvecSubtract,
            ),
            scalar_builtin(
                "-",
                primitive_type(Prim::RatVec),
                primitive_type(Prim::RatVec),
                3,
                ScalarOp::RatvecNegate,
            ),
            scalar_builtin(
                "*",
                Type::tuple(vec![primitive_type(Prim::RatVec), int_type()]),
                primitive_type(Prim::RatVec),
                1,
                ScalarOp::RatvecMultiplyInt,
            ),
            scalar_builtin(
                "/",
                Type::tuple(vec![primitive_type(Prim::RatVec), int_type()]),
                primitive_type(Prim::RatVec),
                1,
                ScalarOp::RatvecDivideInt,
            ),
            scalar_builtin(
                "%",
                Type::tuple(vec![primitive_type(Prim::RatVec), int_type()]),
                primitive_type(Prim::RatVec),
                1,
                ScalarOp::RatvecModuloInt,
            ),
            scalar_builtin(
                "*",
                Type::tuple(vec![primitive_type(Prim::RatVec), rat_type()]),
                primitive_type(Prim::RatVec),
                1,
                ScalarOp::RatvecMultiplyRat,
            ),
            scalar_builtin(
                "/",
                Type::tuple(vec![primitive_type(Prim::RatVec), rat_type()]),
                primitive_type(Prim::RatVec),
                1,
                ScalarOp::RatvecDivideRat,
            ),
            scalar_builtin(
                "+",
                Type::tuple(vec![primitive_type(Prim::Mat), int_type()]),
                primitive_type(Prim::Mat),
                1,
                ScalarOp::MatAddInt,
            ),
            scalar_builtin(
                "-",
                Type::tuple(vec![primitive_type(Prim::Mat), int_type()]),
                primitive_type(Prim::Mat),
                1,
                ScalarOp::MatSubtractInt,
            ),
            scalar_builtin(
                "+",
                Type::tuple(vec![int_type(), primitive_type(Prim::Mat)]),
                primitive_type(Prim::Mat),
                2,
                ScalarOp::IntAddMat,
            ),
            scalar_builtin(
                "-",
                Type::tuple(vec![int_type(), primitive_type(Prim::Mat)]),
                primitive_type(Prim::Mat),
                2,
                ScalarOp::IntSubtractMat,
            ),
            scalar_builtin(
                "*",
                pair(primitive_type(Prim::Vec)),
                int_type(),
                0,
                ScalarOp::VecDot,
            ),
            scalar_builtin(
                "flex_add",
                pair(primitive_type(Prim::Vec)),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::FlexAdd,
            ),
            scalar_builtin(
                "flex_sub",
                pair(primitive_type(Prim::Vec)),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::FlexSub,
            ),
            scalar_builtin(
                "convolve",
                pair(primitive_type(Prim::Vec)),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::VecConvolve,
            ),
            scalar_builtin(
                "+",
                pair(primitive_type(Prim::Mat)),
                primitive_type(Prim::Mat),
                1,
                ScalarOp::MatAdd,
            ),
            scalar_builtin(
                "-",
                pair(primitive_type(Prim::Mat)),
                primitive_type(Prim::Mat),
                1,
                ScalarOp::MatSubtract,
            ),
            scalar_builtin(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::RatVec),
                ]),
                primitive_type(Prim::RatVec),
                2,
                ScalarOp::MatMulRatVec,
            ),
            scalar_builtin(
                "*",
                Type::tuple(vec![primitive_type(Prim::Mat), primitive_type(Prim::Vec)]),
                primitive_type(Prim::Vec),
                2,
                ScalarOp::MatMulVec,
            ),
            scalar_builtin(
                "*",
                pair(primitive_type(Prim::Mat)),
                primitive_type(Prim::Mat),
                1,
                ScalarOp::MatMulMat,
            ),
            scalar_builtin(
                "*",
                Type::tuple(vec![primitive_type(Prim::Vec), primitive_type(Prim::Mat)]),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::VecMulMat,
            ),
            scalar_builtin(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::RatVec),
                    primitive_type(Prim::Mat),
                ]),
                primitive_type(Prim::RatVec),
                1,
                ScalarOp::RatVecMulMat,
            ),
            domain_builtin_validate(
                "*",
                pair(primitive_type(Prim::LieType)),
                primitive_type(Prim::LieType),
                1,
            ),
            domain_builtin(
                "*",
                pair(primitive_type(Prim::WeylElt)),
                primitive_type(Prim::WeylElt),
                1,
            ),
            domain_builtin_validate(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::WeylElt),
                    primitive_type(Prim::Vec),
                ]),
                primitive_type(Prim::Vec),
                2,
            ),
            domain_builtin_validate(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::WeylElt),
                ]),
                primitive_type(Prim::Vec),
                1,
            ),
            // split_times_wrapper (atlas-types.w:5102-5107, hunger 2) is
            // implemented; it keeps this position in the `*` listing.
            domain_builtin(
                "*",
                pair(primitive_type(Prim::Split)),
                primitive_type(Prim::Split),
                2,
            ),
            domain_builtin(
                "*",
                Type::tuple(vec![int_type(), primitive_type(Prim::KTypePol)]),
                primitive_type(Prim::KTypePol),
                2,
            ),
            domain_builtin(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::Split),
                    primitive_type(Prim::KTypePol),
                ]),
                primitive_type(Prim::KTypePol),
                2,
            ),
            domain_builtin(
                "*",
                Type::tuple(vec![primitive_type(Prim::Param), rat_type()]),
                primitive_type(Prim::Param),
                1,
            ),
            domain_builtin(
                "*",
                Type::tuple(vec![int_type(), primitive_type(Prim::ParamPol)]),
                primitive_type(Prim::ParamPol),
                2,
            ),
            domain_builtin(
                "*",
                Type::tuple(vec![
                    primitive_type(Prim::Split),
                    primitive_type(Prim::ParamPol),
                ]),
                primitive_type(Prim::ParamPol),
                2,
            ),
            domain_builtin(
                "*",
                Type::tuple(vec![primitive_type(Prim::ParamPol), rat_type()]),
                primitive_type(Prim::ParamPol),
                1,
            ),
            scalar_builtin("/", pair(rat_type()), rat_type(), 1, ScalarOp::RatDivide),
            scalar_builtin("%", pair(rat_type()), rat_type(), 1, ScalarOp::RatModulo),
            scalar_builtin("-", rat_type(), rat_type(), 3, ScalarOp::RatNegate),
            scalar_builtin("/", rat_type(), rat_type(), 3, ScalarOp::RatInverse),
            // rat decomposers (global.w:3249-3251): floor/ceil/frac.
            scalar_builtin("floor", rat_type(), int_type(), 0, ScalarOp::RatFloor),
            scalar_builtin("ceil", rat_type(), int_type(), 0, ScalarOp::RatCeil),
            scalar_builtin("frac", rat_type(), rat_type(), 3, ScalarOp::RatFrac),
            scalar_builtin(
                "^",
                Type::tuple(vec![rat_type(), int_type()]),
                rat_type(),
                1,
                ScalarOp::RatPower,
            ),
            // transposes and combine_rows (global.w:5185-5194): ^vec is a
            // one-row matrix, ^mat the transpose, n^[rows] stacks rows.
            // Upstream's space-suffixed "transpose " copy stays hidden
            // there and is not registered here.
            scalar_builtin(
                "^",
                primitive_type(Prim::Vec),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::VecTranspose,
            ),
            scalar_builtin(
                "^",
                primitive_type(Prim::Mat),
                primitive_type(Prim::Mat),
                3,
                ScalarOp::MatTranspose,
            ),
            scalar_builtin(
                "^",
                Type::tuple(vec![int_type(), Type::row(primitive_type(Prim::Vec))]),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::CombineRows,
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
            // Container relations (global.w:4405-4420): =/!= everywhere,
            // plus the dominance tests >=/> on vec and ratvec.
            scalar_builtin(
                "=",
                primitive_type(Prim::Vec),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                primitive_type(Prim::Vec),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                "=",
                pair(primitive_type(Prim::Vec)),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                pair(primitive_type(Prim::Vec)),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                ">=",
                primitive_type(Prim::Vec),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::GreaterEqual),
            ),
            scalar_builtin(
                ">",
                primitive_type(Prim::Vec),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Greater),
            ),
            scalar_builtin(
                "=",
                primitive_type(Prim::RatVec),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                primitive_type(Prim::RatVec),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                ">=",
                primitive_type(Prim::RatVec),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::GreaterEqual),
            ),
            scalar_builtin(
                ">",
                primitive_type(Prim::RatVec),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Greater),
            ),
            scalar_builtin(
                "=",
                pair(primitive_type(Prim::RatVec)),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                pair(primitive_type(Prim::RatVec)),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                "=",
                primitive_type(Prim::Mat),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                primitive_type(Prim::Mat),
                bool_type(),
                0,
                ScalarOp::UnaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                "=",
                pair(primitive_type(Prim::Mat)),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::Equal),
            ),
            scalar_builtin(
                "!=",
                pair(primitive_type(Prim::Mat)),
                bool_type(),
                0,
                ScalarOp::BinaryRelation(Relation::NotEqual),
            ),
            scalar_builtin(
                "##",
                pair(string_type()),
                string_type(),
                1,
                ScalarOp::StringConcat,
            ),
            // concatenate_strings (global.w:4387): fold a row of strings.
            scalar_builtin(
                "##",
                Type::row(string_type()),
                string_type(),
                0,
                ScalarOp::StringListConcat,
            ),
            // join_vectors / join_vector_row (global.w:4398-4399): vec
            // concatenation, pairwise and for a row of vecs.
            scalar_builtin(
                "##",
                pair(primitive_type(Prim::Vec)),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::VecJoin,
            ),
            scalar_builtin(
                "##",
                Type::row(primitive_type(Prim::Vec)),
                primitive_type(Prim::Vec),
                0,
                ScalarOp::VecRowJoin,
            ),
            // ascii (global.w:4388-4389): first byte, and its inverse for
            // printable ASCII (plus newline).
            scalar_builtin(
                "ascii",
                string_type(),
                int_type(),
                0,
                ScalarOp::StringToAscii,
            ),
            scalar_builtin("ascii", int_type(), string_type(), 0, ScalarOp::AsciiChar),
            // sizeof instances (global.w:4392-4395): string byte count, vec
            // and ratvec lengths, and the matrix column count.
            scalar_builtin("#", string_type(), int_type(), 0, ScalarOp::SizeOf),
            scalar_builtin(
                "#",
                primitive_type(Prim::Vec),
                int_type(),
                0,
                ScalarOp::SizeOf,
            ),
            scalar_builtin(
                "#",
                primitive_type(Prim::RatVec),
                int_type(),
                0,
                ScalarOp::SizeOf,
            ),
            scalar_builtin(
                "#",
                primitive_type(Prim::Mat),
                int_type(),
                0,
                ScalarOp::SizeOf,
            ),
            // vector suffix/prefix (global.w:4396-4397).
            scalar_builtin(
                "#",
                Type::tuple(vec![primitive_type(Prim::Vec), int_type()]),
                primitive_type(Prim::Vec),
                1,
                ScalarOp::VecSuffix,
            ),
            scalar_builtin(
                "#",
                Type::tuple(vec![int_type(), primitive_type(Prim::Vec)]),
                primitive_type(Prim::Vec),
                2,
                ScalarOp::VecPrefix,
            ),
            // matrix shape and accessors (global.w:4400-4404): shape gives
            // both bounds; rows/columns externalise to a row of vecs.
            scalar_builtin(
                "shape",
                primitive_type(Prim::Mat),
                int_pair(),
                0,
                ScalarOp::MatrixShape,
            ),
            scalar_builtin(
                "row",
                Type::tuple(vec![primitive_type(Prim::Mat), int_type()]),
                primitive_type(Prim::Vec),
                0,
                ScalarOp::MatrixRow,
            ),
            scalar_builtin(
                "column",
                Type::tuple(vec![primitive_type(Prim::Mat), int_type()]),
                primitive_type(Prim::Vec),
                0,
                ScalarOp::MatrixColumn,
            ),
            scalar_builtin(
                "rows",
                primitive_type(Prim::Mat),
                Type::row(primitive_type(Prim::Vec)),
                0,
                ScalarOp::MatrixRows,
            ),
            scalar_builtin(
                "columns",
                primitive_type(Prim::Mat),
                Type::row(primitive_type(Prim::Vec)),
                0,
                ScalarOp::MatrixColumns,
            ),
            // Generic row size is a special `#` instance upstream
            // (axis.w:2542-2550, 8863-8872), rather than an installed
            // concrete overload.  `[*]` is a registry-local wildcard here.
            hidden_scalar_builtin(
                "#",
                Type::row(Type::Undetermined),
                int_type(),
                0,
                ScalarOp::ListCardinality,
            ),
            // global.w:4478-4493: retain zero-row/zero-column dimensions,
            // narrow and bound both dimensions before the no-value gate.
            scalar_builtin(
                "null",
                int_type(),
                primitive_type(Prim::Vec),
                0,
                ScalarOp::NullVector,
            ),
            scalar_builtin(
                "null",
                int_pair(),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::NullMatrix,
            ),
            // Matrix constructors (global.w:5190-5193) and the
            // column-combining `#` instance (global.w:5193).
            scalar_builtin(
                "id_mat",
                int_type(),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::IdMat,
            ),
            scalar_builtin(
                "diagonal",
                primitive_type(Prim::Vec),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::Diagonal,
            ),
            scalar_builtin(
                "stack_rows",
                Type::row(primitive_type(Prim::Vec)),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::StackRows,
            ),
            scalar_builtin(
                "#",
                Type::tuple(vec![int_type(), Type::row(primitive_type(Prim::Vec))]),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::CombineColumns,
            ),
            // gcd(vec->int) (global.w:5200): the plain non-negative integer
            // gcd of the entries (0 for the empty vec); the Bezout/echelon
            // machinery upstream shares stays out of this port.
            scalar_builtin(
                "gcd",
                primitive_type(Prim::Vec),
                int_type(),
                0,
                ScalarOp::VectorGcd,
            ),
            // elapsed_ms (global.w:5245): milliseconds on the program
            // stopwatch.
            scalar_builtin(
                "elapsed_ms",
                Type::void(),
                int_type(),
                0,
                ScalarOp::ElapsedMs,
            ),
            domain_builtin("Lie_type", string_type(), primitive_type(Prim::LieType), 0),
            domain_builtin(
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
            // extend (atlas-types.w:280-289): append a simple factor.
            domain_builtin(
                "extend",
                Type::tuple(vec![
                    primitive_type(Prim::LieType),
                    string_type(),
                    primitive_type(Prim::Int),
                ]),
                primitive_type(Prim::LieType),
                0,
            ),
            domain_builtin(
                "prefers_coroots",
                primitive_type(Prim::RootDatum),
                bool_type(),
                0,
            ),
            domain_builtin(
                "simply_connected",
                Type::tuple(vec![primitive_type(Prim::LieType), bool_type()]),
                primitive_type(Prim::RootDatum),
                0,
            ),
            domain_builtin(
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
            domain_builtin(
                "Cartan_matrix",
                primitive_type(Prim::LieType),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "Cartan_matrix",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "nr_of_posroots",
                primitive_type(Prim::RootDatum),
                int_type(),
                0,
            ),
            domain_builtin("rank", primitive_type(Prim::RootDatum), int_type(), 0),
            domain_builtin("rank", primitive_type(Prim::LieType), int_type(), 0),
            // semisimple_rank (atlas-types.w:1397-1400, installed :2222):
            // the number of simple roots.
            domain_builtin(
                "semisimple_rank",
                primitive_type(Prim::RootDatum),
                int_type(),
                0,
            ),
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
            // root_expression/coroot_expression (atlas-types.w:1487-1504):
            // the simple (co)root coordinates of a signed root number;
            // root_index/coroot_index (:1505-1518): the signed number of a
            // (co)root in the datum's native lattice basis, with the miss
            // sentinel numPosRoots; root_involution (:1519-1526): the
            // reflection in |alpha| as a permutation of all roots in
            // internal RootNbr order.
            domain_builtin(
                "root_expression",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin(
                "coroot_expression",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin(
                "root_index",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::Vec),
                ]),
                int_type(),
                0,
            ),
            domain_builtin(
                "coroot_index",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::Vec),
                ]),
                int_type(),
                0,
            ),
            domain_builtin(
                "root_involution",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                primitive_type(Prim::Vec),
                0,
            ),
            // root_ladder_bottoms/coroot_ladder_bottoms
            // (atlas-types.w:1569-1597, installed :2241-2244): the
            // (co)roots beta for which beta-alpha is not a (co)root
            // (rootdata.h min_roots_for/min_coroots_for), as signed root
            // numbers in ascending internal order.
            domain_builtin(
                "root_ladder_bottoms",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                Type::row(int_type()),
                0,
            ),
            domain_builtin(
                "coroot_ladder_bottoms",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                Type::row(int_type()),
                0,
            ),
            // positive_roots_wrapper / positive_coroots_wrapper
            // (atlas-types.w:1656-1671): the positive (co)root table as a
            // by-columns matrix.
            domain_builtin(
                "posroots",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "poscoroots",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::Mat),
                0,
            ),
            // simple_roots/simple_coroots (atlas-types.w:1638-1658):
            // one row per simple (co)root.
            domain_builtin(
                "simple_roots",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "simple_coroots",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::Mat),
                0,
            ),
            // root_coradical / coroot_radical (atlas-types.w:2254-2255).
            domain_builtin(
                "root_coradical",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "coroot_radical",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::Mat),
                0,
            ),
            // is_Cartan_matrix (atlas-types.w:368-375, 433): a Cartan
            // matrix iff its Dynkin classification succeeds.
            domain_builtin(
                "is_Cartan_matrix",
                primitive_type(Prim::Mat),
                bool_type(),
                0,
            ),
            // components_rank (atlas-types.w:3936).
            domain_builtin(
                "components_rank",
                primitive_type(Prim::RealForm),
                int_type(),
                0,
            ),
            // default_extended (atlas-types.w:7313-7337): the components
            // of a default extended parameter.
            domain_builtin(
                "default_extended",
                Type::tuple(vec![primitive_type(Prim::Param), primitive_type(Prim::Mat)]),
                Type::tuple(vec![
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::Vec),
                ]),
                0,
            ),
            // strong_components (atlas-types.w:7525-7527): the graph is a
            // row of rows of ints; the result is a pair of such rows.
            domain_builtin(
                "strong_components",
                Type::row(Type::row(int_type())),
                Type::tuple(vec![
                    Type::row(Type::row(int_type())),
                    Type::row(Type::row(int_type())),
                ]),
                0,
            ),
            // dual_datum_of_inner_class_wrapper (atlas-types.w:3412-3413).
            domain_builtin(
                "dual_datum",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::RootDatum),
                0,
            ),
            // dual_datum_wrapper (atlas-types.w:1713-1717): the no-value
            // gate precedes the dual build, so skip.
            domain_builtin_skip(
                "dual",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::RootDatum),
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
            domain_builtin_validate(
                "classify_involution",
                primitive_type(Prim::Mat),
                Type::tuple(vec![int_type(), int_type(), int_type()]),
                0,
            ),
            domain_builtin_validate(
                "twisted_involution",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::Mat),
                ]),
                Type::tuple(vec![
                    primitive_type(Prim::WeylElt),
                    primitive_type(Prim::InnerClass),
                ]),
                0,
            ),
            domain_builtin(
                "distinguished_involution",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "nr_of_real_forms",
                primitive_type(Prim::InnerClass),
                int_type(),
                0,
            ),
            domain_builtin(
                "nr_of_dual_real_forms",
                primitive_type(Prim::InnerClass),
                int_type(),
                0,
            ),
            // dual_inner_class_wrapper (atlas-types.w:3254-3258, installed
            // with hunger 3 at atlas-types.w:3414): the no-value gate
            // precedes the dual build, so skip.
            domain_builtin(
                "dual",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::InnerClass),
                3,
            ),
            domain_builtin(
                "form_names",
                primitive_type(Prim::InnerClass),
                Type::row(string_type()),
                0,
            ),
            domain_builtin(
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
            // synthetic_real_form_wrapper (atlas-types.w:3851-3871): the
            // synthetic (InnerClass,mat,ratvec) constructor; every
            // diagnostic fires before its no_value gate, so it validates.
            domain_builtin_validate(
                "real_form",
                Type::tuple(vec![
                    primitive_type(Prim::InnerClass),
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::RatVec),
                ]),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin(
                "dual_real_form",
                Type::tuple(vec![primitive_type(Prim::InnerClass), int_type()]),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin(
                "quasisplit_form",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin(
                "dual_quasisplit_form",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin("form_number", primitive_type(Prim::RealForm), int_type(), 0),
            domain_builtin("KGB_size", primitive_type(Prim::RealForm), int_type(), 0),
            // central_fiber_wrapper (atlas-types.w:3915-3929): only the type
            // layer's conform error precedes its no-value gate, so skip.
            domain_builtin(
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
            // print_KL_basis / print_prim_KL / print_KL_list
            // (atlas-types.w:9117-9119): the KLV table printers over a Block.
            domain_printer_builtin("print_block", primitive_type(Prim::Block)),
            domain_printer_builtin("print_blockd", primitive_type(Prim::Block)),
            domain_printer_builtin("print_blocku", primitive_type(Prim::Block)),
            // print_param_block_wrapper / print_c_block_wrapper
            // (atlas-types.w:6653-6695, installed at 7504-7505): the common
            // block of a Param, fresh-built per call (the Rep_table pool is
            // memoization only).
            domain_printer_builtin("print_block", primitive_type(Prim::Param)),
            domain_printer_builtin("print_common_block", primitive_type(Prim::Param)),
            // print_part_param_block_wrapper / print_pc_block_wrapper
            // (atlas-types.w:6700-6735, installed at 7506-7509): the Bruhat
            // interval below a Param as a partial common block;
            // print_partial_common_block normalises the seed first.
            domain_printer_builtin("print_partial_block", primitive_type(Prim::Param)),
            domain_printer_builtin("print_partial_common_block", primitive_type(Prim::Param)),
            // KGB_Hasse (atlas-types.w:3735-3743): the Bruhat Hasse matrix.
            domain_builtin(
                "KGB_Hasse",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_printer_builtin("print_KL_basis", primitive_type(Prim::Block)),
            domain_printer_builtin("print_prim_KL", primitive_type(Prim::Block)),
            domain_printer_builtin("print_KL_list", primitive_type(Prim::Block)),
            domain_printer_builtin("print_W_graph", primitive_type(Prim::Block)),
            domain_printer_builtin("print_W_cells", primitive_type(Prim::Block)),
            // print_KGB_order / print_KGB_graph (atlas-types.w:9122-9123):
            // the Bruhat Hasse rows and the Graphviz digraph of the KGB.
            domain_printer_builtin("print_KGB_order", primitive_type(Prim::RealForm)),
            domain_printer_builtin("print_KGB_graph", primitive_type(Prim::RealForm)),
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
            domain_builtin("length", primitive_type(Prim::KgbElt), int_type(), 0),
            domain_builtin("length", primitive_type(Prim::Param), int_type(), 0),
            domain_builtin_skip(
                "involution",
                primitive_type(Prim::KgbElt),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "torus_factor",
                primitive_type(Prim::KgbElt),
                primitive_type(Prim::RatVec),
                0,
            ),
            domain_builtin(
                "base_grading_vector",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::RatVec),
                0,
            ),
            domain_builtin(
                "initial_torus_bits",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin(
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
            domain_builtin_validate(
                "from_dominant",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::Vec),
                ]),
                Type::Tuple(vec![
                    primitive_type(Prim::WeylElt),
                    primitive_type(Prim::Vec),
                ]),
                0,
            ),
            domain_builtin_validate(
                "from_dominant",
                Type::tuple(vec![
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::RootDatum),
                ]),
                Type::Tuple(vec![
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::WeylElt),
                ]),
                0,
            ),
            // Weyl orbit / alcove walls surface (atlas-types.w:2271-2281):
            // both argument orders of Weyl_orbit and Weyl_orbit_ws, plus
            // the alcove wall set and the attitude element.
            domain_builtin(
                "Weyl_orbit",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::Vec),
                ]),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "Weyl_orbit",
                Type::tuple(vec![
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::RootDatum),
                ]),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "Weyl_orbit_ws",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::Vec),
                ]),
                Type::row(primitive_type(Prim::WeylElt)),
                0,
            ),
            domain_builtin(
                "Weyl_orbit_ws",
                Type::tuple(vec![
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::RootDatum),
                ]),
                Type::row(primitive_type(Prim::WeylElt)),
                0,
            ),
            domain_builtin(
                "walls",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::RatVec),
                ]),
                Type::Tuple(vec![Type::row(int_type()), int_type()]),
                0,
            ),
            domain_builtin(
                "walls_attitude",
                Type::tuple(vec![primitive_type(Prim::RootDatum), Type::row(int_type())]),
                primitive_type(Prim::WeylElt),
                0,
            ),
            // basic_orbit_ws / affine_orbit_ws (atlas-types.w:2014-2063,
            // installed :2284-2287): the Weyl-word representatives of a
            // pseudo-Levi subquotient, finite or completed affine.
            domain_builtin(
                "basic_orbit_ws",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    Type::row(int_type()),
                    int_type(),
                ]),
                Type::row(primitive_type(Prim::WeylElt)),
                0,
            ),
            domain_builtin(
                "affine_orbit_ws",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::RatVec),
                ]),
                Type::row(primitive_type(Prim::WeylElt)),
                0,
            ),
            // Alcove center / FPP enumerations (atlas-types.w:2279,
            // 2282-2283, 2287-2290).
            domain_builtin(
                "alcove_center",
                primitive_type(Prim::Param),
                primitive_type(Prim::Param),
                0,
            ),
            domain_builtin(
                "alcove_root_vertex",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::RatVec),
                ]),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin(
                "FPP_numers",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::RatVec),
                ]),
                Type::row(primitive_type(Prim::Vec)),
                0,
            ),
            domain_builtin(
                "FPP_w_shifts",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::RatVec),
                ]),
                Type::row(Type::Tuple(vec![
                    primitive_type(Prim::WeylElt),
                    Type::row(primitive_type(Prim::Vec)),
                ])),
                0,
            ),
            domain_builtin(
                "cofolded",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::RootDatum),
                0,
            ),
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
            // root_permutation (atlas-types.w:2604-2618, installed :2649):
            // the images of all roots under w, in internal RootNbr order.
            domain_builtin(
                "root_permutation",
                primitive_type(Prim::WeylElt),
                primitive_type(Prim::Vec),
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
            domain_builtin_validate(
                "#",
                Type::tuple(vec![primitive_type(Prim::WeylElt), int_type()]),
                primitive_type(Prim::WeylElt),
                1,
            ),
            domain_builtin_validate(
                "#",
                Type::tuple(vec![int_type(), primitive_type(Prim::WeylElt)]),
                primitive_type(Prim::WeylElt),
                2,
            ),
            domain_builtin_validate(
                "##",
                Type::tuple(vec![primitive_type(Prim::WeylElt), Type::row(int_type())]),
                primitive_type(Prim::WeylElt),
                1,
            ),
            domain_builtin_validate(
                "##",
                Type::tuple(vec![Type::row(int_type()), primitive_type(Prim::WeylElt)]),
                primitive_type(Prim::WeylElt),
                2,
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
                "Cartan_class",
                primitive_type(Prim::KgbElt),
                primitive_type(Prim::CartanClass),
                0,
            ),
            domain_builtin(
                "nr_of_Cartan_classes",
                primitive_type(Prim::InnerClass),
                int_type(),
                0,
            ),
            domain_builtin(
                "nr_of_Cartan_classes",
                primitive_type(Prim::RealForm),
                int_type(),
                0,
            ),
            domain_builtin(
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
            // KL_sum_at_s / KL_sum_at_s_to_height (atlas-types.w:8583-8588):
            // the KL column of a final parameter evaluated at q = s.
            domain_builtin_validate(
                "KL_sum_at_s",
                primitive_type(Prim::Param),
                primitive_type(Prim::ParamPol),
                0,
            ),
            domain_builtin_validate(
                "KL_sum_at_s_to_height",
                Type::tuple(vec![primitive_type(Prim::Param), int_type()]),
                primitive_type(Prim::ParamPol),
                0,
            ),
            // raw_KL / dual_KL (atlas-types.w:9101-9102): the KL table of
            // a block as (matrix, polynomial pool, length stops).
            domain_builtin(
                "raw_KL",
                primitive_type(Prim::Block),
                Type::tuple(vec![
                    primitive_type(Prim::Mat),
                    Type::row(primitive_type(Prim::Vec)),
                    primitive_type(Prim::Vec),
                ]),
                0,
            ),
            domain_builtin_skip(
                "dual_KL",
                primitive_type(Prim::Block),
                Type::tuple(vec![
                    primitive_type(Prim::Mat),
                    Type::row(primitive_type(Prim::Vec)),
                    primitive_type(Prim::Vec),
                ]),
                0,
            ),
            // extended_block / partial_extended_KL_block
            // (atlas-types.w:7531-7534) and raw_ext_KL
            // (atlas-types.w:9103): the extended block of a standard
            // parameter, its condensed KLV matrix, and its raw KLV table.
            domain_builtin(
                "extended_block",
                Type::tuple(vec![primitive_type(Prim::Param), primitive_type(Prim::Mat)]),
                Type::tuple(vec![
                    Type::row(primitive_type(Prim::Param)),
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::Mat),
                ]),
                0,
            ),
            domain_builtin(
                "raw_ext_KL",
                Type::tuple(vec![primitive_type(Prim::Param), primitive_type(Prim::Mat)]),
                Type::tuple(vec![
                    primitive_type(Prim::Mat),
                    Type::row(primitive_type(Prim::Vec)),
                    primitive_type(Prim::Vec),
                ]),
                0,
            ),
            domain_builtin(
                "partial_extended_KL_block",
                Type::tuple(vec![primitive_type(Prim::Param), primitive_type(Prim::Mat)]),
                Type::tuple(vec![
                    Type::row(primitive_type(Prim::Param)),
                    primitive_type(Prim::Mat),
                    Type::row(primitive_type(Prim::Vec)),
                ]),
                0,
            ),
            // shift_flip (atlas-types.w:7341-7362, installed :7530):
            // whether the parameter's default extension, shifted to the
            // given rational weight, is opposite to the default extension
            // at that weight. test_compatible and both gamma-fix checks
            // precede the wrapper's no_value gate, so it validates.
            domain_builtin_validate(
                "shift_flip",
                Type::tuple(vec![
                    primitive_type(Prim::Param),
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::RatVec),
                ]),
                bool_type(),
                0,
            ),
            // scale_extended / K_type_pol_extended / finalize_extended
            // (atlas-types.w:8449-8537, installed :8591-8596): the
            // ext_finalise trio — scale a final parameter at the extended
            // level, restrict an extended parameter to K, and finalize an
            // extended parameter into an SR_poly. Every precondition
            // (test_final/test_standard, the factor check,
            // test_compatible, is_fixed, commutation) precedes each
            // wrapper's no_value gate, so they validate.
            domain_builtin_validate(
                "scale_extended",
                Type::tuple(vec![
                    primitive_type(Prim::Param),
                    primitive_type(Prim::Mat),
                    rat_type(),
                ]),
                Type::tuple(vec![primitive_type(Prim::Param), bool_type()]),
                1,
            ),
            domain_builtin_validate(
                "K_type_pol_extended",
                Type::tuple(vec![primitive_type(Prim::Param), primitive_type(Prim::Mat)]),
                primitive_type(Prim::KTypePol),
                1,
            ),
            domain_builtin_validate(
                "finalize_extended",
                Type::tuple(vec![primitive_type(Prim::Param), primitive_type(Prim::Mat)]),
                primitive_type(Prim::ParamPol),
                1,
            ),
            // W_graph / W_cells (atlas-types.w:7494-7496): the W-graph and
            // its cell decomposition of a standard parameter's block.
            domain_builtin_validate(
                "W_graph",
                primitive_type(Prim::Param),
                Type::tuple(vec![
                    int_type(),
                    Type::row(Type::tuple(vec![
                        Type::row(int_type()),
                        Type::row(Type::tuple(vec![int_type(), int_type()])),
                    ])),
                ]),
                0,
            ),
            domain_builtin_validate(
                "W_cells",
                primitive_type(Prim::Param),
                Type::tuple(vec![
                    int_type(),
                    Type::row(Type::tuple(vec![
                        Type::row(int_type()),
                        Type::row(Type::tuple(vec![
                            Type::row(int_type()),
                            Type::row(Type::tuple(vec![int_type(), int_type()])),
                        ])),
                    ])),
                ]),
                0,
            ),
            // block_W_graph_wrapper / block_W_cells_wrapper
            // (atlas-types.w:8738-8808, installed :9104-9106): unlike the
            // Param overloads these expose the full Block graph directly,
            // without a distinguished start index.  The upstream wrappers
            // assign their result before testing `no_value`, so discarded
            // calls still perform the graph computation.
            domain_builtin(
                "W_graph",
                primitive_type(Prim::Block),
                Type::row(Type::tuple(vec![
                    Type::row(int_type()),
                    Type::row(Type::tuple(vec![int_type(), int_type()])),
                ])),
                0,
            ),
            domain_builtin(
                "W_cells",
                primitive_type(Prim::Block),
                Type::row(Type::tuple(vec![
                    Type::row(int_type()),
                    Type::row(Type::tuple(vec![
                        Type::row(int_type()),
                        Type::row(Type::tuple(vec![int_type(), int_type()])),
                    ])),
                ])),
                0,
            ),
            // block_Hasse (atlas-types.w:7514): the full block of a
            // standard parameter and its Bruhat Hasse matrix.
            domain_builtin_validate(
                "block_Hasse",
                primitive_type(Prim::Param),
                Type::tuple(vec![
                    Type::row(primitive_type(Prim::Param)),
                    primitive_type(Prim::Mat),
                ]),
                0,
            ),
            // KL_column (atlas-types.w:6882-6905): the KL column of a final
            // standard parameter, over its partial block.
            domain_builtin_validate(
                "KL_column",
                primitive_type(Prim::Param),
                Type::row(Type::tuple(vec![
                    int_type(),
                    primitive_type(Prim::Param),
                    primitive_type(Prim::Vec),
                ])),
                0,
            ),
            // KL_block (atlas-types.w:6868-6912): the condensed KL matrix
            // over the parameter's common block.
            domain_builtin_validate(
                "KL_block",
                primitive_type(Prim::Param),
                Type::tuple(vec![
                    Type::row(primitive_type(Prim::Param)),
                    primitive_type(Prim::Int),
                    primitive_type(Prim::Mat),
                    Type::row(primitive_type(Prim::Vec)),
                ]),
                0,
            ),
            // dual_KL_block (atlas-types.w:7053-7133, installed :7517):
            // the KL matrix of the dual block over the parameter's common
            // block survivors, with no condensing.
            domain_builtin(
                "dual_KL_block",
                primitive_type(Prim::Param),
                Type::tuple(vec![
                    Type::row(primitive_type(Prim::Param)),
                    int_type(),
                    primitive_type(Prim::Mat),
                    Type::row(primitive_type(Prim::Vec)),
                ]),
                0,
            ),
            // partial_block (atlas-types.w:6786-6820): the partial-block
            // parameters of a final standard parameter.
            domain_builtin_validate(
                "partial_block",
                primitive_type(Prim::Param),
                Type::row(primitive_type(Prim::Param)),
                0,
            ),
            // full_deform (atlas-types.w:8213-8227): the full K-type
            // deformation of a final standard parameter.
            domain_builtin(
                "full_deform",
                primitive_type(Prim::Param),
                primitive_type(Prim::KTypePol),
                0,
            ),
            domain_builtin_validate(
                "full_deform",
                Type::tuple(vec![primitive_type(Prim::Param), int_type()]),
                Type::Union(vec![Type::void(), primitive_type(Prim::KTypePol)]),
                0,
            ),
            // partial_KL_block (atlas-types.w:6998-7051): the condensed KL
            // matrix over a parameter's partial-block survivors.
            domain_builtin(
                "partial_KL_block",
                primitive_type(Prim::Param),
                Type::tuple(vec![
                    Type::row(primitive_type(Prim::Param)),
                    primitive_type(Prim::Mat),
                    Type::row(primitive_type(Prim::Vec)),
                ]),
                0,
            ),
            // two_rho / two_rho_check (atlas-types.w:1409-1421): the sum of
            // the positive roots, respectively of the positive coroots.
            domain_builtin(
                "two_rho",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::Vec),
                0,
            ),
            domain_builtin(
                "fundamental_weight",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                primitive_type(Prim::RatVec),
                0,
            ),
            domain_builtin(
                "fundamental_coweight",
                Type::tuple(vec![primitive_type(Prim::RootDatum), int_type()]),
                primitive_type(Prim::RatVec),
                0,
            ),
            domain_builtin(
                "simple_factors",
                primitive_type(Prim::LieType),
                Type::Row(Box::new(Type::tuple(vec![
                    primitive_type(Prim::String),
                    int_type(),
                ]))),
                0,
            ),
            domain_builtin(
                "derived_info",
                primitive_type(Prim::RootDatum),
                Type::Tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::Mat),
                ]),
                0,
            ),
            domain_builtin(
                "mod_central_torus_info",
                primitive_type(Prim::RootDatum),
                Type::Tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::Mat),
                ]),
                0,
            ),
            domain_builtin(
                "integrality_rank",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::RatVec),
                ]),
                int_type(),
                0,
            ),
            domain_builtin(
                "is_integrally_dominant",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::RatVec),
                ]),
                bool_type(),
                0,
            ),
            domain_builtin(
                "integrality_points",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::RatVec),
                ]),
                Type::Row(Box::new(primitive_type(Prim::Rat))),
                0,
            ),
            domain_builtin(
                "integrality_datum",
                Type::tuple(vec![
                    primitive_type(Prim::RootDatum),
                    primitive_type(Prim::RatVec),
                ]),
                primitive_type(Prim::RootDatum),
                0,
            ),
            domain_builtin(
                "Cartan_matrix_type",
                primitive_type(Prim::Mat),
                Type::tuple(vec![
                    primitive_type(Prim::LieType),
                    Type::Row(Box::new(int_type())),
                ]),
                0,
            ),
            domain_builtin(
                "two_rho_check",
                primitive_type(Prim::RootDatum),
                primitive_type(Prim::Vec),
                0,
            ),
            // orientation_nr (atlas-types.w:6546-6552): the orientation
            // number of a standard parameter.
            domain_builtin("orientation_nr", primitive_type(Prim::Param), int_type(), 0),
            // reducibility_points (atlas-types.w:6561-6568, installed
            // :7500-7501): the reducibility fractions of a standard
            // parameter.
            domain_builtin(
                "reducibility_points",
                primitive_type(Prim::Param),
                Type::row(rat_type()),
                0,
            ),
            // Cartan_info (atlas-types.w:4102-4160): the classify triple,
            // the Cartan involution's Weyl word, the orbit/fiber sizes, and
            // the three subsystem types.
            domain_builtin_skip(
                "Cartan_info",
                primitive_type(Prim::CartanClass),
                Type::tuple(vec![
                    Type::tuple(vec![int_type(), int_type(), int_type()]),
                    primitive_type(Prim::Vec),
                    Type::tuple(vec![int_type(), int_type()]),
                    Type::tuple(vec![
                        primitive_type(Prim::LieType),
                        primitive_type(Prim::LieType),
                        primitive_type(Prim::LieType),
                    ]),
                ]),
                0,
            ),
            // parameter_cross_wrapper / parameter_Cayley_wrapper
            // (atlas-types.w:7492-7493): the simple reflection is numbered
            // in the parameter's integral subsystem. Its range check runs
            // before the wrappers' no-value gate.
            domain_builtin_validate(
                "cross",
                Type::tuple(vec![int_type(), primitive_type(Prim::Param)]),
                primitive_type(Prim::Param),
                2,
            ),
            domain_builtin_validate(
                "Cayley",
                Type::tuple(vec![int_type(), primitive_type(Prim::Param)]),
                primitive_type(Prim::Param),
                2,
            ),
            // root_parameter_cross_wrapper/root_parameter_Cayley_wrapper
            // (atlas-types.w:6474-6518, installed at :7494-7496): arbitrary
            // ambient-root coordinates. Both wrappers skip every check when
            // their value is discarded.
            domain_builtin_skip(
                "cross",
                Type::tuple(vec![primitive_type(Prim::Vec), primitive_type(Prim::Param)]),
                primitive_type(Prim::Param),
                2,
            ),
            domain_builtin_skip(
                "Cayley",
                Type::tuple(vec![primitive_type(Prim::Vec), primitive_type(Prim::Param)]),
                primitive_type(Prim::Param),
                2,
            ),
            // basic_involution_wrapper (atlas-types.w:860-880, installed at
            // atlas-types.w:939-940): the permutation size check precedes
            // its no-value gate, so validation checks the size only.
            domain_builtin_validate(
                "involution",
                Type::tuple(vec![
                    primitive_type(Prim::LieType),
                    Type::row(int_type()),
                    string_type(),
                ]),
                primitive_type(Prim::Mat),
                0,
            ),
            // based_involution_wrapper (atlas-types.w:902-927, installed at
            // atlas-types.w:941-943): every diagnostic fires before its
            // result-only no-value gate, so validation runs the full check.
            domain_builtin_validate(
                "involution",
                Type::tuple(vec![
                    primitive_type(Prim::LieType),
                    primitive_type(Prim::Mat),
                    string_type(),
                ]),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "real_forms",
                primitive_type(Prim::CartanClass),
                Type::row(primitive_type(Prim::RealForm)),
                0,
            ),
            domain_builtin(
                "dual_real_forms",
                primitive_type(Prim::CartanClass),
                Type::row(primitive_type(Prim::RealForm)),
                0,
            ),
            domain_builtin(
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
            domain_builtin(
                "occurrence_matrix",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
                "dual_occurrence_matrix",
                primitive_type(Prim::InnerClass),
                primitive_type(Prim::Mat),
                0,
            ),
            domain_builtin(
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
            domain_builtin(
                "Cartan_order",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::Mat),
                0,
            ),
            // print_strongreal_wrapper (atlas-types.w:8850-8859):
            // output::printStrongReal, unconditional like print_KGB.
            domain_printer_builtin("print_strong_real", primitive_type(Prim::CartanClass)),
            // print_gradings_wrapper (atlas-types.w:4260-4300, installed
            // :9108-9109): the imaginary subsystem line plus one grading bit
            // string per fiber element of the real form's partition class.
            domain_printer_builtin(
                "print_gradings",
                Type::tuple(vec![
                    primitive_type(Prim::CartanClass),
                    primitive_type(Prim::RealForm),
                ]),
            ),
            // print_X_wrapper (atlas-types.w:8999-9008, installed :9124):
            // global Tits group X* table, unconditional like print_KGB.
            domain_printer_builtin("print_X", primitive_type(Prim::InnerClass)),
            // print_real_Weyl_wrapper (atlas-types.w:8831-8847, installed
            // :9110-9111) — note the argument order is (RealForm,CartanClass),
            // the reverse of print_gradings.
            domain_printer_builtin(
                "print_real_Weyl",
                Type::tuple(vec![
                    primitive_type(Prim::RealForm),
                    primitive_type(Prim::CartanClass),
                ]),
            ),
            // print_blockstabilizer_wrapper (atlas-types.w:8920-8932,
            // installed :9117-9118).
            domain_printer_builtin(
                "print_blockstabilizer",
                Type::tuple(vec![
                    primitive_type(Prim::Block),
                    primitive_type(Prim::CartanClass),
                ]),
            ),
            // Block surface (atlas-types.w:4994-5005): the Fokko
            // constructor (is_dual gated), the (rf, dual_rf) decomposition,
            // the size, the element/index pair, the dual block, and the
            // per-generator status/cross/Cayley wrappers. block, element,
            // index, and the four per-generator wrappers run their gates
            // before their upstream no-value checks, so they validate;
            // %, #, and dual gate first, so they skip.
            domain_builtin_validate(
                "block",
                Type::tuple(vec![
                    primitive_type(Prim::RealForm),
                    primitive_type(Prim::RealForm),
                ]),
                primitive_type(Prim::Block),
                0,
            ),
            // block(Param) (common_block_wrapper, atlas-types.w:6748-6780,
            // installed :7510): the parameter's common block as survivor
            // parameters plus the start index. test_standard gates before
            // the no-value check, so validate.
            domain_builtin_validate(
                "block",
                primitive_type(Prim::Param),
                Type::tuple(vec![Type::row(primitive_type(Prim::Param)), int_type()]),
                0,
            ),
            domain_builtin_skip(
                "%",
                primitive_type(Prim::Block),
                Type::tuple(vec![
                    primitive_type(Prim::RealForm),
                    primitive_type(Prim::RealForm),
                ]),
                0,
            ),
            domain_builtin_skip("#", primitive_type(Prim::Block), int_type(), 0),
            domain_builtin_validate(
                "element",
                Type::tuple(vec![primitive_type(Prim::Block), int_type()]),
                Type::tuple(vec![
                    primitive_type(Prim::KgbElt),
                    primitive_type(Prim::KgbElt),
                ]),
                0,
            ),
            domain_builtin_validate(
                "index",
                Type::tuple(vec![
                    primitive_type(Prim::Block),
                    primitive_type(Prim::KgbElt),
                    primitive_type(Prim::KgbElt),
                ]),
                int_type(),
                0,
            ),
            domain_builtin_skip(
                "dual",
                primitive_type(Prim::Block),
                primitive_type(Prim::Block),
                0,
            ),
            domain_builtin_validate(
                "status",
                Type::tuple(vec![int_type(), primitive_type(Prim::Block), int_type()]),
                int_type(),
                0,
            ),
            domain_builtin_validate(
                "cross",
                Type::tuple(vec![int_type(), primitive_type(Prim::Block), int_type()]),
                int_type(),
                0,
            ),
            domain_builtin_validate(
                "Cayley",
                Type::tuple(vec![int_type(), primitive_type(Prim::Block), int_type()]),
                int_type(),
                0,
            ),
            domain_builtin_validate(
                "inverse_Cayley",
                Type::tuple(vec![int_type(), primitive_type(Prim::Block), int_type()]),
                int_type(),
                0,
            ),
            // Split surface (atlas-types.w:5136-5145): the unary zero-test
            // relations, componentwise sum and difference, unary negation,
            // and the destructure back to an (int,int) pair. The dual
            // product keeps its position among the arithmetic overloads
            // above; binary equality lives in the relation block below.
            domain_builtin("=", primitive_type(Prim::Split), bool_type(), 0),
            domain_builtin("!=", primitive_type(Prim::Split), bool_type(), 0),
            domain_builtin(
                "+",
                pair(primitive_type(Prim::Split)),
                primitive_type(Prim::Split),
                1,
            ),
            domain_builtin(
                "-",
                pair(primitive_type(Prim::Split)),
                primitive_type(Prim::Split),
                1,
            ),
            domain_builtin(
                "-",
                primitive_type(Prim::Split),
                primitive_type(Prim::Split),
                3,
            ),
            domain_builtin("%", primitive_type(Prim::Split), int_pair(), 0),
            // KType surface (atlas-types.w:6071-6088): the fixture-gated
            // subset. The constructor rank-checks before its no-value gate,
            // so it validates; everything else runs behind the gate (skip).
            domain_builtin_validate(
                "K_type",
                Type::tuple(vec![
                    primitive_type(Prim::KgbElt),
                    primitive_type(Prim::Vec),
                ]),
                primitive_type(Prim::KType),
                0,
            ),
            domain_builtin_skip(
                "K_type",
                primitive_type(Prim::Param),
                primitive_type(Prim::KType),
                0,
            ),
            domain_builtin_skip(
                "%",
                primitive_type(Prim::KType),
                Type::tuple(vec![
                    primitive_type(Prim::KgbElt),
                    primitive_type(Prim::Vec),
                ]),
                0,
            ),
            domain_builtin_skip(
                "real_form",
                primitive_type(Prim::KType),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin("height", primitive_type(Prim::KType), int_type(), 0),
            domain_builtin_validate(
                "equivalent",
                pair(primitive_type(Prim::KType)),
                bool_type(),
                0,
            ),
            domain_builtin("is_standard", primitive_type(Prim::KType), bool_type(), 0),
            domain_builtin("is_dominant", primitive_type(Prim::KType), bool_type(), 0),
            domain_builtin("is_zero", primitive_type(Prim::KType), bool_type(), 0),
            domain_builtin("is_semifinal", primitive_type(Prim::KType), bool_type(), 0),
            domain_builtin("is_final", primitive_type(Prim::KType), bool_type(), 0),
            domain_builtin(
                "dominant",
                primitive_type(Prim::KType),
                primitive_type(Prim::KType),
                3,
            ),
            domain_builtin(
                "normal",
                primitive_type(Prim::KType),
                primitive_type(Prim::KType),
                3,
            ),
            domain_builtin(
                "theta_stable",
                primitive_type(Prim::KType),
                primitive_type(Prim::KType),
                3,
            ),
            domain_builtin(
                "to_canonical_fiber",
                primitive_type(Prim::KType),
                primitive_type(Prim::KType),
                3,
            ),
            // Param surface (atlas-types.w:7472-7480): the fixture-gated
            // subset — constructor, %, height, real_form, K_type(Param),
            // param(KType), and the three predicates. The constructor
            // rank-checks before its no-value gate.
            domain_builtin_validate(
                "param",
                Type::tuple(vec![
                    primitive_type(Prim::KgbElt),
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::RatVec),
                ]),
                primitive_type(Prim::Param),
                0,
            ),
            domain_builtin_skip(
                "param",
                primitive_type(Prim::KType),
                primitive_type(Prim::Param),
                0,
            ),
            domain_builtin_skip(
                "%",
                primitive_type(Prim::Param),
                Type::tuple(vec![
                    primitive_type(Prim::KgbElt),
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::RatVec),
                ]),
                0,
            ),
            domain_builtin_skip(
                "real_form",
                primitive_type(Prim::Param),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin("height", primitive_type(Prim::Param), int_type(), 0),
            domain_builtin("is_standard", primitive_type(Prim::Param), bool_type(), 0),
            domain_builtin("is_dominant", primitive_type(Prim::Param), bool_type(), 0),
            domain_builtin("is_semifinal", primitive_type(Prim::Param), bool_type(), 0),
            domain_builtin("is_final", primitive_type(Prim::Param), bool_type(), 0),
            domain_builtin("is_zero", primitive_type(Prim::Param), bool_type(), 0),
            // Param dominant/normal transforms (atlas-types.w:7484-7485,
            // hunger 3) and equivalence (atlas-types.w:7482), gated by
            // the param_transforms contract.
            domain_builtin(
                "dominant",
                primitive_type(Prim::Param),
                primitive_type(Prim::Param),
                3,
            ),
            domain_builtin(
                "normal",
                primitive_type(Prim::Param),
                primitive_type(Prim::Param),
                3,
            ),
            domain_builtin_skip(
                "twist",
                primitive_type(Prim::Param),
                primitive_type(Prim::Param),
                3,
            ),
            domain_builtin_validate(
                "twist",
                Type::tuple(vec![primitive_type(Prim::Param), primitive_type(Prim::Mat)]),
                primitive_type(Prim::Param),
                1,
            ),
            domain_builtin_validate(
                "equivalent",
                pair(primitive_type(Prim::Param)),
                bool_type(),
                0,
            ),
            // KTypePol surface (atlas-types.w:6091-6117): the
            // fixture-gated subset. add/subtract_K_type_wrapper check the
            // real form identity before their no-value gates, so they
            // validate.
            domain_builtin(
                "null_K_module",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::KTypePol),
                0,
            ),
            domain_builtin(
                "real_form",
                primitive_type(Prim::KTypePol),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin_skip("=", primitive_type(Prim::KTypePol), bool_type(), 0),
            domain_builtin_skip("!=", primitive_type(Prim::KTypePol), bool_type(), 0),
            domain_builtin("#", primitive_type(Prim::KTypePol), int_type(), 0),
            domain_builtin_validate(
                "+",
                Type::tuple(vec![
                    primitive_type(Prim::KTypePol),
                    primitive_type(Prim::KType),
                ]),
                primitive_type(Prim::KTypePol),
                1,
            ),
            domain_builtin_skip(
                "+",
                Type::tuple(vec![
                    primitive_type(Prim::KTypePol),
                    Type::row(Type::tuple(vec![
                        primitive_type(Prim::Split),
                        primitive_type(Prim::KType),
                    ])),
                ]),
                primitive_type(Prim::KTypePol),
                1,
            ),
            domain_builtin_validate(
                "+",
                Type::tuple(vec![
                    primitive_type(Prim::KTypePol),
                    primitive_type(Prim::KTypePol),
                ]),
                primitive_type(Prim::KTypePol),
                1,
            ),
            domain_builtin_validate(
                "+",
                Type::tuple(vec![
                    primitive_type(Prim::KTypePol),
                    Type::tuple(vec![
                        primitive_type(Prim::Split),
                        primitive_type(Prim::KType),
                    ]),
                ]),
                primitive_type(Prim::KTypePol),
                1,
            ),
            domain_builtin_validate(
                "-",
                Type::tuple(vec![
                    primitive_type(Prim::KTypePol),
                    primitive_type(Prim::KType),
                ]),
                primitive_type(Prim::KTypePol),
                1,
            ),
            domain_builtin_validate(
                "-",
                Type::tuple(vec![
                    primitive_type(Prim::KTypePol),
                    primitive_type(Prim::KTypePol),
                ]),
                primitive_type(Prim::KTypePol),
                1,
            ),
            domain_builtin(
                "first_term",
                primitive_type(Prim::KTypePol),
                Type::tuple(vec![
                    primitive_type(Prim::Split),
                    primitive_type(Prim::KType),
                ]),
                0,
            ),
            domain_builtin(
                "last_term",
                primitive_type(Prim::KTypePol),
                Type::tuple(vec![
                    primitive_type(Prim::Split),
                    primitive_type(Prim::KType),
                ]),
                0,
            ),
            domain_builtin(
                "truncate_above_height",
                Type::tuple(vec![primitive_type(Prim::KTypePol), int_type()]),
                primitive_type(Prim::KTypePol),
                1,
            ),
            // KGP_sum_wrapper (atlas-types.w:6120): the KGP set of a
            // semifinal K-type; the semifinal precondition precedes the
            // no-value gate, so it validates.
            domain_builtin_validate(
                "KGP_sum",
                primitive_type(Prim::KType),
                Type::row(Type::tuple(vec![int_type(), primitive_type(Prim::KType)])),
                0,
            ),
            // K_type_formula_wrapper (atlas-types.w:6121-6122): the K-type
            // formula with a height cutoff; semifinal precondition first.
            domain_builtin_validate(
                "K_type_formula",
                Type::tuple(vec![primitive_type(Prim::KType), int_type()]),
                primitive_type(Prim::KTypePol),
                0,
            ),
            // branch_wrapper (atlas-types.w:6123, hunger 1): the branch of
            // a KTypePol at a height cutoff; negative bounds rejected.
            domain_builtin_validate(
                "branch",
                Type::tuple(vec![primitive_type(Prim::KTypePol), int_type()]),
                primitive_type(Prim::KTypePol),
                1,
            ),
            // deform_wrapper (atlas-types.w:8084-8105): the KL deformation
            // of a parameter, producing an SR_poly (ParamPol).
            domain_builtin_validate(
                "deform",
                primitive_type(Prim::Param),
                primitive_type(Prim::ParamPol),
                1,
            ),
            // The twisted deformation family (atlas-types.w:8120-8150,
            // 8178-8204, 8229-8251, 8370-8382, 8420-8431; installed
            // :8572-8590). Every gate of twisted_deform,
            // twisted_full_deform, and both twisted_KL_sum_at_s wrappers
            // precedes the wrapper's no_value gate, so they validate;
            // block_deform's no_value gate comes FIRST
            // (atlas-types.w:8182), so it skips (precedent:
            // KL_sum_at_s_to_height at :5115).
            domain_builtin_validate(
                "twisted_deform",
                primitive_type(Prim::Param),
                primitive_type(Prim::ParamPol),
                1,
            ),
            domain_builtin_skip(
                "block_deform",
                Type::tuple(vec![
                    primitive_type(Prim::Param),
                    primitive_type(Prim::ParamPol),
                    int_type(),
                ]),
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    primitive_type(Prim::ParamPol),
                ]),
                0,
            ),
            domain_builtin_validate(
                "twisted_full_deform",
                primitive_type(Prim::Param),
                primitive_type(Prim::KTypePol),
                1,
            ),
            // The timed variant is installed as a second
            // "twisted_full_deform" overload (atlas-types.w:8585-8586,
            // "(Param,int->|KTypePol)"). Its timer and union semantics are
            // implemented by the domain evaluator after typed dispatch.
            domain_builtin_validate(
                "twisted_full_deform",
                Type::tuple(vec![primitive_type(Prim::Param), int_type()]),
                Type::Union(vec![Type::void(), primitive_type(Prim::KTypePol)]),
                1,
            ),
            domain_builtin_validate(
                "twisted_KL_sum_at_s",
                primitive_type(Prim::Param),
                primitive_type(Prim::ParamPol),
                1,
            ),
            domain_builtin_validate(
                "twisted_KL_sum_at_s",
                Type::tuple(vec![primitive_type(Prim::Param), primitive_type(Prim::Mat)]),
                primitive_type(Prim::ParamPol),
                1,
            ),
            // ParamPol surface (atlas-types.w:8542-8570): the
            // fixture-gated subset. add/subtract_module_wrapper check the
            // real form identity before their no-value gates.
            domain_builtin(
                "null_module",
                primitive_type(Prim::RealForm),
                primitive_type(Prim::ParamPol),
                0,
            ),
            domain_builtin(
                "real_form",
                primitive_type(Prim::ParamPol),
                primitive_type(Prim::RealForm),
                0,
            ),
            domain_builtin_skip("=", primitive_type(Prim::ParamPol), bool_type(), 0),
            domain_builtin_skip("!=", primitive_type(Prim::ParamPol), bool_type(), 0),
            domain_builtin("#", primitive_type(Prim::ParamPol), int_type(), 0),
            domain_builtin_validate(
                "+",
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    primitive_type(Prim::Param),
                ]),
                primitive_type(Prim::ParamPol),
                1,
            ),
            domain_builtin_validate(
                "+",
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    Type::tuple(vec![
                        primitive_type(Prim::Split),
                        primitive_type(Prim::Param),
                    ]),
                ]),
                primitive_type(Prim::ParamPol),
                1,
            ),
            domain_builtin_skip(
                "+",
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    Type::row(Type::tuple(vec![
                        primitive_type(Prim::Split),
                        primitive_type(Prim::Param),
                    ])),
                ]),
                primitive_type(Prim::ParamPol),
                1,
            ),
            domain_builtin_validate(
                "+",
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    primitive_type(Prim::ParamPol),
                ]),
                primitive_type(Prim::ParamPol),
                1,
            ),
            domain_builtin_validate(
                "-",
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    primitive_type(Prim::Param),
                ]),
                primitive_type(Prim::ParamPol),
                1,
            ),
            domain_builtin_validate(
                "-",
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    primitive_type(Prim::ParamPol),
                ]),
                primitive_type(Prim::ParamPol),
                1,
            ),
            domain_builtin(
                "first_term",
                primitive_type(Prim::ParamPol),
                Type::tuple(vec![
                    primitive_type(Prim::Split),
                    primitive_type(Prim::Param),
                ]),
                0,
            ),
            domain_builtin(
                "truncate_above_height",
                Type::tuple(vec![primitive_type(Prim::ParamPol), int_type()]),
                primitive_type(Prim::ParamPol),
                1,
            ),
            // param_poly_to_K_type_poly_wrapper (atlas-types.w:8546): the
            // K-type restriction of a ParamPol.
            domain_builtin(
                "K_type_pol",
                primitive_type(Prim::ParamPol),
                primitive_type(Prim::KTypePol),
                0,
            ),
            domain_builtin(
                "last_term",
                primitive_type(Prim::ParamPol),
                Type::tuple(vec![
                    primitive_type(Prim::Split),
                    primitive_type(Prim::Param),
                ]),
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
            domain_relation_builtin("=", pair(primitive_type(Prim::WeylElt)), Relation::Equal),
            domain_relation_builtin(
                "!=",
                pair(primitive_type(Prim::WeylElt)),
                Relation::NotEqual,
            ),
            domain_relation_builtin("=", pair(primitive_type(Prim::Split)), Relation::Equal),
            domain_relation_builtin("!=", pair(primitive_type(Prim::Split)), Relation::NotEqual),
            domain_relation_builtin("=", pair(primitive_type(Prim::KType)), Relation::Equal),
            domain_relation_builtin("!=", pair(primitive_type(Prim::KType)), Relation::NotEqual),
            domain_relation_builtin("=", pair(primitive_type(Prim::Param)), Relation::Equal),
            domain_relation_builtin("!=", pair(primitive_type(Prim::Param)), Relation::NotEqual),
            domain_relation_builtin("=", pair(primitive_type(Prim::KTypePol)), Relation::Equal),
            domain_relation_builtin(
                "!=",
                pair(primitive_type(Prim::KTypePol)),
                Relation::NotEqual,
            ),
            domain_relation_builtin("=", pair(primitive_type(Prim::ParamPol)), Relation::Equal),
            domain_relation_builtin(
                "!=",
                pair(primitive_type(Prim::ParamPol)),
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
                if !builtin.overload_visible {
                    continue;
                }
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

fn hidden_special_builtin(name: &str) -> Option<usize> {
    builtin_registry()
        .iter()
        .position(|builtin| builtin.name == name && !builtin.overload_visible)
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
            Self::MultiAssignment { plan, value } => {
                // No destination is touched until the complete RHS has been
                // evaluated successfully. Distribution itself cannot fail
                // after the static shape check.
                let value = force(value, context)?;
                execute_multi_assignment(plan, &value, context);
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
                source,
                span,
            } => {
                let upper = expect_integer(force(upper, context)?, *span, "slice upper bound")?;
                let lower = expect_integer(force(lower, context)?, *span, "slice lower bound")?;
                let values = expect_typed_list(force(array, context)?, *span, "slice")?;
                let sliced = evaluate_slice(values, lower, upper, *flags, source, *span)?;
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
            Self::HungryBuiltinCall {
                builtin,
                arguments,
                pilfer,
                pilfer_index,
                span,
            } => {
                let hunger = builtin_registry()[*builtin].hunger;
                let order: &[usize] = match hunger {
                    1 => &[1, 0],
                    2 => &[0, 1],
                    3 => &[0],
                    _ => unreachable!("only a hungry builtin call is rebuilt for pilfering"),
                };
                let mut values = vec![None; arguments.len()];
                for &index in order {
                    let value = if index == *pilfer_index {
                        take_pilfered(pilfer, context)?
                    } else {
                        force(&arguments[index], context)?
                    };
                    values[index] = Some(value);
                }
                let mut values = values
                    .into_iter()
                    .map(|value| value.expect("hunger order covers every builtin operand"))
                    .collect::<Vec<_>>();
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

fn take_pilfered(
    destination: &PilferDestination,
    context: &EvaluationContext,
) -> Result<Value, Control> {
    let (name, value, span) = match destination {
        PilferDestination::Global { name, cell, span } => (name, cell.borrow_mut().take(), *span),
        PilferDestination::Local {
            name,
            depth,
            offset,
            span,
        } => (name, context.take_local(*depth, *offset), *span),
    };
    value
        .map(|value| Rc::try_unwrap(value).unwrap_or_else(|shared| shared.as_ref().clone()))
        .ok_or_else(|| {
            runtime(
                format!("Taking value of uninitialized variable '{name}'"),
                span,
            )
        })
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

/// Send RHS components to resolved destinations in the exact post-order used
/// by upstream: tuple children left-to-right, then the optional whole target.
fn execute_multi_assignment(
    plan: &MultiAssignmentPlan,
    value: &Value,
    context: &EvaluationContext,
) {
    match plan {
        MultiAssignmentPlan::Omitted => {}
        MultiAssignmentPlan::Destination(MultiAssignmentDestination::Global(cell)) => {
            *cell.borrow_mut() = Some(Rc::new(value.clone()));
        }
        MultiAssignmentPlan::Destination(MultiAssignmentDestination::Local { depth, offset }) => {
            let updated = context.set_local(*depth, *offset, Rc::new(value.clone()));
            assert!(
                updated,
                "analysis emitted an invalid local multi-assignment address"
            );
        }
        MultiAssignmentPlan::Tuple { elements, whole } => {
            let Value::Tuple(values) = value else {
                panic!("analysis let a non-tuple value reach a tuple multi-assignment: {value}")
            };
            assert_eq!(
                values.len(),
                elements.len(),
                "analysis let a tuple arity mismatch reach multi-assignment"
            );
            for (element, value) in elements.iter().zip(values) {
                execute_multi_assignment(element, value, context);
            }
            if let Some(whole) = whole {
                execute_multi_assignment(whole, value, context);
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
    source: &str,
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
        // Upstream `slice_range_error` (axis.w:4281-4301): no space after
        // `<=`, and the printed slice node closes the message.
        let message = match (lower_out_of_range, upper_out_of_range) {
            (true, true) => format!(
                "both bounds {lower}:{upper} out of range (should be >=0 respectively <={}) in slice {source}",
                values.len()
            ),
            (true, false) => {
                format!("lower bound {lower} out of range (should be >=0) in slice {source}")
            }
            (false, true) => format!(
                "upper bound {upper} out of range (should be <={}) in slice {source}",
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

fn columns_to_matrix(
    columns: Vec<Vec32>,
    empty_columns: &'static str,
    unequal_sizes: &'static str,
    span: SourceSpan,
) -> Result<Matrix, Control> {
    if columns.is_empty() {
        return Err(runtime(empty_columns, span));
    }
    let rows = columns.first().map_or(0, |column| column.0.len());
    if columns.iter().any(|column| column.0.len() != rows) {
        return Err(runtime(unequal_sizes, span));
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
            Ok(Value::Matrix(columns_to_matrix(
                columns,
                "Implicit conversion to matrix for an empty set of vectors",
                "Vector sizes differ in conversion to matrix",
                span,
            )?))
        }
        "M[[I]]" => {
            let columns = expect_list(value)
                .into_iter()
                .map(|column| list_to_vec32(expect_list(column), span))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::Matrix(columns_to_matrix(
                columns,
                "Cannot convert empty list of lists to matrix",
                "List sizes differ in conversion to matrix",
                span,
            )?))
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
        "LT" | "RdIc" | "IcRf" | "RdRf" | "SpI" | "Sp(I,I)" => {
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
    fn case_empty_tuple_pattern_rejects_a_nonvoid_union_payload() {
        let mut context = TypedContext::new();
        context
            .execute(&command("set_type [ U = (int i | string s) ]"))
            .expect("union type definition");

        let error = context
            .execute(&command("case i(3) | i(()): 1 | s: 0 esac"))
            .expect_err("an empty tuple pattern requires a void payload");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(
            error.message,
            "Pattern () does not match type int for variant i"
        );
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
    fn simple_assignment_pilfers_only_the_matching_hungry_builtin_operand() {
        let mut context = TypedContext::new();
        let mut displays = Vec::new();
        for source in [
            "lt : LieType",
            "lt := Lie_type(\"A1\")",
            "lt_alias : LieType",
            "lt_alias := lt",
            "lt := lt*(begin lt:=Lie_type(\"B2\");Lie_type(\"C2\") end)",
            "lt",
            "lt_alias",
            "let local_lt=Lie_type(\"G2\") then local_alias=local_lt in begin local_lt:=local_lt*Lie_type(\"A1\");(local_lt,local_alias) end",
        ] {
            for event in context.execute(&command(source)).unwrap_or_else(|error| {
                panic!("{source}: {error:?}")
            }) {
                if let TypedCommandEvent::Value { value, .. } = event {
                    displays.push(value.to_string());
                }
            }
        }
        assert_eq!(
            displays,
            [
                "Lie type 'A1'",
                "Lie type 'A1'",
                "Lie type 'B2.C2'",
                "Lie type 'B2.C2'",
                "Lie type 'A1'",
                "(Lie type 'G2.A1',Lie type 'G2')",
            ]
        );
    }

    #[test]
    fn hunger_two_keeps_left_to_right_order_and_aliases_are_copy_on_write() {
        let mut context = TypedContext::new();
        let mut displays = Vec::new();
        for source in [
            "rd : RootDatum",
            "rd := simply_connected(Lie_type(\"A2\"),true)",
            "w : WeylElt",
            "w := W_elt(rd,[0])",
            "v : vec",
            "v := [1,2]",
            "v := (begin v:=[2,3];w end)*v",
            "v_alias : vec",
            "v_alias := v",
            "v := w*v",
            "v_alias",
        ] {
            for event in context
                .execute(&command(source))
                .unwrap_or_else(|error| panic!("{source}: {error:?}"))
            {
                if let TypedCommandEvent::Value { value, .. } = event {
                    displays.push(value.to_string());
                }
            }
        }
        assert_eq!(
            displays,
            [
                "simply connected root datum of Lie type 'A2'",
                "<0>",
                "[ 1, 2 ]",
                "[ -2,  5 ]",
                "[ -2,  5 ]",
                "[ 2, 3 ]",
                "[ -2,  5 ]",
            ]
        );
    }

    #[test]
    fn failed_hungry_assignment_leaves_destination_uninitialized() {
        let mut context = TypedContext::new();
        for source in [
            "rd : RootDatum",
            "rd := simply_connected(Lie_type(\"A2\"),true)",
            "w : WeylElt",
            "w := W_elt(rd,[0])",
            "bad : vec",
            "bad := [1]",
        ] {
            context
                .execute(&command(source))
                .unwrap_or_else(|error| panic!("{source}: {error:?}"));
        }
        assert_eq!(
            context
                .execute(&command("bad := w*bad"))
                .expect_err("rank mismatch")
                .message,
            "Rank and weight size mismatch 2:1"
        );
        assert_eq!(
            context
                .execute(&command("bad"))
                .expect_err("destination was pilfered")
                .message,
            "Taking value of uninitialized variable 'bad'"
        );
    }

    #[test]
    fn multi_assignment_updates_mixed_destinations_in_postorder_and_returns_rhs() {
        let global_a = crate::frames::global_with(Rc::new(Value::Integer(0.into())));
        let global_pair = crate::frames::global_with(Rc::new(Value::Tuple(vec![
            Value::Integer(0.into()),
            Value::Integer(0.into()),
        ])));
        let mut globals = IdTable::new();
        globals.define("a", Type::Primitive(Prim::Int), global_a.clone());
        globals.define(
            "pair",
            Type::Tuple(vec![Type::Primitive(Prim::Int), Type::Primitive(Prim::Int)]),
            global_pair.clone(),
        );

        let (type_, value) =
            convert_and_run_with("let b = 0 in set (a, b):pair := (20, 22)", &globals)
                .expect("mixed global/local multiple assignment");
        assert_eq!(
            type_,
            Type::Tuple(vec![Type::Primitive(Prim::Int), Type::Primitive(Prim::Int),])
        );
        assert_eq!(
            value,
            Value::Tuple(vec![Value::Integer(20.into()), Value::Integer(22.into())])
        );
        assert_eq!(
            global_a.borrow().as_deref(),
            Some(&Value::Integer(20.into()))
        );
        assert_eq!(global_pair.borrow().as_deref(), Some(&value));
    }

    #[test]
    fn multi_assignment_omitted_slots_consume_values_without_constraining_them() {
        let left = crate::frames::global_with(Rc::new(Value::Integer(0.into())));
        let right = crate::frames::global_with(Rc::new(Value::Integer(0.into())));
        let mut globals = IdTable::new();
        globals.define("left", Type::Primitive(Prim::Int), left.clone());
        globals.define("right", Type::Primitive(Prim::Int), right.clone());

        let (type_, value) =
            convert_and_run_with("set (left, , right) := (1, \"ignored\", 3)", &globals)
                .expect("omitted slot accepts any component type");
        assert_eq!(
            type_,
            Type::Tuple(vec![
                Type::Primitive(Prim::Int),
                Type::Primitive(Prim::String),
                Type::Primitive(Prim::Int),
            ])
        );
        assert_eq!(
            value,
            Value::Tuple(vec![
                Value::Integer(1.into()),
                Value::String("ignored".into()),
                Value::Integer(3.into()),
            ])
        );
        assert_eq!(left.borrow().as_deref(), Some(&Value::Integer(1.into())));
        assert_eq!(right.borrow().as_deref(), Some(&Value::Integer(3.into())));
    }

    #[test]
    fn explicit_empty_tuple_in_multi_assignment_voids_its_rhs_component() {
        let x = crate::frames::global_with(Rc::new(Value::Integer(0.into())));
        let mut globals = IdTable::new();
        globals.define("x", Type::Primitive(Prim::Int), x);

        let (type_, value) = convert_and_run_with("set (x, ()) := (1, 2)", &globals)
            .expect("void coercion satisfies the explicit empty tuple");
        assert_eq!(
            type_,
            Type::Tuple(vec![Type::Primitive(Prim::Int), Type::void()])
        );
        assert_eq!(
            value,
            Value::Tuple(vec![Value::Integer(1.into()), Value::Tuple(Vec::new())])
        );
    }

    #[test]
    fn grouped_multi_assignment_name_uses_simple_assignment_diagnostics() {
        let error = convert_and_run("set (missing) := 2").expect_err("undefined target");
        assert_eq!(error.kind, ErrorKind::Name);
        assert_eq!(
            error.message,
            "Undefined identifier 'missing' in assignment missing:=2"
        );
    }

    #[test]
    fn multi_assignment_evaluates_rhs_once_and_commits_only_after_success() {
        let counter = crate::frames::global_with(Rc::new(Value::Integer(0.into())));
        let x = crate::frames::global_with(Rc::new(Value::Integer(10.into())));
        let y = crate::frames::global_with(Rc::new(Value::Integer(20.into())));
        let mut globals = IdTable::new();
        globals.define("counter", Type::Primitive(Prim::Int), counter.clone());
        globals.define("x", Type::Primitive(Prim::Int), x.clone());
        globals.define("y", Type::Primitive(Prim::Int), y.clone());

        let (_, value) =
            convert_and_run_with("set (x, y) := (counter := counter + 1, counter)", &globals)
                .expect("successful RHS runs once before distribution");
        assert_eq!(
            value,
            Value::Tuple(vec![Value::Integer(1.into()), Value::Integer(1.into())])
        );
        assert_eq!(counter.borrow().as_deref(), Some(&Value::Integer(1.into())));
        assert_eq!(x.borrow().as_deref(), Some(&Value::Integer(1.into())));
        assert_eq!(y.borrow().as_deref(), Some(&Value::Integer(1.into())));

        let error = convert_and_run_with("set (x, y) := (7, die)", &globals)
            .expect_err("failing RHS commits no destinations");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(error.message, "I die");
        assert_eq!(x.borrow().as_deref(), Some(&Value::Integer(1.into())));
        assert_eq!(y.borrow().as_deref(), Some(&Value::Integer(1.into())));
    }

    #[test]
    fn multi_assignment_writes_whole_destination_after_children() {
        let aliased = crate::frames::global_with(Rc::new(Value::Integer(0.into())));
        let mut globals = IdTable::new();
        globals.define("child", Type::Primitive(Prim::Int), aliased.clone());
        globals.define(
            "whole",
            Type::Tuple(vec![Type::Primitive(Prim::Int), Type::Primitive(Prim::Int)]),
            aliased.clone(),
        );

        convert_and_run_with("set (child, ):whole := (1, 2)", &globals)
            .expect("aliased destinations are legal under distinct names");
        assert_eq!(
            aliased.borrow().as_deref(),
            Some(&Value::Tuple(vec![
                Value::Integer(1.into()),
                Value::Integer(2.into()),
            ]))
        );
    }

    #[test]
    fn multi_assignment_refines_polymorphic_target_types_after_rhs_conversion() {
        let row = crate::frames::unset_global();
        let mut globals = IdTable::new();
        globals.define("row", Type::row(Type::Undetermined), row);

        convert_and_run_with("set (row, ) := ([1, 2], \"free\")", &globals)
            .expect("RHS fills both target and omitted component holes");
        let (row_type, _) = globals.lookup("row").expect("target remains defined");
        assert_eq!(*row_type.borrow(), Type::row(Type::Primitive(Prim::Int)));
    }

    #[test]
    fn multi_assignment_refinement_ignores_rhs_side_effect_type_conflicts() {
        let row = crate::frames::global_with(Rc::new(Value::List(Vec::new())));
        let mut globals = IdTable::new();
        globals.define("row", Type::row(Type::Undetermined), row.clone());

        let (type_, value) =
            convert_and_run_with("set (row,) := ((row := [1]; [\"x\"]),0)", &globals)
                .expect("upstream refine ignores a failed post-RHS specialisation");
        assert_eq!(
            type_,
            Type::Tuple(vec![
                Type::row(Type::Primitive(Prim::String)),
                Type::Primitive(Prim::Int),
            ])
        );
        assert_eq!(
            value,
            Value::Tuple(vec![
                Value::List(vec![Value::String("x".into())]),
                Value::Integer(0.into()),
            ])
        );
        let (row_type, _) = globals.lookup("row").expect("row remains defined");
        assert_eq!(*row_type.borrow(), Type::row(Type::Primitive(Prim::Int)));
        assert_eq!(
            row.borrow().as_deref(),
            Some(&Value::List(vec![Value::String("x".into())]))
        );
    }

    #[test]
    fn multi_assignment_reports_exact_target_analysis_errors() {
        let x = crate::frames::global_with(Rc::new(Value::Integer(0.into())));
        let constant = crate::frames::global_with(Rc::new(Value::Integer(0.into())));
        let pair = crate::frames::global_with(Rc::new(Value::Tuple(vec![
            Value::Integer(0.into()),
            Value::Integer(0.into()),
        ])));
        let scalar = crate::frames::global_with(Rc::new(Value::String("old".into())));
        let mut globals = IdTable::new();
        globals.define("x", Type::Primitive(Prim::Int), x);
        globals.define("constant", Type::Primitive(Prim::Int), constant);
        globals.mark_const("constant");
        globals.define(
            "pair",
            Type::Tuple(vec![Type::Primitive(Prim::Int), Type::Primitive(Prim::Int)]),
            pair,
        );
        globals.define("scalar", Type::Primitive(Prim::String), scalar);

        for (source, kind, message) in [
            (
                "set (!x, ) := (1, 2)",
                ErrorKind::Type,
                "Cannot constant-qualify '!' identifier 'x' in multi-assignment",
            ),
            (
                "set (missing, ):!x := (1, 2)",
                ErrorKind::Type,
                "Cannot constant-qualify '!' identifier 'x' in multi-assignment",
            ),
            (
                "set (x, x) := (1, 2)",
                ErrorKind::Type,
                "Multiple assignments to same identifier 'x' in multi-assignment",
            ),
            (
                "set (x, missing) := (1, 2)",
                ErrorKind::Name,
                "Undefined identifier 'missing' in multiple assignment set (x,missing):=(1,2)",
            ),
            (
                "set (x, constant) := (1, 2)",
                ErrorKind::Name,
                "Name 'constant' is constant in multiple assignment set (x,constant):=(1,2)",
            ),
            (
                "set (x, ):scalar := (1, 2)",
                ErrorKind::Type,
                "Incompatible type for 'scalar' in multi-assignment: type string does no match pattern (int,*)",
            ),
        ] {
            let error = convert_and_run_with(source, &globals).expect_err(source);
            assert_eq!(error.kind, kind, "source: {source}");
            assert_eq!(error.message, message, "source: {source}");
            let span = error.span.expect("multi-assignment errors carry the whole span");
            assert_eq!(span.byte_start, 0, "source: {source}");
            assert_eq!(span.byte_end, source.len(), "source: {source}");
        }
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
        // Mismatched branches fail balancing (upstream `balance_error` with
        // the "branches of conditional" items name, axis.w:4790).
        let error = convert_and_run("if true then 1 else \"x\" fi").expect_err("mismatch");
        assert_eq!(
            error.message,
            "No common type found between branches of conditional: { int, string }"
        );
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
        assert_eq!(
            error.message,
            "No common type found between components of list expression: { int, bool }"
        );

        let error = convert_and_run("[[if true then 1 else true fi],if true then 2 fi]")
            .expect_err("a conditional failure nested in a list must propagate");
        assert_eq!(
            error.message,
            "No common type found between components of list expression: { int, bool }"
        );
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
    fn global_batch1_builtins_match_the_upstream_scalar_surface() {
        let cases = [
            // rat decomposers (global.w:3249-3251).
            ("floor(7/2)", "3"),
            ("floor(-7/2)", "-4"),
            ("ceil(7/2)", "4"),
            ("ceil(-7/2)", "-3"),
            ("frac(7/2)", "1/2"),
            ("frac(-7/2)", "1/2"),
            ("frac(4/2)", "0/1"),
            // string concatenation of a row (global.w:4387).
            ("##([\"ab\",\"\",\"cd\"])", "\"abcd\""),
            // ascii both ways (global.w:4388-4389).
            ("ascii(\"A\")", "65"),
            ("ascii(\"\")", "-1"),
            ("ascii(65)", "\"A\""),
            ("ascii(10)", "\"\n\""),
            // size-of instances (global.w:4392-4395).
            ("#\"hello\"", "5"),
            ("#\"\"", "0"),
            ("#(vec: [1,2,3])", "3"),
            ("#(ratvec: [1,2])", "2"),
            ("# null(2,3)", "3"),
            // matrix shape and accessors (global.w:4400-4404).
            ("shape(null(2,3))", "(2,3)"),
            ("row(mat: [[1,2],[3,4]], 0)", "[ 1, 3 ]"),
            ("column(mat: [[1,2],[3,4]], 1)", "[ 3, 4 ]"),
            ("rows(mat: [[1,2],[3,4]])", "[[ 1, 3 ],[ 2, 4 ]]"),
            ("columns(mat: [[1,2],[3,4]])", "[[ 1, 2 ],[ 3, 4 ]]"),
        ];
        for (source, expected) in cases {
            let (_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(value.to_string(), expected, "source: {source}");
        }

        for (source, expected) in [
            // The empty row cannot resolve to [string] upstream either.
            ("##([])", "Failed to match '##' with argument type [*]"),
            ("row(null(2,3), 2)", "row index 2 out of range (0<= . <2)"),
            (
                "column(null(2,3), 3)",
                "column index 3 out of range (0<= . <3)",
            ),
            (
                "row(null(2,3), -1)",
                "Negative integer where unsigned is required",
            ),
            ("ascii(31)", "Value 31 out of printable ASCII range"),
            ("ascii(127)", "Value 127 out of printable ASCII range"),
            ("ascii(2147483648)", "Integer value to big for conversion"),
        ] {
            match convert_and_run(source) {
                Err(error) => assert_eq!(error.message, expected, "source: {source}"),
                Ok(value) => panic!("{source} unexpectedly succeeded with {value:?}"),
            }
        }
    }

    #[test]
    fn global_batch2_builtins_match_the_upstream_scalar_surface() {
        let cases = [
            // succ/pred and the call-syntax bitwise int ops
            // (global.w:2966-2994).
            ("succ(5)", "6"),
            ("pred(5)", "4"),
            ("AND(6,3)", "2"),
            ("OR(6,3)", "7"),
            ("XOR(6,3)", "5"),
            ("AND_NOT(6,3)", "4"),
            ("AND(6,-1)", "6"),
            ("XOR(5,-1)", "-6"),
            ("bitwise_subset(2,6)", "true"),
            ("bitwise_subset(1,6)", "false"),
            ("bitwise_subset(-1,0)", "false"),
            ("bitwise_subset(0,-1)", "true"),
            ("bitwise_subset(-2,-3)", "false"),
            ("nth_set_bit(13,0)", "0"),
            ("nth_set_bit(13,1)", "2"),
            ("nth_set_bit(13,2)", "3"),
            ("nth_set_bit(0,0)", "-1"),
            ("nth_set_bit(-1,0)", "0"),
            ("nth_set_bit(-1,5)", "5"),
            ("nth_set_bit(-2,0)", "1"),
            ("nth_set_bit(5,-1)", "1"),
            ("nth_set_bit(-2,-1)", "0"),
            ("nth_set_bit(-2,-2)", "-1"),
            ("nth_set_bit(1,2147483648)", "-1"),
            ("bit_length(0)", "0"),
            ("bit_length(1)", "1"),
            ("bit_length(255)", "8"),
            ("bit_length(-1)", "-1"),
            ("bit_length(-8)", "-4"),
            ("bit_length(-9)", "-5"),
            ("bit_length(-256)", "-9"),
            ("to_bitset(vec: [0,2,5])", "37"),
            ("to_bitset(null(0))", "0"),
            ("to_bitset(vec: [0,63])", "9223372036854775809"),
            ("to_bitset(vec: [64])", "18446744073709551616"),
            // gcd(vec->int) (global.w:5200): plain non-negative gcd, with
            // the machine-int wrap on -2^31.
            ("gcd(vec: [12,18])", "6"),
            ("gcd(vec: [0-6,9])", "3"),
            ("gcd(vec: [0,0])", "0"),
            ("gcd(null(0))", "0"),
            ("gcd(vec: [7])", "7"),
            ("gcd(vec: [0-2147483648])", "-2147483648"),
            // Container relations (global.w:4405-4420).
            ("=(vec: [0,0])", "true"),
            ("!=(vec: [1,0,0])", "true"),
            ("(>=(vec: [0,1,2]))", "true"),
            ("(>=(vec: [0,-1,2]))", "false"),
            ("(>(vec: [1,2]))", "true"),
            ("(>(vec: [0,1]))", "false"),
            ("(vec: [1,2]) = (vec: [1,2])", "true"),
            ("(vec: [1,2]) = (vec: [1,2,0])", "false"),
            ("(vec: [1]) != (vec: [2])", "true"),
            ("=([0,0]/1)", "true"),
            ("!=([1,2]/3)", "true"),
            ("(>=([0,1]/2))", "true"),
            ("(>=([0,-1]/2))", "false"),
            ("(>([1,2]/3))", "true"),
            ("[1,2]/2 = [2,4]/4", "true"),
            ("[1,2]/2 = [1,1]/2", "false"),
            ("[1,2]/2 != [1,1]/2", "true"),
            ("=null(2,3)", "true"),
            ("!=null(2,3)", "false"),
            ("(mat: [[1]]) = (mat: [[1]])", "true"),
            ("null(2,2) = null(2,3)", "false"),
            ("null(2,2) != null(2,3)", "true"),
            // Vector arithmetic (global.w:4421-4428); \ and % take a
            // non-negative remainder even for negative divisors.
            ("(vec: [1,2,3]) + (vec: [4,5,6])", "[ 5, 7, 9 ]"),
            ("(vec: [1,2,3]) - (vec: [4,5,6])", "[ -3, -3, -3 ]"),
            ("-(vec: [1,2,3])", "[ -1, -2, -3 ]"),
            ("(vec: [1,2,3]) * 2", "[ 2, 4, 6 ]"),
            ("(vec: [7,-7]) \\ 2", "[  3, -4 ]"),
            ("(vec: [7,-7]) % 2", "[ 1, 1 ]"),
            ("(vec: [7]) % (0-3)", "[ 1 ]"),
            ("(vec: [7]) \\ (0-3)", "[ -2 ]"),
            ("(vec: [0-7]) % (0-3)", "[ 2 ]"),
            ("(vec: [0-7]) \\ (0-3)", "[ 3 ]"),
            // Rational vector arithmetic (global.w:4428-4436).
            ("([1,2]/3) + ([1,2]/3)", "[ 2, 4 ]/3"),
            ("([1,2]/3) - ([1,2]/3)", "[ 0, 0 ]/1"),
            ("-([1,2]/3)", "[ -1, -2 ]/3"),
            ("([1,2]/3) * 2", "[ 2, 4 ]/3"),
            ("([1,2]/3) / 2", "[ 1, 2 ]/6"),
            ("([1,2]/3) / (0-2)", "[ -1, -2 ]/6"),
            ("([1,2]/3) * (0-2)", "[ -2, -4 ]/3"),
            ("([1,2]/3) % (0-2)", "[ 1, 2 ]/3"),
            ("([1,2]/2) * 2", "[ 1, 2 ]/1"),
            ("([1,2]/2) * (2/3)", "[ 1, 2 ]/3"),
            ("([1,2]/2) / (2/3)", "[ 3, 6 ]/4"),
            ("([1,2]/4) + ([1,0]/4)", "[ 1, 1 ]/2"),
            ("([2]/2) % 2", "[ 1 ]/1"),
            ("([0-7]/2) % 2", "[ 1 ]/2"),
            ("([7]/2) % 2", "[ 3 ]/2"),
            ("%([1,2]/3)", "([ 1, 2 ],3)"),
            (
                "%([2147483647,0]/1 + [1,0]/1)",
                "([ -2147483648,           0 ],1)",
            ),
            // Matrix-integer and matrix-matrix arithmetic
            // (global.w:4437-4446): the integer lands on the main
            // diagonal; square shape is not required.
            ("(mat: [[1,2],[3,4]]) + 1", "\n| 2, 3 |\n| 2, 5 |\n"),
            ("(mat: [[1,2],[3,4]]) - 1", "\n| 0, 3 |\n| 2, 3 |\n"),
            ("1 + (mat: [[1,2],[3,4]])", "\n| 2, 3 |\n| 2, 5 |\n"),
            ("1 - (mat: [[1,2],[3,4]])", "\n|  0, -3 |\n| -2, -3 |\n"),
            ("null(2,3) + 1", "\n| 1, 0, 0 |\n| 0, 1, 0 |\n"),
            (
                "(mat: [[1,2],[3,4]]) + (mat: [[1,2],[3,4]])",
                "\n| 2, 6 |\n| 4, 8 |\n",
            ),
            (
                "(mat: [[1,2],[3,4]]) - (mat: [[1,2],[3,4]])",
                "\n| 0, 0 |\n| 0, 0 |\n",
            ),
            // Products (global.w:4441, 4447-4451); the mat literal lists
            // columns, so its rows are (1,3) and (2,4).
            ("(vec: [1,2]) * (vec: [3,4])", "11"),
            ("(vec: [50000,50000]) * (vec: [50000,50000])", "705032704"),
            ("(mat: [[1,2],[3,4]]) * (vec: [1,1])", "[ 4, 6 ]"),
            ("(vec: [1,1]) * (mat: [[1,2],[3,4]])", "[ 3, 7 ]"),
            (
                "(mat: [[1,2],[3,4]]) * (mat: [[1,2],[3,4]])",
                "\n|  7, 15 |\n| 10, 22 |\n",
            ),
            ("(mat: [[1,2],[3,4]]) * ([1,1]/2)", "[ 2, 3 ]/1"),
            ("([1,1]/2) * (mat: [[1,2],[3,4]])", "[ 3, 7 ]/2"),
            ("null(2,3) * null(3,2)", "\n| 0, 0 |\n| 0, 0 |\n"),
            ("null(0,0) * null(0,0)", "The 0x0 matrix"),
            ("null(2,3) * (vec: [1,2,3])", "[ 0, 0 ]"),
            ("(vec: [1,2]) * null(2,3)", "[ 0, 0, 0 ]"),
            // flex_add/flex_sub/convolve (global.w:4442-4444).
            ("flex_add(vec: [1,2,0], vec: [1,0,3,0,0])", "[ 2, 2, 3 ]"),
            ("flex_sub(vec: [1,2], vec: [1,2,3])", "[  0,  0, -3 ]"),
            ("flex_sub(vec: [1,2,3], vec: [1,2])", "[ 0, 0, 3 ]"),
            ("flex_add(vec: [1,2], vec: [3,4])", "[ 4, 6 ]"),
            ("convolve(vec: [1,2], vec: [3,4])", "[  3, 10,  8 ]"),
            ("convolve(vec: [1,2,0], vec: [3,4])", "[  3, 10,  8 ]"),
            ("convolve(null(0), vec: [1,2])", "[ ]"),
            ("convolve(vec: [0,0], vec: [1])", "[ ]"),
            // Joins and suffix/prefix (global.w:4396-4399).
            ("(vec: [1,2]) ## (vec: [3,4])", "[ 1, 2, 3, 4 ]"),
            ("##([vec: [1,2], vec: [3], vec: []])", "[ 1, 2, 3 ]"),
            ("(vec: [1,2]) # 3", "[ 1, 2, 3 ]"),
            ("3 # (vec: [1,2])", "[ 3, 1, 2 ]"),
            ("1 # null(0)", "[ 1 ]"),
            // Constructors (global.w:5183-5194).
            ("null(3)", "[ 0, 0, 0 ]"),
            ("null(0)", "[ ]"),
            ("^(vec: [1,2,3])", "\n| 1, 2, 3 |\n"),
            ("^(vec: [])", "The 1x0 matrix"),
            ("^(mat: [[1,2],[3,4]])", "\n| 1, 2 |\n| 3, 4 |\n"),
            ("id_mat(3)", "\n| 1, 0, 0 |\n| 0, 1, 0 |\n| 0, 0, 1 |\n"),
            ("id_mat(0)", "The 0x0 matrix"),
            (
                "diagonal(vec: [1,2,3])",
                "\n| 1, 0, 0 |\n| 0, 2, 0 |\n| 0, 0, 3 |\n",
            ),
            ("diagonal(null(0))", "The 0x0 matrix"),
            (
                "stack_rows([vec: [1,2], vec: [3]])",
                "\n| 1, 2 |\n| 3, 0 |\n",
            ),
            ("stack_rows([null(0)])", "The 1x0 matrix"),
            (
                "3 # [vec: [1,2,3], vec: [4,5,6]]",
                "\n| 1, 4 |\n| 2, 5 |\n| 3, 6 |\n",
            ),
            ("0 # [null(0)]", "The 0x1 matrix"),
            ("2 ^ [vec: [1,2], vec: [3,4]]", "\n| 1, 2 |\n| 3, 4 |\n"),
            // elapsed_ms (global.w:5245): the reading itself is
            // nondeterministic, so only its sign is pinned.
            ("elapsed_ms() >= 0", "true"),
        ];
        for (source, expected) in cases {
            let (_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(value.to_string(), expected, "source: {source}");
        }

        for (source, expected) in [
            (
                "to_bitset(vec: [0-1])",
                "Negative entry in conversion to bitset",
            ),
            (
                "nth_set_bit(1,9223372036854775808)",
                "Integer value to big for conversion",
            ),
            ("(vec: [1,2]) + (vec: [1,2,3])", "Size mismatch 2:3"),
            ("(vec: [1]) \\ 0", "Vector division by 0"),
            ("(vec: [1]) % 0", "Vector modulo 0"),
            (
                "(vec: [1]) * 2147483648",
                "Integer value to big for conversion",
            ),
            ("([1,2]/3) + ([1,2,3]/3)", "Size mismatch 2:3"),
            ("([1,2]/3) / 0", "Rational vector division by 0"),
            ("([1,2]/3) % 0", "Rational vector modulo 0"),
            ("([1,2]/3) / (0/1)", "Rational vector division by 0"),
            (
                "([1,2]/3) * 9223372036854775808",
                "Integer value to big for conversion",
            ),
            (
                "(mat: [[1]]) + 2147483648",
                "Integer value to big for conversion",
            ),
            ("(mat: [[1,2],[3,4]]) + null(2,3)", "Size mismatch 2:3"),
            ("(mat: [[1,2],[3,4]]) - null(3,2)", "Size mismatch 2:3"),
            ("(mat: [[1,2],[3,4]]) * (vec: [1,2,3])", "Size mismatch 2:3"),
            ("null(2,3) * null(2,2)", "Size mismatch 3:2"),
            ("(vec: [1,2]) * null(3,2)", "Size mismatch 2:3"),
            ("null(-1)", "Negative integer where unsigned is required"),
            ("id_mat(-1)", "Negative integer where unsigned is required"),
            (
                "(0-1) # [vec: [1]]",
                "Negative integer where unsigned is required",
            ),
            (
                "(0-1) ^ [vec: [1]]",
                "Negative integer where unsigned is required",
            ),
            (
                "2 # [vec: [1,2,3]]",
                "Column 0 size 3 does not match specified size 2",
            ),
            (
                "2 ^ [vec: [1,2,3]]",
                "Row 0 size 3 does not match specified size 2",
            ),
            (
                "0 # [vec: [1,2]]",
                "Column 0 size 2 does not match specified size 0",
            ),
        ] {
            match convert_and_run(source) {
                Err(error) => assert_eq!(error.message, expected, "source: {source}"),
                Ok(value) => panic!("{source} unexpectedly succeeded with {value:?}"),
            }
        }
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
    fn p0_simple_domain_signatures_match_the_upstream_install_table() {
        let signatures = |name: &str| {
            builtin_registry()
                .iter()
                .filter(|builtin| builtin.name == name)
                .map(|builtin| (builtin.arg_type.clone(), builtin.result.clone()))
                .collect::<Vec<_>>()
        };

        let root_datum = primitive_type(Prim::RootDatum);
        let weyl_elt = primitive_type(Prim::WeylElt);
        let vector = primitive_type(Prim::Vec);
        let param = primitive_type(Prim::Param);
        let from_dominant = signatures("from_dominant");
        assert!(from_dominant.contains(&(
            Type::tuple(vec![root_datum.clone(), vector.clone()]),
            Type::tuple(vec![weyl_elt.clone(), vector.clone()]),
        )));
        assert!(from_dominant.contains(&(
            Type::tuple(vec![vector.clone(), root_datum]),
            Type::tuple(vec![vector.clone(), weyl_elt]),
        )));

        assert_eq!(
            signatures("Cartan_info"),
            vec![(
                primitive_type(Prim::CartanClass),
                Type::tuple(vec![
                    Type::tuple(vec![int_type(), int_type(), int_type()]),
                    vector.clone(),
                    Type::tuple(vec![int_type(), int_type()]),
                    Type::tuple(vec![
                        primitive_type(Prim::LieType),
                        primitive_type(Prim::LieType),
                        primitive_type(Prim::LieType),
                    ]),
                ]),
            )]
        );
        assert_eq!(
            signatures("KL_block"),
            vec![(
                param.clone(),
                Type::tuple(vec![
                    Type::row(param.clone()),
                    int_type(),
                    primitive_type(Prim::Mat),
                    Type::row(vector.clone()),
                ]),
            )]
        );
        assert_eq!(
            signatures("KL_column"),
            vec![(
                param.clone(),
                Type::row(Type::tuple(vec![int_type(), param.clone(), vector])),
            )]
        );
        for name in ["cross", "Cayley"] {
            assert!(signatures(name)
                .contains(&(Type::tuple(vec![int_type(), param.clone()]), param.clone(),)));
        }
    }

    #[test]
    fn p1_simple_domain_signatures_match_the_upstream_install_table() {
        let signatures = |name: &str| {
            builtin_registry()
                .iter()
                .filter(|builtin| builtin.name == name)
                .map(|builtin| (builtin.arg_type.clone(), builtin.result.clone()))
                .collect::<Vec<_>>()
        };

        let int = int_type();
        let weyl = primitive_type(Prim::WeylElt);
        let ktype = primitive_type(Prim::KType);
        let ktype_pol = primitive_type(Prim::KTypePol);
        let param = primitive_type(Prim::Param);
        let param_pol = primitive_type(Prim::ParamPol);
        let split = primitive_type(Prim::Split);
        let no_value_policy = |name: &str, arguments: &Type| {
            let builtin = builtin_registry()
                .iter()
                .find(|builtin| builtin.name == name && &builtin.arg_type == arguments)
                .unwrap_or_else(|| panic!("missing {name}({arguments:?})"));
            match builtin.implementation {
                BuiltinImpl::Domain {
                    no_value: DomainNoValue::Skip,
                    ..
                } => "skip",
                BuiltinImpl::Domain {
                    no_value: DomainNoValue::Validate,
                    ..
                } => "validate",
                BuiltinImpl::Domain {
                    no_value: DomainNoValue::BuildAndDrop,
                    ..
                } => "build",
                _ => "other",
            }
        };

        let left_generator = Type::tuple(vec![int.clone(), weyl.clone()]);
        assert!(signatures("#").contains(&(left_generator.clone(), weyl.clone())));
        assert_eq!(no_value_policy("#", &left_generator), "validate");
        let right_generator = Type::tuple(vec![weyl.clone(), int.clone()]);
        assert_eq!(no_value_policy("#", &right_generator), "validate");
        for arguments in [
            Type::tuple(vec![weyl.clone(), Type::row(int.clone())]),
            Type::tuple(vec![Type::row(int), weyl.clone()]),
        ] {
            assert!(signatures("##").contains(&(arguments.clone(), weyl.clone())));
            assert_eq!(no_value_policy("##", &arguments), "validate");
        }
        let kgb = primitive_type(Prim::KgbElt);
        assert!(
            signatures("Cartan_class").contains(&(kgb.clone(), primitive_type(Prim::CartanClass),))
        );
        assert_eq!(no_value_policy("Cartan_class", &kgb), "skip");

        for pol in [ktype_pol.clone(), param_pol.clone()] {
            for name in ["=", "!="] {
                assert!(signatures(name).contains(&(pol.clone(), bool_type())));
                assert_eq!(no_value_policy(name, &pol), "skip");
            }
        }
        let ktype_term_list = Type::tuple(vec![
            ktype_pol,
            Type::row(Type::tuple(vec![split.clone(), ktype])),
        ]);
        assert!(
            signatures("+").contains(&(ktype_term_list.clone(), primitive_type(Prim::KTypePol),))
        );
        assert_eq!(no_value_policy("+", &ktype_term_list), "skip");
        for (terms, policy) in [
            (Type::tuple(vec![split.clone(), param.clone()]), "validate"),
            (Type::row(Type::tuple(vec![split, param])), "skip"),
        ] {
            let arguments = Type::tuple(vec![param_pol.clone(), terms]);
            assert!(signatures("+").contains(&(arguments.clone(), param_pol.clone(),)));
            assert_eq!(no_value_policy("+", &arguments), policy);
        }
    }

    #[test]
    fn p2_block_graph_signatures_match_the_upstream_install_table() {
        let signatures = |name: &str| {
            builtin_registry()
                .iter()
                .filter(|builtin| builtin.name == name)
                .map(|builtin| (builtin.arg_type.clone(), builtin.result.clone()))
                .collect::<Vec<_>>()
        };
        let no_value_policy = |name: &str, arguments: &Type| {
            let builtin = builtin_registry()
                .iter()
                .find(|builtin| builtin.name == name && &builtin.arg_type == arguments)
                .unwrap_or_else(|| panic!("missing {name}({arguments:?})"));
            match builtin.implementation {
                BuiltinImpl::Domain {
                    no_value: DomainNoValue::BuildAndDrop,
                    ..
                } => "build",
                BuiltinImpl::Domain {
                    no_value: DomainNoValue::Skip,
                    ..
                } => "skip",
                BuiltinImpl::Domain {
                    no_value: DomainNoValue::Validate,
                    ..
                } => "validate",
                _ => "other",
            }
        };

        let int = int_type();
        let block = primitive_type(Prim::Block);
        let edge = Type::tuple(vec![int.clone(), int.clone()]);
        let vertex = Type::tuple(vec![Type::row(int.clone()), Type::row(edge)]);
        let graph = Type::row(vertex.clone());
        let cell = Type::tuple(vec![Type::row(int.clone()), Type::row(vertex)]);

        assert!(signatures("W_graph").contains(&(block.clone(), graph.clone())));
        assert!(signatures("W_cells").contains(&(block.clone(), Type::row(cell))));
        assert_eq!(no_value_policy("W_graph", &block), "build");
        assert_eq!(no_value_policy("W_cells", &block), "build");

        let param = primitive_type(Prim::Param);
        let block_params = Type::tuple(vec![Type::row(param.clone()), int_type()]);
        assert!(signatures("block").contains(&(param.clone(), block_params)));
        assert_eq!(no_value_policy("block", &param), "validate");
        assert_eq!(no_value_policy("block_Hasse", &param), "validate");
        assert_eq!(no_value_policy("partial_block", &param), "validate");
        assert_eq!(no_value_policy("KL_sum_at_s", &param), "validate");
        let param_int = Type::tuple(vec![param.clone(), int_type()]);
        assert_eq!(
            no_value_policy("KL_sum_at_s_to_height", &param_int),
            "validate"
        );
    }

    #[test]
    fn param_graph_signatures_preserve_upstream_nested_types() {
        let builtin = |name: &str| {
            builtin_registry()
                .iter()
                .find(|builtin| {
                    builtin.name == name && builtin.arg_type == primitive_type(Prim::Param)
                })
                .unwrap_or_else(|| panic!("missing {name}(Param)"))
        };

        let int = int_type();
        let edge = Type::tuple(vec![int.clone(), int.clone()]);
        let vertex = Type::tuple(vec![Type::row(int.clone()), Type::row(edge)]);
        let graph = Type::tuple(vec![int.clone(), Type::row(vertex.clone())]);
        let cell = Type::tuple(vec![Type::row(int.clone()), Type::row(vertex)]);
        let cells = Type::tuple(vec![int, Type::row(cell)]);

        assert_eq!(builtin("W_graph").result, graph);
        assert_eq!(builtin("W_cells").result, cells);
        for name in ["W_graph", "W_cells"] {
            assert!(matches!(
                builtin(name).implementation,
                BuiltinImpl::Domain {
                    no_value: DomainNoValue::Validate,
                    ..
                }
            ));
        }
    }

    #[test]
    fn list_cardinality_is_polymorphic_and_drops_no_value_results() {
        let signature = builtin_registry()
            .iter()
            .find(|builtin| {
                builtin.name == "#"
                    && builtin.arg_type == Type::row(Type::Undetermined)
                    && builtin.result == int_type()
            })
            .expect("missing #([*]) -> int");
        assert_eq!(signature.hunger, 0);
        assert!(
            overload_variants("#")
                .iter()
                .all(|&index| builtin_registry()[index].arg_type != Type::row(Type::Undetermined)),
            "the generic row primitive is hidden from the overload table"
        );

        let mut overloads = OverloadState::default();
        assert!(
            !overloads.remove("#", &Type::row(Type::Undetermined)),
            "the hidden primitive cannot be forgotten"
        );

        for (source, expected) in [("#[]", 0), ("#[1,2,3]", 3), ("#[\"x\",\"y\"]", 2)] {
            let (_, value) = convert_and_run(source).expect(source);
            assert_eq!(value, Value::Integer(expected.into()), "source: {source}");
        }

        let error = convert_and_run("#[1,true]").expect_err("mixed list stays ill typed");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(
            error.message,
            "No common type found between components of list expression: { int, bool }"
        );

        let source = SourceText::new("#[1,2,3]");
        let program = parse(&source).expect("list cardinality parses");
        let table = TypeTable::new();
        let globals = IdTable::new();
        let overloads = OverloadState::default();
        let analysis = Analysis::new(&table, &globals, &overloads);
        let mut required = Type::Undetermined;
        let typed = convert_expr(&program.expressions[0], &mut required, &analysis)
            .expect("list cardinality converts");
        assert_eq!(
            typed
                .evaluate(&mut EvaluationContext::new(), Level::NoValue)
                .expect("no-value cardinality has no work that can fail"),
            None
        );

        let mut context = TypedContext::new();
        let listing = context
            .execute(&command("whattype # ?"))
            .expect("list ordinary # overloads");
        assert!(matches!(
            &listing[..],
            [TypedCommandEvent::ReportLine { text, .. }]
                if !text.contains("[*]->int")
        ));
        let show_all = context
            .execute(&command("showall"))
            .expect("show all ordinary overloads");
        assert!(matches!(
            &show_all[..],
            [TypedCommandEvent::ReportLine { text, .. }]
                if !text.contains("#: ([*]->int)")
        ));
        context
            .execute(&command("set # ([bool] xs) = 99"))
            .expect("install exact row overload");
        let events = context
            .execute(&command("#[true,false]"))
            .expect("exact user overload preempts generic row cardinality");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::Value {
                value: Value::Integer(value),
                ..
            }] if value == &BigInt::from(99)
        ));
        context
            .execute(&command("set # (ratvec xs) = 88"))
            .expect("install coercible ordinary overload");
        let events = context
            .execute(&command("#[1,2]"))
            .expect("generic row primitive preempts coercible ordinary overload");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::Value {
                value: Value::Integer(value),
                ..
            }] if value == &BigInt::from(2)
        ));
    }

    #[test]
    fn p3_param_twist_signatures_and_no_value_policies_match_upstream() {
        let param = primitive_type(Prim::Param);
        let matrix = primitive_type(Prim::Mat);
        let unary = builtin_registry()
            .iter()
            .find(|builtin| builtin.name == "twist" && builtin.arg_type == param)
            .expect("missing twist(Param)");
        assert_eq!(unary.result, param);
        assert_eq!(unary.hunger, 3);
        assert!(matches!(
            unary.implementation,
            BuiltinImpl::Domain {
                no_value: DomainNoValue::Skip,
                ..
            }
        ));

        let binary_arguments = Type::tuple(vec![param.clone(), matrix]);
        let binary = builtin_registry()
            .iter()
            .find(|builtin| builtin.name == "twist" && builtin.arg_type == binary_arguments)
            .expect("missing twist(Param,mat)");
        assert_eq!(binary.result, param);
        assert_eq!(binary.hunger, 1);
        assert!(matches!(
            binary.implementation,
            BuiltinImpl::Domain {
                no_value: DomainNoValue::Validate,
                ..
            }
        ));

        let datum = "simply_connected(Lie_type(\"A2\"),true)";
        let inner = format!("inner_class({datum},[[0,1],[1,0]])");
        let real = format!("real_form({inner},0)");
        let parameter = format!("param(KGB({real},1),[0,0],[0,0]/1)");

        let (_, value) = convert_and_run(&format!("begin twist({parameter});7 end"))
            .expect("discarded unary twist skips its result");
        assert_eq!(value, Value::Integer(BigInt::from(7)));

        let error = convert_and_run(&format!("begin twist({parameter},[[1]]);7 end"))
            .expect_err("discarded outer twist still validates the matrix");
        assert_eq!(
            error.message,
            "Involution should be a 2x2 matrix; received a 1x1 matrix"
        );
    }

    #[test]
    fn timed_full_deform_signature_and_no_value_policy_match_upstream() {
        let arguments = Type::tuple(vec![primitive_type(Prim::Param), int_type()]);
        let builtin = builtin_registry()
            .iter()
            .find(|builtin| builtin.name == "full_deform" && builtin.arg_type == arguments)
            .expect("missing full_deform(Param,int)");
        assert_eq!(
            builtin.result,
            Type::Union(vec![Type::void(), primitive_type(Prim::KTypePol)])
        );
        assert_eq!(builtin.hunger, 0);
        assert!(matches!(
            builtin.implementation,
            BuiltinImpl::Domain {
                no_value: DomainNoValue::Validate,
                ..
            }
        ));
    }

    #[test]
    fn p3_undefined_twist_values_support_strict_relations() {
        let datum = "simply_connected(Lie_type(\"A3\"),true)";
        let identity = "[[1,0,0],[0,1,0],[0,0,1]]";
        let anti_diagonal = "[[0,0,1],[0,1,0],[1,0,0]]";
        let inner = format!("inner_class({datum},{identity})");
        let real = format!("real_form({inner},0)");
        let element = format!("twist(KGB({real},0),{anti_diagonal})");
        let (_, equal) = convert_and_run(&format!("{element}={element}"))
            .expect("undefined KGB relations compare the sentinel structurally");
        assert_eq!(equal, Value::Boolean(true));

        let parameter = format!("param(KGB({real},0),[0,0,0],[0,0,0]/1)");
        let undefined = format!("twist({parameter},{anti_diagonal})");
        let (_, equal) = convert_and_run(&format!("{undefined}={undefined}"))
            .expect("undefined Param relations compare cached state structurally");
        assert_eq!(equal, Value::Boolean(true));
    }

    #[test]
    fn p2_block_graph_values_match_the_a1_oracle_contract() {
        let datum = "simply_connected(Lie_type(\"A1\"),true)";
        let inner = format!("inner_class({datum},[[1]])");
        let real = format!("real_form({inner},1)");
        let dual = format!("dual_real_form({inner},1)");
        let block = format!("block({real},{dual})");
        let (_, graph) = convert_and_run(&format!("W_graph({block})")).expect("block W-graph");
        assert_eq!(
            graph.to_string(),
            "[([],[(2,1)]),([],[(2,1)]),([0],[(0,1),(1,1)])]"
        );
        let (_, cells) = convert_and_run(&format!("W_cells({block})")).expect("block W-cells");
        assert_eq!(
            cells.to_string(),
            "[([0],[([],[])]),([1],[([],[])]),([2],[([0],[])])]"
        );
        for (call, expected) in [("W_graph", 7), ("W_cells", 8)] {
            let (_, value) = convert_and_run(&format!("begin {call}({block});{expected} end"))
                .expect("discarded Block graph call still completes");
            assert_eq!(value, Value::Integer(BigInt::from(expected)));
        }
    }

    #[test]
    fn p2_block_param_returns_survivors_and_start_index() {
        let datum = "simply_connected(Lie_type(\"A1\"),true)";
        let inner = format!("inner_class({datum},[[1]])");
        let real = format!("real_form({inner},1)");
        let parameter = format!("param(KGB({real},1),[0],[1]/2)");
        let (_, value) = convert_and_run(&format!("block({parameter})"))
            .expect("full-integral Param block should evaluate");
        assert_eq!(
            value.to_string(),
            "([final parameter(x=0,lambda=[1]/1,nu=[0]/1),final parameter(x=1,lambda=[1]/1,nu=[0]/1),final parameter(x=2,lambda=[1]/1,nu=[1]/1)],1)"
        );

        let (_, value) = convert_and_run(&format!("begin block({parameter});7 end"))
            .expect("discarded Param block should still validate and continue");
        assert_eq!(value, Value::Integer(BigInt::from(7)));
    }

    #[test]
    fn p0_simple_domain_signatures_accept_and_reject_oracle_inputs() {
        let root_datum = "simply_connected(Lie_type(\"A1\"),true)";
        let inner_class = format!("inner_class({root_datum},[[1]])");
        let real_form = format!("real_form({inner_class},1)");
        let param = format!("param(KGB({real_form},1),[0],[1]/2)");
        let accepted = [
            format!("from_dominant({root_datum},vec: [3])"),
            format!("from_dominant(vec: [3],{root_datum})"),
            format!("Cartan_info(Cartan_class({inner_class},0))"),
            format!("KL_block({param})"),
            format!("KL_column({param})"),
            format!("cross(0,{param})"),
            format!("Cayley(0,{param})"),
        ];
        for source in accepted {
            convert_and_run(&source)
                .unwrap_or_else(|error| panic!("{source} should be accepted: {error:?}"));
        }

        let rejected = [
            (
                format!("from_dominant({root_datum},mat: [[1]])"),
                "Failed to match 'from_dominant' with argument type (RootDatum,mat)",
            ),
            (
                format!("Cartan_info({root_datum})"),
                "found RootDatum while CartanClass was needed.",
            ),
            (
                format!("KL_block({real_form})"),
                "found RealForm while Param was needed.",
            ),
            (
                format!("KL_column({real_form})"),
                "found RealForm while Param was needed.",
            ),
            (
                format!("cross(1,{param})"),
                "Illegal simple reflection: 1, should be <1",
            ),
            (
                format!("Cayley(1,{param})"),
                "Illegal simple reflection: 1, should be <1",
            ),
            (
                format!("cross(-1,{param})"),
                "Illegal simple reflection: -1, should be <1",
            ),
            (
                format!("Cayley(999999999999999999999999,{param})"),
                "Integer value to big for conversion",
            ),
        ];
        for (source, expected) in rejected {
            assert_eq!(
                convert_and_run(&source)
                    .expect_err("oracle-rejected input")
                    .message,
                expected,
                "source: {source}"
            );
        }
    }

    #[test]
    fn parameter_cross_uses_the_integral_subsystem_generator() {
        let root_datum = "simply_connected(Lie_type(\"B2\"),true)";
        let inner_class = format!("inner_class({root_datum},[[1,0],[0,1]])");
        let real_form = format!("real_form({inner_class},1)");
        let parameter = format!("param(KGB({real_form},2),[-1,0],[1,3]/2)");

        let (_, crossed) =
            convert_and_run(&format!("cross(0,{parameter})")).expect("integral-root cross");
        assert_eq!(
            crossed.to_string(),
            "final parameter(x=3,lambda=[1,1]/1,nu=[3,0]/4)"
        );

        let (_, cayley) =
            convert_and_run(&format!("Cayley(0,{parameter})")).expect("integral-root Cayley");
        assert_eq!(
            cayley.to_string(),
            "non-dominant parameter(x=2,lambda=[0,1]/1,nu=[-3,6]/4)"
        );
    }

    #[test]
    fn arbitrary_root_parameter_transforms_match_the_a2_oracle() {
        let rd = "simply_connected(Lie_type(\"A2\"),true)";
        let ic = format!("inner_class({rd},[[1,0],[0,1]])");
        let rf = format!("real_form({ic},1)");
        let p = format!("param(KGB({rf},5),[0,0],[0,0]/1)");
        let cases = [
            (
                format!("cross(root({rd},0),{p})"),
                "non-final parameter(x=3,lambda=[-1,2]/1,nu=[0,0]/1)",
            ),
            (format!("Cayley(root({rd},0),{p})={p}"), "true"),
            (
                format!("cross(root({rd},0)+root({rd},1),{p})"),
                "non-final parameter(x=5,lambda=[1,1]/1,nu=[0,0]/1)",
            ),
            (
                format!("Cayley(root({rd},0)+root({rd},1),{p})"),
                "zero parameter(x=1,lambda=[0,0]/1,nu=[0,0]/1)",
            ),
            (
                format!("cross(-root({rd},0),{p})"),
                "non-final parameter(x=3,lambda=[-1,2]/1,nu=[0,0]/1)",
            ),
            (
                format!("Cayley(-root({rd},0),{p})"),
                "non-final parameter(x=5,lambda=[1,1]/1,nu=[0,0]/1)",
            ),
        ];
        for (expression, expected) in cases {
            let (_, value) = convert_and_run(&expression)
                .unwrap_or_else(|error| panic!("{expression}: {error:?}"));
            assert_eq!(value.to_string(), expected, "{expression}");
        }
    }

    #[test]
    fn arbitrary_root_cayley_transports_a_noncommuting_dominance_word() {
        let rd = "simply_connected(Lie_type(\"A3\"),true)";
        let ic = format!("inner_class({rd},[[1,0,0],[0,1,0],[0,0,1]])");
        let rf = format!("real_form({ic},1)");
        let p = format!("param(KGB({rf},7),[0,0,0],[-2,-2,-2]/1)");
        let (_, value) = convert_and_run(&format!("Cayley([-1,1,1],{p})"))
            .expect("A3 Cayley after the noncommuting [1,2,1] dominance word");
        assert_eq!(
            value.to_string(),
            "final parameter(x=1,lambda=[0,2,2]/1,nu=[0,0,0]/1)"
        );
    }

    #[test]
    fn arbitrary_root_parameter_transforms_keep_oracle_diagnostics() {
        let rd = "simply_connected(Lie_type(\"A2\"),true)";
        let ic = format!("inner_class({rd},[[1,0],[0,1]])");
        let rf = format!("real_form({ic},1)");
        let p = format!("param(KGB({rf},5),[0,0],[0,0]/1)");
        let q = format!("param(KGB({rf},5),[0,0],[1,0]/2)");
        for (expression, expected) in [
            (format!("cross(root({rd},0),{q})"), "Not an integral root"),
            (format!("Cayley(root({rd},0),{q})"), "Not an integral root"),
            (format!("cross([2],{p})"), "Not a root"),
            (format!("Cayley([2],{p})"), "Not an integral root"),
        ] {
            let error = convert_and_run(&expression).expect_err("oracle-rejected root transform");
            assert_eq!(error.message, expected, "{expression}");
        }

        let a1 = "simply_connected(Lie_type(\"A1\"),true)";
        let inner = format!("inner_class({a1},[[1]])");
        let real = format!("real_form({inner},1)");
        let nonstandard = format!("param(KGB({real},1),[-2],[0]/1)");
        let error = convert_and_run(&format!("Cayley([2,3],{nonstandard})"))
            .expect_err("Cayley makes dominant before rejecting its root argument");
        assert_eq!(
            error.message,
            "Cannot make non-standard parameter integrally dominant"
        );
    }

    #[test]
    fn arbitrary_root_parameter_signatures_use_skip_no_value_policy() {
        let vector_param =
            Type::tuple(vec![primitive_type(Prim::Vec), primitive_type(Prim::Param)]);
        for name in ["cross", "Cayley"] {
            let builtin = builtin_registry()
                .iter()
                .find(|builtin| builtin.name == name && builtin.arg_type == vector_param)
                .unwrap_or_else(|| panic!("missing {name}(vec,Param)"));
            assert_eq!(builtin.result, primitive_type(Prim::Param));
            assert!(matches!(
                builtin.implementation,
                BuiltinImpl::Domain {
                    no_value: DomainNoValue::Skip,
                    ..
                }
            ));
            assert_eq!(builtin.hunger, 2);
        }
    }

    #[test]
    fn arbitrary_root_parameter_transforms_reject_undefined_sources_safely() {
        let datum = "simply_connected(Lie_type(\"A3\"),true)";
        let inner = format!("inner_class({datum},[[1,0,0],[0,1,0],[0,0,1]])");
        let real = format!("real_form({inner},0)");
        let parameter = format!("param(KGB({real},0),[0,0,0],[0,0,0]/1)");
        let undefined = format!("twist({parameter},[[0,0,1],[0,1,0],[1,0,0]])");
        for name in ["cross", "Cayley"] {
            let error = convert_and_run(&format!("{name}(root({datum},0),{undefined})"))
                .expect_err("graph-dependent transform rejects UndefKGB");
            assert!(
                error.message.contains("undefined parameter operation"),
                "{name}: {}",
                error.message
            );
        }
    }

    #[test]
    fn vector_addition_is_elementwise_and_validates_size_before_no_value() {
        let (type_, value) = convert_and_run("[1,2]+[3,4]").expect("vec addition");
        assert_eq!(type_, primitive_type(Prim::Vec));
        assert_eq!(value, Value::Vector(Vec32(vec![4, 6])));

        let error = convert_and_run("[1]+[2,3]").expect_err("mismatched vectors");
        assert_eq!(error.message, "Size mismatch 1:2");

        let error = convert_and_run("begin [1]+[2,3];7 end")
            .expect_err("size is checked before no-value gate");
        assert_eq!(error.message, "Size mismatch 1:2");

        let (_, value) =
            convert_and_run("begin [1,2]+[3,4];7 end").expect("discarded matching vectors");
        assert_eq!(value, Value::Integer(BigInt::from(7)));
    }

    #[test]
    fn vector_unary_negation_is_elementwise_and_skips_at_no_value() {
        let (type_, value) = convert_and_run("-[1,-2]").expect("vec negation");
        assert_eq!(type_, primitive_type(Prim::Vec));
        assert_eq!(value, Value::Vector(Vec32(vec![-1, 2])));

        let (_, value) = convert_and_run("begin -[1,-2];7 end").expect("discarded vec negation");
        assert_eq!(value, Value::Integer(BigInt::from(7)));

        let builtin = builtin_registry()
            .iter()
            .find(|builtin| builtin.name == "-" && builtin.arg_type == primitive_type(Prim::Vec))
            .expect("-(vec) registry entry");
        assert_eq!(builtin.result, primitive_type(Prim::Vec));
        assert_eq!(builtin.hunger, 3);
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
                pair(primitive_type(Prim::Vec)),
                pair(primitive_type(Prim::RatVec)),
                Type::tuple(vec![primitive_type(Prim::Mat), int_type()]),
                Type::tuple(vec![int_type(), primitive_type(Prim::Mat)]),
                pair(primitive_type(Prim::Mat)),
                pair(primitive_type(Prim::Split)),
                Type::tuple(vec![
                    primitive_type(Prim::KTypePol),
                    primitive_type(Prim::KType),
                ]),
                Type::tuple(vec![
                    primitive_type(Prim::KTypePol),
                    Type::row(Type::tuple(vec![
                        primitive_type(Prim::Split),
                        primitive_type(Prim::KType),
                    ])),
                ]),
                Type::tuple(vec![
                    primitive_type(Prim::KTypePol),
                    primitive_type(Prim::KTypePol),
                ]),
                Type::tuple(vec![
                    primitive_type(Prim::KTypePol),
                    Type::tuple(vec![
                        primitive_type(Prim::Split),
                        primitive_type(Prim::KType),
                    ]),
                ]),
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    primitive_type(Prim::Param),
                ]),
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    Type::tuple(vec![
                        primitive_type(Prim::Split),
                        primitive_type(Prim::Param),
                    ]),
                ]),
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    Type::row(Type::tuple(vec![
                        primitive_type(Prim::Split),
                        primitive_type(Prim::Param),
                    ])),
                ]),
                Type::tuple(vec![
                    primitive_type(Prim::ParamPol),
                    primitive_type(Prim::ParamPol),
                ]),
            ]
        );
        assert_eq!(
            plus.iter()
                .map(|&index| builtin_registry()[index].hunger)
                .collect::<Vec<_>>(),
            vec![1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
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

        let error = convert_and_run(
            "begin KL_block(param(KGB(real_form(inner_class(simply_connected(\
             Lie_type(\"A1\"),true),[[1]]),1),1),[-2],[0]/1));7 end",
        )
        .expect_err("KL_block validates standardness before its no-value gate");
        assert_eq!(
            error.message,
            "KL_block requires a standard parameter:\n  \
             non-standard parameter(x=1,lambda=[-1]/1,nu=[0]/1)\n  \
             Parameter not standard"
        );

        let error = convert_and_run(
            "begin from_dominant(simply_connected(Lie_type(\"A1\"),true),[1,2]);7 end",
        )
        .expect_err("from_dominant checks rank before its no-value gate");
        assert_eq!(error.message, "Rank and weight size mismatch 1:2");

        let error = convert_and_run(
            "begin KL_column(param(KGB(real_form(inner_class(simply_connected(\
             Lie_type(\"A1\"),true),[[1]]),1),1),[-2],[0]/1));7 end",
        )
        .expect_err("KL_column validates standardness before its no-value gate");
        assert_eq!(
            error.message,
            "Cannot compute Kazhdan-Lusztig column:\n  \
             non-standard parameter(x=1,lambda=[-1]/1,nu=[0]/1)\n  \
             Parameter not standard"
        );

        let (_, value) = convert_and_run(
            "begin Cartan_info(Cartan_class(inner_class(simply_connected(\
             Lie_type(\"A1\"),true),[[1]]),0));7 end",
        )
        .expect("Cartan_info skips all work at no-value");
        assert_eq!(value, Value::Integer(BigInt::from(7)));
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
    fn null_matrix_preserves_shape_and_checks_dimensions_before_no_value() {
        let (type_, value) = convert_and_run("null(0,2)").expect("zero-row matrix");
        assert_eq!(type_, primitive_type(Prim::Mat));
        assert_eq!(value.to_string(), "The 0x2 matrix");

        let (_, value) = convert_and_run("begin null(2,3);7 end")
            .expect("discarded construction validates without allocating its value");
        assert_eq!(value, Value::Integer(BigInt::from(7)));

        for (source, expected) in [
            (
                "begin null(-1,0);7 end",
                "Negative integer where unsigned is required",
            ),
            (
                "begin null(4294967296,0);7 end",
                "Number of rows 4294967296 exceeds implementation limit",
            ),
            (
                "begin null(0,4294967296);7 end",
                "Number of columns 4294967296 exceeds implementation limit",
            ),
        ] {
            assert_eq!(
                convert_and_run(source)
                    .expect_err("invalid matrix dimension is rejected")
                    .message,
                expected,
                "source: {source}"
            );
        }
    }

    #[test]
    fn involution_decomposition_surface_matches_the_frozen_a2_contract() {
        let (type_, value) =
            convert_and_run("classify_involution([[0,-1],[-1,0]])").expect("classification");
        assert_eq!(type_, Type::tuple(vec![int_type(), int_type(), int_type()]));
        assert_eq!(value.to_string(), "(0,1,0)");

        let (type_, value) = convert_and_run(
            "twisted_involution(simply_connected(Lie_type(\"A2\"),true),[[0,-1],[-1,0]])",
        )
        .expect("A2 opposition decomposition");
        assert_eq!(
            type_,
            Type::tuple(vec![
                primitive_type(Prim::WeylElt),
                primitive_type(Prim::InnerClass),
            ])
        );
        assert_eq!(
            value.to_string(),
            "(<0.1.0>,Complex reductive group of type A2, with involution defining\n\
             inner class of type 'c', with 2 real forms and 1 dual real form)"
        );

        let (_, value) = convert_and_run(
            "distinguished_involution(inner_class(simply_connected(Lie_type(\"A2\"),true),[[0,-1],[-1,0]]))",
        )
        .expect("distinguished A2 involution");
        assert_eq!(value.to_string(), "\n| 1, 0 |\n| 0, 1 |\n");
    }

    #[test]
    fn involution_decomposition_diagnostics_and_no_value_gates_are_exact() {
        let cases = [
            (
                "classify_involution([[1],[0]])",
                "Involution should be a 1x1 matrix; received a 1x2 matrix",
            ),
            (
                "classify_involution([[2,0],[0,2]])",
                "Given transformation is not an involution",
            ),
            (
                "classify_involution([[1,0],[0]])",
                "Vector sizes differ in conversion to matrix",
            ),
            (
                "twisted_involution(simply_connected(Lie_type(\"A2\"),true),[[1]])",
                "Involution should be a 2x2 matrix; received a 1x1 matrix",
            ),
            (
                "twisted_involution(simply_connected(Lie_type(\"B2\"),true),[[1,2],[0,-1]])",
                "Matrix maps simple root 0 to non-root",
            ),
        ];
        for (source, expected) in cases {
            assert_eq!(
                convert_and_run(source)
                    .expect_err("fixture is rejected")
                    .message,
                expected,
                "source: {source}"
            );
        }

        let type_cases = [
            (
                "classify_involution([1])",
                "found [int] while mat was needed.",
            ),
            (
                "twisted_involution(simply_connected(Lie_type(\"A1\"),true),[1])",
                "found (RootDatum,[int]) while (RootDatum,mat) was needed.",
            ),
            (
                "distinguished_involution(simply_connected(Lie_type(\"A1\"),true))",
                "found RootDatum while InnerClass was needed.",
            ),
        ];
        for (source, expected) in type_cases {
            assert_eq!(
                convert_and_run(source)
                    .expect_err("overload does not match")
                    .message,
                expected,
                "source: {source}"
            );
        }

        let error = convert_and_run("begin classify_involution([[2]]);7 end")
            .expect_err("classifier checks M squared before no_value");
        assert_eq!(error.message, "Given transformation is not an involution");
        let error = convert_and_run(
            "begin twisted_involution(simply_connected(Lie_type(\"B2\"),true),[[1,2],[0,-1]]);7 end",
        )
        .expect_err("twisted constructor validates roots before no_value");
        assert_eq!(error.message, "Matrix maps simple root 0 to non-root");
        let (_, value) = convert_and_run(
            "begin twisted_involution(simply_connected(Lie_type(\"A1\"),true),[[1]]);7 end",
        )
        .expect("a valid discarded constructor does not assemble its result pair");
        assert_eq!(value, Value::Integer(BigInt::from(7)));
    }

    #[test]
    fn twisted_family_overload_mismatch_wordings_are_exact() {
        // The oracle-pinned type diagnostics of
        // tests/fixtures/domain/twisted_family_rejected.atlas (job
        // 3536421) and block_deform_rejected.atlas (job 3536583): a
        // single-variant builtin reports "found … while … was needed.",
        // while the multi-variant names (twisted_full_deform carries the
        // timed second overload, twisted_KL_sum_at_s the external-delta
        // overload) report "Failed to match …".
        let rf_a1 = "real_form(inner_class(simply_connected(Lie_type(\"A1\"),true),[[1]]),1)";
        let rf_a2 =
            "real_form(inner_class(simply_connected(Lie_type(\"A2\"),true),[[1,0],[0,1]]),1)";
        let p_a1 = format!("param(KGB({rf_a1},0),[1],[0]/1)");
        let p_a2 = format!("param(KGB({rf_a2},3),[0,0],[1,1]/1)");
        let d_a2 = format!("deform({p_a2})");
        let cases = [
            (
                format!("twisted_deform({rf_a1})"),
                "found RealForm while Param was needed.".to_string(),
            ),
            (
                format!("twisted_full_deform({rf_a1})"),
                "Failed to match 'twisted_full_deform' with argument type RealForm".to_string(),
            ),
            (
                format!("twisted_KL_sum_at_s({p_a1},{rf_a1})"),
                "Failed to match 'twisted_KL_sum_at_s' with argument type (Param,RealForm)"
                    .to_string(),
            ),
            (
                format!("block_deform({p_a2},{d_a2})"),
                "found (Param,ParamPol) while (Param,ParamPol,int) was needed.".to_string(),
            ),
            (
                format!("block_deform({rf_a2},{d_a2},0)"),
                "found (RealForm,ParamPol,int) while (Param,ParamPol,int) was needed.".to_string(),
            ),
        ];
        for (source, expected) in cases {
            assert_eq!(
                convert_and_run(&source)
                    .expect_err("overload does not match")
                    .message,
                expected,
                "source: {source}"
            );
        }
    }

    #[test]
    fn involution_primitive_surface_matches_the_frozen_contract() {
        // The accepted fixture's eleven anchors (domain/involution_primitive).
        let cases = [
            ("involution(Lie_type(\"A1\"),[0],\"c\")", "\n| 1 |\n"),
            ("involution(Lie_type(\"A1\"),[0],\"s\")", "\n| 1 |\n"),
            (
                "involution(Lie_type(\"A2\"),[0,1],\"c\")",
                "\n| 1, 0 |\n| 0, 1 |\n",
            ),
            (
                "involution(Lie_type(\"A2\"),[1,0],\"s\")",
                "\n| 0, 1 |\n| 1, 0 |\n",
            ),
            (
                "involution(Lie_type(\"A1.A1\"),[0,1],\"C\")",
                "\n| 0, 1 |\n| 1, 0 |\n",
            ),
            (
                "involution(Lie_type(\"B2\"),[0,1],\"s\")",
                "\n| 1, 0 |\n| 0, 1 |\n",
            ),
            (
                "involution(Lie_type(\"A2\"),[0,1],\"s\")",
                "\n| 0, 1 |\n| 1, 0 |\n",
            ),
            (
                "involution(Lie_type(\"A2\"),[0,1],\"u\")",
                "\n| 0, 1 |\n| 1, 0 |\n",
            ),
            ("involution(Lie_type(\"A1\"),[[2]],\"c\")", "\n| 1 |\n"),
            ("involution(Lie_type(\"A1\"),[[1]],\"s\")", "\n| 1 |\n"),
            (
                "involution(Lie_type(\"A2\"),[[1,1],[0,1]],\"s\")",
                "\n| 1,  1 |\n| 0, -1 |\n",
            ),
        ];
        for (source, expected) in cases {
            let (type_, value) =
                convert_and_run(source).unwrap_or_else(|error| panic!("{source}: {error:?}"));
            assert_eq!(type_, primitive_type(Prim::Mat), "source: {source}");
            assert_eq!(value.to_string(), expected, "source: {source}");
        }
    }

    #[test]
    fn involution_primitive_diagnostics_and_no_value_gates_are_exact() {
        // The rejected fixture's six diagnostics (domain/involution_primitive_rejected).
        let cases = [
            (
                "involution(Lie_type(\"A1\"),[1],\"c\")",
                "Permutation entry 1 too big",
            ),
            (
                "involution(Lie_type(\"A2\"),[0,1],\"ec\")",
                "Too many inner class symbols",
            ),
            (
                "involution(Lie_type(\"A2\"),[0],\"c\")",
                "Permutation size 1 does not match rank 2 of Lie type",
            ),
            (
                "involution(Lie_type(\"A1\"),[0],\"x\")",
                "Unknown inner class symbol `x'",
            ),
            (
                "involution(Lie_type(\"A1.A2\"),[0,1,2],\"Cs\")",
                "Complex inner class needs two identical consecutive types",
            ),
            (
                "involution(Lie_type(\"A2\"),[[1,0],[0,2]],\"s\")",
                "Inner class is not compatible with given lattice",
            ),
        ];
        for (source, expected) in cases {
            assert_eq!(
                convert_and_run(source)
                    .expect_err("fixture line is rejected")
                    .message,
                expected,
                "source: {source}"
            );
        }

        // basic_involution_wrapper's size check precedes its no-value gate;
        // the letter and permutation-entry checks are suppressed with it.
        let error = convert_and_run("begin involution(Lie_type(\"A2\"),[0],\"c\");7 end")
            .expect_err("the size check fires in a no-value context");
        assert_eq!(
            error.message,
            "Permutation size 1 does not match rank 2 of Lie type"
        );
        let (_, value) = convert_and_run("begin involution(Lie_type(\"A1\"),[1],\"c\");7 end")
            .expect("the permutation-entry check follows the no-value gate");
        assert_eq!(value, Value::Integer(BigInt::from(7)));
        let (_, value) = convert_and_run("begin involution(Lie_type(\"A1\"),[0],\"x\");7 end")
            .expect("the letter check follows the no-value gate");
        assert_eq!(value, Value::Integer(BigInt::from(7)));

        // based_involution_wrapper has no early gate: the letters and the
        // lattice compatibility fire in a no-value context too.
        let error = convert_and_run("begin involution(Lie_type(\"A1\"),[[1]],\"x\");7 end")
            .expect_err("the based wrapper checks letters in a no-value context");
        assert_eq!(error.message, "Unknown inner class symbol `x'");
        let error = convert_and_run("begin involution(Lie_type(\"A2\"),[[1,0],[0,2]],\"s\");7 end")
            .expect_err("the based wrapper checks the lattice in a no-value context");
        assert_eq!(
            error.message,
            "Inner class is not compatible with given lattice"
        );
    }

    #[test]
    fn matrix_conversion_size_diagnostics_follow_the_source_route() {
        let span = SourceText::new("").span(0, 0);
        let vector_columns = Value::List(vec![
            Value::Vector(Vec32(vec![1, 0])),
            Value::Vector(Vec32(vec![0])),
        ]);
        let error = apply_conversion("M[V]", vector_columns, span)
            .expect_err("vector columns have unequal sizes");
        assert!(matches!(
            error,
            Control::Runtime(Diagnostic { message, .. })
                if message == "Vector sizes differ in conversion to matrix"
        ));

        let integer_columns = Value::List(vec![
            Value::List(vec![Value::Integer(1.into()), Value::Integer(0.into())]),
            Value::List(vec![Value::Integer(0.into())]),
        ]);
        let error = apply_conversion("M[[I]]", integer_columns, span)
            .expect_err("integer lists have unequal sizes");
        assert!(matches!(
            error,
            Control::Runtime(Diagnostic { message, .. })
                if message == "List sizes differ in conversion to matrix"
        ));

        for (tag, expected) in [
            (
                "M[V]",
                "Implicit conversion to matrix for an empty set of vectors",
            ),
            ("M[[I]]", "Cannot convert empty list of lists to matrix"),
        ] {
            let error = apply_conversion(tag, Value::List(Vec::new()), span)
                .expect_err("empty conversion reports its source route");
            assert!(matches!(
                error,
                Control::Runtime(Diagnostic { message, .. })
                    if message == expected
            ));
        }
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
        assert_eq!(error.message, "found (int,int) while bool was needed.");
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
            ("while true; 1 + 1; dont od", "[]"),
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
