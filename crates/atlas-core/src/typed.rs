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
use crate::formula::FormulaOperator;
use crate::frames::{EvaluationContext, Frame, GlobalCell, SharedValue};
use crate::linear_values::{Matrix, RatVec, Vec32};
use crate::matreduc;
use crate::ratfast;
use crate::syntax::{
    compact_expression, compact_pattern, Command, ComponentAssignmentExpr, ComponentTransformExpr,
    Expr, FieldAssignmentExpr, FieldTransformExpr, ForLoop, LambdaParam, LetBinding,
    MultiAssignmentExpr, Pattern, SpannedValue, TypeExpr, TypeSpec,
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
    Return(SharedValue),
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

/// The resolved destination of a component or field assignment (the whole
/// assignment family shares the layer lookup of axis.w:6863+): locals by
/// lexical coordinates, globals by the cell captured at analysis time.
#[derive(Clone, Debug, PartialEq)]
pub enum AssignTarget {
    Local { depth: usize, offset: usize },
    Global(GlobalCell),
}

/// The resolved operation of a transform assignment `a[i] op:= v` /
/// `p.f op:= v` (axis.w:8268+): the builtin the overload resolution found,
/// or a user overload's closure (upstream reassembles an ordinary call for
/// user operations, which is observably the desugared application).
#[derive(Clone, Debug, PartialEq)]
pub enum TransformOperation {
    Builtin(usize),
    Closure(Rc<Closure>),
}

/// A typed executable expression.
#[derive(Clone, Debug, PartialEq)]
pub enum TypedExpr {
    /// A literal value built once at analysis time and shared per
    /// evaluation (an `Rc` bump instead of a value copy).
    Denotation(SharedValue),
    /// `$` captured at analysis time: evaluates to the snapshotted last
    /// value like a denotation, but the verbose trace prints the oracle's
    /// `(type:$)` spelling (axis.w:596-602), not the value itself.
    CapturedLastValue {
        value: Value,
        type_display: String,
    },
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
    /// `a[i] := v` (axis.w:8131-8192 `comp_ass_stat`): evaluates the value,
    /// then the index, then replaces the component and yields the value.
    ComponentAssignment {
        target: AssignTarget,
        /// The variable name, quoted by the uninitialized diagnostic.
        name: String,
        index: Box<TypedExpr>,
        reversed: bool,
        value: Box<TypedExpr>,
        /// Compact rendering of the source expression, quoted by the
        /// out-of-range diagnostic exactly like the oracle's `range_mess`
        /// prints the assignment node (axis.w:7953).
        source: String,
        span: SourceSpan,
    },
    /// `a[i] op:= v` (axis.w:8495+ `comp_trans_stat`): evaluates the right
    /// operand, then the index, then applies the resolved operation to the
    /// old component and writes the result back, yielding that result.
    ComponentTransform {
        target: AssignTarget,
        name: String,
        index: Box<TypedExpr>,
        reversed: bool,
        operation: TransformOperation,
        rhs: Box<TypedExpr>,
        /// Result coercion of the operator call, applied before write-back.
        conversion: Option<&'static str>,
        /// Compact of the synthetic read `a[i]` (pair indices without the
        /// tuple parentheses, `M[5,0]`): the vec/mat transform range check
        /// fires on this READ, so the oracle quotes the selection
        /// ("in subscription v[5]", "in matrix column selection M[5]").
        selection: String,
        source: String,
        span: SourceSpan,
    },
    /// `p.f := v` (axis.w:8194-8239 `field_ass_stat`).
    FieldAssignment {
        target: AssignTarget,
        name: String,
        position: usize,
        value: Box<TypedExpr>,
        span: SourceSpan,
    },
    /// `p.f op:= v` (axis.w:8286+ `field_trans_stat`).
    FieldTransform {
        target: AssignTarget,
        name: String,
        position: usize,
        operation: TransformOperation,
        rhs: Box<TypedExpr>,
        conversion: Option<&'static str>,
        span: SourceSpan,
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
        /// Column bounds of the two-dimensional form `M[rlo:rhi, clo:chi]`;
        /// both `None` for the one-dimensional row slice.
        column_lower: Option<Box<TypedExpr>>,
        column_upper: Option<Box<TypedExpr>>,
        flags: crate::syntax::SliceFlags,
        /// Compact rendering of the source expression, quoted by the
        /// out-of-range diagnostic exactly like the oracle's
        /// `slice_range_error` prints the slice node (axis.w:4299).
        source: String,
        span: SourceSpan,
    },
    /// `[a,b | c,d]` after conversion: every entry converted against `int`,
    /// the matrix built at evaluation (parser.y:370-376 desugars to the
    /// hidden `"transpose "` applied to a `mat` cast; here the construction
    /// is direct so user `^`/`mat` overloads cannot intercept it).
    BarList {
        rows: Vec<Vec<TypedExpr>>,
        span: SourceSpan,
    },
    LetGroup {
        /// One (shape, initializer) pair per binding in the group; each
        /// value distributes into the frame slots its shape describes.
        initializers: Vec<(SlotShape, TypedExpr)>,
        /// Frame slot names in bind order across the group, for the
        /// back-trace frame dump (axis.w:2896-2909).
        names: Rc<[String]>,
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
        /// The resolved `name@argtype` for the back-trace call line
        /// (axis.w:1647-1648 builds it at overload resolution).
        name: String,
        span: SourceSpan,
    },
    /// A top-level builtin RHS of a simple assignment whose hungry operand
    /// is exactly that assignment's destination. Evaluation moves the value
    /// out of the destination immediately before that operand is supplied.
    HungryBuiltinCall {
        builtin: usize,
        arguments: Vec<TypedExpr>,
        /// The resolved `name@argtype` for the back-trace call line.
        name: String,
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
        /// The lambda's source location (`defined at ...` in back-traces).
        span: SourceSpan,
        /// Frame slot names in bind order, for the back-trace frame dump.
        param_names: Rc<[String]>,
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
        /// The resolved `name@argtype` for the back-trace call line; `None`
        /// for a dynamically computed callee (upstream prints the callee
        /// expression there, axis.w:1913-1915).
        name: Option<String>,
        span: SourceSpan,
    },
    /// `first; second`: the first half evaluates for effects at `NoValue`;
    /// the sequence yields the second half's value.
    Sequence {
        first: Box<TypedExpr>,
        second: Box<TypedExpr>,
    },
    /// A while loop over a single do_expr body (axis.w:5373-5403): each
    /// iteration evaluates the body, then reads the while-condition flag
    /// the body set — `false` ends the loop without collecting. Completed
    /// iterations' values collect into a row. `out_reversed` is the tilde
    /// before `od` (parser.y:364, flags bit 1): the collected row is
    /// reversed. `yields_count` is the upstream int-context variant
    /// (axis.w:5424-5435, make_while_loop flag 0x8): the body is converted
    /// against void and the loop yields the number of iterations done.
    While {
        body: Box<TypedExpr>,
        out_reversed: bool,
        yields_count: bool,
    },
    /// A do_expr guard (axis.w:5476-5580 `do_expression` /
    /// `forever_expression` / `dont_expression`): the condition evaluates
    /// against bool; when false the flag is cleared and NO value is
    /// produced even at a value level, when true the body evaluates and
    /// the flag is set afterwards. `condition: None` is the
    /// constant-folded `forever` case (bare `do` or `true`).
    Do {
        condition: Option<Box<TypedExpr>>,
        body: Box<TypedExpr>,
    },
    /// A for loop over a row value; each iteration pushes the 0-based
    /// index slot when `index` is set, then distributes the element per
    /// `shape` (the upstream (index, pattern) pair wrap). `in_reversed`
    /// traverses the components in reverse (the index counting down from
    /// n-1); `out_reversed` reverse-collects the output row.
    For {
        shape: SlotShape,
        index: bool,
        /// Frame slot names in bind order (the index name, then the
        /// pattern leaves), for the back-trace frame dump
        /// (axis.w:6124-6161).
        names: Rc<[String]>,
        iterable: Box<TypedExpr>,
        in_reversed: bool,
        body: Box<TypedExpr>,
        out_reversed: bool,
    },
    /// `break N`, unwound through `levels + 1` enclosing loop boundaries
    /// (parser.y:385-386, axis.w:665 `loop_break(depth)`).
    Break {
        levels: usize,
    },
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
    /// iterations collecting each body value; with `name` the counter is
    /// bound (as a constant) in a per-iteration frame, increasing from the
    /// bound (default 0), or decreasing to it inclusive when `decreasing`.
    /// `in_reversed` (the count-side tilde, parser.y:550-567) likewise
    /// counts down from bound+count-1; `out_reversed` (the body-side
    /// tilde) reverse-collects the output row.
    CountedFor {
        name: Option<String>,
        decreasing: bool,
        in_reversed: bool,
        out_reversed: bool,
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
    /// The enclosing function's result-type cell (axis.w:313,342-365):
    /// `return e` converts `e` against THIS type regardless of the local
    /// context, and specialisations flow back into the function type.
    /// `None` outside function bodies (upstream `return_type==nullptr`).
    return_type: Option<TypeCell>,
    /// Number of enclosing loops: `break` is legal only when nonzero
    /// (mirrors `in_function`; upstream rejects a stray `break` during
    /// analysis, before anything evaluates).
    loop_depth: usize,
    /// The `$` captures (axis.w:573-582): the last top-level value and its
    /// type cell, snapshotted into a denotation at analysis time.
    last_value: &'a Value,
    last_value_type: &'a TypeCell,
}

impl<'a> Analysis<'a> {
    pub fn new(
        types: &'a TypeTable,
        globals: &'a IdTable,
        overloads: &'a OverloadState,
        last_value: &'a Value,
        last_value_type: &'a TypeCell,
    ) -> Self {
        Self {
            types,
            globals,
            overloads,
            locals: BTreeMap::new(),
            constant_locals: BTreeSet::new(),
            in_function: false,
            return_type: None,
            loop_depth: 0,
            last_value,
            last_value_type,
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
            return_type: self.return_type.clone(),
            loop_depth: self.loop_depth + 1,
            last_value: self.last_value,
            last_value_type: self.last_value_type,
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
    /// Merged per-name variant lists (see [`merged_variants`]), rebuilt only
    /// after a `set`/`forget` mutation. Call sites per session number in the
    /// thousands while the variant list for a name is fixed between
    /// mutations — upstream keeps ONE persistent table whose order is fixed
    /// at insertion time (global.w:1004-1023), so caching matches the oracle
    /// and removes the per-call rebuild (deep clones + `is_close` insert
    /// scans) that dominated type-checking the corpus scripts.
    merged_cache: std::cell::RefCell<std::collections::HashMap<String, Rc<Vec<MergedVariant>>>>,
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
        // Only THIS name's merged view changes; leave the other cached
        // names warm (a corpus script interleaves hundreds of `set`
        // commands with call sites of unrelated names).
        self.merged_cache.borrow_mut().remove(name);
        // A tabled function type (lazy_lists.at's inf_list) keeps its
        // name for the report but contributes its expansion's argument
        // type to overload matching.
        let arg_type = match &function_type {
            Type::Function(parts) => parts.0.clone(),
            Type::Tabled(number) => match types.expansion(*number) {
                Type::Function(parts) => parts.0.clone(),
                _ => unreachable!("tabled function type expands to a function"),
            },
            _ => unreachable!("only function-typed values enter the overload table"),
        };
        // Build the pre-mutation view directly: the caching wrapper would
        // re-populate the just-cleared cache with the stale list.
        let merged = build_merged_variants(name, self, types);
        let old_n = merged.len();
        let mut lower = 0;
        let mut upper = old_n;
        let mut replacement = None;
        for (slot, existing) in merged.iter().enumerate() {
            match crate::coercions::is_close(&arg_type, &existing.arg_type, types) {
                0x6 => lower = slot + 1,
                0x5 => upper = upper.min(slot),
                0x7 if arg_type.equals(&existing.arg_type, types) => {
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
        self.merged_cache.borrow_mut().remove(name);
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

/// Print a converted (typed) expression the way upstream prints its
/// converted expression tree (`expression_base::print`): denotations print
/// their value (axis.w:516-517), identifiers their name (axis.w:1255),
/// builtin calls `name@argtype(args)` (overloaded_call::print,
/// axis.w:2031-2036), dynamic calls `function(argument)` with the function
/// parenthesised unless an identifier and the argument unless a tuple
/// display (call_expression::print, axis.w:1925-1935), conditionals as
/// ` if c then t else e fi ` with `elif` chaining (axis.w:4754-4766), and
/// tuple/list displays as `(a,b)` / `[a,b]` (axis.w:761-768, 946-951).
/// Used by the verbose analysis trace (main.w:528-540 `Converted
/// expression:`), the dynamic-callee back-trace name
/// (call_expression::function_name, axis.w:1911-1913), and the closure
/// body print in frame dumps (print_lambda, axis.w:3045-3053). Node shapes
/// beyond these keep the `<expression>` fallback rather than claiming
/// oracle fidelity for unverified prints.
fn typed_expression_print(expression: &TypedExpr) -> String {
    match expression {
        TypedExpr::Denotation(value) => value.as_ref().to_string(),
        TypedExpr::CapturedLastValue { type_display, .. } => type_display.clone(),
        TypedExpr::GlobalIdent { name, .. } | TypedExpr::LocalIdent { name, .. } => name.clone(),
        TypedExpr::BuiltinCall {
            builtin: _,
            arguments,
            name,
            ..
        }
        | TypedExpr::HungryBuiltinCall {
            builtin: _,
            arguments,
            name,
            ..
        } => {
            if let Some(special) = special_int_unary_print(name, arguments) {
                return special;
            }
            let mut out = name.clone();
            out.push('(');
            for (index, argument) in arguments.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&typed_expression_print(argument));
            }
            out.push(')');
            out
        }
        TypedExpr::TupleDisplay(elements) => {
            let inner = elements
                .iter()
                .map(typed_expression_print)
                .collect::<Vec<_>>()
                .join(",");
            format!("({inner})")
        }
        TypedExpr::ListDisplay(elements) => {
            let inner = elements
                .iter()
                .map(typed_expression_print)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{inner}]")
        }
        TypedExpr::Conversion { inner, .. } => typed_expression_print(inner),
        TypedExpr::FunctionCall {
            function,
            argument,
            name,
            ..
        } => {
            // A call whose function slot holds a `set_type`-installed
            // projector closure is upstream's projector_call
            // (axis.w:4495-4532): it prints postfix, `argument.field`,
            // with the field name taken from the call-site trace name
            // (so a projector aliased under another `set` name prints
            // that name, like upstream's build_call(name)).
            if let TypedExpr::Denotation(value) = function.as_ref() {
                if let Value::Closure(closure) = value.as_ref() {
                    if matches!(closure.body.as_ref(), TypedExpr::TupleProject { .. }) {
                        if let Some(trace) = name {
                            let field = trace.split('@').next().unwrap_or(trace);
                            return format!(
                                "{}.{}",
                                typed_expression_print(argument),
                                field
                            );
                        }
                    }
                }
            }
            let function_print = typed_expression_print(function);
            let mut out = match function.as_ref() {
                TypedExpr::GlobalIdent { .. } | TypedExpr::LocalIdent { .. } => function_print,
                _ => format!("({function_print})"),
            };
            match argument.as_ref() {
                TypedExpr::TupleDisplay(_) => out.push_str(&typed_expression_print(argument)),
                _ => {
                    out.push('(');
                    out.push_str(&typed_expression_print(argument));
                    out.push(')');
                }
            }
            out
        }
        TypedExpr::Sequence { first, second } => format!(
            "{};{}",
            typed_expression_print(first),
            typed_expression_print(second)
        ),
        TypedExpr::Next { first, second } => format!(
            "{} next {}",
            typed_expression_print(first),
            typed_expression_print(second)
        ),
        TypedExpr::Conditional { .. } => {
            let mut out = String::from(" if ");
            let mut current = expression;
            loop {
                let TypedExpr::Conditional {
                    condition,
                    then_branch,
                    else_branch,
                } = current
                else {
                    unreachable!("the loop only continues on conditional else branches")
                };
                out.push_str(&typed_expression_print(condition));
                out.push_str(" then ");
                out.push_str(&typed_expression_print(then_branch));
                match else_branch.as_ref() {
                    next @ TypedExpr::Conditional { .. } => {
                        out.push_str(" elif ");
                        current = next;
                    }
                    other => {
                        out.push_str(" else ");
                        out.push_str(&typed_expression_print(other));
                        out.push_str(" fi ");
                        break;
                    }
                }
            }
            out
        }
        _ => "<expression>".to_string(),
    }
}

/// The rewrites upstream's special integer builtins apply at call
/// construction (global.w:2916-2983), reproduced at PRINT time because the
/// Rust converter keeps the desugared call in its typed tree: `x+1` prints
/// as `succ@int(x)`, `x-1` as `pred@int(x)`, `-1-x` as `~@int(x)`, and
/// unary minus on an integer denotation folds to the negative denotation.
fn special_int_unary_print(name: &str, arguments: &[TypedExpr]) -> Option<String> {
    fn integer_denotation(expression: &TypedExpr) -> Option<&BigInt> {
        match expression {
            TypedExpr::Denotation(value) => match value.as_ref() {
                Value::Integer(value) => Some(value),
                _ => None,
            },
            _ => None,
        }
    }
    match (name, arguments) {
        ("+@(int,int)", [left, right]) if integer_denotation(right) == Some(&BigInt::from(1)) => {
            Some(format!("succ@int({})", typed_expression_print(left)))
        }
        ("-@(int,int)", [left, right]) if integer_denotation(right) == Some(&BigInt::from(1)) => {
            Some(format!("pred@int({})", typed_expression_print(left)))
        }
        ("-@(int,int)", [left, right]) if integer_denotation(left) == Some(&BigInt::from(-1)) => {
            Some(format!("~@int({})", typed_expression_print(right)))
        }
        ("-@int", [operand]) => integer_denotation(operand).map(|value| format!("-{value}")),
        _ => None,
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
/// single-table ordering gives them. Cached on the overload state; the
/// `add_user`/`remove` mutations invalidate the entry for their name.
fn merged_variants(name: &str, overloads: &OverloadState, types: &TypeTable) -> Rc<Vec<MergedVariant>> {
    if let Some(cached) = overloads.merged_cache.borrow().get(name) {
        return Rc::clone(cached);
    }
    let merged = Rc::new(build_merged_variants(name, overloads, types));
    overloads
        .merged_cache
        .borrow_mut()
        .insert(name.to_owned(), Rc::clone(&merged));
    merged
}

fn build_merged_variants(name: &str, overloads: &OverloadState, types: &TypeTable) -> Vec<MergedVariant> {
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
        compact_matrix_display: bool,
        span: SourceSpan,
    },
    ReportLine {
        text: String,
        span: SourceSpan,
    },
    /// A report line that upstream prints WITHOUT the include-depth
    /// indentation (`global_forget_identifier` global.w:1241-1248 and
    /// `global_forget_overload` global.w:1253-1261 write to
    /// `*output_stream` directly, unlike the definition reports).
    PlainReportLine {
        text: String,
        span: SourceSpan,
    },
    Output {
        text: String,
        span: SourceSpan,
    },
}

/// The startup completion names (buffer.w:1175-1192): every
/// `main_hash_table` entry present at session start, in hash-code order —
/// 34 keywords, 21 primitive type names, then the builtins in upstream
/// registration order (NOT this crate's registry order). Captured
/// verbatim from the oracle (`readline_completions("")` on a fresh
/// session); "transpose " (trailing space) and "matrix slicer" are the
/// deliberately unregistered hidden builtins (see the registry batch-4
/// comment). The three startup system variables (main.w:408-435) are NOT
/// here: they are session globals, seeded in `TypedContext::new`.
const STARTUP_COMPLETION_NAMES: &[&str] = &[
    "quit",
    "set",
    "let",
    "in",
    "begin",
    "end",
    "if",
    "then",
    "else",
    "elif",
    "fi",
    "and",
    "or",
    "not",
    "next",
    "do",
    "dont",
    "from",
    "downto",
    "while",
    "for",
    "od",
    "case",
    "esac",
    "rec_fun",
    "true",
    "false",
    "die",
    "break",
    "return",
    "set_type",
    "whattype",
    "showall",
    "forget",
    "int",
    "rat",
    "string",
    "bool",
    "vec",
    "mat",
    "ratvec",
    "LieType",
    "RootDatum",
    "WeylElt",
    "InnerClass",
    "RealForm",
    "CartanClass",
    "KGBElt",
    "Block",
    "Split",
    "KType",
    "KTypePol",
    "Param",
    "ParamPol",
    "void",
    "-",
    "succ",
    "+",
    "pred",
    "~",
    "*",
    "\\",
    "%",
    "\\%",
    "^",
    "AND",
    "OR",
    "XOR",
    "AND_NOT",
    "bitwise_subset",
    "nth_set_bit",
    "bit_length",
    "to_bitset",
    "/",
    "floor",
    "ceil",
    "frac",
    "=",
    "!=",
    ">=",
    ">",
    "<=",
    "<",
    "##",
    "ascii",
    "readline_completions",
    "#",
    "shape",
    "row",
    "column",
    "rows",
    "columns",
    "flex_add",
    "flex_sub",
    "convolve",
    "null",
    "transpose ",
    "id_mat",
    "diagonal",
    "stack_rows",
    "swiss_matrix_knife",
    "matrix slicer",
    "gcd",
    "Bezout",
    "echelon",
    "linear_solve",
    "diagonalize",
    "adapted_basis",
    "kernel",
    "eigen_lattice",
    "row_saturate",
    "Smith",
    "invert",
    "mod2_section",
    "subspace_normal",
    "elapsed_ms",
    "Lie_type",
    "extend",
    "Cartan_matrix",
    "Cartan_matrix_type",
    "is_Cartan_matrix",
    "simple_factors",
    "rank",
    "Smith_Cartan",
    "filter_units",
    "ann_mod",
    "replace_gen",
    "quotient_basis",
    "involution",
    "prefers_coroots",
    "root_datum",
    "simply_connected",
    "adjoint",
    "semisimple_rank",
    "nr_of_posroots",
    "two_rho",
    "two_rho_check",
    "root",
    "coroot",
    "root_index",
    "coroot_index",
    "root_expression",
    "coroot_expression",
    "is_long_root",
    "is_long_coroot",
    "root_involution",
    "root_ladder_bottoms",
    "coroot_ladder_bottoms",
    "fundamental_weight",
    "fundamental_coweight",
    "simple_roots",
    "simple_coroots",
    "posroots",
    "poscoroots",
    "root_coradical",
    "coroot_radical",
    "dual",
    "derived_info",
    "mod_central_torus_info",
    "integrality_datum",
    "integrality_rank",
    "is_integrally_dominant",
    "integrality_points",
    "Weyl_orbit",
    "Weyl_orbit_ws",
    "cofolded",
    "walls",
    "alcove_center",
    "walls_attitude",
    "alcove_root_vertex",
    "basic_orbit_ws",
    "affine_orbit_ws",
    "FPP_numers",
    "FPP_w_shifts",
    "W_elt",
    "word",
    "length",
    "from_dominant",
    "root_permutation",
    "classify_involution",
    "inner_class",
    "twisted_involution",
    "distinguished_involution",
    "dual_datum",
    "form_names",
    "dual_form_names",
    "nr_of_real_forms",
    "nr_of_dual_real_forms",
    "nr_of_Cartan_classes",
    "block_sizes",
    "block_size",
    "occurrence_matrix",
    "dual_occurrence_matrix",
    "real_form",
    "form_number",
    "quasisplit_form",
    "components_rank",
    "KGB_size",
    "base_grading_vector",
    "Cartan_order",
    "KGB_Hasse",
    "dual_real_form",
    "dual_quasisplit_form",
    "central_fiber",
    "initial_torus_bits",
    "Cartan_class",
    "most_split_Cartan",
    "Cartan_info",
    "real_forms",
    "dual_real_forms",
    "fiber_partition",
    "square_classes",
    "KGB",
    "cross",
    "Cayley",
    "status",
    "KGB_elt",
    "twist",
    "torus_bits",
    "torus_factor",
    "block",
    "element",
    "index",
    "inverse_Cayley",
    "K_type",
    "height",
    "equivalent",
    "is_standard",
    "is_dominant",
    "is_zero",
    "is_semifinal",
    "is_final",
    "dominant",
    "to_canonical_fiber",
    "normal",
    "theta_stable",
    "null_K_module",
    "last_term",
    "first_term",
    "truncate_above_height",
    "KGP_sum",
    "K_type_formula",
    "branch",
    "param",
    "orientation_nr",
    "reducibility_points",
    "print_block",
    "print_common_block",
    "print_partial_block",
    "print_partial_common_block",
    "partial_block",
    "block_Hasse",
    "KL_block",
    "dual_KL_block",
    "partial_KL_block",
    "W_graph",
    "W_cells",
    "strong_components",
    "default_extended",
    "shift_flip",
    "extended_block",
    "partial_extended_KL_block",
    "null_module",
    "K_type_pol",
    "deform",
    "twisted_deform",
    "block_deform",
    "full_deform",
    "twisted_full_deform",
    "KL_sum_at_s",
    "KL_sum_at_s_to_height",
    "twisted_KL_sum_at_s",
    "KL_column",
    "scale_extended",
    "K_type_pol_extended",
    "finalize_extended",
    "raw_KL",
    "dual_KL",
    "raw_ext_KL",
    "print_gradings",
    "print_real_Weyl",
    "print_strong_real",
    "print_blocku",
    "print_blockd",
    "print_blockstabilizer",
    "print_KGB",
    "print_KGB_order",
    "print_KGB_graph",
    "print_X",
    "print_KL_basis",
    "print_prim_KL",
    "print_KL_list",
    "print_W_cells",
    "print_W_graph",
];

/// The startup system variables (main.w:408-435), in definition order:
/// the `-path` search list (empty in batch mode), the prelude log
/// (declared constant upstream), and the error back-trace, rewritten with
/// the trace lines of every caught runtime error (global.w:1135-1148).
const SYSTEM_VARIABLE_NAMES: &[&str] = &["input_path", "prelude_log", "back_trace"];

/// Persistent state for command-at-a-time typed execution.
pub struct TypedContext {
    types: TypeTable,
    globals: IdTable,
    evaluation: EvaluationContext,
    /// The `$` state (axis.w:573-582): value and type of the last
    /// non-void top-level expression. Both start at the empty tuple/void,
    /// and a failed or void evaluation leaves them untouched (sticky).
    last_value: Value,
    last_type: TypeCell,
    /// The overload table: startup overloads hidden by
    /// `forget name @ type` plus user `set` definitions.
    overloads: OverloadState,
    /// The session verbosity (main.w `verbosity`): `set quiet` = 0,
    /// `set verbose` = 1; the verbose analysis trace prints when this is
    /// nonzero.
    verbosity: u8,
    /// Session completion names in order of first definition (the
    /// upstream hash codes allocated after the startup entries; they are
    /// never recycled, so a forgotten-then-redefined name revives at its
    /// original position). Seeded with the system variables.
    completion_order: Vec<String>,
    /// The recorded names that are currently live (bound in the identifier
    /// table or carrying user overloads); kept in sync by the note/forget
    /// paths so the candidate snapshot can be maintained incrementally.
    completion_live: BTreeSet<String>,
    /// Set when a name left the live set or a recorded name revived (the
    /// append-only fast path cannot represent middle removals or position
    /// restoration); the next command rebuilds the snapshot in order.
    completion_dirty: bool,
    /// Names of user-defined types (lexer.w:419-448): the lexer consults
    /// this set to emit `TYPE_ID` for defined type names in later commands.
    /// Shared (Rc) with the session so each new command's lexer sees the
    /// same live set. `forget` removes the name here even though the
    /// TypeTable keeps the equation (upstream behaviour: the name stops
    /// lexing as TYPE_ID once forgotten).
    defined_type_names: Rc<RefCell<BTreeSet<String>>>,
}

impl Default for TypedContext {
    fn default() -> Self {
        Self {
            types: TypeTable::default(),
            globals: IdTable::default(),
            evaluation: EvaluationContext::default(),
            // Upstream seeds `$` with the empty tuple at void so a reference
            // before any assignment type-checks (and prints nothing).
            last_value: Value::Tuple(Vec::new()),
            last_type: Rc::new(RefCell::new(Type::void())),
            overloads: OverloadState::default(),
            verbosity: 0,
            completion_order: Vec::new(),
            completion_live: BTreeSet::new(),
            completion_dirty: true,
            defined_type_names: Rc::new(RefCell::new(BTreeSet::new())),
        }
    }
}

impl TypedContext {
    pub fn new() -> Self {
        let mut context = Self::default();
        for &name in SYSTEM_VARIABLE_NAMES {
            context.globals.define(
                name,
                Type::row(string_type()),
                crate::frames::global_with(Rc::new(Value::List(Vec::new()))),
            );
            context.completion_order.push(name.to_owned());
        }
        context.globals.mark_const("prelude_log");
        context
    }

    pub fn globals(&self) -> &IdTable {
        &self.globals
    }

    /// The live set of user-defined type names, shared with the lexers the
    /// session builds for later commands (lexer.w:419-448).
    pub fn defined_type_names(&self) -> Rc<RefCell<BTreeSet<String>>> {
        Rc::clone(&self.defined_type_names)
    }

    /// The top-level `Value: ` rendering (main.w:533-540 prints
    /// `*last_value` with the standard value printer): closures take the
    /// multi-line `closure_value::print` (axis.w:3254-3271) at ANY depth
    /// inside containers, every other value uses `Display`.
    pub fn render_value(&self, value: &Value) -> String {
        value_string(&self.evaluation, value)
    }

    /// Record a source buffer's trace display name (buffer.w:694): the
    /// session frame calls this as it registers each buffer.
    pub fn note_source_name(&mut self, id: crate::diagnostic::SourceId, name: String) {
        self.evaluation.note_source_name(id, name);
    }

    /// Evaluate one top-level expression; on a runtime error with a
    /// non-empty back-trace, store the trace lines in the `back_trace`
    /// system variable (global.w:1127, 1135-1148). An empty trace keeps the
    /// previous value (the upstream `set_back_trace` no-ops on empty).
    fn evaluate_with_trace(&mut self, typed: &TypedExpr) -> Result<Value, Diagnostic> {
        match evaluate_command_expr(typed, &mut self.evaluation) {
            Ok(value) => Ok(unwrap_shared(value)),
            Err(diagnostic) => {
                if diagnostic.kind == ErrorKind::Runtime && !diagnostic.back_trace.is_empty() {
                    // Upstream writes through a raw pointer captured at
                    // startup; here the name is looked up, so a user who
                    // forgot or redefined `back_trace` loses the update.
                    if let Some((_, cell)) = self.globals.lookup("back_trace") {
                        *cell.borrow_mut() = Some(Rc::new(Value::List(
                            diagnostic
                                .back_trace
                                .iter()
                                .map(|line| Rc::new(Value::String(line.clone())))
                                .collect(),
                        )));
                    }
                }
                Err(diagnostic)
            }
        }
    }

    pub fn execute(&mut self, command: &Command) -> Result<Vec<TypedCommandEvent>, Diagnostic> {
        self.refresh_completion_candidates();
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
                            crate::syntax::parse_tree_print(expression)
                        ),
                        span: expression.span(),
                    });
                }
                let mut type_ = Type::Undetermined;
                let typed = convert_expr(
                    expression,
                    &mut type_,
                    &Analysis::new(
                        &self.types,
                        &self.globals,
                        &self.overloads,
                        &self.last_value,
                        &self.last_type,
                    ),
                )?;
                if self.verbosity > 0 {
                    events.push(TypedCommandEvent::Output {
                        text: format!("Type found: {}\n", type_.display(&self.types)),
                        span: expression.span(),
                    });
                    events.push(TypedCommandEvent::Output {
                        text: format!("Converted expression: {}\n", typed_expression_print(&typed)),
                        span: expression.span(),
                    });
                }
                let value = match self.evaluate_with_trace(&typed) {
                    Ok(value) => value,
                    Err(diagnostic) => {
                        // Upstream keeps text printed before the failure
                        // (ext_kl.cpp:947); the buffer survives here and the
                        // session layer drains it ahead of the diagnostic.
                        return Err(diagnostic);
                    }
                };
                events.extend(self.drain_printed(expression.span()));
                // A non-void result refreshes `$` (main.w:533-540); a void
                // value (`()`, `prints(...)`) and any failure leave the
                // previous value sticky.
                if !type_.is_void() {
                    self.last_value = value.clone();
                    *self.last_type.borrow_mut() = type_.clone();
                }
                events.push(TypedCommandEvent::Value {
                    compact_matrix_display: matches!(typed, TypedExpr::GlobalAssignment { .. })
                        || matches!(typed, TypedExpr::Slice { column_lower: None, .. }),
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
                // parser.y disambiguates `id : id` lexically: a defined type
                // name lexes as TYPE_ID, making the command a declaration.
                // Our lexer has no type-table token, so a bare-identifier
                // right side naming a known type (alias or tabled) is
                // re-routed to the declaration path here.
                if let Expr::Identifier {
                    name: type_name, ..
                } = value
                {
                    if let Some(type_) = self.types.resolve_name(type_name) {
                        self.globals.define(
                            name.clone(),
                            type_.clone(),
                            crate::frames::unset_global(),
                        );
                        self.note_completion_name(name);
                        return Ok(vec![TypedCommandEvent::ReportLine {
                            text: format!(
                                "Declaring identifier '{name}': {}\n",
                                type_.display(&self.types)
                            ),
                            span: *span,
                        }]);
                    }
                }
                let mut type_ = Type::Undetermined;
                let typed = convert_expr(
                    value,
                    &mut type_,
                    &Analysis::new(
                        &self.types,
                        &self.globals,
                        &self.overloads,
                        &self.last_value,
                        &self.last_type,
                    ),
                )?;
                let value = match self.evaluate_with_trace(&typed) {
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
                self.note_completion_name(name);
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
                // tabled type map supports no removal, so only the lexer
                // set is updated: the name stops lexing as TYPE_ID.
                // global_forget_identifier (global.w:1241-1248): the report
                // goes to standard output and never fails the command. A
                // TYPE name also counts as known (upstream removes it from
                // the id table via clean_out_type_identifier): Levi_
                // subgroups.at's closing `forget orbit_data` reports
                // "forgotten", not "not known".
                let was_type = self.defined_type_names.borrow_mut().remove(&name.value);
                let was_known = self.globals.remove(&name.value) || was_type;
                self.invalidate_completion_candidates();
                // global.w:1241-1248: no input-level indentation here.
                let state = if was_known { "forgotten" } else { "not known" };
                Ok(vec![TypedCommandEvent::PlainReportLine {
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
                self.invalidate_completion_candidates();
                let state = if removed { "forgotten" } else { "not known" };
                // global.w:1253-1261: no input-level indentation here either.
                Ok(vec![TypedCommandEvent::PlainReportLine {
                    text: format!(
                        "Definition of '{}@{}' {state}\n",
                        name.value,
                        resolved.display(&self.types)
                    ),
                    span: *span,
                }])
            }
            Command::Set { bindings, span } => {
                self.execute_set(bindings, *span).map_err(|mut diagnostic| {
                    // Upstream appends the command context to any error
                    // escaping a 'set' command (global.w:1116-1130): the
                    // original message, then "Error in 'set' command at
                    // <loc>:" (indented like every multi-line message
                    // continuation). The parser rule `SET declarations
                    // '\n'` (parser.y:140) makes @$ cover the terminating
                    // newline, so the location end extends one column past
                    // the last initializer token.
                    let mut location_span = *span;
                    location_span.end.column += 1;
                    location_span.byte_end += 1;
                    let location = trace_location(&self.evaluation, &location_span);
                    diagnostic
                        .message
                        .push_str(&format!("\n  Error in 'set' command {location}:"));
                    diagnostic
                })
            }
            Command::ShowOverloads { name, span } => {
                // show_overloads (global.w:1790-1799): one line per active
                // variant, argument and result types printed independently.
                let variants = merged_variants(&name.value, &self.overloads, &self.types);
                let mut text = if variants.is_empty() {
                    format!("No overloads for '{}'\n", name.value)
                } else {
                    format!("Overloaded instances of '{}'\n", name.value)
                };
                for variant in variants.iter() {
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

    /// Refresh the completion candidate snapshot the
    /// `readline_completions` builtin reads (buffer.w:1175-1192): the
    /// static startup names (upstream hash codes are never recycled, so a
    /// redefined builtin keeps its startup position) followed by the
    /// session names still present in the identifier or overload table.
    fn refresh_completion_candidates(&mut self) {
        // The snapshot is the constant startup names plus the live
        // completion-order names. Definitions append through
        // `note_completion_name`; only a forget/revive (rare) forces this
        // full rebuild. Skipping the per-command rebuild removes hundreds
        // of string clones for every command in long include chains.
        if !self.completion_dirty {
            return;
        }
        let mut candidates: Vec<String> = STARTUP_COMPLETION_NAMES
            .iter()
            .map(|name| (*name).to_owned())
            .collect();
        self.completion_live.clear();
        for name in &self.completion_order {
            if self.globals.lookup(name).is_some() || !self.overloads.user_variants(name).is_empty()
            {
                self.completion_live.insert(name.clone());
                candidates.push(name.clone());
            }
        }
        self.evaluation.set_completion_candidates(candidates);
        self.completion_dirty = false;
    }

    /// Record a session name for completions (buffer.w:1175-1192): the
    /// upstream hash code is allocated at first definition and never
    /// recycled, so a name enters the order once; a name already in the
    /// startup hash table keeps its startup position (no duplicate).
    fn note_completion_name(&mut self, name: &str) {
        if STARTUP_COMPLETION_NAMES.contains(&name) {
            return;
        }
        if self.completion_order.iter().any(|known| known == name) {
            // A re-definition of a recorded name. It was live before unless
            // an intervening `forget` dropped it; reviving restores its
            // original position, which the append-only path cannot express.
            if !self.completion_live.contains(name) {
                self.completion_dirty = true;
            }
            return;
        }
        self.completion_order.push(name.to_owned());
        self.completion_live.insert(name.to_owned());
        // The caller defines the name immediately before noting it, so it
        // is live; append to the snapshot unless a rebuild is pending (the
        // rebuild then picks the name up from completion_order).
        if !self.completion_dirty {
            self.evaluation.push_completion_candidate(name.to_owned());
        }
    }

    /// Mark the completion snapshot stale after a `forget`/`forget @`
    /// command or a type-member definition: those can drop a recorded name
    /// from the live set or revive one without a note.
    fn invalidate_completion_candidates(&mut self) {
        self.completion_dirty = true;
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
            let variants = merged_variants(name, &self.overloads, &self.types);
            for variant in variants.iter() {
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
        let previous = self.globals.lookup(name).map(|(type_, _)| {
            // global.w:911-994: an overridden CONSTANT binding is noted
            // with a ` (constant)` suffix after its type.
            let mut text = type_.borrow().display(&self.types).to_string();
            if self.globals.is_const(name) {
                text.push_str(" (constant)");
            }
            text
        });
        self.globals.define(
            name.to_owned(),
            type_.clone(),
            crate::frames::global_with(Rc::new(value)),
        );
        if constant {
            self.globals.mark_const(name);
        }
        self.note_completion_name(name);
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
        self.note_completion_name(name);
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
            let analysis = Analysis::new(
                &self.types,
                &self.globals,
                &self.overloads,
                &self.last_value,
                &self.last_type,
            );
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
            let value = match self.evaluate_with_trace(&pending.typed) {
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
            distribute(Rc::new(value), &shape, &mut slots);
            debug_assert_eq!(slots.len(), leaves.len());
            for ((name, name_span, constant, leaf_type), slot) in leaves.into_iter().zip(slots) {
                let value = Rc::try_unwrap(slot).unwrap_or_else(|rc| (*rc).clone());
                // global.w:938: the routing tests kind()==function_type,
                // which untables — a tabled function type (lazy_lists.at's
                // inf_list = (->inf_node)) also joins the overload table
                // and reports "Defined", never "Variable".
                let is_function = match &leaf_type {
                    Type::Function(_) => true,
                    Type::Tabled(number) => {
                        matches!(self.types.expansion(*number), Type::Function(_))
                    }
                    _ => false,
                };
                let event = if is_function {
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
                        merged_into: None,
                    })
                })
                .collect();
            // Pass 2: resolve each spec with every group name visible.
            for (definition, number) in definitions.iter().zip(numbers) {
                let (mut expansion, mut fields) = resolve_type_spec(&definition.spec, &self.types)?;
                // A bracketed member written without any field names
                // parses as a plain type expression (Alias-shaped spec)
                // and yields no field slots; the tabled entry still needs
                // one (empty) slot per component, or `whattype` zips the
                // components with an empty field list and prints "(  )".
                if fields.is_empty() {
                    match &expansion {
                        Type::Tuple(components) | Type::Union(components)
                            if !components.is_empty() =>
                        {
                            fields = vec![None; components.len()];
                        }
                        _ => {}
                    }
                }
                // Upstream reduces every structural equivalence class to
                // one type_map entry (axis-types.w:1024-1051): an anonymous
                // sub-type equal to an earlier named one references it.
                self.types.canonicalise_anonymous(&mut expansion);
                self.types.update(number, expansion, fields);
                targets.push(Type::Tabled(number));
            }
            // Pass 3 (axis-types.w:1024-1051): every equivalence class
            // reduces to one entry. A re-included identical definition
            // merges into the earlier number, so functions written against
            // the first include still match.
            for target in targets.iter_mut() {
                let Type::Tabled(number) = target else { continue };
                let canonical = (0..number.0)
                    .map(TypeNumber)
                    .find(|candidate| {
                        self.types.binding(*candidate).merged_into.is_none()
                            && self.types.equivalent(*number, *candidate)
                    });
                if let Some(canonical) = canonical {
                    let fields = self.types.binding(*number).fields.clone();
                    self.types.merge_into(*number, canonical, fields);
                    *target = Type::Tabled(canonical);
                }
            }
            self.types.canonicalise_references();
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
        // global.w:1384: the single-name form says "redefined as" when the
        // name was already a defined type; capture the flags before the
        // lexer set below is updated. The bracketed form always prints
        // "defined as" (global.w:1635-1647 cleans out silently).
        let redefinitions: Vec<bool> = definitions
            .iter()
            .map(|definition| {
                self.defined_type_names
                    .borrow()
                    .contains(&definition.name.value)
            })
            .collect();
        // lexer.w:419-448: every freshly defined type name now lexes as
        // TYPE_ID in later commands, so the lexer set is updated here.
        {
            let mut names = self.defined_type_names.borrow_mut();
            for definition in definitions {
                names.insert(definition.name.value.clone());
            }
        }
        let mut events = Vec::with_capacity(definitions.len());
        for ((definition, target), redefine) in
            definitions.iter().zip(&targets).zip(&redefinitions)
        {
            let text =
                self.define_type_members(definition, target, tabled, *redefine, span)?;
            events.push(TypedCommandEvent::ReportLine { text, span });
        }
        Ok(events)
    }

    /// Install the projector (struct) or injector (union) functions of one
    /// definition in the overload table (global.w:1398-1410 adds each one
    /// with `overload_table::add`, so same-named members of later types
    /// overload or replace exactly like user `set` definitions), and render
    /// the report line.
    fn define_type_members(
        &mut self,
        definition: &crate::syntax::TypeDefinition,
        target: &Type,
        tabled: bool,
        redefine: bool,
        span: SourceSpan,
    ) -> Result<String, Diagnostic> {
        let expansion = match target {
            Type::Tabled(number) => self.types.expansion(*number).clone(),
            other => other.clone(),
        };
        // The bracketed (tabled) form echoes the expansion with void
        // arrow sides shown (global.w:1647, "defined as (void->int)") —
        // even when no field is named and the spec parses as a plain
        // type expression (`set_type [ t0 = (int,int) ]` echoes
        // "(int,int)"). The single-name alias form echoes the type AS
        // WRITTEN (global.w:1390): a tabled right-hand side prints its
        // NAME ("Type name 'Parabolic' defined as KGPElt"), a
        // structural one its plain spelling ("(->int)").
        let heading = if tabled {
            format!(
                "Type name '{}' defined as {}\n",
                definition.name.value,
                match target {
                    Type::Tabled(_) => expansion.display_in_set_type(&self.types).to_string(),
                    _ => expansion.display(&self.types).to_string(),
                }
            )
        } else {
            format!(
                "Type name '{}' {} as {}\n",
                definition.name.value,
                if redefine { "redefined" } else { "defined" },
                target.display(&self.types)
            )
        };
        let fields = match &definition.spec {
            TypeSpec::Alias(_) => return Ok(heading),
            TypeSpec::Struct(fields) | TypeSpec::Union(fields) => fields,
        };
        let components: &[Type] = match &expansion {
            Type::Tuple(components) | Type::Union(components) => components,
            _ => &[],
        };
        let union = matches!(definition.spec, TypeSpec::Union(_));
        // The echo prints EVERY field position (global.w:1705-1725): an
        // unnamed hole contributes just its separator, so `(, int x, int
        // mu)` reports "with projectors: , x, mu.".
        let display_names: Vec<String> = fields
            .iter()
            .map(|field| {
                field
                    .name
                    .as_ref()
                    .map(|name| name.value.clone())
                    .unwrap_or_default()
            })
            .collect();
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
            self.overloads.add_user(
                &field_name.value,
                function_type,
                member_closure(body, field.span),
                &self.types,
                span,
            )?;
            names.push(field_name.value.clone());
        }
        // Type members join the overload table without a completion note;
        // a member sharing a recorded name could revive it, so rebuild.
        if !names.is_empty() {
            self.invalidate_completion_candidates();
        }
        if names.is_empty() {
            return Ok(heading);
        }
        let role = if union { "injectors" } else { "projectors" };
        Ok(format!(
            "{heading}  with {role}: {}.\n",
            display_names.join(", ")
        ))
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
                                // Oracle: `whattype il2` on a tabled
                                // function type shows (void->int) — the
                                // same spelling as the set_type echo.
                                return Ok(vec![TypedCommandEvent::ReportLine {
                                    text: format!(
                                        "Defined type: {}\n",
                                        other.display_in_set_type(&self.types)
                                    ),
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
            &Analysis::new(
                &self.types,
                &self.globals,
                &self.overloads,
                &self.last_value,
                &self.last_type,
            ),
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
) -> Result<SharedValue, Diagnostic> {
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
/// projector and injector globals a `set_type` definition installs. These
/// stand in for upstream's `projector_value`/`injector_value`; their
/// back-trace origin wording (`projector defined ...`, axis.w:4479/4577) is
/// not ported, and they push no traced frame (empty `param_names`).
fn member_closure(body: TypedExpr, span: SourceSpan) -> Value {
    Value::Closure(Rc::new(Closure {
        parameters: 1,
        shapes: Rc::from(vec![SlotShape::Leaf]),
        recursive: false,
        body: Rc::new(body),
        frame: None,
        span,
        param_names: Rc::from(Vec::new()),
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
    // The row-context test untables transparently (axis-types.w:375-384
    // `kind()`): a tabled row like sparse.at's `sparse_column` hands its
    // expansion's component type down; conform_types still specialises
    // against the original tabled pattern.
    let expanded_required;
    let required_kind: &Type = match &*required {
        Type::Tabled(number) => {
            expanded_required = analysis.types.expansion(*number).clone();
            &expanded_required
        }
        other => other,
    };
    let (mut component, coercion_tag) = match required_kind {
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
                TypedExpr::Denotation(Rc::new(Value::Integer(value.clone()))),
                *span,
                analysis,
            )
        }
        Expr::Boolean { value, span } => conform_types(
            &Type::Primitive(Prim::Bool),
            required,
            TypedExpr::Denotation(Rc::new(Value::Boolean(*value))),
            *span,
            analysis,
        ),
        Expr::String { value, span } => conform_types(
            &Type::Primitive(Prim::String),
            required,
            TypedExpr::Denotation(Rc::new(Value::String(value.clone()))),
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
            // `return` is legal only lexically inside a function body
            // (upstream `layer::may_return`, axis.w:381-384); the value
            // converts against the function's result type — shared through
            // the analysis cell so specialisations stick (axis.w:717-721) —
            // NEVER against the local context, which may be void (a `return`
            // inside a statement-position loop body still returns a value).
            let Some(return_cell) = &analysis.return_type else {
                return Err(type_error(
                    "One can only use 'return' within a function body".into(),
                    *span,
                ));
            };
            let mut target = return_cell.borrow().clone();
            let converted = convert_expr(value, &mut target, analysis)?;
            *return_cell.borrow_mut() = target;
            // The expression itself produces no value: `required` untouched.
            Ok(TypedExpr::Return {
                value: Box::new(converted),
            })
        }
        Expr::Group { inner, .. } => convert_expr(inner, required, analysis),
        Expr::Cast { target, body, .. } => {
            // The cast's whole effect is conversion-time: convert the body
            // against the denoted type, then conform THAT to the context.
            // The target resolves against the session table so user type
            // names (TYPE_ID) are valid cast targets.
            let mut cast_type = target.resolve_in(analysis.types).map_err(|unknown| {
                Diagnostic::new(
                    ErrorKind::Name,
                    format!("undefined type name '{}'", unknown.value),
                    Some(unknown.span),
                )
            })?;
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
                // A void context voids the display as a whole
                // (conform_types' voiding), e.g. the non-final component
                // `(e:=S1[i],j:=i+1)` of an if-condition sequence
                // (combinatorics.at:826).
                if required.is_void() {
                    return conform_types(
                        &found,
                        required,
                        TypedExpr::TupleDisplay(converted),
                        *span,
                        analysis,
                    );
                }
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
        Expr::OpCast {
            name,
            arg_type,
            span,
        } => convert_op_cast(name, arg_type, *span, required, analysis),
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
        Expr::ComponentAssignment(assignment) => {
            convert_component_assignment(expression, assignment, required, analysis)
        }
        Expr::ComponentTransform(transform) => {
            convert_component_transform(expression, transform, required, analysis)
        }
        Expr::FieldAssignment(assignment) => {
            convert_field_assignment(expression, assignment, required, analysis)
        }
        Expr::FieldTransform(transform) => {
            convert_field_transform(expression, transform, required, analysis)
        }
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
            // string, a vec/ratvec to its entry, a mat to its column; a
            // two-int tuple subscripts a mat to the entry (parser.y:585-598,
            // axis.w matrix subscription). Anything else is the
            // analysis-time `not_so` error (axis.w:4101-4105).
            let int_index = matches!(index_type, Type::Primitive(Prim::Int));
            let pair_index = matches!(
                &index_type,
                Type::Tuple(parts)
                    if parts.len() == 2
                        && parts
                            .iter()
                            .all(|part| matches!(part, Type::Primitive(Prim::Int)))
            );
            // `aggr.kind()` untables transparently (axis-types.w:375-384),
            // so a tabled row like Levi_subgroups.at's `orbit_data`
            // subscripts through its expansion; the not_so diagnostic still
            // prints the original (named) type, as upstream prints `aggr`.
            let expanded_array;
            let array_kind: &Type = match &array_type {
                Type::Tabled(number) => {
                    expanded_array = analysis.types.expansion(*number).clone();
                    &expanded_array
                }
                other => other,
            };
            let found = match array_kind {
                Type::Row(component) if int_index => (**component).clone(),
                Type::Primitive(Prim::String) if int_index => Type::Primitive(Prim::String),
                Type::Primitive(Prim::Vec) if int_index => Type::Primitive(Prim::Int),
                Type::Primitive(Prim::RatVec) if int_index => Type::Primitive(Prim::Rat),
                Type::Primitive(Prim::Mat) if int_index => Type::Primitive(Prim::Vec),
                Type::Primitive(Prim::Mat) if pair_index => Type::Primitive(Prim::Int),
                // Term-coefficient selection (axis.w:3962-3969,
                // index_kind's K_type_poly_term and mod_poly_term).
                Type::Primitive(Prim::KTypePol)
                    if matches!(index_type, Type::Primitive(Prim::KType)) =>
                {
                    if *reversed {
                        return Err(type_error(
                            "Cannot do reversed subscription of a KTypePol".into(),
                            *span,
                        ));
                    }
                    Type::Primitive(Prim::Split)
                }
                Type::Primitive(Prim::ParamPol)
                    if matches!(index_type, Type::Primitive(Prim::Param)) =>
                {
                    if *reversed {
                        return Err(type_error(
                            "Cannot do reversed subscription of a ParamPol".into(),
                            *span,
                        ));
                    }
                    Type::Primitive(Prim::Split)
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
            // The pair-index subscription prints without the tuple
            // parentheses in the range diagnostic (`M[5,0]`, while the
            // assignment compact keeps them: `M[(5,0)]:=1`). The oracle
            // quotes the CONVERTED (typed) sub-expressions in the range
            // message (`subscription [1,2,3][+@(int,int)(1,2)]`), so the
            // single-index source uses the typed printer, not the parse tree.
            let source = match &**index {
                Expr::Tuple { elements, .. } if elements.len() == 2 => format!(
                    "{}{}[{},{}]",
                    compact_expression(array),
                    if *reversed { "~" } else { "" },
                    compact_expression(&elements[0]),
                    compact_expression(&elements[1])
                ),
                _ => format!(
                    "{}{}[{}]",
                    typed_expression_print(&converted_array),
                    if *reversed { "~" } else { "" },
                    typed_expression_print(&converted_index)
                ),
            };
            conform_types(
                &found,
                required,
                TypedExpr::Subscription {
                    array: Box::new(converted_array),
                    index: Box::new(converted_index),
                    reversed: *reversed,
                    source,
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
            column_lower,
            column_upper,
            flags,
            span,
        } => {
            if let (Some(column_lower), Some(column_upper)) = (column_lower, column_upper) {
                // Two-dimensional slice (parser.y:660-705): upstream wraps
                // the base in an explicit `mat` cast, so a non-matrix base
                // fails with the coercion error ("found [int] while mat was
                // needed.") and a row DISPLAY in mat position coerces via
                // the row coercion ([[int]] -> mat, columns-first).
                let mut array_type = Type::Primitive(Prim::Mat);
                let converted_array = convert_expr(array, &mut array_type, analysis)?;
                // The desugared "matrix slicer" call converts the bounds
                // with UNDETERMINED a-priori types and rejects the whole
                // argument tuple when one is not int (probed oracle wording:
                // "found (int,mat,int,string,int,int) while
                // (int,mat,int,int,int,int) was needed.").
                let bounds = [lower, upper, column_lower, column_upper];
                let mut bound_types = Vec::with_capacity(4);
                let mut converted_bounds = Vec::with_capacity(4);
                for bound in bounds {
                    let mut bound_type = Type::Undetermined;
                    converted_bounds.push(convert_expr(bound, &mut bound_type, analysis)?);
                    bound_types.push(bound_type);
                }
                if bound_types
                    .iter()
                    .any(|bound_type| *bound_type != Type::Primitive(Prim::Int))
                {
                    let mut tuple_components =
                        vec![Type::Primitive(Prim::Int), Type::Primitive(Prim::Mat)];
                    tuple_components.extend(bound_types);
                    let found = Type::tuple(tuple_components);
                    let needed = Type::tuple(vec![
                        Type::Primitive(Prim::Int),
                        Type::Primitive(Prim::Mat),
                        Type::Primitive(Prim::Int),
                        Type::Primitive(Prim::Int),
                        Type::Primitive(Prim::Int),
                        Type::Primitive(Prim::Int),
                    ]);
                    return Err(type_error(
                        format!(
                            "found {} while {} was needed.",
                            found.display(analysis.types),
                            needed.display(analysis.types)
                        ),
                        *span,
                    ));
                }
                let mut converted_bounds = converted_bounds.into_iter();
                let row_lower = converted_bounds.next().expect("row lower bound");
                let row_upper = converted_bounds.next().expect("row upper bound");
                let column_lower = converted_bounds.next().expect("column lower bound");
                let column_upper = converted_bounds.next().expect("column upper bound");
                let found = Type::Primitive(Prim::Mat);
                let source = format!(
                    "{}[{}:{},{}:{}]",
                    typed_expression_print(&converted_array),
                    typed_expression_print(&row_lower),
                    typed_expression_print(&row_upper),
                    typed_expression_print(&column_lower),
                    typed_expression_print(&column_upper)
                );
                return conform_types(
                    &found,
                    required,
                    TypedExpr::Slice {
                        array: Box::new(converted_array),
                        lower: Box::new(row_lower),
                        upper: Box::new(row_upper),
                        column_lower: Some(Box::new(column_lower)),
                        column_upper: Some(Box::new(column_upper)),
                        flags: *flags,
                        source,
                        span: *span,
                    },
                    *span,
                    analysis,
                );
            }
            let mut array_type = Type::Undetermined;
            let converted_array = convert_expr(array, &mut array_type, analysis)?;
            // Rows, strings, vecs, ratvecs and matrices use the
            // one-dimensional slice families (axis.w:3846-3897). The string
            // case is byte-indexed by the upstream `std::string` wrapper
            // (axis.w:4379-4402), while the result remains a string; the
            // matrix case selects COLUMNS (matrix_slice, axis.w:4407-4427).
            let found = match &array_type {
                Type::Row(component) => Type::Row(component.clone()),
                Type::Primitive(
                    primitive @ (Prim::String | Prim::Vec | Prim::RatVec | Prim::Mat),
                ) => Type::Primitive(*primitive),
                _ => {
                    return Err(type_error(
                        format!(
                            "Cannot slice value of type {}",
                            array_type.display(analysis.types)
                        ),
                        *span,
                    ));
                }
            };
            let mut bound_type = Type::Primitive(Prim::Int);
            let converted_lower = convert_expr(lower, &mut bound_type, analysis)?;
            let mut bound_type = Type::Primitive(Prim::Int);
            let converted_upper = convert_expr(upper, &mut bound_type, analysis)?;
            let source = format!(
                "{}[{}:{}]",
                typed_expression_print(&converted_array),
                typed_expression_print(&converted_lower),
                typed_expression_print(&converted_upper)
            );
            conform_types(
                &found,
                required,
                TypedExpr::Slice {
                    array: Box::new(converted_array),
                    lower: Box::new(converted_lower),
                    upper: Box::new(converted_upper),
                    column_lower: None,
                    column_upper: None,
                    flags: *flags,
                    source,
                    span: *span,
                },
                *span,
                analysis,
            )
        }
        Expr::BarList { rows, span } => {
            // Each segment is a comma-list of `int` entries (upstream routes
            // the row-of-rows through a `mat` cast, which balances every
            // entry against int: "found string while int was needed.").
            let converted_rows = rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|entry| {
                            let mut entry_type = Type::Primitive(Prim::Int);
                            convert_expr(entry, &mut entry_type, analysis)
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?;
            let found = Type::Primitive(Prim::Mat);
            conform_types(
                &found,
                required,
                TypedExpr::BarList {
                    rows: converted_rows,
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
                            return_type: analysis.return_type.clone(),
                            loop_depth: analysis.loop_depth,
                            last_value: analysis.last_value,
                            last_value_type: analysis.last_value_type,
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
                // Slot names in bind order across the group's bindings,
                // for the error-time frame dump (axis.w:2882-2909).
                let names: Vec<String> = pending
                    .iter()
                    .flat_map(|(_, leaves, _)| leaves.iter().map(|(name, _, _, _)| name.clone()))
                    .collect();
                groups.push((
                    pending
                        .into_iter()
                        .map(|(shape, _, converted)| (shape, converted))
                        .collect::<Vec<_>>(),
                    names,
                ));
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
                    return_type: analysis.return_type.clone(),
                    loop_depth: analysis.loop_depth,
                    last_value: analysis.last_value,
                    last_value_type: analysis.last_value_type,
                },
            )?;
            for (initializers, names) in groups.into_iter().rev() {
                converted = TypedExpr::LetGroup {
                    initializers,
                    names: Rc::from(names),
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
                        name: format!("/@{}", int_type().display(analysis.types)),
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
                // `local_type_p->kind()==function_type` untables (axis.w:2431,
                // axis-types.w:378): a local whose NAMED type expands to a
                // function (e.g. inf_list = (->inf_node)) still shadows the
                // overload table.
                let local_function = local.is_some_and(|(type_, _, _)| {
                    let type_ = type_.borrow();
                    match &*type_ {
                        Type::Function(_) => true,
                        Type::Tabled(number) => matches!(
                            analysis.types.expansion(*number),
                            Type::Function(_)
                        ),
                        _ => false,
                    }
                });
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
                    // A dynamically computed callee has no resolved overload
                    // name; the trace falls back to the callee rendering.
                    name: None,
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
            body,
            out_reversed,
            span,
        } => {
            // Upstream (axis.w:5424-5435): in a void context the body
            // converts against void and no row is built; in an int context
            // the loop yields the number of iterations done
            // (make_while_loop flag 0x8); otherwise it collects a row of
            // body values against a fresh component pattern. The body is a
            // single do_expr whose flag drives termination at evaluation.
            if required.is_void() {
                let mut void = Type::void();
                let body = convert_expr(body, &mut void, &analysis.in_loop())?;
                return conform_types(
                    &Type::void(),
                    required,
                    TypedExpr::While {
                        body: Box::new(body),
                        out_reversed: *out_reversed,
                        yields_count: false,
                    },
                    *span,
                    analysis,
                );
            }
            if matches!(required, Type::Primitive(Prim::Int)) {
                let mut void = Type::void();
                let body = convert_expr(body, &mut void, &analysis.in_loop())?;
                return conform_types(
                    &Type::Primitive(Prim::Int),
                    required,
                    TypedExpr::While {
                        body: Box::new(body),
                        out_reversed: *out_reversed,
                        yields_count: true,
                    },
                    *span,
                    analysis,
                );
            }
            // The body converts against a fresh component pattern with the
            // loop depth raised; the loop's type is a row of that pattern.
            let mut component = Type::Undetermined;
            let body = convert_expr(body, &mut component, &analysis.in_loop())?;
            conform_types(
                &Type::row(component),
                required,
                TypedExpr::While {
                    body: Box::new(body),
                    out_reversed: *out_reversed,
                    yields_count: false,
                },
                *span,
                analysis,
            )
        }
        Expr::Do {
            condition,
            body,
            span,
        } => {
            if analysis.loop_depth == 0 {
                return Err(type_error(
                    "Using 'do' not in the reach of any loop".into(),
                    *span,
                ));
            }
            // axis.w:5532-5550: the body converts against the required type
            // in ALL cases; a constant-false condition then drops it (the
            // `dont` expression), a constant-true one keeps just the body
            // (the `forever` expression).
            let body = convert_expr(body, required, analysis)?;
            if let Expr::Boolean { value, .. } = &**condition {
                if !value {
                    return Ok(TypedExpr::Dont);
                }
                return Ok(TypedExpr::Do {
                    condition: None,
                    body: Box::new(body),
                });
            }
            let mut bool_type = Type::Primitive(Prim::Bool);
            let condition = convert_expr(condition, &mut bool_type, analysis)?;
            Ok(TypedExpr::Do {
                condition: Some(Box::new(condition)),
                body: Box::new(body),
            })
        }
        Expr::For(loop_) => convert_for_loop(loop_, required, analysis),
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
                Type::Tabled(number) => {
                    // Follow the merge chain and a Tabled definition: a
                    // group member whose expansion equals an earlier type
                    // merges into it, and its stored definition may just
                    // reference the canonical entry (conjugate.at's
                    // maybe_a_conjugator = (void no_w| WeylElt w) merging
                    // into maybe_a_mover). Upstream keeps ONE type_map
                    // entry per class, so discrimination always sees the
                    // canonical union.
                    let mut current = analysis.types.canonical(*number);
                    loop {
                        let binding = analysis.types.binding(current);
                        match &binding.definition {
                            Type::Tabled(next) => {
                                let next = analysis.types.canonical(*next);
                                if next == current {
                                    break None;
                                }
                                current = next;
                            }
                            Type::Union(variants)
                                if binding.fields.len() == variants.len()
                                    && binding.fields.iter().all(Option::is_some) =>
                            {
                                break Some((variants.clone(), binding.fields.clone()));
                            }
                            _ => break None,
                        }
                    }
                }
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
            // Branch bodies convert against ONE shared type pattern seeded
            // with the required type (axis.w:5179-5189: discrimination
            // branches use "the type provided by the context as possibly
            // modified by the conversion of previous branches" — no
            // balancing), so a void context voids every branch.
            let mut common = required.clone();
            let mut converted_branches = Vec::new();
            let mut fallback = None;
            let mut seen_labels = BTreeSet::new();
            for branch in branches {
                let Some(tag) = &branch.tag else {
                    let body = convert_expr(&branch.body, &mut common, analysis)?;
                    fallback = Some(Box::new(body));
                    continue;
                };
                let Some(index) = injector_names
                    .iter()
                    .position(|field| field.as_deref() == Some(tag.value.as_str()))
                else {
                    // Capture 3604622: the oracle reports the first unknown
                    // label against the subject's union type by name.
                    return Err(type_error(
                        format!(
                            "Branch has label {} not associated to any variant of the union type {}",
                            tag.value,
                            subject_type.display(analysis.types)
                        ),
                        tag.span,
                    ));
                };
                // A repeated label is rejected before its branch is
                // converted (axis.w:5712-5717: `choices[k]` already
                // filled), with the whole discrimination as context.
                if !seen_labels.insert(tag.value.as_str()) {
                    return Err(type_error(
                        format!("Multiple branches with label {}", tag.value),
                        tag.span,
                    ));
                }
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
                let body = convert_expr(
                    &branch.body,
                    &mut common,
                    &Analysis {
                        types: analysis.types,
                        globals: analysis.globals,
                        overloads: analysis.overloads,
                        locals,
                        constant_locals,
                        in_function: analysis.in_function,
                        return_type: analysis.return_type.clone(),
                        loop_depth: analysis.loop_depth,
                        last_value: analysis.last_value,
                        last_value_type: analysis.last_value_type,
                    },
                )?;
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
        Expr::CountedFor(loop_) => convert_counted_for_loop(loop_, required, analysis),
        Expr::Break { levels, span } => {
            // `break N` is legal only when N+1 loops lexically enclose it;
            // upstream rejects it during analysis (axis.w:673-685,
            // layer::may_break), before anything evaluates.
            if analysis.loop_depth <= *levels {
                let message = if *levels == 0 {
                    "Using 'break' not in the reach of any loop".to_string()
                } else {
                    format!(
                        "Using 'break {}' requires {} nested levels of loops",
                        levels,
                        levels + 1
                    )
                };
                return Err(type_error(message, *span));
            }
            // A break yields no value and, like the upstream breaker
            // (axis.w:673-685), converts in ANY context without touching the
            // required type — the enclosing balance (e.g. the `else break`
            // branch of a desugared while-let, parser.y:438-443) must not
            // see void drag the common type down.
            Ok(TypedExpr::Break { levels: *levels })
        }
        Expr::Dont { span } => {
            if analysis.loop_depth == 0 {
                return Err(type_error(
                    "Using 'dont' not in the reach of any loop".into(),
                    *span,
                ));
            }
            // Upstream `dont` is the kind-2 sequence (false, die): its type
            // is the DROPPED body's, converted against the required type —
            // and `die` leaves the required type untouched (axis.w:634-638).
            // So `dont` passes analysis against any context, exactly like
            // `die`; evaluation just clears the while-condition flag.
            Ok(TypedExpr::Dont)
        }
        Expr::Die { span } => {
            // `die` passes analysis trivially in ANY context, leaving the
            // required type untouched (upstream die_expr, axis.w:634-638);
            // only evaluation throws.
            Ok(TypedExpr::Die { span: *span })
        }
        Expr::LastValue { span } => convert_last_value(*span, required, analysis),
    }
}

/// Convert `$` (axis.w:610-624 `last_value_computed`): the last top-level
/// value is snapshotted into a denotation AT ANALYSIS TIME (a captured
/// occurrence inside a function body does not track later updates), then
/// conformed to the required type like any denotation.
fn convert_last_value(
    span: SourceSpan,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let found = analysis.last_value_type.borrow().clone();
    let type_display = format!("({}:$)", found.display(analysis.types));
    conform_types(
        &found,
        required,
        TypedExpr::CapturedLastValue {
            value: analysis.last_value.clone(),
            type_display,
        },
        span,
        analysis,
    )
}

/// The function value a builtin overload instance casts to (the upstream
/// capture_expression around the table's shared_function): the display
/// name uses the REGISTERED argument type, with the generic `*`
/// (undetermined) pattern printing as `T` like the oracle's polymorphic
/// variable (`{prints@T}`, capture 3604640).
fn builtin_function_value(index: usize, types: &TypeTable) -> Value {
    let builtin = &builtin_registry()[index];
    let argument = generic_type_display(&builtin.arg_type, types);
    Value::BuiltinFunction {
        builtin: index,
        name: format!("{}@{}", builtin.name, argument),
    }
}

/// Render a generic special operator's registered argument pattern. The
/// upstream capture printer calls an undetermined component `T`, including
/// when it is nested inside a row or tuple (`#@[T]`, `##@([T],[T])`), while
/// ordinary type diagnostics continue to print the same component as `*`.
fn generic_type_display(type_: &Type, types: &TypeTable) -> String {
    match type_ {
        Type::Undetermined => "T".to_owned(),
        Type::Row(component) => {
            let display = format!("[{}]", generic_type_display(component, types));
            if matches!(component.as_ref(), Type::Row(_)) {
                format!("({display})")
            } else {
                display
            }
        }
        Type::Tuple(components) => {
            if components.is_empty() {
                "void".to_owned()
            } else {
                format!(
                    "({})",
                    components
                        .iter()
                        .map(|component| generic_type_display(component, types))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
        Type::Union(variants) => format!(
            "({})",
            variants
                .iter()
                .map(|variant| generic_type_display(variant, types))
                .collect::<Vec<_>>()
                .join("|")
        ),
        other => other.display(types).to_string(),
    }
}

/// Convert an operator cast `name @ type` (parser.y:381-382;
/// axis.w:7356-7391 op_cast_expr). An exact overload-table entry (startup
/// builtin or user `set` variant) wins; otherwise the hidden generic special
/// instances accept the argument patterns described below, with `prints@@T`
/// accepting any argument type. The other generic special operators (`print`, `to_string`,
/// `error`, `#`, and `##`) use the same controlled fallback described by
/// `axis.w:6743-6848`; ordinary overloads are never selected by mere
/// specialisability. Anything else is the oracle's `No instance for
/// name@type found` (capture 3604640).
fn convert_op_cast(
    name: &SpannedValue<String>,
    arg_type: &TypeExpr,
    span: SourceSpan,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let cast_type = arg_type.resolve_in(analysis.types).map_err(|unknown| {
        Diagnostic::new(
            ErrorKind::Name,
            format!("undefined type name '{}'", unknown.value),
            Some(unknown.span),
        )
    })?;
    let merged = merged_variants(&name.value, analysis.overloads, analysis.types);
    // The instance test is structural (no wildcard specialisation — see
    // the operator-cast guard in AGENTS.md) but upstream's equality is
    // type_expr::operator==, which treats a tabled type as equal to its
    // expansion (axis-types.w:807-825): `maximal@KGPElt` selects the
    // ([int],KGBElt) overload (parabolics.at:124).
    if let Some(variant) = merged
        .iter()
        .find(|variant| variant.arg_type.equals(&cast_type, analysis.types))
    {
        // The cast value's type uses the WRITTEN argument type, not the
        // stored one (axis.w:6761-6764: type_expr(ctype.copy(), res_t)) —
        // `complex_Levi@ComplexParabolic` reports
        // (ComplexParabolic->RootDatum) even when the selected instance
        // was stored with the structural (RootDatum,[int]).
        let value = match variant.origin {
            OverloadOrigin::Builtin(index) => builtin_function_value(index, analysis.types),
            OverloadOrigin::User(user_index) => {
                analysis.overloads.user_variants(&name.value)[user_index]
                    .value
                    .clone()
            }
        };
        let deduced = Type::function(cast_type.clone(), variant.result_type.clone());
        return conform_types(
            &deduced,
            required,
            TypedExpr::Denotation(Rc::new(value)),
            span,
            analysis,
        );
    }
    if let Some((index, result_type)) = hidden_special_cast_variant(&name.value, &cast_type)
    {
        return conform_types(
            &Type::function(cast_type.clone(), result_type),
            required,
            TypedExpr::Denotation(Rc::new(builtin_function_value(index, analysis.types))),
            span,
            analysis,
        );
    }
    Err(type_error(
        format!(
            "No instance for {}@{} found",
            name.value,
            cast_type.display(analysis.types)
        ),
        span,
    ))
}

/// The component type when `type_` is (or expands to) a row; `None`
/// for an undetermined slot, which the body's a priori type fills.
fn row_component(type_: &Type, table: &TypeTable) -> Option<Type> {
    match type_ {
        Type::Row(component) => Some(component.as_ref().clone()),
        Type::Tabled(number) => match table.expansion(*number) {
            Type::Row(component) => Some(component.as_ref().clone()),
            _ => None,
        },
        _ => None,
    }
}

/// Convert a `for pattern[@index] in iterable do body od` loop
/// (parser.y:506-531). Kept out of `convert_expr`'s frame: the branch
/// locals are heavy, and the giant dispatcher recurses per subexpression.
fn convert_for_loop(
    loop_: &ForLoop,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let ForLoop {
        pattern,
        index,
        iterable,
        in_reversed,
        body,
        out_reversed,
        iffor_body,
        span,
    } = loop_;
    let mut found = Type::Undetermined;
    let iterable = convert_expr(iterable, &mut found, analysis)?;
    // The iterated component (parser.y:506-531 accepts every
    // subscriptable aggregate): a row yields its component, a
    // string its one-character strings, a vec its int entries, a
    // ratvec its rat entries, a mat its column vecs. When the
    // aggregate is not int-indexable, the polynomial types index by
    // the term itself (axis.w:5926-5936 `index_kind` retries with
    // KType and Param index types) and the component is the Split
    // coefficient.
    // The aggregate's STRUCTURE decides iterability, so a tabled type
    // stands for its expansion (axis-types.w:375-384: `kind()` untables);
    // a tabled row like `sparse_mat` iterates its (tabled) component.
    let untabled = match &found {
        Type::Tabled(number) => analysis.types.expansion(*number).clone(),
        other => other.clone(),
    };
    let (index_type, component) = match &untabled {
        Type::Row(component) => (int_type(), component.as_ref().clone()),
        Type::Primitive(Prim::String) => (int_type(), string_type()),
        Type::Primitive(Prim::Vec) => (int_type(), int_type()),
        Type::Primitive(Prim::RatVec) => (int_type(), rat_type()),
        Type::Primitive(Prim::Mat) => (int_type(), primitive_type(Prim::Vec)),
        Type::Primitive(Prim::KTypePol) => (
            primitive_type(Prim::KType),
            primitive_type(Prim::Split),
        ),
        Type::Primitive(Prim::ParamPol) => (
            primitive_type(Prim::Param),
            primitive_type(Prim::Split),
        ),
        _ => {
            return Err(type_error(
                format!(
                    "Cannot iterate over value of type {}",
                    found.display(analysis.types)
                ),
                loop_.iterable.span(),
            ));
        }
    };
    // The pattern claims the iterated component; the `@` name binds the
    // index (the upstream (index, pattern) pair wrap, in that slot order):
    // the 0-based position as int for ordinary aggregates, the KType or
    // Param term for the polynomial types.
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
    // The upstream (pattern, index) pair wrap is push_front-built
    // (parser.y:533-537, parsetree.w:1383-1385), so the INDEX takes
    // slot 0 and the pattern leaves follow (observable in the
    // back-trace frame dump: `{ k=0, i=2 }` for `for i@k`).
    if let Some(index) = index {
        locals.insert(
            index.value.clone(),
            (Rc::new(RefCell::new(index_type.clone())), 0, offset),
        );
        constant_locals.remove(&index.value);
        offset += 1;
    }
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
    let shape = pattern
        .as_ref()
        .map(pattern_slot_shape)
        .unwrap_or(SlotShape::Discard);
    // Slot names in bind order (the index name, then the leaves),
    // for the error-time frame dump (axis.w:6124-6161).
    let names: Vec<String> = index
        .iter()
        .map(|index| index.value.clone())
        .chain(leaves.iter().map(|(name, _, _, _)| name.clone()))
        .collect();
    // The required type steers the body (axis.w:5883-5924 for_expr
    // case): a void context evaluates the body for its side effects; a
    // row context hands its component type to the body, so e.g. a
    // `[(Split,KType)]` requirement narrows a body yielding
    // `(int,KType)` per component; any other required type must be
    // reachable by a registered row coercion, which then wraps the
    // whole loop.
    let row_of_type = Type::row(Type::Undetermined);
    let mut conv: Option<&crate::coercions::Coercion> = None;
    let mut body_type = if required.is_void() && !*iffor_body {
        Type::void()
    } else if required.is_void() || required.can_specialise(&row_of_type, analysis.types) {
        // An iffor loop in void context still builds its row of rows;
        // the enclosing `## ` join is what gets voided (axis.w: the
        // parser wraps the loop in a protected-concatenate call, so the
        // loop's own required type is never void).
        match row_component(required, analysis.types) {
            Some(component) => component,
            None => Type::Undetermined,
        }
    } else if let Some((coercion, component)) = row_coercion(required, analysis.types) {
        conv = Some(coercion);
        component.clone()
    } else {
        return Err(type_error(
            format!(
                "found {} while {} was needed.",
                row_of_type.display(analysis.types),
                required.display(analysis.types)
            ),
            *span,
        ));
    };
    // An iffor body is a conditional producing ROWS (parser.y:509-522
    // wraps the branches in list displays and the loop in a `## ` join):
    // its component is itself a row of the loop's eventual component.
    if *iffor_body && !body_type.is_void() {
        body_type = Type::row(body_type);
    }
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
            return_type: analysis.return_type.clone(),
            loop_depth: analysis.loop_depth + 1,
            last_value: analysis.last_value,
            last_value_type: analysis.last_value_type,
        },
    )?;
    let loop_type = Type::row(body_type);
    let converted = TypedExpr::For {
        shape,
        index: index.is_some(),
        names: Rc::from(names),
        iterable: Box::new(iterable),
        in_reversed: *in_reversed,
        body: Box::new(body),
        out_reversed: *out_reversed,
    };
    if *iffor_body {
        return join_iffor_body(converted, &loop_type, required, *span, analysis);
    }
    if let Some(coercion) = conv {
        let converted = TypedExpr::Conversion {
            tag: coercion.tag,
            inner: Box::new(converted),
            span: *span,
        };
        let target = coercion.to.clone();
        return conform_types(&target, required, converted, *span, analysis);
    }
    conform_types(&loop_type, required, converted, *span, analysis)
}

/// Convert a counted `for` loop (parser.y:550-573); kept out of
/// `convert_expr`'s frame like `convert_for_loop`.
fn convert_counted_for_loop(
    loop_: &crate::syntax::CountedForLoop,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let crate::syntax::CountedForLoop {
        name,
        count,
        bound,
        decreasing,
        in_reversed,
        body,
        out_reversed,
        iffor_body,
        span,
    } = loop_;
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
    // The required type steers the body exactly as in convert_for_loop
    // (axis.w:6457-6464 cfor_expr case): a void context evaluates the body
    // for its side effects; a row context hands its component type to the
    // body; any other required type must be reachable by a registered row
    // coercion, which then wraps the whole loop.
    let row_of_type = Type::row(Type::Undetermined);
    let mut conv: Option<&crate::coercions::Coercion> = None;
    let mut body_type = if required.is_void() && !*iffor_body {
        Type::void()
    } else if required.is_void() || required.can_specialise(&row_of_type, analysis.types) {
        match row_component(required, analysis.types) {
            Some(component) => component,
            None => Type::Undetermined,
        }
    } else if let Some((coercion, component)) = row_coercion(required, analysis.types) {
        conv = Some(coercion);
        component.clone()
    } else {
        return Err(type_error(
            format!(
                "found {} while {} was needed.",
                row_of_type.display(analysis.types),
                required.display(analysis.types)
            ),
            *span,
        ));
    };
    if *iffor_body && !body_type.is_void() {
        body_type = Type::row(body_type);
    }
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
            return_type: analysis.return_type.clone(),
            loop_depth: analysis.loop_depth + 1,
            last_value: analysis.last_value,
            last_value_type: analysis.last_value_type,
        },
    )?;
    let loop_type = Type::row(body_type);
    let converted = TypedExpr::CountedFor {
        name: name.as_ref().map(|name| name.value.clone()),
        decreasing: *decreasing,
        in_reversed: *in_reversed,
        out_reversed: *out_reversed,
        count: Box::new(count),
        bound,
        body: Box::new(body),
        span: *span,
    };
    if *iffor_body {
        return join_iffor_body(converted, &loop_type, required, *span, analysis);
    }
    if let Some(coercion) = conv {
        let converted = TypedExpr::Conversion {
            tag: coercion.tag,
            inner: Box::new(converted),
            span: *span,
        };
        let target = coercion.to.clone();
        return conform_types(&target, required, converted, *span, analysis);
    }
    conform_types(&loop_type, required, converted, *span, analysis)
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

/// The variable lookup shared by the component/field assignment family
/// (axis.w:8148-8160, 8216-8228): locals shadow globals, and both report
/// the assignment-specific undefined/constant diagnostics quoting the
/// compact rendering of the whole expression.
fn lookup_assignable(
    name: &str,
    span: SourceSpan,
    context: &str,
    compact: &str,
    analysis: &Analysis<'_>,
) -> Result<(AssignTarget, Type), Diagnostic> {
    if let Some((target, depth, offset)) = analysis.locals.get(name) {
        if analysis.constant_locals.contains(name) {
            return Err(Diagnostic::new(
                ErrorKind::Name,
                format!("Name '{name}' is constant in {context} {compact}"),
                Some(span),
            ));
        }
        return Ok((
            AssignTarget::Local {
                depth: *depth,
                offset: *offset,
            },
            target.borrow().clone(),
        ));
    }
    let Some((target, cell)) = analysis.globals.lookup(name) else {
        return Err(Diagnostic::new(
            ErrorKind::Name,
            format!("Undefined identifier '{name}' in {context} {compact}"),
            Some(span),
        ));
    };
    if analysis.globals.is_const(name) {
        return Err(Diagnostic::new(
            ErrorKind::Name,
            format!("Name '{name}' is constant in {context} {compact}"),
            Some(span),
        ));
    }
    Ok((AssignTarget::Global(cell.clone()), target.borrow().clone()))
}

/// The subscriptability check of a component assignment or transform
/// (axis.w:8163-8172, 8531-8546: `subscr_base::index_kind` gated on
/// `assignable`): rows, vec, and mat (column or two-index entry) admit
/// component assignment; ratvec is read-only upstream.
fn component_type_for_assignment(
    aggregate_type: &Type,
    index_type: &Type,
    transform: bool,
    span: SourceSpan,
    analysis: &Analysis<'_>,
) -> Result<Type, Diagnostic> {
    let int_index = matches!(index_type, Type::Primitive(Prim::Int));
    let pair_index = matches!(
        index_type,
        Type::Tuple(parts)
            if parts.len() == 2
                && parts
                    .iter()
                    .all(|part| matches!(part, Type::Primitive(Prim::Int)))
    );
    // axis.w:8163-8172 (`comp_ass_stat::assignability`): rows, vec, and mat
    // (column or two-index entry) admit component assignment; ratvec is
    // read-only upstream and falls to the generic diagnostic. index_kind
    // untables the aggregate (axis-types.w:375-384), so a tabled row like
    // sparse.at's sparse_mat assigns through its expansion; the diagnostic
    // still prints the original named type.
    let expanded_aggregate;
    let aggregate_kind: &Type = match aggregate_type {
        Type::Tabled(number) => {
            expanded_aggregate = analysis.types.expansion(*number).clone();
            &expanded_aggregate
        }
        other => other,
    };
    match aggregate_kind {
        Type::Row(component) if int_index => return Ok((**component).clone()),
        Type::Primitive(Prim::Vec) if int_index => return Ok(Type::Primitive(Prim::Int)),
        Type::Primitive(Prim::Mat) if int_index => return Ok(Type::Primitive(Prim::Vec)),
        Type::Primitive(Prim::Mat) if pair_index => return Ok(Type::Primitive(Prim::Int)),
        // Term-coefficient assignment `P[t]:=s` (axis.w:3962-3969 term
        // kinds are assignable; atlas-types.w:5650-5659, 7766-7782
        // `assign_coef`).
        Type::Primitive(Prim::KTypePol)
            if matches!(index_type, Type::Primitive(Prim::KType)) =>
        {
            return Ok(Type::Primitive(Prim::Split))
        }
        Type::Primitive(Prim::ParamPol)
            if matches!(index_type, Type::Primitive(Prim::Param)) =>
        {
            return Ok(Type::Primitive(Prim::Split))
        }
        _ => {}
    }
    let message = if transform {
        format!(
            "Cannot assign to component of value of type {} selected by index of type {} in transforming assignment",
            aggregate_type.display(analysis.types),
            index_type.display(analysis.types)
        )
    } else {
        format!(
            "Cannot subscript value of type {} with index of type {} in assignment",
            aggregate_type.display(analysis.types),
            index_type.display(analysis.types)
        )
    };
    Err(type_error(message, span))
}

/// The projector lookup of a field assignment or transform
/// (axis.w:8240-8266): the selector must resolve against the EXACT tuple
/// type (a tabled type compares by its expansion), and the bound value must
/// be a `set_type`-installed projector closure.
fn resolve_projector(
    field: &str,
    tuple_type: &Type,
    span: SourceSpan,
    analysis: &Analysis<'_>,
) -> Result<(usize, Type), Diagnostic> {
    let expanded = match tuple_type {
        Type::Tabled(number) => analysis.types.expansion(*number).clone(),
        other => other.clone(),
    };
    let improper = || type_error("Improper selection in field assignment".to_string(), span);
    let not_projector = || {
        type_error(
            "Selector in field assignment is not a projector function".to_string(),
            span,
        )
    };
    let matches = |argument: &Type| argument == tuple_type || argument == &expanded;
    // A user (`set`) overload with the exact argument type shadows the plain
    // global the `set_type` definition installed.
    let value = analysis
        .overloads
        .user_variants(field)
        .iter()
        .find(|variant| match &variant.function_type {
            Type::Function(parts) => matches(&parts.0),
            _ => false,
        })
        .map(|variant| variant.value.clone())
        .or_else(|| {
            let (target, cell) = analysis.globals.lookup(field)?;
            let Type::Function(parts) = &*target.borrow() else {
                return None;
            };
            matches(&parts.0)
                .then(|| cell.borrow().as_ref().map(|value| value.as_ref().clone()))
                .flatten()
        });
    let Some(value) = value else {
        return Err(improper());
    };
    let Value::Closure(closure) = &value else {
        return Err(not_projector());
    };
    let TypedExpr::TupleProject { index, .. } = closure.body.as_ref() else {
        return Err(not_projector());
    };
    let position = *index;
    let Type::Tuple(components) = &expanded else {
        return Err(improper());
    };
    let Some(component) = components.get(position) else {
        return Err(improper());
    };
    Ok((position, component.clone()))
}

/// Try to factor the converted desugared operator call of a transform
/// assignment into the builtin operation to apply and its converted right
/// operand. Upstream only builds an in-place transform when the resolved
/// call is a `builtin_call` whose FIRST argument is the unconverted
/// selection (a subscription for `a[i] op:= v`, a projector call for
/// `p.f op:= v`); a user overload, an implicit conversion, or any other
/// shape reverts to an ordinary assignment of the whole call
/// (axis.w:8422-8455 `field_trans_stat`, axis.w:8572-8596
/// `comp_trans_stat`). Returns the call back when the optimisation does
/// not apply. The Rust converter never applies the upstream `x+1` →
/// `succ(x)` argument-dropping optimisation, so an optimisable binary
/// call keeps both operands.
fn factor_transform_call(
    call: TypedExpr,
    selection_is_subscription: bool,
) -> Result<(TransformOperation, Box<TypedExpr>), TypedExpr> {
    match call {
        TypedExpr::BuiltinCall {
            builtin,
            mut arguments,
            ..
        } if arguments.len() == 2
            && if selection_is_subscription {
                matches!(arguments[0], TypedExpr::Subscription { .. })
            } else {
                matches!(arguments[0], TypedExpr::FunctionCall { .. })
            } =>
        {
            let rhs = arguments
                .pop()
                .expect("a binary operator call holds its right operand");
            Ok((TransformOperation::Builtin(builtin), Box::new(rhs)))
        }
        other => Err(other),
    }
}

/// `a[i] := v` (parser.y:265, axis.w:8131-8192 `comp_ass_stat`).
fn convert_component_assignment(
    expression: &Expr,
    assignment: &ComponentAssignmentExpr,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let compact = compact_expression(expression);
    let (target, aggregate_type) = lookup_assignable(
        &assignment.name,
        assignment.span,
        "component assignment",
        &compact,
        analysis,
    )?;
    // The index converts with an undetermined a-priori type, like upstream.
    let mut index_type = Type::Undetermined;
    let converted_index = convert_expr(&assignment.index, &mut index_type, analysis)?;
    let component_type = component_type_for_assignment(
        &aggregate_type,
        &index_type,
        false,
        assignment.span,
        analysis,
    )?;
    let mut required_value = component_type;
    let converted_value = convert_expr(&assignment.value, &mut required_value, analysis)?;
    // The oracle's range_mess prints the CONVERTED assignment node, so a
    // coerced right side shows its conversion tag: `r[5]:=QI:1`,
    // `M[5]:=V[I]:[1,2]` (parsetree.w:2989-3020 with the conversion print).
    let compact = match &converted_value {
        TypedExpr::Conversion { tag, .. } => format!(
            "{}{}[{}]:={tag}:{}",
            assignment.name,
            if assignment.reversed { "~" } else { "" },
            compact_expression(&assignment.index),
            compact_expression(&assignment.value)
        ),
        _ => compact,
    };
    conform_types(
        &required_value,
        required,
        TypedExpr::ComponentAssignment {
            target,
            name: assignment.name.clone(),
            index: Box::new(converted_index),
            reversed: assignment.reversed,
            value: Box::new(converted_value),
            source: compact,
            span: assignment.span,
        },
        assignment.span,
        analysis,
    )
}

/// `a[i] op:= v` (parser.y:272, axis.w:8495+ `comp_trans_stat`): after the
/// assignability checks the desugared call `op(a[i], v)` converts against
/// the component type, which resolves the operation and produces exactly
/// the upstream diagnostics (`Failed to match …`, `found … while …`).
fn convert_component_transform(
    expression: &Expr,
    transform: &ComponentTransformExpr,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let compact = compact_expression(expression);
    let (target, aggregate_type) = lookup_assignable(
        &transform.name,
        transform.span,
        "component transform",
        &compact,
        analysis,
    )?;
    let mut index_type = Type::Undetermined;
    let converted_index = convert_expr(&transform.index, &mut index_type, analysis)?;
    let component_type = component_type_for_assignment(
        &aggregate_type,
        &index_type,
        true,
        transform.span,
        analysis,
    )?;
    let subscription = Expr::Subscription {
        array: Box::new(Expr::Identifier {
            name: transform.name.clone(),
            span: transform.name_span,
        }),
        index: Box::new(transform.index.clone()),
        reversed: transform.reversed,
        span: transform.span,
    };
    let call = Expr::OperatorCall {
        operator: FormulaOperator::new(transform.operator.clone(), 0)
            .with_span(transform.operator_span),
        arguments: vec![subscription, transform.value.clone()],
        span: transform.span,
    };
    let mut call_required = component_type.clone();
    let converted_call = convert_expr(&call, &mut call_required, analysis)?;
    // The optimised in-place transform applies only to a row/vec entry
    // selected by an int index (subscr_base::row_entry, axis.w:8560-8562);
    // matrix selections and any call the factorer rejects (user overload,
    // implicit conversion) become an ordinary component assignment whose
    // value is the whole call (axis.w:8597-8604). Unlike upstream we do
    // not let-wrap a side-effecting index expression here; the converted
    // subscription inside the call re-evaluates it.
    let row_entry = matches!(&aggregate_type, Type::Row(_) | Type::Primitive(Prim::Vec))
        && matches!(index_type, Type::Primitive(Prim::Int));
    let factored = if row_entry {
        factor_transform_call(converted_call, true)
    } else {
        Err(converted_call)
    };
    let converted = match factored {
        Ok((operation, rhs)) => {
            // The vec/mat transform range check fires on the component READ,
            // whose diagnostic quotes the selection (`M[5]`, pair index
            // without the tuple parentheses `M[5,0]`) — not the whole
            // transform compact.
            let selection = match &transform.index {
                Expr::Tuple { elements, .. } if elements.len() == 2 => format!(
                    "{}{}[{},{}]",
                    transform.name,
                    if transform.reversed { "~" } else { "" },
                    compact_expression(&elements[0]),
                    compact_expression(&elements[1])
                ),
                index => format!(
                    "{}{}[{}]",
                    transform.name,
                    if transform.reversed { "~" } else { "" },
                    compact_expression(index)
                ),
            };
            TypedExpr::ComponentTransform {
                target,
                name: transform.name.clone(),
                index: Box::new(converted_index),
                reversed: transform.reversed,
                operation,
                rhs,
                conversion: None,
                selection,
                source: compact,
                span: transform.span,
            }
        }
        Err(call) => TypedExpr::ComponentAssignment {
            target,
            name: transform.name.clone(),
            index: Box::new(converted_index),
            reversed: transform.reversed,
            value: Box::new(call),
            source: compact,
            span: transform.span,
        },
    };
    conform_types(&component_type, required, converted, transform.span, analysis)
}

/// `p.f := v` (parser.y:266, axis.w:8194-8239 `field_ass_stat`).
fn convert_field_assignment(
    expression: &Expr,
    assignment: &FieldAssignmentExpr,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let compact = compact_expression(expression);
    let (target, tuple_type) = lookup_assignable(
        &assignment.name,
        assignment.span,
        "field assignment",
        &compact,
        analysis,
    )?;
    let (position, component_type) =
        resolve_projector(&assignment.field, &tuple_type, assignment.span, analysis)?;
    let mut required_value = component_type;
    let converted_value = convert_expr(&assignment.value, &mut required_value, analysis)?;
    conform_types(
        &required_value,
        required,
        TypedExpr::FieldAssignment {
            target,
            name: assignment.name.clone(),
            position,
            value: Box::new(converted_value),
            span: assignment.span,
        },
        assignment.span,
        analysis,
    )
}

/// `p.f op:= v` (parser.y:274, axis.w:8286+ `field_trans_stat`): like the
/// component transform, the desugared call `op(f(p), v)` converts against
/// the component type.
fn convert_field_transform(
    expression: &Expr,
    transform: &FieldTransformExpr,
    required: &mut Type,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let compact = compact_expression(expression);
    let (target, tuple_type) = lookup_assignable(
        &transform.name,
        transform.span,
        "field transform",
        &compact,
        analysis,
    )?;
    let (position, component_type) =
        resolve_projector(&transform.field, &tuple_type, transform.span, analysis)?;
    let selection = Expr::Call {
        callee: Box::new(Expr::Identifier {
            name: transform.field.clone(),
            span: transform.field_span,
        }),
        arguments: vec![Expr::Identifier {
            name: transform.name.clone(),
            span: transform.name_span,
        }],
        span: transform.span,
    };
    let call = Expr::OperatorCall {
        operator: FormulaOperator::new(transform.operator.clone(), 0)
            .with_span(transform.operator_span),
        arguments: vec![selection, transform.value.clone()],
        span: transform.span,
    };
    let mut call_required = component_type.clone();
    let converted_call = convert_expr(&call, &mut call_required, analysis)?;
    // As for the component transform: only a builtin call whose first
    // argument is the unconverted projector call becomes an in-place
    // transform; anything else assigns the whole call (axis.w:8422-8455).
    let converted = match factor_transform_call(converted_call, false) {
        Ok((operation, rhs)) => TypedExpr::FieldTransform {
            target,
            name: transform.name.clone(),
            position,
            operation,
            rhs,
            conversion: None,
            span: transform.span,
        },
        Err(call) => TypedExpr::FieldAssignment {
            target,
            name: transform.name.clone(),
            position,
            value: Box::new(call),
            span: transform.span,
        },
    };
    conform_types(&component_type, required, converted, transform.span, analysis)
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
        name,
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
                name,
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
                name,
                span,
            };
        }
    };
    TypedExpr::HungryBuiltinCall {
        builtin,
        arguments,
        name,
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
            // A tabled (named) type is transparent here: upstream's
            // `type.kind()`/`type.tuple()` accessors untable, so a tuple
            // pattern destructures the expansion (axis-types.w:376-382,
            // thread_bindings axis.w:2743-2754). The `whole` name, if any,
            // still binds to the tabled type itself.
            let expanded;
            let expanded_found = if let Type::Tabled(number) = found {
                expanded = types.expansion(*number).clone();
                &expanded
            } else {
                found
            };
            let Type::Tuple(components) = expanded_found else {
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
            // Resolve against the session table so user-defined type names
            // (TYPE_ID annotations like `(maybe_a_vec x)`) bind; an unknown
            // name is the same diagnostic as a set_type spec reference.
            let declared = typed.type_expr.resolve_in(types).map_err(|unknown| {
                Diagnostic::new(
                    ErrorKind::Name,
                    format!("undefined type name '{}'", unknown.value),
                    Some(unknown.span),
                )
            })?;
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
    // Frame slot names in bind order, retained for the error-time frame
    // dump (axis.w:2896-2909).
    let mut param_names = Vec::new();
    let mut offset = 0;
    for (parameter_type, shape, leaves) in converted_parameters {
        parameter_types.push(parameter_type);
        shapes.push(shape);
        for (name, _, constant, leaf_type) in leaves {
            param_names.push(name.clone());
            locals.insert(name.clone(), (Rc::new(RefCell::new(leaf_type)), 0, offset));
            if constant {
                constant_locals.insert(name.clone());
            } else {
                constant_locals.remove(&name);
            }
            offset += 1;
        }
    }
    // The shared result-type cell (axis.w:313): `return` clauses convert
    // against it even from void contexts, and narrowings they alone make
    // flow back into the function type after the body is converted.
    let return_cell: TypeCell = Rc::new(RefCell::new(Type::Undetermined));
    let body_analysis = Analysis {
        types: analysis.types,
        globals: analysis.globals,
        overloads: analysis.overloads,
        locals,
        constant_locals,
        in_function: true,
        return_type: Some(return_cell.clone()),
        // A closure evaluates in its captured context, not the defining
        // loop's; `break` legality starts over at the function boundary.
        loop_depth: 0,
        last_value: analysis.last_value,
        last_value_type: analysis.last_value_type,
    };
    let closure = |body: TypedExpr| TypedExpr::Closure {
        parameters: parameters.len(),
        shapes: Rc::from(shapes),
        recursive: false,
        body: Rc::new(body),
        span,
        param_names: Rc::from(param_names),
    };
    if required.is_void() {
        // Upstream converts the body against a dummy result type
        // (axis.w:3105-3109): the cell stays undetermined, and `return`
        // clauses check against it without affecting anything.
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
    if let Type::Tabled(number) = required {
        // A tabled required type only CHECKS its expansion (specialise does
        // not unwrap it, and the tabled type has no holes to fill): convert
        // the body against the expansion's fixed result type instead.
        let Type::Function(parts) = analysis.types.expansion(*number).clone() else {
            unreachable!("specialise accepted a non-function expansion")
        };
        let mut result = parts.1;
        // The expansion is fully determined, so `return` clauses cannot
        // narrow it; seeding the cell lets them type-check against it.
        *return_cell.borrow_mut() = result.clone();
        let converted = convert_expr(body, &mut result, &body_analysis)?;
        return Ok(closure(converted));
    }
    let Type::Function(parts) = required else {
        unreachable!("specialising to a function pattern yields a function type")
    };
    *return_cell.borrow_mut() = parts.1.clone();
    // `set f(...) = T: body` desugars the declared result into a `T: body`
    // cast (syntax.rs lambda_with_result), so the cell seeded above is
    // still undetermined when the context gave nothing. Seed it from that
    // cast — upstream seeds the shared result cell from the lambda's
    // declared result directly (axis.w:313) — otherwise a `return` clause
    // narrows the cell to its raw a-priori type (`return null(0)` inside
    // `set f(...)=[int]: ...` leaked vec and then contradicted the body's
    // coerced [int]).
    if matches!(*return_cell.borrow(), Type::Undetermined) {
        if let Expr::Cast { target, .. } = body {
            if let Ok(declared) = target.resolve_in(analysis.types) {
                *return_cell.borrow_mut() = declared;
            }
        }
    }
    let converted = convert_expr(body, &mut parts.1, &body_analysis)?;
    // Propagate narrowings that only `return` clauses made (the body's own
    // narrowings are already in `parts.1`; a conflict means a return type
    // disagreed with the body, which upstream rejects at the return).
    if !parts.1.specialise(&return_cell.borrow(), analysis.types) {
        return Err(type_error(
            format!(
                "type {} does not match required pattern {}",
                return_cell.borrow().display(analysis.types),
                parts.1.display(analysis.types)
            ),
            span,
        ));
    }
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
    let resolved_result = result_type.resolve_in(analysis.types).map_err(|unknown| {
        Diagnostic::new(
            ErrorKind::Name,
            format!("undefined type name '{}'", unknown.value),
            Some(unknown.span),
        )
    })?;
    let function_type = Type::function(Type::tuple(parameter_types), resolved_result.clone());
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
    // Frame slot names in bind order: the self binding at slot 0, then the
    // parameter leaves (for the error-time frame dump, axis.w:2896-2909).
    let mut param_names = vec![self_name.clone()];
    for (_, _, leaves) in converted_parameters {
        for (name, _, constant, leaf_type) in leaves {
            param_names.push(name.clone());
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
        // The declared result type is fully determined, so a shared cell
        // suffices for `return` clauses (axis.w:3154 f_type.func()->result).
        return_type: Some(Rc::new(RefCell::new(resolved_result.clone()))),
        // A closure evaluates in its captured context, not the defining
        // loop's; `break` legality starts over at the function boundary.
        loop_depth: 0,
        last_value: analysis.last_value,
        last_value_type: analysis.last_value_type,
    };
    let closure = |body: TypedExpr| TypedExpr::Closure {
        parameters: parameters.len(),
        shapes: Rc::from(shapes),
        recursive: true,
        body: Rc::new(body),
        span: *span,
        param_names: Rc::from(param_names),
    };
    if required.is_void() {
        let mut dummy = resolved_result.clone();
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
    let mut result_required = resolved_result;
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
    let has_hidden_special = hidden_special_builtin(name).is_some();
    if resolve_name_first && variants.is_empty() && !has_hidden_special {
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
    // recognises the hidden generic row `#`/`##` instances, and only
    // afterwards considers coercible ordinary overloads (axis.w:2458-2595).
    // Upstream's exact test is type_expr::operator==, which treats a tabled
    // type as equal to its expansion (axis-types.w:807-825) — a call with a
    // KGBElt_gen argument exactly matches a (InnerClass,mat,ratvec) overload.
    let exact = variants
        .iter()
        .position(|variant| variant.arg_type.equals(&a_priori_type, analysis.types));
    let hidden = if exact.is_none() {
        hidden_special_variant(name, &a_priori_type, analysis.types)
    } else {
        None
    };
    let inexact = if exact.is_none() && hidden.is_none() {
        variants.iter().position(|variant| {
            crate::coercions::is_close(&a_priori_type, &variant.arg_type, analysis.types) & 0x1 != 0
        })
    } else {
        None
    };
    let position = exact.or(inexact);
    if position.is_none() && hidden.is_none() {
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
    let variant = if let Some((index, result_type)) = hidden {
        hidden_variant = MergedVariant {
            arg_type: a_priori_type.clone(),
            result_type,
            origin: OverloadOrigin::Builtin(index),
        };
        &hidden_variant
    } else {
        &variants[position.expect("ordinary overload was found")]
    };
    let expected: Vec<Type> = if expressions.len() == 1 {
        vec![variant.arg_type.clone()]
    } else {
        // A tabled parameter type stands for its expansion: upstream converts
        // the argument tuple against arg_type whose accessors untable
        // (axis-types.w:376-382), so the per-argument slots are the
        // expansion's components, not the named type itself.
        let mut argument_type = &variant.arg_type;
        while let Type::Tabled(number) = argument_type {
            argument_type = analysis.types.expansion(*number);
        }
        match argument_type {
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
    // The resolved call name retained for the error back-trace
    // (axis.w:1647-1648): the overload name and its argument type, e.g.
    // `g@int`, `%@(int,int)`, `z@void` for a zero-parameter variant.
    let trace_name = format!("{name}@{}", variant.arg_type.display(analysis.types));
    match variant.origin {
        OverloadOrigin::Builtin(index) => conform_types(
            &variant.result_type,
            required,
            TypedExpr::BuiltinCall {
                builtin: index,
                arguments,
                name: trace_name,
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
                    function: Box::new(TypedExpr::Denotation(Rc::new(user.value.clone()))),
                    argument: Box::new(argument),
                    name: Some(trace_name),
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
    /// readline_completions (global.w:3546-3561): filters the completion
    /// candidate snapshot the command layer stashed in the evaluation
    /// context by the argument prefix.
    Completions,
    /// The variadic generic `prints@@T` (axis.w:8773, wrapper :8850-8853):
    /// a hidden special instance matched by `hidden_special_variant` for any
    /// a-priori argument type. Writes its arguments unseparated and
    /// unquoted (a tuple argument expands one level), followed by a newline,
    /// at BOTH levels; yields the empty tuple at single_value
    /// (axis.w:8821-8848 to_string_aux).
    Prints,
    /// The variadic generic `print@@T` (axis.w:8767, wrapper :8796-8802):
    /// prints the argument verbatim (strings quoted) and returns it
    /// unchanged at value-demanding levels.
    Print,
    /// The variadic generic `to_string@@T` (axis.w:8769, wrapper
    /// :8841-8846): the stripped concatenation as a string value.
    ToString,
    /// The variadic generic `error@@T` (axis.w:8771, wrapper :8855-8859):
    /// raises the stripped concatenation as a runtime error.
    Error,
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
    RowSuffixElement,
    RowPrefixElement,
    RowJoinRows,
    RowJoinRowOfRows,
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
    MatEchelon,
    MatKernel,
    MatEigenLattice,
    MatRowSaturate,
    MatSmith,
    MatAdaptedBasis,
    MatDiagonalize,
    MatInvert,
    VecBezout,
    LinearSolve,
    SwissMatrixKnife,
    Mod2Section,
    SubspaceNormal,
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
    /// Arguments arrive SHARED: read-only implementations (the scalar
    /// operators, relations, printers) borrow them through the `Rc`; the
    /// domain dispatch and the consuming scalar operators take ownership of
    /// just what they consume via `own_all`/`unwrap_shared`, which moves
    /// uniquely held values and copies genuinely shared ones (the same
    /// copy-on-write step the old call-site unwrap performed on every
    /// argument).
    fn run(
        &self,
        arguments: Vec<SharedValue>,
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
                            domain_builtins::validate(name, &own_all(arguments), span)
                                .map_err(Control::Runtime)?;
                            return Ok(None);
                        }
                        DomainNoValue::BuildAndDrop => {}
                    }
                }
                domain_builtins::call_owned_with_printed(
                    name,
                    own_all(arguments),
                    span,
                    context.printed_buffer(),
                )
                .map(|value| at_builtin_level(level, || value))
                .map_err(Control::Runtime)
            }
            BuiltinImpl::DomainPrinter { name } => {
                let text = domain_builtins::print_text(name, &own_all(arguments), span)
                    .map_err(Control::Runtime)?;
                context.print_text(text);
                Ok(at_builtin_level(level, || Value::Tuple(Vec::new())))
            }
            BuiltinImpl::DomainRelation(relation) => {
                let (first, second) = expect_pair(&arguments);
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
            BuiltinImpl::Completions => {
                let Value::String(prefix) = expect_unary(&arguments) else {
                    panic!("readline_completions saw a non-string argument")
                };
                Ok(at_builtin_level(level, || {
                    Value::List(
                        context
                            .completion_candidates()
                            .iter()
                            .filter(|name| name.starts_with(prefix.as_str()))
                            .map(|name| Rc::new(Value::String(name.clone())))
                            .collect(),
                    )
                }))
            }
            BuiltinImpl::Prints => {
                // axis.w:8850-8853: the report is written at every level;
                // only single_value yields the (empty tuple) value.
                let text = prints_text(context, &arguments);
                context.print_text(text);
                Ok(at_builtin_level(level, || Value::Tuple(Vec::new())))
            }
            BuiltinImpl::Print => {
                // axis.w:8796-8802: the argument prints verbatim (the
                // standard value printer, quotes and all — closures print
                // their full multi-line form at any depth) at every level,
                // and is returned unchanged when a value is demanded.
                let value = if arguments.len() == 1 {
                    unwrap_shared(arguments.into_iter().next().expect("one argument"))
                } else {
                    Value::Tuple(arguments)
                };
                let text = value_string(context, &value);
                context.print_text(format!("{text}\n"));
                Ok(at_builtin_level(level, || value))
            }
            BuiltinImpl::ToString => {
                // axis.w:8841-8846: the stripped concatenation, no trailing
                // newline; no value is produced in void context.
                let text = stripped_text(context, &arguments);
                Ok(at_builtin_level(level, || Value::String(text)))
            }
            BuiltinImpl::Error => {
                // axis.w:8855-8859: always throws; the level is irrelevant.
                Err(runtime(stripped_text(context, &arguments), span))
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

/// `to_string_aux` (axis.w:8819-8840) applied to the variadic argument
/// tuple: string components print without quotes, every other value prints
/// like `print`. No trailing newline — `prints` adds the wrapper's
/// `std::endl` itself, `to_string` and `error` do not.
fn stripped_text(context: &EvaluationContext, arguments: &[SharedValue]) -> String {
    fn component(context: &EvaluationContext, text: &mut String, value: &Value) {
        match value {
            Value::String(string) => text.push_str(string),
            // The standard value printer, so a closure argument (or one
            // nested in a tuple) prints its full multi-line form
            // (axis.w:3254-3271), not the bare Display head.
            other => text.push_str(&value_string(context, other)),
        }
    }
    let mut text = String::new();
    // A single argument arrives unwrapped (the variadic tuple collapses), so
    // a lone tuple's components print individually; anything else prints as
    // one value.
    if arguments.len() == 1 {
        match arguments[0].as_ref() {
            Value::Tuple(components) => {
                for value in components {
                    component(context, &mut text, value);
                }
            }
            value => component(context, &mut text, value),
        }
    } else {
        for value in arguments {
            component(context, &mut text, value);
        }
    }
    text
}

/// `prints_wrapper`'s output (axis.w:8850-8853): the stripped text plus one
/// trailing newline.
fn prints_text(context: &EvaluationContext, arguments: &[SharedValue]) -> String {
    format!("{}\n", stripped_text(context, arguments))
}

fn expect_unary(arguments: &[SharedValue]) -> &Value {
    match arguments {
        [value] => value.as_ref(),
        _ => panic!("unary builtin saw {} arguments", arguments.len()),
    }
}

fn expect_pair(arguments: &[SharedValue]) -> (&Value, &Value) {
    match arguments {
        [first, second] => (first.as_ref(), second.as_ref()),
        _ => panic!("binary builtin saw {} arguments", arguments.len()),
    }
}

fn expect_ints(arguments: &[SharedValue]) -> (&BigInt, &BigInt) {
    match expect_pair(arguments) {
        (Value::Integer(first), Value::Integer(second)) => (first, second),
        other => panic!("int builtin saw {other:?}"),
    }
}

fn expect_rationals(arguments: &[SharedValue]) -> (&BigRational, &BigRational) {
    match expect_pair(arguments) {
        (Value::Rational(first), Value::Rational(second)) => (first, second),
        other => panic!("rat builtin saw {other:?}"),
    }
}

fn expect_rat_int(arguments: &[SharedValue]) -> (&BigRational, &BigInt) {
    match expect_pair(arguments) {
        (Value::Rational(first), Value::Integer(second)) => (first, second),
        other => panic!("rat-int builtin saw {other:?}"),
    }
}

/// Materialize owned arguments for a builtin that consumes its operands:
/// uniquely held values move (a fresh temporary or a pilfered operand), a
/// genuinely shared value copies — exactly the per-argument step the old
/// call-site unwrap performed.
fn own_all(arguments: Vec<SharedValue>) -> Vec<Value> {
    arguments.into_iter().map(unwrap_shared).collect()
}

fn expect_unary_owned(mut arguments: Vec<Value>) -> Value {
    let value = arguments.pop().expect("unary builtin has one argument");
    assert!(arguments.is_empty(), "unary builtin saw extra arguments");
    value
}

fn expect_pair_owned(mut arguments: Vec<Value>) -> (Value, Value) {
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
    arguments: Vec<SharedValue>,
    span: SourceSpan,
    level: Level,
) -> Result<Option<Value>, Control> {
    match operation {
        ScalarOp::IntNegate => match expect_unary(&arguments) {
            Value::Integer(value) => Ok(at_builtin_level(level, || Value::Integer(-value))),
            other => panic!("integer negation saw {other:?}"),
        },
        ScalarOp::IntAdd | ScalarOp::IntSubtract | ScalarOp::IntMultiply => {
            let (first, second) = expect_ints(&arguments);
            Ok(at_builtin_level(level, || {
                Value::Integer(match operation {
                    ScalarOp::IntAdd => first + second,
                    ScalarOp::IntSubtract => first - second,
                    ScalarOp::IntMultiply => first * second,
                    _ => unreachable!(),
                })
            }))
        }
        ScalarOp::IntComplement => match expect_unary(&arguments) {
            Value::Integer(value) => Ok(at_builtin_level(level, || Value::Integer(!value))),
            other => panic!("integer complement saw {other:?}"),
        },
        ScalarOp::IntQuotient | ScalarOp::IntModulo | ScalarOp::IntDivMod => {
            let (first, second) = expect_ints(&arguments);
            if *second == 0 {
                let message = match operation {
                    ScalarOp::IntQuotient => "Division by zero",
                    ScalarOp::IntModulo => "Modulo zero",
                    ScalarOp::IntDivMod => "DivMod by zero",
                    _ => unreachable!(),
                };
                return Err(runtime(message, span));
            }
            Ok(at_builtin_level(level, || {
                let (quotient, remainder) = euclidean_divmod(first, second);
                match operation {
                    ScalarOp::IntQuotient => Value::Integer(quotient),
                    ScalarOp::IntModulo => Value::Integer(remainder),
                    ScalarOp::IntDivMod => Value::Tuple(vec![
                        Rc::new(Value::Integer(quotient)),
                        Rc::new(Value::Integer(remainder)),
                    ]),
                    _ => unreachable!(),
                }
            }))
        }
        ScalarOp::IntPower => {
            let (base, exponent) = expect_ints(&arguments);
            let unit_base = *base == 1 || *base == -1;
            if !unit_base && *exponent < 0 {
                return Err(runtime("Negative power of integer", span));
            }
            if !unit_base && *base != 0 && i32::try_from(exponent).is_err() {
                return Err(runtime("Exponent too large in power of integer", span));
            }
            Ok(at_builtin_level(level, || {
                if unit_base {
                    if exponent % BigInt::from(2) != 0 {
                        Value::Integer(base.clone())
                    } else {
                        Value::Integer(BigInt::from(1))
                    }
                } else if *base == 0 {
                    Value::Integer(if *exponent == 0 {
                        BigInt::from(1)
                    } else {
                        base.clone()
                    })
                } else {
                    Value::Integer(
                        base.pow(u64::from(
                            u32::try_from(i32::try_from(exponent).expect("validated exponent"))
                                .expect("validated exponent is nonnegative"),
                        )),
                    )
                }
            }))
        }
        ScalarOp::IntInverse => match expect_unary(&arguments) {
            Value::Integer(value) => {
                if *value == 0 {
                    return Err(runtime("Inverse of zero", span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Rational(BigRational::from_integers(BigInt::from(1), value.clone()))
                }))
            }
            other => panic!("integer inverse saw {other:?}"),
        },
        ScalarOp::IntFraction => {
            let (numerator, denominator) = expect_ints(&arguments);
            if *denominator == 0 {
                return Err(runtime("fraction with zero denominator", span));
            }
            Ok(at_builtin_level(level, || {
                Value::Rational(BigRational::from_integers(
                    numerator.clone(),
                    denominator.clone(),
                ))
            }))
        }
        ScalarOp::RatUnfraction => match expect_unary(&arguments) {
            Value::Rational(value) => Ok(at_builtin_level(level, || {
                let negative = *value < 0;
                let (numerator, denominator) = value.clone().into_numerator_and_denominator();
                let numerator = BigInt::from(numerator);
                Value::Tuple(vec![
                    Rc::new(Value::Integer(if negative { -numerator } else { numerator })),
                    Rc::new(Value::Integer(BigInt::from(denominator))),
                ])
            })),
            other => panic!("rational unfraction saw {other:?}"),
        },
        ScalarOp::VecAdd => match expect_pair(&arguments) {
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
                            .iter()
                            .zip(&right.0)
                            .map(|(&a, &b)| a.wrapping_add(b))
                            .collect(),
                    ))
                }))
            }
            other => panic!("vector addition saw {other:?}"),
        },
        ScalarOp::VecNegate => match expect_unary(&arguments) {
            Value::Vector(vector) => Ok(at_builtin_level(level, || {
                Value::Vector(Vec32(
                    vector
                        .0
                        .iter()
                        .map(|&entry| i32::wrapping_neg(entry))
                        .collect(),
                ))
            })),
            other => panic!("vector negation saw {other:?}"),
        },
        ScalarOp::VecDivideInt => match expect_pair(&arguments) {
            (Value::Vector(vector), Value::Integer(denominator)) => {
                // Upstream constructs the rational vector inside its
                // no_value gate, so the diagnostics fire only when the
                // value is actually produced.
                if level == Level::NoValue {
                    return Ok(None);
                }
                if *denominator == 0 {
                    return Err(runtime("Denominator 0 in rational vector", span));
                }
                let negative = *denominator < 0;
                let magnitude = if negative {
                    -denominator
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
            let (rational, integer) = expect_rat_int(&arguments);
            if *integer == 0 {
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
                // Machine-word fast path (ratfast): the same normalized
                // value, without malachite limb churn; None falls through
                // to the generic operators below.
                let fast = match operation {
                    ScalarOp::RatAddInt => ratfast::add_int(rational, integer),
                    ScalarOp::RatSubtractInt => ratfast::sub_int(rational, integer),
                    ScalarOp::RatMultiplyInt => ratfast::mul_int(rational, integer),
                    ScalarOp::RatDivideInt => ratfast::div_int(rational, integer),
                    _ => None,
                };
                if let Some(value) = fast {
                    return Value::Rational(value);
                }
                let integer_as_rational = BigRational::from(integer.clone());
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
            let (first, second) = expect_rationals(&arguments);
            if *second == 0 {
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
                // Machine-word fast path (ratfast): the same normalized
                // value; None falls through to the generic operators.
                let fast = match operation {
                    ScalarOp::RatAdd => ratfast::add(first, second),
                    ScalarOp::RatSubtract => ratfast::sub(first, second),
                    ScalarOp::RatMultiply => ratfast::mul(first, second),
                    ScalarOp::RatDivide => ratfast::div(first, second),
                    _ => None,
                };
                Value::Rational(match fast {
                    Some(value) => value,
                    None => match operation {
                        ScalarOp::RatAdd => first + second,
                        ScalarOp::RatSubtract => first - second,
                        ScalarOp::RatMultiply => first * second,
                        ScalarOp::RatDivide => first / second,
                        ScalarOp::RatModulo => first.mod_op(second),
                        _ => unreachable!(),
                    },
                })
            }))
        }
        ScalarOp::RatNegate => match expect_unary(&arguments) {
            Value::Rational(value) => Ok(at_builtin_level(level, || Value::Rational(-value))),
            other => panic!("rational negation saw {other:?}"),
        },
        ScalarOp::RatInverse => match expect_unary(&arguments) {
            Value::Rational(value) => {
                if *value == 0 {
                    return Err(runtime("Inverse of zero", span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Rational(BigRational::from(1) / value)
                }))
            }
            other => panic!("rational inverse saw {other:?}"),
        },
        ScalarOp::RatPower => {
            let (base, exponent) = expect_rat_int(&arguments);
            let unit_base = *base == 1 || *base == -1;
            if *base == 0 && *exponent < 0 {
                return Err(runtime("Negative power of rational zero", span));
            }
            if *base != 0 && !unit_base && i32::try_from(exponent).is_err() {
                return Err(runtime(
                    "Exponent too large in power of rational number",
                    span,
                ));
            }
            if level == Level::NoValue {
                return Ok(None);
            }
            if *base != 0 && !unit_base && *exponent < 0 {
                return Err(runtime("Negative integer where unsigned is required", span));
            }
            Ok(Some({
                if unit_base {
                    if exponent % BigInt::from(2) != 0 {
                        Value::Rational(base.clone())
                    } else {
                        Value::Rational(BigRational::from(1))
                    }
                } else if *base == 0 {
                    Value::Rational(if *exponent == 0 {
                        BigRational::from(1)
                    } else {
                        base.clone()
                    })
                } else {
                    Value::Rational(base.pow(i64::from(
                        i32::try_from(exponent).expect("validated exponent"),
                    )))
                }
            }))
        }
        ScalarOp::NullMatrix => {
            let (row_value, column_value) = expect_ints(&arguments);
            // Atlas pops/converts the column count first, then the row count.
            let columns = unsigned_long(column_value, span)?;
            let rows = unsigned_long(row_value, span)?;
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
            let value = expect_unary(&arguments);
            Ok(at_builtin_level(level, || {
                // Containers compare against an implicit zero of the same
                // kind: only =/!=/>=/> are registered for vec and ratvec,
                // only =/!= for mat (global.w:4405-4420).
                let result = match value {
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
            let (first, second) = expect_pair(&arguments);
            Ok(at_builtin_level(level, || {
                let result = match (first, second) {
                    (Value::Integer(first), Value::Integer(second)) => {
                        relation_matches(relation, first.cmp(second))
                    }
                    (Value::Rational(first), Value::Rational(second)) => {
                        // Cross-multiplied i128 comparison when both are
                        // machine-sized; identical ordering to the generic
                        // cmp since denominators are positive.
                        let ordering =
                            ratfast::cmp(first, second).unwrap_or_else(|| first.cmp(second));
                        relation_matches(relation, ordering)
                    }
                    (Value::Boolean(first), Value::Boolean(second)) => {
                        relation_matches(relation, first.cmp(second))
                    }
                    (Value::String(first), Value::String(second)) => {
                        relation_matches(relation, first.cmp(second))
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
        ScalarOp::ListCardinality => match expect_unary(&arguments) {
            Value::List(values) => Ok(at_builtin_level(level, || {
                Value::Integer(BigInt::from(values.len()))
            })),
            other => panic!("list cardinality saw {other:?}"),
        },
        ScalarOp::StringConcat => {
            let (first, second) = expect_pair(&arguments);
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
            match expect_unary(&arguments) {
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
        ScalarOp::StringListConcat => match expect_unary(&arguments) {
            Value::List(values) => Ok(at_builtin_level(level, || {
                let mut joined = String::new();
                for value in values {
                    match value.as_ref() {
                        Value::String(text) => joined.push_str(text),
                        other => panic!("string list concatenation saw {other:?}"),
                    }
                }
                Value::String(joined)
            })),
            other => panic!("string list concatenation saw {other:?}"),
        },
        // string_to_ascii (global.w:3516-3521): first byte unsigned, -1 when
        // the string is empty.
        ScalarOp::StringToAscii => match expect_unary(&arguments) {
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
        ScalarOp::AsciiChar => match expect_unary(&arguments) {
            Value::Integer(value) => {
                let code = i32::try_from(value)
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
            let size = match expect_unary(&arguments) {
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
        ScalarOp::MatrixShape => match expect_unary(&arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                Value::Tuple(vec![
                    Rc::new(Value::Integer(BigInt::from(matrix.rows()))),
                    Rc::new(Value::Integer(BigInt::from(matrix.cols()))),
                ])
            })),
            other => panic!("matrix shape saw {other:?}"),
        },
        // matrix_row/column (global.w:3626-3648): the index narrows through
        // ulong_val and bounds-checks before the no-value gate.
        ScalarOp::MatrixRow | ScalarOp::MatrixColumn => {
            let (matrix, index) = match expect_pair(&arguments) {
                (Value::Matrix(matrix), Value::Integer(index)) => (matrix, index),
                other => panic!("matrix row/column saw {other:?}"),
            };
            let index = unsigned_long(index, span)?;
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
        ScalarOp::MatrixRows | ScalarOp::MatrixColumns => match expect_unary(&arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                let count = match operation {
                    ScalarOp::MatrixRows => matrix.rows(),
                    ScalarOp::MatrixColumns => matrix.cols(),
                    _ => unreachable!(),
                };
                Value::List(
                    (0..count)
                        .map(|index| {
                            Rc::new(Value::Vector(match operation {
                                ScalarOp::MatrixRows => matrix.row(index),
                                ScalarOp::MatrixColumns => matrix.column(index),
                                _ => unreachable!(),
                            }))
                        })
                        .collect(),
                )
            })),
            other => panic!("matrix rows/columns saw {other:?}"),
        },
        // succ/pred (global.w:2761-2773): upstream's parse-time rewrite
        // turns x+1/x-1 into these; as builtins they are plain increments.
        ScalarOp::IntSuccessor | ScalarOp::IntPredecessor => match expect_unary(&arguments) {
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
            let (first, second) = expect_ints(&arguments);
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
            let (first, second) = expect_ints(&arguments);
            Ok(at_builtin_level(level, || {
                Value::Boolean(first & second == *first)
            }))
        }
        // nth_set_bit (global.w:2859-2870): the index narrows through
        // long_val before the no-value gate; a negative index counts cleared
        // bits of the operand instead.
        ScalarOp::IntNthSetBit => {
            let (value, index) = expect_ints(&arguments);
            let index = long_int(index, span)?;
            Ok(at_builtin_level(level, || {
                Value::Integer(if index >= 0 {
                    index_of_set_bit(value, index as u64)
                } else {
                    index_of_set_bit(&!value, index.wrapping_neg().wrapping_sub(1) as u64)
                })
            }))
        }
        // bit_length (global.w:2872-2877): significant bits for n>=0; for
        // n<0 the negated two's-complement size, -(bits(~n)+1).
        ScalarOp::IntBitLength => match expect_unary(&arguments) {
            Value::Integer(value) => Ok(at_builtin_level(level, || {
                Value::Integer(if *value >= 0 {
                    BigInt::from(value.significant_bits())
                } else {
                    -BigInt::from((!value).significant_bits() + 1)
                })
            })),
            other => panic!("bit_length saw {other:?}"),
        },
        // to_bitset (global.w:2887-2899): the negative-entry scan runs
        // before the no-value gate.
        ScalarOp::VecToBitset => match expect_unary(&arguments) {
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
        ScalarOp::VecJoin => match expect_pair(&arguments) {
            (Value::Vector(left), Value::Vector(right)) => Ok(at_builtin_level(level, || {
                let mut joined = Vec::with_capacity(left.0.len() + right.0.len());
                joined.extend_from_slice(&left.0);
                joined.extend_from_slice(&right.0);
                Value::Vector(Vec32(joined))
            })),
            other => panic!("vector join saw {other:?}"),
        },
        ScalarOp::VecRowJoin => match expect_unary(&arguments) {
            Value::List(parts) => Ok(at_builtin_level(level, || {
                let mut joined = Vec::new();
                for part in parts {
                    match part.as_ref() {
                        Value::Vector(vector) => joined.extend_from_slice(&vector.0),
                        other => panic!("vector row join saw {other:?}"),
                    }
                }
                Value::Vector(Vec32(joined))
            })),
            other => panic!("vector row join saw {other:?}"),
        },
        // Generic row operators (axis.w:8895-8938): suffix/prefix extend a
        // row by one element; the joins concatenate two rows or fold a row
        // of rows. Elements keep whatever value shape they carry.
        ScalarOp::RowSuffixElement | ScalarOp::RowPrefixElement => {
            let (first, second) = expect_pair_owned(own_all(arguments));
            let (mut entries, element, at_back) = match operation {
                ScalarOp::RowSuffixElement => match (first, second) {
                    (Value::List(entries), element) => (entries, element, true),
                    (first, second) => panic!("row suffix saw {first:?} and {second:?}"),
                },
                ScalarOp::RowPrefixElement => match (first, second) {
                    (element, Value::List(entries)) => (entries, element, false),
                    (first, second) => panic!("row prefix saw {first:?} and {second:?}"),
                },
                _ => unreachable!(),
            };
            Ok(at_builtin_level(level, || {
                if at_back {
                    entries.push(Rc::new(element));
                } else {
                    entries.insert(0, Rc::new(element));
                }
                Value::List(entries)
            }))
        }
        ScalarOp::RowJoinRows => match expect_pair_owned(own_all(arguments)) {
            (Value::List(mut left), Value::List(right)) => Ok(at_builtin_level(level, || {
                left.extend(right);
                Value::List(left)
            })),
            (first, second) => panic!("row join saw {first:?} and {second:?}"),
        },
        ScalarOp::RowJoinRowOfRows => match expect_unary_owned(own_all(arguments)) {
            Value::List(rows) => Ok(at_builtin_level(level, || {
                let mut joined = Vec::new();
                for row in rows {
                    match unwrap_shared(row) {
                        Value::List(entries) => joined.extend(entries),
                        other => panic!("row-of-rows join saw {other:?}"),
                    }
                }
                Value::List(joined)
            })),
            other => panic!("row-of-rows join saw {other:?}"),
        },
        // vector suffix/prefix (global.w:3657-3673): the element narrows
        // through int_val before the gate.
        ScalarOp::VecSuffix | ScalarOp::VecPrefix => {
            let (first, second) = expect_pair_owned(own_all(arguments));
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
        ScalarOp::VecSubtract => match expect_pair(&arguments) {
            (Value::Vector(left), Value::Vector(right)) => {
                if left.0.len() != right.0.len() {
                    return Err(size_mismatch(left.0.len(), right.0.len(), span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Vector(Vec32(
                        left.0
                            .iter()
                            .zip(&right.0)
                            .map(|(&a, &b)| a.wrapping_sub(b))
                            .collect(),
                    ))
                }))
            }
            other => panic!("vector subtraction saw {other:?}"),
        },
        // vec*int (global.w:3909-3915): int_val narrowing before the gate.
        ScalarOp::VecMultiplyInt => match expect_pair(&arguments) {
            (Value::Vector(vector), Value::Integer(factor)) => {
                let factor = plain_int(factor, span)?;
                Ok(at_builtin_level(level, || {
                    Value::Vector(Vec32(
                        vector
                            .0
                            .iter()
                            .map(|&entry| entry.wrapping_mul(factor))
                            .collect(),
                    ))
                }))
            }
            other => panic!("vector scaling saw {other:?}"),
        },
        // vec\int and vec%int (global.w:3917-3937): narrowing and the zero
        // divisor diagnostic fire before the gate; the remainder is always
        // non-negative (see vec_divmod_entry).
        ScalarOp::VecQuotientInt | ScalarOp::VecModuloInt => match expect_pair(&arguments) {
            (Value::Vector(vector), Value::Integer(divisor)) => {
                let divisor = plain_int(divisor, span)?;
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
                            .iter()
                            .map(|&entry| {
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
        ScalarOp::RatvecUnfraction => match expect_unary(&arguments) {
            Value::RatVector(ratvec) => Ok(at_builtin_level(level, || {
                Value::Tuple(vec![
                    Rc::new(Value::Vector(Vec32(
                        ratvec.numerators().iter().map(|&n| n as i32).collect(),
                    ))),
                    Rc::new(Value::Integer(BigInt::from(ratvec.denominator()))),
                ])
            })),
            other => panic!("ratvec unfraction saw {other:?}"),
        },
        // ratvec+ratvec/ratvec-ratvec (global.w:4127-4139): check_size
        // fires before the gate.
        ScalarOp::RatvecAdd | ScalarOp::RatvecSubtract => match expect_pair(&arguments) {
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
                        left,
                        right,
                        operation == ScalarOp::RatvecSubtract,
                    ))
                }))
            }
            other => panic!("ratvec addition/subtraction saw {other:?}"),
        },
        ScalarOp::RatvecNegate => match expect_unary(&arguments) {
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
            match expect_pair(&arguments) {
                (Value::RatVector(ratvec), Value::Integer(factor)) => {
                    let factor = long_int(factor, span)?;
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
        ScalarOp::RatvecMultiplyRat | ScalarOp::RatvecDivideRat => match expect_pair(&arguments) {
            (Value::RatVector(ratvec), Value::Rational(factor)) => {
                if operation == ScalarOp::RatvecDivideRat && *factor == 0 {
                    return Err(runtime("Rational vector division by 0", span));
                }
                if level == Level::NoValue {
                    return Ok(None);
                }
                // Malachite splits off the sign; the magnitude narrows
                // through machine long, as upstream's ratvec arithmetic.
                let negative = *factor < 0;
                let (magnitude, denominator) = factor.clone().into_numerator_and_denominator();
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
            let (first, second) = expect_pair(&arguments);
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
            let value = plain_int(value, span)?;
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
        ScalarOp::VecDot => match expect_pair(&arguments) {
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
        ScalarOp::FlexAdd | ScalarOp::FlexSub => match expect_pair(&arguments) {
            (Value::Vector(left), Value::Vector(right)) => Ok(at_builtin_level(level, || {
                Value::Vector(Vec32(flex_add_sub(
                    &left.0,
                    &right.0,
                    operation == ScalarOp::FlexSub,
                )))
            })),
            other => panic!("flex add/sub saw {other:?}"),
        },
        ScalarOp::VecConvolve => match expect_pair(&arguments) {
            (Value::Vector(left), Value::Vector(right)) => Ok(at_builtin_level(level, || {
                Value::Vector(Vec32(convolve(&left.0, &right.0)))
            })),
            other => panic!("convolve saw {other:?}"),
        },
        // mat±mat (global.w:4253-4275): the row check fires before the
        // column check, both before the gate.
        ScalarOp::MatAdd | ScalarOp::MatSubtract => match expect_pair(&arguments) {
            (Value::Matrix(left), Value::Matrix(right)) => {
                if left.rows() != right.rows() {
                    return Err(size_mismatch(left.rows(), right.rows(), span));
                }
                if left.cols() != right.cols() {
                    return Err(size_mismatch(left.cols(), right.cols(), span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Matrix(match operation {
                        ScalarOp::MatAdd => left.added(right),
                        ScalarOp::MatSubtract => left.subtracted(right),
                        _ => unreachable!(),
                    })
                }))
            }
            other => panic!("matrix addition/subtraction saw {other:?}"),
        },
        // The matrix/vector products (global.w:4284-4342): each dimension
        // diagnostic fires before the gate, with the product's own wording
        // ("Size mismatch <inner left>:<inner right>").
        ScalarOp::MatMulVec => match expect_pair(&arguments) {
            (Value::Matrix(matrix), Value::Vector(vector)) => {
                if matrix.cols() != vector.0.len() {
                    return Err(size_mismatch(matrix.cols(), vector.0.len(), span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Vector(matrix.multiplied_vec(vector))
                }))
            }
            other => panic!("matrix-vector product saw {other:?}"),
        },
        ScalarOp::MatMulRatVec => match expect_pair(&arguments) {
            (Value::Matrix(matrix), Value::RatVector(vector)) => {
                if matrix.cols() != vector.numerators().len() {
                    return Err(size_mismatch(
                        matrix.cols(),
                        vector.numerators().len(),
                        span,
                    ));
                }
                Ok(at_builtin_level(level, || {
                    Value::RatVector(matrix.multiplied_ratvec(vector))
                }))
            }
            other => panic!("matrix-ratvec product saw {other:?}"),
        },
        ScalarOp::MatMulMat => match expect_pair(&arguments) {
            (Value::Matrix(left), Value::Matrix(right)) => {
                if left.cols() != right.rows() {
                    return Err(size_mismatch(left.cols(), right.rows(), span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Matrix(left.multiplied(right))
                }))
            }
            other => panic!("matrix product saw {other:?}"),
        },
        ScalarOp::VecMulMat => match expect_pair(&arguments) {
            (Value::Vector(vector), Value::Matrix(matrix)) => {
                if vector.0.len() != matrix.rows() {
                    return Err(size_mismatch(vector.0.len(), matrix.rows(), span));
                }
                Ok(at_builtin_level(level, || {
                    Value::Vector(matrix.left_multiplied_vec(vector))
                }))
            }
            other => panic!("vector-matrix product saw {other:?}"),
        },
        ScalarOp::RatVecMulMat => match expect_pair(&arguments) {
            (Value::RatVector(vector), Value::Matrix(matrix)) => {
                if vector.numerators().len() != matrix.rows() {
                    return Err(size_mismatch(
                        vector.numerators().len(),
                        matrix.rows(),
                        span,
                    ));
                }
                Ok(at_builtin_level(level, || {
                    Value::RatVector(matrix.left_multiplied_ratvec(vector))
                }))
            }
            other => panic!("ratvec-matrix product saw {other:?}"),
        },
        // null(int->vec) (global.w:4471-4475): ulong_val narrowing, then
        // the gate; the allocation guard mirrors NullMatrix.
        ScalarOp::NullVector => match expect_unary(&arguments) {
            Value::Integer(value) => {
                let size = unsigned_long(value, span)?;
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
        ScalarOp::VecTranspose => match expect_unary_owned(own_all(arguments)) {
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
        ScalarOp::MatTranspose => match expect_unary(&arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                Value::Matrix(matrix.transposed())
            })),
            other => panic!("matrix transpose saw {other:?}"),
        },
        // id_mat (global.w:4518-4528): ulong_val narrowing and the size
        // limit fire before the gate.
        ScalarOp::IdMat => match expect_unary(&arguments) {
            Value::Integer(value) => {
                let size = unsigned_long(value, span)?;
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
        ScalarOp::Diagonal => match expect_unary(&arguments) {
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
                    Value::Matrix(Matrix::diagonal(vector))
                }))
            }
            other => panic!("diagonal matrix saw {other:?}"),
        },
        // stack_rows (global.w:4557-4584): a ragged row of vecs becomes a
        // zero-padded matrix; both limit checks fire before the gate.
        ScalarOp::StackRows => match expect_unary_owned(own_all(arguments)) {
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
                    match unwrap_shared(row) {
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
        ScalarOp::CombineColumns | ScalarOp::CombineRows => {
            match expect_pair_owned(own_all(arguments)) {
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
                        match unwrap_shared(part) {
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
            }
        }
        // gcd(vec->int) (global.w:4820-4828): the non-negative gcd of the
        // entries, computed in machine int arithmetic — gcd([-2^31]) prints
        // -2147483648 upstream, so the fold runs in u32 and wraps back.
        ScalarOp::VectorGcd => match expect_unary(&arguments) {
            Value::Vector(vector) => Ok(at_builtin_level(level, || {
                let mut divisor = 0u64;
                for &entry in &vector.0 {
                    divisor = gcd_u64(divisor, u64::from(entry.unsigned_abs()));
                }
                Value::Integer(BigInt::from(divisor as u32 as i32))
            })),
            other => panic!("vector gcd saw {other:?}"),
        },
        // Bezout(vec->int,mat) (global.w:4830-4841): the gcd plus the
        // unimodular recorder with `v*C == [d,0,...]`; `det(C)` may be -1
        // (the flip is computed upstream but not reported).
        ScalarOp::VecBezout => match expect_unary(&arguments) {
            Value::Vector(vector) => Ok(at_builtin_level(level, || {
                let mut flip = false;
                let (d, recorder) = matreduc::gcd_recorder(vector.0.clone(), &mut flip, 0);
                Value::Tuple(vec![
                    Rc::new(Value::Integer(BigInt::from(d))),
                    Rc::new(Value::Matrix(recorder.to_matrix())),
                ])
            })),
            other => panic!("Bezout saw {other:?}"),
        },
        // echelon(mat->mat,mat,[int],int) (global.w:4848-4865): E has its
        // zero columns REMOVED, the kernel columns are rotated right in C,
        // pivots are ascending, flip = sign det(C).
        ScalarOp::MatEchelon => match expect_unary(&arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                let mut reduced = matreduc::PidMatrix::from_matrix(matrix);
                let (recorder, pivots, flip) = matreduc::column_echelon(&mut reduced);
                Value::Tuple(vec![
                    Rc::new(Value::Matrix(reduced.to_matrix())),
                    Rc::new(Value::Matrix(recorder.to_matrix())),
                    Rc::new(Value::List(
                        pivots
                            .into_iter()
                            .map(|pivot| Rc::new(Value::Integer(BigInt::from(pivot))))
                            .collect(),
                    )),
                    Rc::new(Value::Integer(BigInt::from(if flip { -1 } else { 1 }))),
                ])
            })),
            other => panic!("echelon saw {other:?}"),
        },
        // kernel(mat->mat) (global.w:4975-4979, lattice.cpp:133-140): the
        // m×(m−rank) recorder block spanning ker(M) over the integers.
        ScalarOp::MatKernel => match expect_unary(&arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                Value::Matrix(
                    matreduc::kernel(&matreduc::PidMatrix::from_matrix(matrix)).to_matrix(),
                )
            })),
            other => panic!("kernel saw {other:?}"),
        },
        // eigen_lattice(mat,int->mat) (global.w:4981-4987,
        // lattice.cpp:142-145): kernel(M−λI); NO square check, the diagonal
        // touch runs up to min(rows,cols); the int narrowing fires BEFORE
        // the no-value gate (upstream pops int_val() first).
        ScalarOp::MatEigenLattice => match expect_pair(&arguments) {
            (Value::Matrix(matrix), Value::Integer(lambda)) => {
                let lambda = plain_int(lambda, span)?;
                Ok(at_builtin_level(level, || {
                    Value::Matrix(
                        matreduc::eigen_lattice(&matreduc::PidMatrix::from_matrix(matrix), lambda)
                            .to_matrix(),
                    )
                }))
            }
            other => panic!("eigen_lattice saw {other:?}"),
        },
        // row_saturate(mat->mat) (global.w:4989-4993, lattice.cpp:147-160,
        // installed with hunger 3): adapted_basis of the transpose, rows.
        ScalarOp::MatRowSaturate => match expect_unary(&arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                Value::Matrix(
                    matreduc::row_saturate(&matreduc::PidMatrix::from_matrix(matrix)).to_matrix(),
                )
            })),
            other => panic!("row_saturate saw {other:?}"),
        },
        // Smith(mat->mat,vec) (global.w:5000-5010, matreduc.cpp:359-385):
        // (B, inv_factors) with positive divisibility-ordered factors.
        ScalarOp::MatSmith => match expect_unary(&arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                let (basis, factors) =
                    matreduc::smith_basis(&matreduc::PidMatrix::from_matrix(matrix));
                Value::Tuple(vec![
                    Rc::new(Value::Matrix(basis.to_matrix())),
                    Rc::new(Value::Vector(Vec32(factors))),
                ])
            })),
            other => panic!("Smith saw {other:?}"),
        },
        // adapted_basis(mat->mat,vec) (global.w:4949-4959,
        // matreduc.cpp:261-336): image(M) = span{d_i·B.col(i)}; the
        // diagonal is NOT divisibility-ordered.
        ScalarOp::MatAdaptedBasis => match expect_unary(&arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                let (basis, diagonal) =
                    matreduc::adapted_basis(&matreduc::PidMatrix::from_matrix(matrix));
                Value::Tuple(vec![
                    Rc::new(Value::Matrix(basis.to_matrix())),
                    Rc::new(Value::Vector(Vec32(diagonal))),
                ])
            })),
            other => panic!("adapted_basis saw {other:?}"),
        },
        // diagonalize(mat->vec,mat,mat) (global.w:4934-4947,
        // matreduc.cpp:145-226): (diagonal, row, column) — diagonal FIRST,
        // entries positive except possibly the first, det(row)=det(col)=1.
        ScalarOp::MatDiagonalize => match expect_unary(&arguments) {
            Value::Matrix(matrix) => Ok(at_builtin_level(level, || {
                let (row, column, diagonal) =
                    matreduc::diagonalise(&matreduc::PidMatrix::from_matrix(matrix));
                Value::Tuple(vec![
                    Rc::new(Value::Vector(Vec32(diagonal))),
                    Rc::new(Value::Matrix(row.to_matrix())),
                    Rc::new(Value::Matrix(column.to_matrix())),
                ])
            })),
            other => panic!("diagonalize saw {other:?}"),
        },
        // invert(mat->mat,int) (global.w:5017-5032, matrix.cpp:471-498):
        // (N, d) with N/d = M⁻¹. The non-square diagnostic fires BEFORE the
        // no-value gate; a singular square matrix returns the zero matrix
        // with d=0 and NO error.
        ScalarOp::MatInvert => match expect_unary(&arguments) {
            Value::Matrix(matrix) => {
                if matrix.rows() != matrix.cols() {
                    return Err(runtime(
                        format!("Cannot invert a {}x{} matrix", matrix.rows(), matrix.cols()),
                        span,
                    ));
                }
                if level == Level::NoValue {
                    return Ok(None);
                }
                let (numerator, denominator) =
                    matreduc::inverse(&matreduc::PidMatrix::from_matrix(matrix))
                        .map_err(|message| runtime(message, span))?;
                Ok(Some(Value::Tuple(vec![
                    Rc::new(Value::Matrix(numerator.to_matrix())),
                    Rc::new(Value::Integer(denominator)),
                ])))
            }
            other => panic!("invert saw {other:?}"),
        },
        // linear_solve(mat,vec->|vec,int,mat) (global.w:4891-4923): the
        // first union-returning builtin; the size-mismatch diagnostic fires
        // BEFORE the no-value gate, and `echelon_solve` failure is caught
        // into the `empty_set` variant rather than thrown.
        ScalarOp::LinearSolve => match expect_pair(&arguments) {
            (Value::Matrix(matrix), Value::Vector(rhs)) => {
                if matrix.rows() != rhs.0.len() {
                    return Err(runtime(
                        format!(
                            "Linear system size mismatch {}:{}",
                            matrix.rows(),
                            rhs.0.len()
                        ),
                        span,
                    ));
                }
                if level == Level::NoValue {
                    return Ok(None);
                }
                let solution = matreduc::linear_solve(
                    &matreduc::PidMatrix::from_matrix(matrix),
                    rhs.0.clone(),
                );
                Ok(Some(match solution {
                    matreduc::LinearSolution::Empty => Value::Union {
                        tag: 0,
                        injector_name: "empty_set".into(),
                        value: Box::new(Value::Tuple(Vec::new())),
                    },
                    matreduc::LinearSolution::Affine {
                        solution,
                        factor,
                        kernel,
                    } => Value::Union {
                        tag: 1,
                        injector_name: "affine_subspace".into(),
                        value: Box::new(Value::Tuple(vec![
                            Rc::new(Value::Vector(Vec32(solution))),
                            Rc::new(Value::Integer(factor)),
                            Rc::new(Value::Matrix(kernel.to_matrix())),
                        ])),
                    },
                }))
            }
            other => panic!("linear_solve saw {other:?}"),
        },
        // swiss_matrix_knife(int,mat,int,int,int,int->mat)
        // (global.w:4675-4809): the flag-bitfield slicer. Upstream pops
        // l, j, k, i as ulong_val() (in THAT order), then M, then flags via
        // int_val() truncated to the low 8 bits (BitSet<8>: NO range or
        // negativity check — -1 sets all bits, 256 == 0). The bounds
        // diagnostic fires AFTER all six arguments were evaluated but BEFORE
        // the no-value gate, using the RAW bounds (the from-end bits do not
        // relax the check). Negation (bit 7) is wrapping i32, as C++ int.
        ScalarOp::SwissMatrixKnife => {
            let mut arguments = own_all(arguments);
            let (Some(l), Some(j), Some(k), Some(i), Some(src), Some(flags)) = (
                arguments.pop(),
                arguments.pop(),
                arguments.pop(),
                arguments.pop(),
                arguments.pop(),
                arguments.pop(),
            ) else {
                panic!("swiss_matrix_knife saw wrong arity")
            };
            let (
                Value::Integer(l),
                Value::Integer(j),
                Value::Integer(k),
                Value::Integer(i),
                Value::Matrix(src),
                Value::Integer(flags),
            ) = (l, j, k, i, src, flags)
            else {
                panic!("swiss_matrix_knife saw ill-typed arguments")
            };
            // Narrow in the upstream pop order: l, j, k, i, then flags.
            let l = unsigned_long(&l, span)?;
            let j = unsigned_long(&j, span)?;
            let k = unsigned_long(&k, span)?;
            let i = unsigned_long(&i, span)?;
            let flags = (plain_int(&flags, span)? & 0xFF) as u8;
            let sliced = matreduc::swiss_matrix_knife(
                flags,
                &matreduc::PidMatrix::from_matrix(&src),
                i,
                k,
                j,
                l,
            )
            .map_err(|message| runtime(message, span))?;
            Ok(at_builtin_level(level, || {
                Value::Matrix(sliced.to_matrix())
            }))
        }
        // mod2_section(mat->mat) (global.w:5043-5053, bitvector.cpp:346-405):
        // the GF(2) section (ABA=A, BAB=B) with TRANSPOSE-shaped output. NO
        // validation and NO no-value gate before the compute (upstream gates
        // only the push); row bits >= 64 are masked on input, reproducing
        // the pinned NDEBUG oracle's silent drop (upstream UB regime).
        ScalarOp::Mod2Section => match expect_unary(&arguments) {
            Value::Matrix(matrix) => {
                let section = matreduc::mod2_section(&matreduc::PidMatrix::from_matrix(matrix));
                Ok(at_builtin_level(level, || {
                    Value::Matrix(section.to_matrix())
                }))
            }
            other => panic!("mod2_section saw {other:?}"),
        },
        // subspace_normal(mat->mat,mat,mat,[int]) (global.w:5062-5174): the
        // GF(2) reduced column-echelon normal form with combination and
        // relation tracking; output columns are PIVOT-ASCENDING via
        // permutations::standardization, NOT loop order. The two size
        // diagnostics fire BEFORE the no-value gate, dim first.
        ScalarOp::SubspaceNormal => match expect_unary(&arguments) {
            Value::Matrix(matrix) => {
                if matrix.rows() > 64 {
                    return Err(runtime(
                        format!("Dimension too large: {}>64", matrix.rows()),
                        span,
                    ));
                }
                if matrix.cols() > 64 {
                    return Err(runtime(
                        format!("Too many generators: {}>64", matrix.cols()),
                        span,
                    ));
                }
                if level == Level::NoValue {
                    return Ok(None);
                }
                let (basis, combination, relations, pivots) =
                    matreduc::subspace_normal(&matreduc::PidMatrix::from_matrix(matrix));
                Ok(Some(Value::Tuple(vec![
                    Rc::new(Value::Matrix(basis.to_matrix())),
                    Rc::new(Value::Matrix(combination.to_matrix())),
                    Rc::new(Value::Matrix(relations.to_matrix())),
                    Rc::new(Value::List(
                        pivots
                            .into_iter()
                            .map(|pivot| Rc::new(Value::Integer(BigInt::from(pivot))))
                            .collect(),
                    )),
                ])))
            }
            other => panic!("subspace_normal saw {other:?}"),
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
            // readline_completions (global.w:4390-4391, wrapper
            // :3546-3561): the line-editing completion list exposed as a
            // builtin; the run arm reads the per-command candidate
            // snapshot from the evaluation context.
            Builtin {
                name: "readline_completions",
                arg_type: string_type(),
                result: Type::row(string_type()),
                hunger: 0,
                overload_visible: true,
                implementation: BuiltinImpl::Completions,
            },
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
            // The remaining generic row operators are special instances too
            // (axis.w:2549-2595, 8776-8786): suffix/prefix element extension
            // on ([*],*)/(*,[*]) and row concatenation on ([*],[*])/[[*]].
            // Their `[*]` components are registry-local wildcards; the
            // recognition logic computes the concrete result type.
            hidden_scalar_builtin(
                "#",
                Type::tuple(vec![Type::row(Type::Undetermined), Type::Undetermined]),
                Type::row(Type::Undetermined),
                1,
                ScalarOp::RowSuffixElement,
            ),
            hidden_scalar_builtin(
                "#",
                Type::tuple(vec![Type::Undetermined, Type::row(Type::Undetermined)]),
                Type::row(Type::Undetermined),
                2,
                ScalarOp::RowPrefixElement,
            ),
            hidden_scalar_builtin(
                "##",
                Type::tuple(vec![
                    Type::row(Type::Undetermined),
                    Type::row(Type::Undetermined),
                ]),
                Type::row(Type::Undetermined),
                0,
                ScalarOp::RowJoinRows,
            ),
            hidden_scalar_builtin(
                "##",
                Type::row(Type::row(Type::Undetermined)),
                Type::row(Type::Undetermined),
                0,
                ScalarOp::RowJoinRowOfRows,
            ),
            // prints@@T (axis.w:8773): the variadic generic printer is a
            // hidden special instance; the `*` argument is a registry-local
            // wildcard matched by `hidden_special_variant` for any a-priori
            // type, and the result is always void.
            Builtin {
                name: "prints",
                arg_type: Type::Undetermined,
                result: Type::void(),
                hunger: 0,
                overload_visible: false,
                implementation: BuiltinImpl::Prints,
            },
            // print@@T / to_string@@T / error@@T (axis.w:8767-8771): the
            // remaining variadic specials, likewise matched by
            // `hidden_special_variant` for any a-priori type; print's
            // identity result and error's unknown result are produced
            // there, not from these placeholder rows.
            Builtin {
                name: "print",
                arg_type: Type::Undetermined,
                result: Type::Undetermined,
                hunger: 0,
                overload_visible: false,
                implementation: BuiltinImpl::Print,
            },
            Builtin {
                name: "to_string",
                arg_type: Type::Undetermined,
                result: Type::Primitive(Prim::String),
                hunger: 0,
                overload_visible: false,
                implementation: BuiltinImpl::ToString,
            },
            Builtin {
                name: "error",
                arg_type: Type::Undetermined,
                result: Type::Undetermined,
                hunger: 0,
                overload_visible: false,
                implementation: BuiltinImpl::Error,
            },
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
            // global.w batch 3 (installs at global.w:5201-5210): the
            // matreduc/lattice linear algebra builtins. All but
            // `linear_solve` return plain tuples; `row_saturate` keeps its
            // upstream operator hunger 3.
            scalar_builtin(
                "Bezout",
                primitive_type(Prim::Vec),
                Type::tuple(vec![int_type(), primitive_type(Prim::Mat)]),
                0,
                ScalarOp::VecBezout,
            ),
            scalar_builtin(
                "echelon",
                primitive_type(Prim::Mat),
                Type::tuple(vec![
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::Mat),
                    Type::row(int_type()),
                    int_type(),
                ]),
                0,
                ScalarOp::MatEchelon,
            ),
            scalar_builtin(
                "linear_solve",
                Type::tuple(vec![primitive_type(Prim::Mat), primitive_type(Prim::Vec)]),
                Type::union_of(vec![
                    Type::void(),
                    Type::tuple(vec![
                        primitive_type(Prim::Vec),
                        int_type(),
                        primitive_type(Prim::Mat),
                    ]),
                ]),
                0,
                ScalarOp::LinearSolve,
            ),
            scalar_builtin(
                "diagonalize",
                primitive_type(Prim::Mat),
                Type::tuple(vec![
                    primitive_type(Prim::Vec),
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::Mat),
                ]),
                0,
                ScalarOp::MatDiagonalize,
            ),
            scalar_builtin(
                "adapted_basis",
                primitive_type(Prim::Mat),
                Type::tuple(vec![primitive_type(Prim::Mat), primitive_type(Prim::Vec)]),
                0,
                ScalarOp::MatAdaptedBasis,
            ),
            scalar_builtin(
                "kernel",
                primitive_type(Prim::Mat),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::MatKernel,
            ),
            scalar_builtin(
                "eigen_lattice",
                Type::tuple(vec![primitive_type(Prim::Mat), int_type()]),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::MatEigenLattice,
            ),
            scalar_builtin(
                "row_saturate",
                primitive_type(Prim::Mat),
                primitive_type(Prim::Mat),
                3,
                ScalarOp::MatRowSaturate,
            ),
            scalar_builtin(
                "Smith",
                primitive_type(Prim::Mat),
                Type::tuple(vec![primitive_type(Prim::Mat), primitive_type(Prim::Vec)]),
                0,
                ScalarOp::MatSmith,
            ),
            scalar_builtin(
                "invert",
                primitive_type(Prim::Mat),
                Type::tuple(vec![primitive_type(Prim::Mat), int_type()]),
                0,
                ScalarOp::MatInvert,
            ),
            // global.w batch 4: the flag-bitfield matrix slicer
            // (global.w:5195-5196) and the GF(2) builtins (:5211-5213).
            // The hidden "matrix slicer" (:5197-5198) and "transpose "
            // (:5188) copies are deliberately NOT registered: the 2-D slice
            // and commabarlist row-display syntaxes desugar directly in the
            // grammar (grammar.lalrpop:422-441, :504), so no builtin copy is
            // ever called.
            scalar_builtin(
                "swiss_matrix_knife",
                Type::tuple(vec![
                    int_type(),
                    primitive_type(Prim::Mat),
                    int_type(),
                    int_type(),
                    int_type(),
                    int_type(),
                ]),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::SwissMatrixKnife,
            ),
            scalar_builtin(
                "mod2_section",
                primitive_type(Prim::Mat),
                primitive_type(Prim::Mat),
                0,
                ScalarOp::Mod2Section,
            ),
            scalar_builtin(
                "subspace_normal",
                primitive_type(Prim::Mat),
                Type::tuple(vec![
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::Mat),
                    primitive_type(Prim::Mat),
                    Type::row(int_type()),
                ]),
                0,
                ScalarOp::SubspaceNormal,
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
            // of a parameter, producing an SR_poly (ParamPol). Its
            // no_value gate comes FIRST (atlas-types.w:8085-8087, right
            // after the type-level get_own), so it skips — precedent
            // block_deform (atlas-types.w:8182).
            domain_builtin_skip(
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

/// The names that have at least one hidden (non-overload-visible) builtin:
/// a static set so the per-call-site check is one hash lookup instead of a
/// full registry scan.
fn hidden_special_names() -> &'static std::collections::HashSet<&'static str> {
    static NAMES: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        builtin_registry()
            .iter()
            .filter(|builtin| !builtin.overload_visible)
            .map(|builtin| builtin.name)
            .collect()
    })
}

fn hidden_special_builtin(name: &str) -> Option<usize> {
    if !hidden_special_names().contains(name) {
        return None;
    }
    builtin_registry()
        .iter()
        .position(|builtin| builtin.name == name && !builtin.overload_visible)
}

/// Locate a hidden builtin for `name` by its argument pattern.
fn hidden_builtin_by_pattern(name: &str, pattern: impl Fn(&Type) -> bool) -> Option<usize> {
    builtin_registry().iter().position(|builtin| {
        builtin.name == name && !builtin.overload_visible && pattern(&builtin.arg_type)
    })
}

/// The generic special operators `#`/`##` (axis.w:2473-2595): recognised
/// from the a-priori type when no exact ordinary overload matched, taking
/// precedence over every coercible one. Returns the registry index and the
/// concrete result type; the a-priori type itself serves as the argument
/// pattern, since upstream reuses the already-converted arguments unchanged.
fn hidden_special_variant(
    name: &str,
    a_priori_type: &Type,
    types: &TypeTable,
) -> Option<(usize, Type)> {
    // Every upstream shape test goes through kind()/component_type(), which
    // untable transparently (axis-types.w:375-384): a tabled row like
    // orbit_data still matches the generic `#`/`##` instances.
    let expanded_top;
    let a_priori_type: &Type = match a_priori_type {
        Type::Tabled(number) => {
            expanded_top = types.expansion(*number).clone();
            &expanded_top
        }
        other => other,
    };
    // Untabled view of one level down, for the row-component shape tests.
    fn untabled<'t>(type_: &'t Type, types: &'t TypeTable) -> &'t Type {
        match type_ {
            Type::Tabled(number) => types.expansion(*number),
            other => other,
        }
    }
    match name {
        "#" => match a_priori_type {
            // sizeof_row (axis.w:2544-2548): the length of any row value.
            Type::Row(_) => {
                let index = hidden_builtin_by_pattern("#", |arg| matches!(arg, Type::Row(_)))?;
                Some((index, int_type()))
            }
            Type::Tuple(components) if components.len() == 2 => {
                // suffix_element (axis.w:2552-2560), tried before prefix:
                // ([T],element) where T specialises the element type. A `*`
                // component adopts the element type, so `[]#3` works
                // (upstream mutates the a-priori component the same way).
                if let Type::Row(component) = untabled(&components[0], types) {
                    let mut component = component.as_ref().clone();
                    if component.specialise(&components[1], types) {
                        let index = hidden_builtin_by_pattern(
                            "#",
                            |arg| matches!(arg, Type::Tuple(parts) if matches!(parts.first(), Some(Type::Row(_)))),
                        )?;
                        return Some((index, Type::row(component)));
                    }
                }
                // prefix_element (axis.w:2561-2569): (element,[T]).
                if let Type::Row(component) = untabled(&components[1], types) {
                    let mut component = component.as_ref().clone();
                    if component.specialise(&components[0], types) {
                        let index = hidden_builtin_by_pattern(
                            "#",
                            |arg| matches!(arg, Type::Tuple(parts) if matches!(parts.get(1), Some(Type::Row(_)))),
                        )?;
                        return Some((index, Type::row(component)));
                    }
                }
                None
            }
            _ => None,
        },
        "##" => match a_priori_type {
            // join_rows_row (axis.w:2577-2582): fold a row of rows; the
            // component may itself be a tabled row (a row of orbit_data).
            Type::Row(component) if matches!(untabled(component, types), Type::Row(_)) => {
                let index = hidden_builtin_by_pattern(
                    "##",
                    |arg| matches!(arg, Type::Row(inner) if matches!(inner.as_ref(), Type::Row(_))),
                )?;
                Some((index, component.as_ref().clone()))
            }
            // join_rows (axis.w:2583-2595): two rows of the same type.
            Type::Tuple(components)
                if components.len() == 2
                    && matches!(untabled(&components[0], types), Type::Row(_))
                    && components[0].equals(&components[1], types) =>
            {
                let index = hidden_builtin_by_pattern(
                    "##",
                    |arg| matches!(arg, Type::Tuple(parts) if parts.iter().all(|part| matches!(part, Type::Row(_)))),
                )?;
                Some((index, components[0].clone()))
            }
            _ => None,
        },
        // prints@@T (axis.w:8773, wrapper :8850-8853): the variadic generic
        // printer matches any a-priori type; the result is always void.
        "prints" => {
            let index = hidden_special_builtin("prints")?;
            Some((index, Type::void()))
        }
        // print@@T (axis.w:8767, selection :6780-6783): identity function
        // type — the call's result type is the argument type itself.
        "print" => {
            let index = hidden_special_builtin("print")?;
            Some((index, a_priori_type.clone()))
        }
        // to_string@@T (axis.w:8769, selection :6788-6790): always string.
        "to_string" => {
            let index = hidden_special_builtin("to_string")?;
            Some((index, string_type()))
        }
        // error@@T (axis.w:8771, selection :6791-6794): the upstream result
        // is unknown_type, fitting every context (the call always throws
        // before the value is used); Undetermined specialises the same way.
        "error" => {
            let index = hidden_special_builtin("error")?;
            Some((index, Type::Undetermined))
        }
        _ => None,
    }
}

/// Operator casts use the exact structural predicates in axis.w:6750-6857.
/// They must not reuse the ordinary-call wildcard specialisation rules: for
/// example, `#@([*],int)` is rejected by the oracle rather than binding the
/// scalar wildcard to a row.
fn hidden_special_cast_variant(name: &str, cast_type: &Type) -> Option<(usize, Type)> {
    match name {
        "print" => Some((hidden_special_builtin("print")?, cast_type.clone())),
        "prints" => Some((hidden_special_builtin("prints")?, Type::void())),
        "to_string" => Some((hidden_special_builtin("to_string")?, string_type())),
        "error" => Some((hidden_special_builtin("error")?, Type::Undetermined)),
        "#" => match cast_type {
            Type::Row(_) => Some((
                hidden_builtin_by_pattern("#", |arg| matches!(arg, Type::Row(_)))?,
                int_type(),
            )),
            Type::Tuple(components) if components.len() == 2 => {
                if let Type::Row(component) = &components[0] {
                    if component.as_ref() == &components[1] {
                        return Some((
                            hidden_builtin_by_pattern("#", |arg| {
                                matches!(arg, Type::Tuple(parts) if matches!(parts.first(), Some(Type::Row(_))))
                            })?,
                            components[0].clone(),
                        ));
                    }
                }
                if let Type::Row(component) = &components[1] {
                    if component.as_ref() == &components[0] {
                        return Some((
                            hidden_builtin_by_pattern("#", |arg| {
                                matches!(arg, Type::Tuple(parts) if matches!(parts.get(1), Some(Type::Row(_))))
                            })?,
                            components[1].clone(),
                        ));
                    }
                }
                None
            }
            _ => None,
        },
        "##" => match cast_type {
            Type::Row(component) if matches!(component.as_ref(), Type::Row(_)) => Some((
                hidden_builtin_by_pattern("##", |arg| {
                    matches!(arg, Type::Row(inner) if matches!(inner.as_ref(), Type::Row(_)))
                })?,
                component.as_ref().clone(),
            )),
            Type::Tuple(components)
                if components.len() == 2
                    && matches!(&components[0], Type::Row(_))
                    && components[0] == components[1] => Some((
                hidden_builtin_by_pattern("##", |arg| {
                    matches!(arg, Type::Tuple(parts) if parts.iter().all(|part| matches!(part, Type::Row(_))))
                })?,
                components[0].clone(),
            )),
            _ => None,
        },
        _ => None,
    }
}

/// Wrap an iffor-bodied loop in the protected `## ` concatenation
/// (parser.y:317-324, 528-571; axis.w:1785 protected_concatenate_name):
/// the loop yields a row of rows that is joined by a DIRECT call to the
/// hidden row-of-rows `##` instance, bypassing overload resolution so a
/// user-defined `##` cannot shadow it.
fn join_iffor_body(
    loop_typed: TypedExpr,
    loop_type: &Type,
    required: &mut Type,
    span: SourceSpan,
    analysis: &Analysis<'_>,
) -> Result<TypedExpr, Diagnostic> {
    let (builtin, result_type) = hidden_special_variant("##", loop_type, analysis.types)
        .expect("an iffor-bodied loop yields a row of rows");
    let call = TypedExpr::BuiltinCall {
        builtin,
        arguments: vec![loop_typed],
        name: format!("## @{}", loop_type.display(analysis.types)),
        span,
    };
    conform_types(&result_type, required, call, span, analysis)
}

impl TypedExpr {
    /// Evaluate at the demanded level. `NoValue` returns `None`.
    ///
    /// Results are shared (`Rc<Value>`): identifier reads hand the slot's
    /// own reference to the consumer instead of deep-copying the aggregate,
    /// and consumers that need ownership go through [`unwrap_shared`], which
    /// moves when the value is uniquely held and copies only when it is
    /// genuinely shared (copy-on-write; Atlas assignment semantics copy, so
    /// a shared aggregate is never mutated behind an alias's back).
    pub fn evaluate(
        &self,
        context: &mut EvaluationContext,
        level: Level,
    ) -> Result<Option<SharedValue>, Control> {
        let _ = context;
        match self {
            Self::Denotation(value) => Ok(at_shared(level, value)),
            Self::CapturedLastValue { value, .. } => Ok(at_level(level, || value.clone())),
            Self::TupleDisplay(elements) => {
                let values = elements
                    .iter()
                    .map(|element| force(element, context))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(at_level(level, move || Value::Tuple(values)))
            }
            Self::ListDisplay(elements) => {
                let values = elements
                    .iter()
                    .map(|element| force(element, context))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(at_level(level, move || Value::List(values)))
            }
            Self::Conversion { tag, inner, span } => {
                let value = force(inner, context)?;
                let converted = apply_conversion(tag, unwrap_shared(value), *span)?;
                Ok(at_level(level, move || converted))
            }
            Self::Void(inner) => {
                inner.evaluate(context, Level::NoValue)?;
                Ok(at_level(level, || Value::Tuple(Vec::new())))
            }
            Self::GlobalIdent { name, cell, span } => {
                let value = cell.borrow().clone();
                match value {
                    Some(value) => Ok(at_shared(level, &value)),
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
                Some(value) => Ok(at_shared(level, &value)),
                None => Err(runtime(
                    format!("Taking value of uninitialized variable '{name}'"),
                    *span,
                )),
            },
            Self::GlobalAssignment { cell, value } => {
                let value = force(value, context)?;
                *cell.borrow_mut() = Some(Rc::clone(&value));
                Ok(at_shared(level, &value))
            }
            Self::LocalAssignment {
                depth,
                offset,
                value,
            } => {
                let value = force(value, context)?;
                let updated = context.set_local(*depth, *offset, Rc::clone(&value));
                assert!(
                    updated,
                    "analysis emitted an invalid local assignment address"
                );
                Ok(at_shared(level, &value))
            }
            Self::MultiAssignment { plan, value } => {
                // No destination is touched until the complete RHS has been
                // evaluated successfully. Distribution itself cannot fail
                // after the static shape check.
                let value = force(value, context)?;
                execute_multi_assignment(plan, &value, context);
                Ok(at_shared(level, &value))
            }
            Self::ComponentAssignment {
                target,
                name,
                index,
                reversed,
                value,
                source,
                span,
            } => {
                // axis.w:7940-7957: the value evaluates before the index,
                // and the range check comes last.
                let aggregate =
                    read_assign_target(target, name, context, *span, "Assigning to", "component")?;
                let value = force(value, context)?;
                let index = force(index, context)?;
                mutate_aggregate(target, aggregate, context, |aggregate| {
                    match aggregate {
                        Value::List(values) => {
                            let index =
                                expect_integer(unwrap_shared(index), *span, "assignment index")?;
                            let position = checked_index_in(
                                &index,
                                values.len(),
                                *reversed,
                                "component assignment",
                                source,
                                *span,
                            )?;
                            values[position] = Rc::clone(&value);
                            Ok(())
                        }
                        Value::Vector(Vec32(entries)) => {
                            let index =
                                expect_integer(unwrap_shared(index), *span, "assignment index")?;
                            let position = checked_index_in(
                                &index,
                                entries.len(),
                                *reversed,
                                "component assignment",
                                source,
                                *span,
                            )?;
                            let Value::Integer(component) = value.as_ref() else {
                                panic!(
                                    "analysis let a non-integer vec component through: {}",
                                    value.as_ref()
                                )
                            };
                            entries[position] = narrow_i32(component, *span)?;
                            Ok(())
                        }
                        Value::Matrix(matrix) => match index.as_ref() {
                            Value::Tuple(pair) if pair.len() == 2 => {
                                let row =
                                    expect_integer(pair[0].clone(), *span, "assignment index")?;
                                let column =
                                    expect_integer(pair[1].clone(), *span, "assignment index")?;
                                let row = checked_index_word(
                                    "initial index",
                                    &row,
                                    matrix.rows(),
                                    *reversed,
                                    "matrix entry assignment",
                                    source,
                                    *span,
                                )?;
                                let column = checked_index_word(
                                    "final index",
                                    &column,
                                    matrix.cols(),
                                    *reversed,
                                    "matrix entry assignment",
                                    source,
                                    *span,
                                )?;
                                let Value::Integer(component) = value.as_ref() else {
                                    panic!(
                                        "analysis let a non-integer mat entry through: {}",
                                        value.as_ref()
                                    )
                                };
                                matrix.set_entry(row, column, narrow_i32(component, *span)?);
                                Ok(())
                            }
                            _ => {
                                let index = expect_integer(
                                    unwrap_shared(index),
                                    *span,
                                    "assignment index",
                                )?;
                                let position = checked_index_in(
                                    &index,
                                    matrix.cols(),
                                    *reversed,
                                    "matrix column assignment",
                                    source,
                                    *span,
                                )?;
                                let Value::Vector(column) = value.as_ref() else {
                                    panic!(
                                        "analysis let a non-vec mat column through: {}",
                                        value.as_ref()
                                    )
                                };
                                if column.0.len() != matrix.rows() {
                                    return Err(runtime(
                                        format!(
                                            "Cannot replace column of size {} by one of size {}",
                                            matrix.rows(),
                                            column.0.len()
                                        ),
                                        *span,
                                    ));
                                }
                                matrix.set_column(position, column.clone());
                                Ok(())
                            }
                        },
                        // Term-coefficient assignment `P[t]:=s`
                        // (atlas-types.w:5650-5659, 7766-7782 `assign_coef`):
                        // finality is tested on the term, then a zero
                        // coefficient clears it and a nonzero one sets it.
                        Value::Domain(crate::domain_builtins::DomainValue::KTypePol(
                            polynomial,
                        )) => {
                            let Value::Domain(crate::domain_builtins::DomainValue::KType(ktype)) =
                                index.as_ref()
                            else {
                                panic!(
                                    "analysis let a non-KType index into a KTypePol assignment: {}",
                                    index.as_ref()
                                )
                            };
                            let Value::Domain(crate::domain_builtins::DomainValue::Split(
                                coefficient,
                            )) = value.as_ref()
                            else {
                                panic!(
                                    "analysis let a non-Split coefficient through: {}",
                                    value.as_ref()
                                )
                            };
                            crate::domain_builtins::ktype_pol_assign_coef(
                                polynomial,
                                ktype,
                                *coefficient,
                                *span,
                            )
                            .map_err(Control::Runtime)
                        }
                        Value::Domain(crate::domain_builtins::DomainValue::ParamPol(
                            polynomial,
                        )) => {
                            let Value::Domain(crate::domain_builtins::DomainValue::Param(
                                parameter,
                            )) = index.as_ref()
                            else {
                                panic!(
                                    "analysis let a non-Param index into a ParamPol assignment: {}",
                                    index.as_ref()
                                )
                            };
                            let Value::Domain(crate::domain_builtins::DomainValue::Split(
                                coefficient,
                            )) = value.as_ref()
                            else {
                                panic!(
                                    "analysis let a non-Split coefficient through: {}",
                                    value.as_ref()
                                )
                            };
                            crate::domain_builtins::param_pol_assign_coef(
                                polynomial,
                                parameter,
                                *coefficient,
                                *span,
                            )
                            .map_err(Control::Runtime)
                        }
                        other => {
                            panic!(
                                "analysis let a non-aggregate component assignment through: {other}"
                            )
                        }
                    }
                })?;
                Ok(at_shared(level, &value))
            }
            Self::ComponentTransform {
                target,
                name,
                index,
                reversed,
                operation,
                rhs,
                conversion,
                selection,
                source,
                span,
            } => {
                // axis.w:7989-8035: the right operand evaluates before the
                // index so the aggregate stays intact during its evaluation.
                let aggregate =
                    read_assign_target(target, name, context, *span, "Transforming", "component")?;
                let operand = force(rhs, context)?;
                let index = force(index, context)?;
                // Phase 1 locates the component and reads its old value out
                // of the read aggregate; the transform itself evaluates
                // after, outside any slot borrow; phase 3 writes the result
                // through the deferred plan (in place when unaliased).
                let (old, write) = match aggregate.as_ref() {
                    Value::List(values) => {
                        let index = expect_integer(unwrap_shared(index), *span, "transform index")?;
                        let position = checked_index_in(
                            &index,
                            values.len(),
                            *reversed,
                            "component assignment",
                            source,
                            *span,
                        )?;
                        (values[position].clone(), ComponentWrite::List(position))
                    }
                    Value::Vector(Vec32(entries)) => {
                        // The vec range check fires on the synthetic READ, so
                        // the oracle quotes the selection ("in subscription").
                        let index = expect_integer(unwrap_shared(index), *span, "transform index")?;
                        let position =
                            checked_index(&index, entries.len(), *reversed, selection, *span)?;
                        (
                            Rc::new(Value::Integer(BigInt::from(entries[position]))),
                            ComponentWrite::VectorEntry(position),
                        )
                    }
                    Value::Matrix(matrix) => match index.as_ref() {
                        Value::Tuple(pair) if pair.len() == 2 => {
                            let row = expect_integer(pair[0].clone(), *span, "transform index")?;
                            let column =
                                expect_integer(pair[1].clone(), *span, "transform index")?;
                            let row = checked_index_word(
                                "initial index",
                                &row,
                                matrix.rows(),
                                *reversed,
                                "matrix subscription",
                                selection,
                                *span,
                            )?;
                            let column = checked_index_word(
                                "final index",
                                &column,
                                matrix.cols(),
                                *reversed,
                                "matrix subscription",
                                selection,
                                *span,
                            )?;
                            let old = Rc::new(Value::Integer(BigInt::from(
                                matrix
                                    .entry(row, column)
                                    .expect("range-checked matrix entry is in bounds"),
                            )));
                            (old, ComponentWrite::MatrixEntry(row, column))
                        }
                        _ => {
                            let index =
                                expect_integer(unwrap_shared(index), *span, "transform index")?;
                            let position = checked_index_in(
                                &index,
                                matrix.cols(),
                                *reversed,
                                "matrix column selection",
                                selection,
                                *span,
                            )?;
                            (
                                Rc::new(Value::Vector(matrix.column(position))),
                                ComponentWrite::MatrixColumn(position),
                            )
                        }
                    },
                    other => {
                        panic!("analysis let a non-aggregate component transform through: {other}")
                    }
                };
                let result = Rc::new(apply_transform(
                    operation,
                    old,
                    operand,
                    *conversion,
                    *span,
                    context,
                )?);
                mutate_aggregate(target, aggregate, context, |aggregate| {
                    match (aggregate, &write) {
                        (Value::List(values), ComponentWrite::List(position)) => {
                            values[*position] = Rc::clone(&result);
                            Ok(())
                        }
                        (Value::Vector(Vec32(entries)), ComponentWrite::VectorEntry(position)) => {
                            let Value::Integer(component) = result.as_ref() else {
                                panic!(
                                    "analysis let a non-integer vec component through: {}",
                                    result.as_ref()
                                )
                            };
                            entries[*position] = narrow_i32(component, *span)?;
                            Ok(())
                        }
                        (Value::Matrix(matrix), ComponentWrite::MatrixEntry(row, column)) => {
                            let Value::Integer(component) = result.as_ref() else {
                                panic!(
                                    "analysis let a non-integer mat entry through: {}",
                                    result.as_ref()
                                )
                            };
                            matrix.set_entry(*row, *column, narrow_i32(component, *span)?);
                            Ok(())
                        }
                        (Value::Matrix(matrix), ComponentWrite::MatrixColumn(position)) => {
                            let Value::Vector(column) = result.as_ref() else {
                                panic!(
                                    "analysis let a non-vec mat column through: {}",
                                    result.as_ref()
                                )
                            };
                            if column.0.len() != matrix.rows() {
                                return Err(runtime(
                                    format!(
                                        "Cannot replace column of size {} by one of size {}",
                                        matrix.rows(),
                                        column.0.len()
                                    ),
                                    *span,
                                ));
                            }
                            matrix.set_column(*position, column.clone());
                            Ok(())
                        }
                        _ => unreachable!("the write plan matches the read aggregate's shape"),
                    }
                })?;
                Ok(at_shared(level, &result))
            }
            Self::FieldAssignment {
                target,
                name,
                position,
                value,
                span,
            } => {
                let aggregate =
                    read_assign_target(target, name, context, *span, "Assigning to", "field")?;
                let value = force(value, context)?;
                mutate_aggregate(target, aggregate, context, |aggregate| {
                    let Value::Tuple(components) = aggregate else {
                        panic!(
                            "analysis let a non-tuple field assignment through: {}",
                            &*aggregate
                        )
                    };
                    components[*position] = Rc::clone(&value);
                    Ok(())
                })?;
                Ok(at_shared(level, &value))
            }
            Self::FieldTransform {
                target,
                name,
                position,
                operation,
                rhs,
                conversion,
                span,
            } => {
                let aggregate =
                    read_assign_target(target, name, context, *span, "Transforming", "field")?;
                let operand = force(rhs, context)?;
                let old = {
                    let Value::Tuple(components) = aggregate.as_ref() else {
                        panic!(
                            "analysis let a non-tuple field assignment through: {}",
                            aggregate.as_ref()
                        )
                    };
                    components[*position].clone()
                };
                let result = Rc::new(apply_transform(
                    operation,
                    old,
                    operand,
                    *conversion,
                    *span,
                    context,
                )?);
                mutate_aggregate(target, aggregate, context, |aggregate| {
                    let Value::Tuple(components) = aggregate else {
                        panic!(
                            "analysis let a non-tuple field assignment through: {}",
                            &*aggregate
                        )
                    };
                    components[*position] = Rc::clone(&result);
                    Ok(())
                })?;
                Ok(at_shared(level, &result))
            }
            Self::Subscription {
                array,
                index,
                reversed,
                source,
                span,
            } => {
                let index = force(index, context)?;
                let array = force(array, context)?;
                match array.as_ref() {
                    Value::List(values) => {
                        let index =
                            expect_integer(unwrap_shared(index), *span, "subscription index")?;
                        let position =
                            checked_index(&index, values.len(), *reversed, source, *span)?;
                        Ok(at_shared(level, &values[position]))
                    }
                    // Upstream `string_subscription` (axis.w:4229-4239): the
                    // result is the one-character string at the position.
                    Value::String(text) => {
                        let index =
                            expect_integer(unwrap_shared(index), *span, "subscription index")?;
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
                    Value::Vector(Vec32(entries)) => {
                        let index =
                            expect_integer(unwrap_shared(index), *span, "subscription index")?;
                        let position =
                            checked_index(&index, entries.len(), *reversed, source, *span)?;
                        Ok(at_level(level, || {
                            Value::Integer(BigInt::from(entries[position]))
                        }))
                    }
                    Value::RatVector(ratvec) => {
                        let index =
                            expect_integer(unwrap_shared(index), *span, "subscription index")?;
                        let position = checked_index(
                            &index,
                            ratvec.numerators().len(),
                            *reversed,
                            source,
                            *span,
                        )?;
                        Ok(at_level(level, || {
                            Value::Rational(BigRational::from_integers(
                                BigInt::from(ratvec.numerators()[position]),
                                BigInt::from(ratvec.denominator()),
                            ))
                        }))
                    }
                    Value::Matrix(matrix) => match index.as_ref() {
                        // Two-index entry selection (parser.y:585-598): the
                        // first index is the ROW, the second the COLUMN
                        // (oracle: (mat:[[1,2],[3,4]])[0,1] = 3). The
                        // reversed form counts BOTH indices from the end.
                        Value::Tuple(pair) if pair.len() == 2 => {
                            let row =
                                expect_integer(pair[0].clone(), *span, "subscription index")?;
                            let column =
                                expect_integer(pair[1].clone(), *span, "subscription index")?;
                            let row = checked_index_word(
                                "initial index",
                                &row,
                                matrix.rows(),
                                *reversed,
                                "matrix subscription",
                                source,
                                *span,
                            )?;
                            let column = checked_index_word(
                                "final index",
                                &column,
                                matrix.cols(),
                                *reversed,
                                "matrix subscription",
                                source,
                                *span,
                            )?;
                            let entry = matrix.entry(row, column).expect("checked indices");
                            Ok(at_level(level, || Value::Integer(BigInt::from(entry))))
                        }
                        _ => {
                            let index = expect_integer(
                                unwrap_shared(index),
                                *span,
                                "subscription index",
                            )?;
                            let position = checked_index_in(
                                &index,
                                matrix.cols(),
                                *reversed,
                                "matrix column selection",
                                source,
                                *span,
                            )?;
                            Ok(at_level(level, || Value::Vector(matrix.column(position))))
                        }
                    },
                    // Term-coefficient selection
                    // (atlas-types.w:5631-5643, 7744-7759): the mismatch
                    // and finality checks run at every level, only the
                    // push is gated.
                    Value::Domain(crate::domain_builtins::DomainValue::KTypePol(
                        polynomial,
                    )) => {
                        let Value::Domain(crate::domain_builtins::DomainValue::KType(ktype)) =
                            index.as_ref()
                        else {
                            panic!(
                                "analysis let a non-KType index into a KTypePol: {}",
                                index.as_ref()
                            )
                        };
                        let coefficient = crate::domain_builtins::ktype_pol_coefficient(
                            polynomial, ktype, *span,
                        )
                        .map_err(Control::Runtime)?;
                        Ok(at_level(level, || {
                            Value::Domain(crate::domain_builtins::DomainValue::Split(coefficient))
                        }))
                    }
                    Value::Domain(crate::domain_builtins::DomainValue::ParamPol(polynomial)) => {
                        let Value::Domain(crate::domain_builtins::DomainValue::Param(parameter)) =
                            index.as_ref()
                        else {
                            panic!(
                                "analysis let a non-Param index into a ParamPol: {}",
                                index.as_ref()
                            )
                        };
                        let coefficient = crate::domain_builtins::param_pol_coefficient(
                            polynomial, parameter, *span,
                        )
                        .map_err(Control::Runtime)?;
                        Ok(at_level(level, || {
                            Value::Domain(crate::domain_builtins::DomainValue::Split(coefficient))
                        }))
                    }
                    other => panic!("analysis let a non-subscriptable value through: {other}"),
                }
            }
            Self::Slice {
                array,
                lower,
                upper,
                column_lower,
                column_upper,
                flags,
                source,
                span,
            } => {
                if let (Some(column_lower), Some(column_upper)) = (column_lower, column_upper) {
                    // Two-dimensional slice: all bounds evaluate before any
                    // narrowing, which then runs in the upstream pop order
                    // l, j, k, i (global.w:4719-4723); the range check fires
                    // at every level (only the push is gated upstream).
                    let array = force(array, context)?;
                    let Value::Matrix(matrix) = array.as_ref() else {
                        panic!(
                            "analysis let a non-matrix slice base through: {}",
                            array.as_ref()
                        )
                    };
                    let row_lower = expect_integer(
                        unwrap_shared(force(lower, context)?),
                        *span,
                        "slice lower bound",
                    )?;
                    let row_upper = expect_integer(
                        unwrap_shared(force(upper, context)?),
                        *span,
                        "slice upper bound",
                    )?;
                    let column_lower = expect_integer(
                        unwrap_shared(force(column_lower, context)?),
                        *span,
                        "slice column lower bound",
                    )?;
                    let column_upper = expect_integer(
                        unwrap_shared(force(column_upper, context)?),
                        *span,
                        "slice column upper bound",
                    )?;
                    let l = unsigned_long(&column_upper, *span)?;
                    let j = unsigned_long(&column_lower, *span)?;
                    let k = unsigned_long(&row_upper, *span)?;
                    let i = unsigned_long(&row_lower, *span)?;
                    let packed = (u8::from(flags.lower_from_end) * 0x02)
                        | (u8::from(flags.upper_from_end) * 0x04)
                        | (u8::from(flags.column_lower_from_end) * 0x10)
                        | (u8::from(flags.column_upper_from_end) * 0x20);
                    let sliced = matreduc::swiss_matrix_knife(
                        packed,
                        &matreduc::PidMatrix::from_matrix(matrix),
                        i,
                        k,
                        j,
                        l,
                    )
                    .map_err(|message| runtime(message, *span))?;
                    return Ok(at_level(level, || Value::Matrix(sliced.to_matrix())));
                }
                let upper = expect_integer(
                    unwrap_shared(force(upper, context)?),
                    *span,
                    "slice upper bound",
                )?;
                let lower = expect_integer(
                    unwrap_shared(force(lower, context)?),
                    *span,
                    "slice lower bound",
                )?;
                let array = force(array, context)?;
                match array.as_ref() {
                    Value::List(values) => {
                        let sliced = evaluate_slice(values, lower, upper, *flags, source, *span)?;
                        Ok(at_level(level, move || Value::List(sliced)))
                    }
                    Value::String(value) => {
                        let sliced = evaluate_string_slice(
                            value, lower, upper, *flags, source, *span,
                        )?;
                        Ok(at_level(level, move || Value::String(sliced)))
                    }
                    Value::Vector(vector) => {
                        let sliced =
                            evaluate_vec_slice(vector, lower, upper, *flags, source, *span)?;
                        Ok(at_level(level, move || Value::Vector(sliced)))
                    }
                    Value::RatVector(value) => {
                        let sliced =
                            evaluate_ratvec_slice(value, lower, upper, *flags, source, *span)?;
                        Ok(at_level(level, move || Value::RatVector(sliced)))
                    }
                    Value::Matrix(matrix) => {
                        let sliced =
                            evaluate_matrix_slice(matrix, lower, upper, *flags, source, *span)?;
                        Ok(at_level(level, move || Value::Matrix(sliced)))
                    }
                    other => panic!("analysis let a non-slice value through: {other}"),
                }
            }
            Self::BarList { rows, span } => {
                // Evaluate every entry first, then narrow row by row (the
                // per-row [int]->vec coercion precedes the rectangularity
                // check upstream), then stack the segments as matrix ROWS
                // (upstream: they are the columns of the `mat` cast, which
                // the hidden transpose then flips). Both diagnostics fire at
                // every level; only the push is gated.
                let mut evaluated = Vec::with_capacity(rows.len());
                for row in rows {
                    let values = row
                        .iter()
                        .map(|entry| force(entry, context))
                        .collect::<Result<Vec<_>, _>>()?;
                    evaluated.push(list_to_vec32(values, *span)?);
                }
                let width = evaluated.first().map_or(0, |row| row.0.len());
                if evaluated.iter().any(|row| row.0.len() != width) {
                    return Err(runtime(
                        "Vector sizes differ in conversion to matrix",
                        *span,
                    ));
                }
                let height = evaluated.len();
                let mut data = Vec::with_capacity(width * height);
                for column in 0..width {
                    for row in &evaluated {
                        data.push(row.0[column]);
                    }
                }
                Ok(at_level(level, move || {
                    Value::Matrix(
                        crate::linear_values::Matrix::from_columns(height, width, data)
                            .expect("commabarlist rows are rectangular"),
                    )
                }))
            }
            Self::LetGroup {
                initializers,
                names,
                body,
            } => {
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
                    evaluate_let_frame(slots, names, body, context, level)
                }
            }
            Self::Conditional {
                condition,
                then_branch,
                else_branch,
            } => match force(condition, context)?.as_ref() {
                Value::Boolean(true) => then_branch.evaluate(context, level),
                Value::Boolean(false) => else_branch.evaluate(context, level),
                other => panic!("analysis let a non-boolean condition through: {other}"),
            },
            Self::BuiltinCall {
                builtin,
                arguments,
                name,
                span,
            } => {
                let mut values = arguments
                    .iter()
                    .map(|argument| force(argument, context))
                    .collect::<Result<Vec<_>, _>>()?;
                if values.len() == 1
                    && matches!(builtin_registry()[*builtin].arg_type, Type::Tuple(_))
                    && matches!(
                        values.first().map(|value| value.as_ref()),
                        Some(Value::Tuple(_))
                    )
                {
                    let Value::Tuple(components) =
                        unwrap_shared(values.pop().expect("one tuple argument"))
                    else {
                        unreachable!("tuple shape checked above")
                    };
                    values = components;
                }
                match builtin_registry()[*builtin].run(values, *span, level, context) {
                    // Arguments evaluate OUTSIDE the traced region
                    // (axis.w:2184-2189): only errors from the builtin
                    // itself earn the call line.
                    Err(Control::Runtime(mut diagnostic)) => {
                        diagnostic.trace(format!(
                            "In call of {name} {}, built-in.",
                            trace_location(context, span)
                        ));
                        Err(Control::Runtime(diagnostic))
                    }
                    other => other.map(|value| value.map(Rc::new)),
                }
            }
            Self::HungryBuiltinCall {
                builtin,
                arguments,
                name,
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
                        Rc::new(take_pilfered(pilfer, context)?)
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
                    && matches!(
                        values.first().map(|value| value.as_ref()),
                        Some(Value::Tuple(_))
                    )
                {
                    let Value::Tuple(components) =
                        unwrap_shared(values.pop().expect("one tuple argument"))
                    else {
                        unreachable!("tuple shape checked above")
                    };
                    values = components;
                }
                match builtin_registry()[*builtin].run(values, *span, level, context) {
                    Err(Control::Runtime(mut diagnostic)) => {
                        diagnostic.trace(format!(
                            "In call of {name} {}, built-in.",
                            trace_location(context, span)
                        ));
                        Err(Control::Runtime(diagnostic))
                    }
                    other => other.map(|value| value.map(Rc::new)),
                }
            }
            Self::Closure {
                parameters,
                shapes,
                recursive,
                body,
                span,
                param_names,
            } => Ok(at_level(level, || {
                Value::Closure(Rc::new(Closure {
                    parameters: *parameters,
                    shapes: shapes.clone(),
                    recursive: *recursive,
                    body: Rc::clone(body),
                    frame: context.capture(),
                    span: *span,
                    param_names: param_names.clone(),
                }))
            })),
            Self::Return { value } => {
                let value = force(value, context)?;
                Err(Control::Return(value))
            }
            Self::FunctionCall {
                function,
                argument,
                name,
                span,
            } => {
                let callee = force(function, context)?;
                // The callee and argument evaluate OUTSIDE the traced
                // region (axis.w:2184-2189): only errors from the call
                // itself earn the call line.
                let argument = force(argument, context)?;
                match callee.as_ref() {
                    Value::Closure(closure) => {
                        match apply_closure(closure, argument, context, level) {
                            Err(Control::Runtime(mut diagnostic)) => {
                                // A dynamically computed callee prints its function
                                // expression (call_expression::function_name,
                                // axis.w:1911-1913); `defined` is the closure's
                                // lambda location (report_origin, axis.w:3273-3274).
                                let callee = name
                                    .clone()
                                    .unwrap_or_else(|| typed_expression_print(function));
                                diagnostic.trace(format!(
                                    "In call of {callee} {}, defined {}.",
                                    trace_location(context, span),
                                    trace_location(context, &closure.span)
                                ));
                                Err(Control::Runtime(diagnostic))
                            }
                            other => other,
                        }
                    }
                    // An operator-cast builtin value applies like a
                    // BuiltinCall: a tuple argument against a tuple
                    // parameter unpacks before the run.
                    Value::BuiltinFunction { builtin, name } => {
                        let mut values = vec![argument];
                        if matches!(builtin_registry()[*builtin].arg_type, Type::Tuple(_))
                            && matches!(
                                values.first().map(|value| value.as_ref()),
                                Some(Value::Tuple(_))
                            )
                        {
                            let Value::Tuple(components) =
                                unwrap_shared(values.pop().expect("one argument"))
                            else {
                                unreachable!("tuple shape checked above")
                            };
                            values = components;
                        }
                        match builtin_registry()[*builtin].run(values, *span, level, context) {
                            Err(Control::Runtime(mut diagnostic)) => {
                                diagnostic.trace(format!(
                                    "In call of {name} {}, built-in.",
                                    trace_location(context, span)
                                ));
                                Err(Control::Runtime(diagnostic))
                            }
                            other => other.map(|value| value.map(Rc::new)),
                        }
                    }
                    other => panic!("analysis let a non-function callee through: {other}"),
                }
            }
            Self::Sequence { first, second } => {
                first.evaluate(context, Level::NoValue)?;
                second.evaluate(context, level)
            }
            Self::While {
                body,
                out_reversed,
                yields_count,
            } => {
                // axis.w:5620-5652: evaluate the do_expr body, then read
                // the while-condition flag IT set — `false` ends the loop
                // without collecting that iteration. The flag (not the
                // value) drives termination, so a void-context body that
                // produces nothing still terminates correctly.
                let mut collected = Vec::new();
                let mut iterations: usize = 0;
                loop {
                    let body_level = if *yields_count {
                        Level::NoValue
                    } else {
                        Level::SingleValue
                    };
                    match body.evaluate(context, body_level) {
                        Ok(value) => {
                            if !context.while_condition_result() {
                                break;
                            }
                            iterations += 1;
                            match value {
                                // Collect only when the caller demands the
                                // row value.
                                Some(value) => {
                                    if level == Level::SingleValue {
                                        collected.push(value);
                                    }
                                }
                                None if *yields_count || matches!(body_level, Level::NoValue) => {}
                                None => {
                                    unreachable!("single-value loop body yields a value")
                                }
                            }
                        }
                        // The breaking iteration contributes no value.
                        Err(Control::Break(0)) => break,
                        Err(Control::Break(levels)) => {
                            return Err(Control::Break(levels - 1));
                        }
                        Err(control) => return Err(control),
                    }
                }
                if *yields_count {
                    return Ok(at_level(level, || Value::Integer(BigInt::from(iterations))));
                }
                // The tilde before `od` reverse-collects the row.
                if *out_reversed {
                    collected.reverse();
                }
                Ok(at_level(level, move || Value::List(collected)))
            }
            Self::Do { condition, body } => {
                // axis.w:5564-5576: on a false guard, clear the flag and
                // produce NO value (even at a value level); on a true
                // guard, evaluate the body and only THEN set the flag, so
                // a nested loop's flag writes cannot clobber it.
                let guard = match condition {
                    Some(condition) => match force(condition, context)?.as_ref() {
                        Value::Boolean(guard) => *guard,
                        other => {
                            panic!("analysis let a non-boolean do condition through: {other}")
                        }
                    },
                    None => true,
                };
                if !guard {
                    context.set_while_condition_result(false);
                    return Ok(None);
                }
                let value = body.evaluate(context, level)?;
                context.set_while_condition_result(true);
                Ok(value)
            }
            Self::For {
                shape,
                index,
                names,
                iterable,
                in_reversed,
                body,
                out_reversed,
            } => eval_for_loop(
                shape,
                *index,
                names,
                iterable,
                *in_reversed,
                body,
                *out_reversed,
                level,
                context,
            ),
            Self::Break { levels } => Err(Control::Break(*levels)),
            // dont_expression (axis.w:5579-5580): clear the flag, push
            // nothing — the enclosing while ends without collecting.
            Self::Dont => {
                context.set_while_condition_result(false);
                Ok(None)
            }
            Self::Die { span } => Err(runtime("I die", *span)),
            Self::UnionInject {
                tag,
                injector_name,
                payload,
            } => {
                let value = force(payload, context)?;
                Ok(at_level(level, move || Value::Union {
                    tag: *tag,
                    injector_name: injector_name.clone(),
                    value: Box::new(unwrap_shared(value)),
                }))
            }
            Self::TupleProject { index, inner } => {
                let value = force(inner, context)?;
                let Value::Tuple(components) = value.as_ref() else {
                    panic!(
                        "analysis let a non-tuple projection through: {}",
                        value.as_ref()
                    )
                };
                Ok(at_shared(level, &components[*index]))
            }
            Self::Case {
                subject,
                branches,
                fallback,
                span,
            } => {
                let subject = force(subject, context)?;
                let Value::Union { tag, value, .. } = subject.as_ref() else {
                    panic!(
                        "analysis let a non-union discrimination subject through: {}",
                        subject.as_ref()
                    )
                };
                for (branch_tag, shape, body) in branches {
                    if branch_tag != tag {
                        continue;
                    }
                    let mut slots = Vec::new();
                    distribute(Rc::new(value.as_ref().clone()), shape, &mut slots);
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
                let selector = expect_integer(
                    unwrap_shared(force(condition, context)?),
                    *span,
                    "case selector",
                )?;
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
                let Value::Union { tag, value, .. } = subject.as_ref() else {
                    panic!(
                        "analysis let a non-union union-case subject through: {}",
                        subject.as_ref()
                    )
                };
                // The positional branch evaluates to a function, applied
                // to the payload (axis.w:5041-5049).
                let function = force(&branches[usize::from(*tag)], context)?;
                let Value::Closure(closure) = function.as_ref() else {
                    panic!(
                        "analysis let a non-function union-case branch through: {}",
                        function.as_ref()
                    )
                };
                apply_closure(closure, Rc::new(value.as_ref().clone()), context, level)
            }
            Self::CountedFor {
                name,
                decreasing,
                in_reversed,
                out_reversed,
                count,
                bound,
                body,
                span,
            } => {
                let count = expect_integer(
                    unwrap_shared(force(count, context)?),
                    *span,
                    "loop count",
                )?;
                // A negative count yields an empty row (axis.w:6521).
                let count = if count < 0 { BigInt::from(0) } else { count };
                let lower = match bound {
                    Some(bound) => expect_integer(
                        unwrap_shared(force(bound, context)?),
                        *span,
                        "loop bound",
                    )?,
                    None => BigInt::from(0),
                };
                // Increasing takes `count` steps from the bound;
                // decreasing (downto, or the count-side tilde) runs from
                // bound+count-1 down to the bound inclusive
                // (axis.w:6638-6670).
                let descending = *decreasing || *in_reversed;
                let mut index = if descending {
                    &lower + &count - BigInt::from(1)
                } else {
                    lower.clone()
                };
                let mut collected = Vec::new();
                let mut position = 0usize;
                loop {
                    let active = if descending {
                        index >= lower
                    } else {
                        index < &lower + &count
                    };
                    if !active {
                        break;
                    }
                    let result = if name.is_some() {
                        context
                            .with_frame(vec![Rc::new(Value::Integer(index.clone()))], |context| {
                                body.evaluate(context, Level::SingleValue)
                            })
                    } else {
                        body.evaluate(context, Level::SingleValue)
                    };
                    match result {
                        // Collect only when the caller demands the row value.
                        Ok(Some(value)) => {
                            if level == Level::SingleValue {
                                collected.push(value);
                            }
                        }
                        Ok(None) => unreachable!("single-value loop body yields a value"),
                        // The breaking iteration contributes no value.
                        Err(Control::Break(0)) => break,
                        Err(Control::Break(levels)) => return Err(Control::Break(levels - 1)),
                        Err(Control::Runtime(mut diagnostic)) => {
                            // Iteration line only, no frame dump
                            // (axis.w:6587-6594, 6685-6698). A named loop
                            // reports its counter by name and notes
                            // `reversed` when decreasing; the anonymous
                            // catch shares one format string, keeping its
                            // double space and no `reversed` mention.
                            let line = match name {
                                Some(name) if descending => format!(
                                    "During iteration {position} ({name}={index}) of the counted reversed for-loop"
                                ),
                                Some(name) => format!(
                                    "During iteration {position} ({name}={index}) of the counted for-loop"
                                ),
                                None => {
                                    format!("During iteration {position} of the  counted for-loop")
                                }
                            };
                            diagnostic.trace(line);
                            return Err(Control::Runtime(diagnostic));
                        }
                        Err(control) => return Err(control),
                    }
                    position += 1;
                    if descending {
                        index -= BigInt::from(1);
                    } else {
                        index += BigInt::from(1);
                    }
                }
                // The tilde before `od` reverse-collects the row.
                if *out_reversed {
                    collected.reverse();
                }
                Ok(at_level(level, move || Value::List(collected)))
            }
        }
    }
}

/// Gate a freshly computed value on the demanded level, sharing it.
fn at_level(level: Level, value: impl FnOnce() -> Value) -> Option<SharedValue> {
    match level {
        Level::NoValue => None,
        Level::SingleValue => Some(Rc::new(value())),
    }
}

/// Gate an already shared value on the demanded level (identifier reads and
/// assignments hand out the slot's own reference — no copy).
fn at_shared(level: Level, value: &SharedValue) -> Option<SharedValue> {
    match level {
        Level::NoValue => None,
        Level::SingleValue => Some(Rc::clone(value)),
    }
}

/// Take ownership of a shared value: move when uniquely held, copy when it
/// is genuinely shared (the copy-on-write half of the shared evaluator —
/// Atlas copy-on-assignment semantics mean a shared aggregate is never
/// mutated in place behind an alias).
fn unwrap_shared(value: SharedValue) -> Value {
    Rc::try_unwrap(value).unwrap_or_else(|shared| shared.as_ref().clone())
}

fn force(expression: &TypedExpr, context: &mut EvaluationContext) -> Result<SharedValue, Control> {
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
///
/// The argument arrives shared: a single plain parameter binds it with an
/// `Rc` bump, and only tuple destructuring unpacks it (copy-on-write).
fn apply_closure(
    closure: &Rc<Closure>,
    argument: SharedValue,
    context: &mut EvaluationContext,
    level: Level,
) -> Result<Option<SharedValue>, Control> {
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
        _ => match unwrap_shared(argument) {
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
        let (result, frame) = match slots {
            Some(slots) => {
                let (result, frame) = context
                    .with_frame_traced(slots, |context| closure.body.evaluate(context, level));
                (result, Some(frame))
            }
            None => (closure.body.evaluate(context, level), None),
        };
        match result {
            // An explicit `return` ends the call and supplies its value
            // (upstream function_return caught in apply, axis.w:3569-3571).
            Err(Control::Return(value)) => Ok(at_shared(level, &value)),
            // A runtime error unwinding through a call with named slots
            // earns the local-variable trace line (axis.w:3525-3533);
            // parameterless closures push no frame and no line.
            Err(Control::Runtime(mut diagnostic)) => {
                if let Some(frame) = frame {
                    if !closure.param_names.is_empty() {
                        diagnostic.trace(frame_dump(context, &closure.param_names, &frame));
                    }
                }
                Err(Control::Runtime(diagnostic))
            }
            other => other,
        }
    })
}

/// The upstream source-location rendering (parsetree.w:173-180):
/// `at NAME:LINE:COL-COL` with 1-based lines and 0-based columns (Rust
/// spans are 1-based), end exclusive on a single line, and
/// `at NAME:LINE:COL--ENDLINE:ENDCOL` across lines (a doubled dash).
fn trace_location(context: &EvaluationContext, span: &SourceSpan) -> String {
    let name = context.source_name(span.source_id());
    let start_column = span.start.column.saturating_sub(1);
    let end_column = span.end.column.saturating_sub(1);
    if span.start.line == span.end.line {
        format!("at {name}:{}:{start_column}-{end_column}", span.start.line)
    } else {
        // Upstream prints `'-'` unconditionally, then `'-':EL:':'` when the
        // span crosses lines — a doubled dash (parsetree.w:173-180).
        format!(
            "at {name}:{}:{start_column}--{}:{end_column}",
            span.start.line, span.end.line
        )
    }
}

/// Evaluate a `for pattern[@index] in iterable do body od` loop. Kept out
/// of `TypedExpr::evaluate`'s frame: the branch locals are heavy, and the
/// giant dispatcher recurses per subexpression.
#[allow(clippy::too_many_arguments)]
fn eval_for_loop(
    shape: &SlotShape,
    index: bool,
    names: &[String],
    iterable: &TypedExpr,
    in_reversed: bool,
    body: &TypedExpr,
    out_reversed: bool,
    level: Level,
    context: &mut EvaluationContext,
) -> Result<Option<SharedValue>, Control> {
    // The traversal pairs the `@` index value with the component — the
    // index value is built only when the pattern names one. Ordinary
    // aggregates index by position (int); the polynomial types
    // index by the term itself (axis.w:5926-5936: KTypePol by KType,
    // ParamPol by Param), the component being the Split coefficient.
    let position_index = |position: usize| index.then(|| Value::Integer(BigInt::from(position)));
    let mut collected = match Rc::try_unwrap(force(iterable, context)?) {
        // A uniquely held iterable (a fresh display or call result): move
        // the components out without copying anything.
        Ok(owned) => {
            let values: Vec<(Option<Value>, SharedValue)> = match owned {
                // List elements are already shared handles: they pass
                // straight through with no copy at all.
                Value::List(values) => values
                    .into_iter()
                    .enumerate()
                    .map(|(position, element)| (position_index(position), element))
                    .collect(),
                // A string iterates its one-character strings, a vec its
                // int entries, a ratvec its rat entries, a mat its
                // column vecs (the aggregate for-in of parser.y:506-531,
                // mirroring subscription's component types).
                Value::String(text) => text
                    .as_bytes()
                    .iter()
                    .enumerate()
                    .map(|(position, byte)| {
                        (
                            position_index(position),
                            Rc::new(Value::String(
                                String::from_utf8_lossy(&[*byte]).into_owned(),
                            )),
                        )
                    })
                    .collect(),
                Value::Vector(Vec32(entries)) => entries
                    .iter()
                    .enumerate()
                    .map(|(position, entry)| {
                        (
                            position_index(position),
                            Rc::new(Value::Integer(BigInt::from(*entry))),
                        )
                    })
                    .collect(),
                Value::RatVector(ratvec) => ratvec
                    .numerators()
                    .iter()
                    .enumerate()
                    .map(|(position, numerator)| {
                        (
                            position_index(position),
                            Rc::new(Value::Rational(BigRational::from_integers(
                                BigInt::from(*numerator),
                                BigInt::from(ratvec.denominator()),
                            ))),
                        )
                    })
                    .collect(),
                Value::Matrix(matrix) => (0..matrix.cols())
                    .map(|column| {
                        (
                            position_index(column),
                            Rc::new(Value::Vector(matrix.column(column))),
                        )
                    })
                    .collect(),
                Value::Domain(crate::domain_builtins::DomainValue::KTypePol(polynomial)) => {
                    polynomial
                        .iteration_terms()
                        .into_iter()
                        .map(|(term, coefficient)| {
                            (
                                index.then(|| Value::Domain(term)),
                                Rc::new(Value::Domain(coefficient)),
                            )
                        })
                        .collect()
                }
                Value::Domain(crate::domain_builtins::DomainValue::ParamPol(polynomial)) => {
                    polynomial
                        .iteration_terms()
                        .into_iter()
                        .map(|(term, coefficient)| {
                            (
                                index.then(|| Value::Domain(term)),
                                Rc::new(Value::Domain(coefficient)),
                            )
                        })
                        .collect()
                }
                other => panic!("analysis let a non-iterable value through: {other}"),
            };
            // The tilde after the in-part traverses the components in
            // reverse; the `@` index still names the original position,
            // so it counts down from n-1 (axis.w:6017-6026).
            let mut iterations = values;
            if in_reversed {
                iterations.reverse();
            }
            for_loop_iterations(shape, names, body, in_reversed, level, context, iterations)?
        }
        // A shared iterable (the common case: a variable read): borrow the
        // aggregate and build one component per step instead of deep-copying
        // the whole value up front. Upstream does the same — for_expression
        // holds its own shared pointer to the evaluated aggregate and
        // indexes it per iteration (axis.w:5990-6026). Snapshot semantics
        // are preserved because the pinned handle keeps the Rc count above
        // one for the whole loop: every write path that could reach the
        // iterable through another alias copy-on-writes first
        // (mutate_aggregate's Rc::make_mut, take_pilfered's try_unwrap),
        // so the traversal always observes the entry-time aggregate,
        // exactly what the former upfront deep clone guaranteed.
        Err(shared) => {
            let view = IterableView::new(shared.as_ref());
            let length = view.len();
            let iterations = (0..length).map(|position| {
                let position = if in_reversed {
                    length - 1 - position
                } else {
                    position
                };
                view.component(position, index)
            });
            for_loop_iterations(shape, names, body, in_reversed, level, context, iterations)?
        }
    };
    // The tilde before `od` reverse-collects the row.
    if out_reversed {
        collected.reverse();
    }
    Ok(at_level(level, move || Value::List(collected)))
}

/// The per-iteration half of a for loop, shared by the owned and borrowed
/// traversals: bind the index slot ahead of the pattern slots, run the
/// body in a traced frame, and collect the row values.
#[allow(clippy::too_many_arguments)]
fn for_loop_iterations(
    shape: &SlotShape,
    names: &[String],
    body: &TypedExpr,
    in_reversed: bool,
    level: Level,
    context: &mut EvaluationContext,
    iterations: impl IntoIterator<Item = (Option<Value>, SharedValue)>,
) -> Result<Vec<SharedValue>, Control> {
    let mut collected = Vec::new();
    // The trace reports the traversal-order iteration counter
    // (0-based), which differs from the `@` index position
    // under reversed traversal (axis.w:6124-6161).
    for (iteration, (index_value, element)) in iterations.into_iter().enumerate() {
        // The index slot precedes the pattern slots, matching
        // the analysis-time layout (upstream pair wrap).
        let mut slots = Vec::new();
        if let Some(index_value) = index_value {
            slots.push(Rc::new(index_value));
        }
        distribute(element, shape, &mut slots);
        let (result, frame) = if slots.is_empty() {
            // A pure-discard layer pushes no frame, matching the
            // analysis-time empty-layer rule.
            (body.evaluate(context, Level::SingleValue), None)
        } else {
            let (result, frame) = context
                .with_frame_traced(slots, |context| body.evaluate(context, Level::SingleValue));
            (result, Some(frame))
        };
        match result {
            // Collect only when the caller demands the row value.
            Ok(Some(value)) => {
                if level == Level::SingleValue {
                    collected.push(value);
                }
            }
            Ok(None) => unreachable!("single-value loop body yields a value"),
            Err(Control::Break(0)) => break,
            Err(Control::Break(levels)) => {
                return Err(Control::Break(levels - 1));
            }
            Err(Control::Runtime(mut diagnostic)) => {
                // The per-iteration frame dump, then the
                // iteration line ahead of it (axis.w:6124-6161).
                if let Some(frame) = frame {
                    diagnostic.trace(frame_dump(context, names, &frame));
                }
                diagnostic.trace(format!(
                    "During iteration {iteration} of the {}for-loop",
                    if in_reversed { "reversed " } else { "" },
                ));
                return Err(Control::Runtime(diagnostic));
            }
            Err(control) => return Err(control),
        }
    }
    Ok(collected)
}

/// Borrowed traversal of a shared for-loop iterable: the aggregate stays
/// put and one component value is built per step (see `eval_for_loop` for
/// the copy-on-write argument that keeps this a snapshot traversal).
enum IterableView<'a> {
    List(&'a [SharedValue]),
    String(&'a [u8]),
    Vector(&'a [i32]),
    RatVector(&'a RatVec),
    Matrix(&'a Matrix),
    /// Polynomials iterate (term, coefficient) pairs built fresh per
    /// traversal (`iteration_terms`), so the view owns them.
    Terms(Vec<(domain_builtins::DomainValue, domain_builtins::DomainValue)>),
}

impl<'a> IterableView<'a> {
    fn new(value: &'a Value) -> Self {
        match value {
            Value::List(values) => Self::List(values),
            Value::String(text) => Self::String(text.as_bytes()),
            Value::Vector(Vec32(entries)) => Self::Vector(entries),
            Value::RatVector(ratvec) => Self::RatVector(ratvec),
            Value::Matrix(matrix) => Self::Matrix(matrix),
            Value::Domain(domain_builtins::DomainValue::KTypePol(polynomial)) => {
                Self::Terms(polynomial.iteration_terms())
            }
            Value::Domain(domain_builtins::DomainValue::ParamPol(polynomial)) => {
                Self::Terms(polynomial.iteration_terms())
            }
            other => panic!("analysis let a non-iterable value through: {other}"),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::List(values) => values.len(),
            Self::String(bytes) => bytes.len(),
            Self::Vector(entries) => entries.len(),
            Self::RatVector(ratvec) => ratvec.numerators().len(),
            Self::Matrix(matrix) => matrix.cols(),
            Self::Terms(terms) => terms.len(),
        }
    }

    /// The `(index, component)` pair of one position: ordinary aggregates
    /// index by position (int), polynomials by the term (axis.w:5926-5936).
    /// A list component is the element's own shared handle (an `Rc` bump);
    /// the other aggregates build a fresh component value per step.
    fn component(&self, position: usize, index: bool) -> (Option<Value>, SharedValue) {
        let position_index = || index.then(|| Value::Integer(BigInt::from(position)));
        match self {
            Self::List(values) => (position_index(), Rc::clone(&values[position])),
            Self::String(bytes) => (
                position_index(),
                Rc::new(Value::String(
                    String::from_utf8_lossy(&[bytes[position]]).into_owned(),
                )),
            ),
            Self::Vector(entries) => (
                position_index(),
                Rc::new(Value::Integer(BigInt::from(entries[position]))),
            ),
            Self::RatVector(ratvec) => (
                position_index(),
                Rc::new(Value::Rational(BigRational::from_integers(
                    BigInt::from(ratvec.numerators()[position]),
                    BigInt::from(ratvec.denominator()),
                ))),
            ),
            Self::Matrix(matrix) => (
                position_index(),
                Rc::new(Value::Vector(matrix.column(position))),
            ),
            Self::Terms(terms) => {
                let (term, coefficient) = &terms[position];
                (
                    index.then(|| Value::Domain(term.clone())),
                    Rc::new(Value::Domain(coefficient.clone())),
                )
            }
        }
    }
}

/// The traced frame of one let group (let_expression::evaluate catch,
/// axis.w:2882-2909): an error unwinding through the bindings dumps the
/// frame ahead of the inner trace lines, exactly like a call frame.
/// Outlined from `evaluate` so the frame-dump machinery does not inflate
/// the evaluator's (recursive) stack frame.
#[inline(never)]
fn evaluate_let_frame(
    slots: Vec<Rc<Value>>,
    names: &[String],
    body: &TypedExpr,
    context: &mut EvaluationContext,
    level: Level,
) -> Result<Option<SharedValue>, Control> {
    let (result, frame) = context.with_frame_traced(slots, |context| body.evaluate(context, level));
    match result {
        Err(Control::Runtime(mut diagnostic)) => {
            diagnostic.trace(frame_dump(context, names, &frame));
            Err(Control::Runtime(diagnostic))
        }
        other => other,
    }
}

/// The local-variable trace line of one frame (axis.w:2896-2909):
/// `{ name=value, ... }` with the standard value printer, read after
/// unwinding so a slot reassigned before the error prints its current value.
fn frame_dump(context: &EvaluationContext, names: &[String], frame: &Frame) -> String {
    let slots = frame.slot_snapshot();
    debug_assert_eq!(
        names.len(),
        slots.len(),
        "analysis keeps slot names and values in step"
    );
    let mut out = String::from("{ ");
    for (index, (name, slot)) in names.iter().zip(slots.iter()).enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(name);
        out.push('=');
        match slot {
            Some(value) => out.push_str(&trace_value_string(context, value)),
            // A call frame binds every slot before the body runs; an empty
            // slot can only appear after a pilfering builtin moved it out.
            None => out.push('*'),
        }
    }
    out.push_str(" }");
    out
}

/// The frame-dump rendering of one slot value (axis.w:2905 prints `**it`,
/// the standard value printer): closures use the multi-line
/// `closure_value::print` (axis.w:3254-3271), everything else `Display`.
fn trace_value_string(context: &EvaluationContext, value: &Value) -> String {
    value_string(context, value)
}

/// The standard value printer recurses through containers, so a closure
/// nested in a tuple, list, union payload, or domain wrapper still prints
/// its full multi-line `closure_value::print` form (axis.w:3254-3271) —
/// not the bare "Function defined" head that `Display` falls back to.
/// (Corpus 3617953: example.at printed `(Function defined,…)` where the
/// oracle prints `(Function defined at hodge_tensor.at:36:3-37 …)`.)
fn value_string(context: &EvaluationContext, value: &Value) -> String {
    match value {
        Value::Closure(closure) => closure_trace_string(context, closure),
        Value::Tuple(values) => {
            let mut out = String::from("(");
            for (index, element) in values.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&value_string(context, element));
            }
            out.push(')');
            out
        }
        Value::List(values) => {
            let mut out = String::from("[");
            for (index, element) in values.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&value_string(context, element));
            }
            out.push(']');
            out
        }
        Value::Union {
            injector_name,
            value: payload,
            ..
        } => format!("{}.{}", value_string(context, payload), injector_name),
        // DomainValue leaves are domain handles (real forms, params, …);
        // they never contain closures, so plain Display suffices.
        Value::Domain(_) => value.to_string(),
        other => other.to_string(),
    }
}

/// `closure_value::print` (axis.w:3254-3271): a header naming the lambda's
/// source location, then `print_lambda` (axis.w:3045-3053) — the parameter
/// pattern in parentheses (`@@` when it binds no identifier) followed by
/// `": "` and the converted body. A recursive closure prints its self name
/// ahead of the parameter pattern (`name = `, axis.w:3265-3271). Tuple
/// parameter patterns print flat (`(a,b)`); upstream additionally shows a
/// whole-value name (`(a,b):w`), which this port does not retain.
fn closure_trace_string(context: &EvaluationContext, closure: &Closure) -> String {
    let mut out = String::new();
    let names: &[String] = if closure.recursive {
        out.push_str("Recursive function defined ");
        out.push_str(&trace_location(context, &closure.span));
        out.push('\n');
        // The recursive self slot precedes the argument slots
        // (axis.w:3548-3560 `maybe_push`).
        match closure.param_names.split_first() {
            Some((self_name, rest)) => {
                out.push_str(self_name);
                out.push_str(" = ");
                rest
            }
            None => &[],
        }
    } else {
        out.push_str("Function defined ");
        out.push_str(&trace_location(context, &closure.span));
        out.push('\n');
        &closure.param_names
    };
    if names.is_empty() {
        out.push_str("@@");
    } else {
        out.push('(');
        out.push_str(&names.join(","));
        out.push(')');
    }
    out.push_str(": ");
    out.push_str(&typed_expression_print(&closure.body));
    out
}

/// Bind one value against a slot shape, pushing leaves left-to-right
/// (upstream `bind_pattern` at evaluation time). A plain leaf moves the
/// shared handle straight into its slot; tuple destructuring unpacks the
/// container (copy-on-write: a genuinely shared tuple spine is cloned once
/// here) and the components — already shared handles — pass straight down.
fn distribute(value: SharedValue, shape: &SlotShape, slots: &mut Vec<Rc<Value>>) {
    match shape {
        SlotShape::Leaf => slots.push(value),
        SlotShape::Discard => {}
        SlotShape::Tuple { elements, whole } => {
            if *whole {
                slots.push(Rc::clone(&value));
            }
            let values = match unwrap_shared(value) {
                Value::Tuple(values) => values,
                other => panic!("analysis let a non-tuple value reach a tuple pattern: {other}"),
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
/// Destinations bind the RHS's own shared handles (tuple components included);
/// Atlas copy-on-assignment semantics are preserved by the copy-on-write
/// mutation paths (`mutate_aggregate`).
fn execute_multi_assignment(
    plan: &MultiAssignmentPlan,
    value: &SharedValue,
    context: &EvaluationContext,
) {
    match plan {
        MultiAssignmentPlan::Omitted => {}
        MultiAssignmentPlan::Destination(MultiAssignmentDestination::Global(cell)) => {
            *cell.borrow_mut() = Some(Rc::clone(value));
        }
        MultiAssignmentPlan::Destination(MultiAssignmentDestination::Local { depth, offset }) => {
            let updated = context.set_local(*depth, *offset, Rc::clone(value));
            assert!(
                updated,
                "analysis emitted an invalid local multi-assignment address"
            );
        }
        MultiAssignmentPlan::Tuple { elements, whole } => {
            let Value::Tuple(values) = value.as_ref() else {
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

fn expect_list(value: Value) -> Vec<SharedValue> {
    match value {
        Value::List(values) => values,
        other => panic!("conversion applied to non-list value {other}"),
    }
}

/// The shared integer-index gate. Takes the index by any borrow — an owned
/// `Value`, a `SharedValue`, or a tuple pair's element handle — and copies
/// the (small) machine-range payload out.
fn expect_integer(
    value: impl std::borrow::Borrow<Value>,
    span: SourceSpan,
    operation: &str,
) -> Result<BigInt, Control> {
    match value.borrow() {
        Value::Integer(value) => Ok(value.clone()),
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
    checked_index_in(index, length, reversed, "subscription", source, span)
}

/// The shared range check of subscription and component assignment
/// (`range_mess`, axis.w:4188-4194): the context word distinguishes the
/// read (`subscription`) from the write (`component assignment`).
fn checked_index_in(
    index: &BigInt,
    length: usize,
    reversed: bool,
    context_name: &str,
    source: &str,
    span: SourceSpan,
) -> Result<usize, Control> {
    checked_index_word("index", index, length, reversed, context_name, source, span)
}

/// `checked_index_in` with a qualified index word: matrix entry operations
/// report "initial index …" / "final index …" (axis.w matrix subscription
/// and entry assignment range messages).
fn checked_index_word(
    index_word: &str,
    index: &BigInt,
    length: usize,
    reversed: bool,
    context_name: &str,
    source: &str,
    span: SourceSpan,
) -> Result<usize, Control> {
    let original = index.clone();
    let out_of_range = || {
        runtime(
            format!(
                "{index_word} {original} out of range (0<= . <{length}) in {context_name} {source}"
            ),
            span,
        )
    };
    let index = usize::try_from(index).map_err(|_| out_of_range())?;
    if index >= length {
        return Err(out_of_range());
    }
    Ok(if reversed { length - 1 - index } else { index })
}

/// The current value of a component/field assignment target, or the
/// uninitialized diagnostic of the assignment family (axis.w:7746-7785).
/// Returns the slot's own shared reference; [`mutate_aggregate`] uses
/// pointer identity against it to mutate the slot in place when nothing
/// aliased or reassigned the value in between.
fn read_assign_target(
    target: &AssignTarget,
    name: &str,
    context: &EvaluationContext,
    span: SourceSpan,
    verb: &str,
    noun: &str,
) -> Result<SharedValue, Control> {
    let value = match target {
        AssignTarget::Local { depth, offset } => context.local(*depth, *offset),
        AssignTarget::Global(cell) => cell.borrow().clone(),
    };
    match value {
        Some(value) => Ok(value),
        None => Err(runtime(
            format!("{verb} {noun} of uninitialized variable {name}"),
            span,
        )),
    }
}

/// The deferred component write of a transform assignment: phase 1 locates
/// the component and reads its old value, the transform itself evaluates
/// outside any slot borrow, then phase 3 writes the result back through
/// this plan.
enum ComponentWrite {
    List(usize),
    VectorEntry(usize),
    MatrixEntry(usize, usize),
    MatrixColumn(usize),
}

/// Mutate the aggregate of a component/field assignment target. When the
/// slot still holds exactly the value `read` saw, the mutation happens in
/// place through `Rc::make_mut` (copying only when a genuine alias shares
/// the value — Atlas copy-on-assignment semantics); otherwise the RHS
/// evaluations reassigned or pilfered the variable, and a mutated copy of
/// the read value replaces the slot, the same clobber the old
/// copy-read/write-back implementation produced.
///
/// `mutate` runs under a short slot borrow, so it must not evaluate
/// anything nested (a same-frame access would double-borrow), and it must
/// run every fallible check BEFORE touching the aggregate: an in-place
/// mutation that failed halfway would stay visible in the slot.
///
/// Global slots are borrowed with `try_borrow_mut`: when a foreign borrow
/// is held (only possible from outside the evaluator, e.g. tests), the
/// mutation runs on a detached copy and the cell is borrowed mutably only
/// after it succeeded, so failed checks keep producing errors instead of
/// `RefCell` panics while a successful write keeps the legacy panic.
fn mutate_aggregate(
    target: &AssignTarget,
    read: SharedValue,
    context: &mut EvaluationContext,
    mutate: impl FnOnce(&mut Value) -> Result<(), Control>,
) -> Result<(), Control> {
    fn apply(
        slot: &mut Option<SharedValue>,
        read: SharedValue,
        mutate: impl FnOnce(&mut Value) -> Result<(), Control>,
    ) -> Result<(), Control> {
        match slot.as_mut() {
            Some(current) if Rc::ptr_eq(current, &read) => {
                // The slot kept the read value: drop our handle so a solely
                // slot-held aggregate mutates in place.
                drop(read);
                mutate(Rc::make_mut(current))
            }
            _ => {
                let mut work = unwrap_shared(read);
                mutate(&mut work)?;
                *slot = Some(Rc::new(work));
                Ok(())
            }
        }
    }
    match target {
        AssignTarget::Global(cell) => match cell.try_borrow_mut() {
            Ok(mut slot) => apply(&mut slot, read, mutate),
            // A foreign borrow of the cell is held (only possible from a
            // caller outside the evaluator, e.g. tests): never borrow
            // mutably until the mutation has fully succeeded, so failed
            // checks return errors instead of panicking.
            Err(_) => {
                let mut work = unwrap_shared(read);
                mutate(&mut work)?;
                *cell.borrow_mut() = Some(Rc::new(work));
                Ok(())
            }
        },
        AssignTarget::Local { depth, offset } => context
            .update_local_slot(*depth, *offset, |slot| apply(slot, read, mutate))
            .expect("analysis emitted an invalid local assignment address"),
    }
}

/// Apply the resolved operation of a transform assignment to the old
/// component value and the evaluated right operand (axis.w:8014-8035),
/// then apply the result coercion the call conversion recorded. Both
/// operands arrive shared: a builtin call hands the handles straight to the
/// registry, and a closure call shares them through the argument tuple.
fn apply_transform(
    operation: &TransformOperation,
    old: SharedValue,
    operand: SharedValue,
    conversion: Option<&'static str>,
    span: SourceSpan,
    context: &mut EvaluationContext,
) -> Result<Value, Control> {
    let result = match operation {
        TransformOperation::Builtin(builtin) => builtin_registry()[*builtin]
            .run(vec![old, operand], span, Level::SingleValue, context)?
            .expect("a transform builtin call yields a single value"),
        TransformOperation::Closure(closure) => unwrap_shared(
            apply_closure(
                closure,
                Rc::new(Value::Tuple(vec![old, operand])),
                context,
                Level::SingleValue,
            )?
            .expect("a transform closure call yields a single value"),
        ),
    };
    match conversion {
        Some(tag) => apply_conversion(tag, result, span),
        None => Ok(result),
    }
}

fn evaluate_slice(
    values: &[SharedValue],
    lower: BigInt,
    upper: BigInt,
    flags: crate::syntax::SliceFlags,
    source: &str,
    span: SourceSpan,
) -> Result<Vec<SharedValue>, Control> {
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
    let mut result = if flags.reverse_output {
        let reverse_lower = values.len() - upper_index;
        let reverse_upper = values.len() - lower_index;
        values[reverse_lower..reverse_upper].to_vec()
    } else {
        values[lower_index..upper_index].to_vec()
    };
    if flags.reverse_output {
        result.reverse();
    }
    Ok(result)
}

fn evaluate_string_slice(
    value: &str,
    lower: BigInt,
    upper: BigInt,
    flags: crate::syntax::SliceFlags,
    source: &str,
    span: SourceSpan,
) -> Result<String, Control> {
    let Some((lower, upper)) = slice_bounds(
        lower,
        upper,
        value.len(),
        flags,
        source,
        span,
    )?
    else {
        return Ok(String::new());
    };
    let bytes = value.as_bytes();
    let (selected_lower, selected_upper) = if flags.reverse_output {
        (bytes.len() - upper, bytes.len() - lower)
    } else {
        (lower, upper)
    };
    let selected = &bytes[selected_lower..selected_upper];
    if flags.reverse_output {
        Ok(String::from_utf8_lossy(&selected.iter().rev().copied().collect::<Vec<_>>()).into_owned())
    } else {
        Ok(String::from_utf8_lossy(selected).into_owned())
    }
}

/// `vector_slice` (axis.w:4322-4342): the reversed form takes the range from
/// the reversed storage, i.e. slices then reverses.
fn evaluate_vec_slice(
    vector: &Vec32,
    lower: BigInt,
    upper: BigInt,
    flags: crate::syntax::SliceFlags,
    source: &str,
    span: SourceSpan,
) -> Result<Vec32, Control> {
    let Some((lower, upper)) = slice_bounds(lower, upper, vector.0.len(), flags, source, span)?
    else {
        return Ok(Vec32(Vec::new()));
    };
    let entries = &vector.0;
    let selected: Vec<i32> = if flags.reverse_output {
        entries[entries.len() - upper..entries.len() - lower]
            .iter()
            .rev()
            .copied()
            .collect()
    } else {
        entries[lower..upper].to_vec()
    };
    Ok(Vec32(selected))
}

/// `ratvec_slice` (axis.w:4348-4368): slice the numerators and keep the
/// common denominator; the constructor re-normalises.
fn evaluate_ratvec_slice(
    value: &RatVec,
    lower: BigInt,
    upper: BigInt,
    flags: crate::syntax::SliceFlags,
    source: &str,
    span: SourceSpan,
) -> Result<RatVec, Control> {
    let Some((lower, upper)) =
        slice_bounds(lower, upper, value.numerators().len(), flags, source, span)?
    else {
        return Ok(RatVec::new(Vec::new(), 1).expect("empty ratvec is valid"));
    };
    let numerators = value.numerators();
    let selected: Vec<i64> = if flags.reverse_output {
        numerators[numerators.len() - upper..numerators.len() - lower]
            .iter()
            .rev()
            .copied()
            .collect()
    } else {
        numerators[lower..upper].to_vec()
    };
    Ok(RatVec::new(selected, value.denominator()).expect("slicing keeps the common denominator"))
}

/// `matrix_slice` (axis.w:4407-4427): one-dimensional matrix slices select
/// COLUMNS; the reversed form walks the columns from the end.
fn evaluate_matrix_slice(
    matrix: &Matrix,
    lower: BigInt,
    upper: BigInt,
    flags: crate::syntax::SliceFlags,
    source: &str,
    span: SourceSpan,
) -> Result<Matrix, Control> {
    let columns = matrix.cols();
    let Some((lower, upper)) = slice_bounds(lower, upper, columns, flags, source, span)? else {
        return Ok(Matrix::from_columns(matrix.rows(), 0, Vec::new()).expect("zero columns"));
    };
    let selected: Vec<usize> = if flags.reverse_output {
        ((columns - upper)..(columns - lower)).rev().collect()
    } else {
        (lower..upper).collect()
    };
    let mut data = Vec::with_capacity(matrix.rows() * (upper - lower));
    for column in selected {
        data.extend_from_slice(&matrix.column(column).0);
    }
    Ok(Matrix::from_columns(matrix.rows(), upper - lower, data)
        .expect("slice column count matches"))
}

fn slice_bounds(
    lower: BigInt,
    upper: BigInt,
    length: usize,
    flags: crate::syntax::SliceFlags,
    source: &str,
    span: SourceSpan,
) -> Result<Option<(usize, usize)>, Control> {
    let length_big = BigInt::from(length);
    let lower = if flags.lower_from_end {
        &length_big - lower
    } else {
        lower
    };
    let upper = if flags.upper_from_end {
        &length_big - upper
    } else {
        upper
    };
    let lower_out_of_range = lower < 0;
    let upper_out_of_range = upper > length_big;
    if lower_out_of_range || upper_out_of_range {
        let message = match (lower_out_of_range, upper_out_of_range) {
            (true, true) => format!(
                "both bounds {lower}:{upper} out of range (should be >=0 respectively <={length}) in slice {source}"
            ),
            (true, false) => {
                format!("lower bound {lower} out of range (should be >=0) in slice {source}")
            }
            (false, true) => format!(
                "upper bound {upper} out of range (should be <={length}) in slice {source}"
            ),
            (false, false) => unreachable!(),
        };
        return Err(runtime(message, span));
    }
    if lower >= upper {
        return Ok(None);
    }
    let lower_index = usize::try_from(&lower)
        .map_err(|_| runtime("slice lower bound is not a machine index", span))?;
    let upper_index = usize::try_from(&upper)
        .map_err(|_| runtime("slice upper bound is not a machine index", span))?;
    Ok(Some((lower_index, upper_index)))
}

fn list_to_vec32(values: Vec<SharedValue>, span: SourceSpan) -> Result<Vec32, Control> {
    let entries = values
        .iter()
        .map(|value| match value.as_ref() {
            Value::Integer(value) => narrow_i32(value, span),
            other => panic!("vec conversion saw non-integer {other}"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Vec32(entries))
}

fn rationals_to_ratvec(values: Vec<SharedValue>, span: SourceSpan) -> Result<RatVec, Control> {
    // Bring all entries over a common denominator, then normalise.
    let rationals: Vec<BigRational> = values
        .iter()
        .map(|value| match value.as_ref() {
            Value::Rational(value) => value.clone(),
            Value::Integer(value) => BigRational::from(value.clone()),
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
            .map(|column| Rc::new(Value::Vector(Vec32(column))))
            .collect(),
    )
}

fn matrix_to_integer_rows(matrix: Matrix) -> Value {
    Value::List(
        matrix_columns(matrix)
            .into_iter()
            .map(|column| {
                Rc::new(Value::List(
                    column
                        .into_iter()
                        .map(|entry| Rc::new(Value::Integer(BigInt::from(entry))))
                        .collect(),
                ))
            })
            .collect(),
    )
}

fn matrix_to_ratvectors(matrix: Matrix) -> Value {
    Value::List(
        matrix_columns(matrix)
            .into_iter()
            .map(|column| {
                Rc::new(Value::RatVector(
                    RatVec::new(column.into_iter().map(i64::from).collect(), 1)
                        .expect("unit denominator is nonzero"),
                ))
            })
            .collect(),
    )
}

fn matrix_to_rational_rows(matrix: Matrix) -> Value {
    Value::List(
        matrix_columns(matrix)
            .into_iter()
            .map(|column| {
                Rc::new(Value::List(
                    column
                        .into_iter()
                        .map(|entry| Rc::new(rational_value(entry, 1)))
                        .collect(),
                ))
            })
            .collect(),
    )
}

fn vectors_to_ratvectors(value: Value) -> Value {
    Value::List(
        expect_list(value)
            .into_iter()
            .map(|entry| match unwrap_shared(entry) {
                Value::Vector(vector) => Rc::new(Value::RatVector(
                    RatVec::new(vector.0.into_iter().map(i64::from).collect(), 1)
                        .expect("unit denominator is nonzero"),
                )),
                other => panic!("[Qv][V] conversion saw {other}"),
            })
            .collect(),
    )
}

fn vectors_to_rational_rows(value: Value) -> Value {
    Value::List(
        expect_list(value)
            .into_iter()
            .map(|entry| match unwrap_shared(entry) {
                Value::Vector(vector) => Rc::new(Value::List(
                    vector
                        .0
                        .into_iter()
                        .map(|item| Rc::new(rational_value(item, 1)))
                        .collect(),
                )),
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
                    .map(|&numerator| {
                        Rc::new(rational_value(numerator, BigInt::from(vector.denominator())))
                    })
                    .collect(),
            )),
            other => panic!("[Q]Qv conversion saw {other}"),
        },
        "[Q]V" => match value {
            Value::Vector(vector) => Ok(Value::List(
                vector
                    .0
                    .into_iter()
                    .map(|entry| Rc::new(rational_value(entry, 1)))
                    .collect(),
            )),
            other => panic!("[Q]V conversion saw {other}"),
        },
        "M[V]" => {
            let columns = expect_list(value)
                .into_iter()
                .map(|column| match unwrap_shared(column) {
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
                .map(|column| list_to_vec32(expect_list(unwrap_shared(column)), span))
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
                .map(|entry| {
                    Ok(Rc::new(Value::Vector(list_to_vec32(
                        expect_list(unwrap_shared(entry)),
                        span,
                    )?)))
                })
                .collect::<Result<Vec<_>, _>>()?,
        )),
        "[[I]][V]" => Ok(Value::List(
            expect_list(value)
                .into_iter()
                .map(|entry| match unwrap_shared(entry) {
                    Value::Vector(vector) => Ok(Rc::new(Value::List(
                        vector
                            .0
                            .into_iter()
                            .map(|item| Rc::new(Value::Integer(BigInt::from(item))))
                            .collect(),
                    ))),
                    other => panic!("[[I]][V] conversion saw {other}"),
                })
                .collect::<Result<Vec<_>, Control>>()?,
        )),
        "[I]V" => match value {
            Value::Vector(vector) => Ok(Value::List(
                vector
                    .0
                    .into_iter()
                    .map(|entry| Rc::new(Value::Integer(BigInt::from(entry))))
                    .collect(),
            )),
            other => panic!("[I]V conversion saw {other}"),
        },
        "[Q][I]" => Ok(Value::List(
            expect_list(value)
                .into_iter()
                .map(|entry| match unwrap_shared(entry) {
                    Value::Integer(value) => Rc::new(Value::Rational(BigRational::from(value))),
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
                    let vector = list_to_vec32(expect_list(unwrap_shared(entry)), span)?;
                    Ok(Rc::new(Value::RatVector(
                        RatVec::new(vector.0.into_iter().map(i64::from).collect(), 1)
                            .expect("unit denominator is nonzero"),
                    )))
                })
                .collect::<Result<Vec<_>, Control>>()?,
        )),
        "[[Q]][[I]]" => Ok(Value::List(
            expect_list(value)
                .into_iter()
                .map(|entry| {
                    Ok(Rc::new(Value::List(
                        list_to_vec32(expect_list(unwrap_shared(entry)), span)?
                            .0
                            .into_iter()
                            .map(|item| Rc::new(rational_value(item, 1)))
                            .collect(),
                    )))
                })
                .collect::<Result<Vec<_>, Control>>()?,
        )),
        "LT" | "RdIc" | "IcRf" | "RdRf" | "SpI" | "Sp(I,I)" | "KpolK" | "PolP" => {
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
        let last_value = Value::Tuple(Vec::new());
        let last_type: TypeCell = Rc::new(RefCell::new(Type::void()));
        let analysis = Analysis::new(&table, globals, &overloads, &last_value, &last_type);
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
        Ok((required, unwrap_shared(value)))
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
    fn component_assignment_updates_row_components() {
        let cell = crate::frames::global_with(Rc::new(Value::List(vec![
            Rc::new(Value::Integer(1.into())),
            Rc::new(Value::Integer(2.into())),
            Rc::new(Value::Integer(3.into())),
        ])));
        let mut globals = IdTable::new();
        globals.define("a", Type::row(Type::Primitive(Prim::Int)), cell.clone());

        // Plain, reversed, and transform forms (axis.w:7940-8035).
        let (type_, value) =
            convert_and_run_with("a[0] := 9", &globals).expect("component assignment");
        assert_eq!(type_, Type::Primitive(Prim::Int));
        assert_eq!(value, Value::Integer(9.into()));
        assert_eq!(
            cell.borrow().as_ref().map(|value| value.to_string()),
            Some("[9,2,3]".into())
        );
        let (_, value) = convert_and_run_with("a~[0] := 7", &globals).expect("reversed form");
        assert_eq!(value, Value::Integer(7.into()));
        let (_, value) = convert_and_run_with("a[1] *:= 10", &globals).expect("transform");
        assert_eq!(value, Value::Integer(20.into()));
        assert_eq!(
            cell.borrow().as_ref().map(|value| value.to_string()),
            Some("[9,20,7]".into())
        );

        // The out-of-range diagnostic quotes the compact source node
        // (range_mess, axis.w:7953).
        let error = convert_and_run_with("a[5] := 1", &globals).expect_err("out of range");
        assert_eq!(
            error.message,
            "index 5 out of range (0<= . <3) in component assignment a[5]:=1"
        );
        // A string aggregates no assignable components (axis.w:8163-8172).
        let string_cell = crate::frames::global_with(Rc::new(Value::String("abc".into())));
        globals.define("s", Type::Primitive(Prim::String), string_cell);
        let error = convert_and_run_with("s[0] := \"x\"", &globals).expect_err("string target");
        assert_eq!(
            error.message,
            "Cannot subscript value of type string with index of type int in assignment"
        );
        // The transform checks assignability before resolving the operator.
        let error = convert_and_run_with("s[0] +:= \"x\"", &globals).expect_err("string transform");
        assert_eq!(
            error.message,
            "Cannot assign to component of value of type string selected by index of type int in transforming assignment"
        );
        // The operator result must match the component type (the desugared
        // call converts against it, axis.w:8516-8519).
        let error = convert_and_run_with("a[1] /:= 2", &globals).expect_err("rat result");
        assert_eq!(error.message, "found rat while int was needed.");
        // Uninitialized, undefined, and constant diagnostics.
        let unset = crate::frames::unset_global();
        globals.define("u", Type::row(Type::Primitive(Prim::Int)), unset);
        let error = convert_and_run_with("u[0] := 5", &globals).expect_err("uninitialized");
        assert_eq!(
            error.message,
            "Assigning to component of uninitialized variable u"
        );
        let error =
            convert_and_run_with("u[0] +:= 5", &globals).expect_err("uninitialized transform");
        assert_eq!(
            error.message,
            "Transforming component of uninitialized variable u"
        );
        let error = convert_and_run("zz[0] := 1").expect_err("undefined");
        assert_eq!(
            error.message,
            "Undefined identifier 'zz' in component assignment zz[0]:=1"
        );
        let error = convert_and_run("zz[0] +:= 1").expect_err("undefined transform");
        assert_eq!(
            error.message,
            "Undefined identifier 'zz' in component transform zz[0] +:= 1"
        );
        globals.mark_const("a");
        let error = convert_and_run_with("a[0] := 1", &globals).expect_err("constant");
        assert_eq!(
            error.message,
            "Name 'a' is constant in component assignment a[0]:=1"
        );
    }

    #[test]
    fn component_assignment_copies_on_write_through_aliases() {
        // Two names bound to one aggregate: mutating through one must not
        // disturb the other (Atlas copy-on-assignment), even though the
        // evaluator now writes uniquely held aggregates in place.
        let (_, value) = convert_and_run(
            "let x = [1,2,3] in let y = x in begin x[0] := 9; (x, y) end",
        )
        .expect("aliased row component write");
        assert_eq!(value.to_string(), "([9,2,3],[1,2,3])");

        let (_, value) = convert_and_run(
            "let v = vec: [1,2,3] in let w = v in begin v[1] := 7; (v, w) end",
        )
        .expect("aliased vec component write");
        assert_eq!(
            value,
            Value::Tuple(vec![
                Rc::new(Value::Vector(Vec32(vec![1, 7, 3]))),
                Rc::new(Value::Vector(Vec32(vec![1, 2, 3]))),
            ])
        );

        let (_, value) = convert_and_run(
            "let M = null(2,2) in let N = M in begin M[1,1] := 2; (M[1,1], N[1,1]) end",
        )
        .expect("aliased matrix entry write");
        assert_eq!(
            value,
            Value::Tuple(vec![
                Rc::new(Value::Integer(2.into())),
                Rc::new(Value::Integer(0.into()))
            ])
        );

        // A transform on an aliased aggregate also copies on write.
        let (_, value) = convert_and_run(
            "let x = [1,2,3] in let y = x in begin x[1] +:= 10; (x, y) end",
        )
        .expect("aliased transform");
        assert_eq!(value.to_string(), "([1,12,3],[1,2,3])");

        // The index expression may read the very aggregate being assigned:
        // the slot stays populated until the write phase.
        let (_, value) = convert_and_run("let x = [1,2,3] in begin x[x[0]] := 9; x end")
            .expect("self-reading index");
        assert_eq!(value.to_string(), "[1,9,3]");

        // An RHS that reassigns the target gets clobbered by the modified
        // read value — the legacy copy-read/write-back behavior, kept
        // verbatim by mutate_aggregate's replace branch.
        let (_, value) =
            convert_and_run("let x = [1,2,3] in begin x[0] := (x := [7,8]; 1); x end")
                .expect("reassigning rhs");
        assert_eq!(value.to_string(), "[1,2,3]");
    }

    #[test]
    fn vector_and_matrix_subscriptions_read_and_write_components() {
        let vector_cell = crate::frames::global_with(Rc::new(Value::Vector(Vec32(vec![1, 2, 3]))));
        let ratvec_cell = crate::frames::global_with(Rc::new(Value::RatVector(
            RatVec::new(vec![1, 2], 2).expect("valid ratvec"),
        )));
        let matrix_cell = crate::frames::global_with(Rc::new(Value::Matrix(
            Matrix::from_columns(2, 2, vec![1, 3, 2, 4]).expect("valid matrix"),
        )));
        let mut globals = IdTable::new();
        globals.define("v", primitive_type(Prim::Vec), vector_cell.clone());
        globals.define("rv", primitive_type(Prim::RatVec), ratvec_cell);
        globals.define("M", primitive_type(Prim::Mat), matrix_cell.clone());

        for (source, expected) in [
            ("v[0]", "1"),
            ("v~[0]", "3"),
            ("rv[0]", "1/2"),
            ("rv~[1]", "1/2"),
            ("M[0]", "[ 1, 3 ]"),
            ("M[0,1]", "2"),
            ("M[1,0]", "3"),
            ("M~[1,0]", "2"),
        ] {
            let (_, value) = convert_and_run_with(source, &globals).expect(source);
            assert_eq!(value.to_string(), expected, "source: {source}");
        }

        convert_and_run_with("v[0] := 7", &globals).expect("vec write");
        convert_and_run_with("v[0] +:= 2", &globals).expect("vec transform");
        convert_and_run_with("M[1] := [9,9]", &globals).expect("matrix column write");
        convert_and_run_with("M[0,1] := 9", &globals).expect("matrix entry write");
        convert_and_run_with("M[1,1] +:= 10", &globals).expect("matrix entry transform");

        assert_eq!(
            vector_cell.borrow().as_ref().unwrap().to_string(),
            "[ 9, 2, 3 ]"
        );
        let matrix_binding = matrix_cell.borrow();
        let matrix = match matrix_binding.as_ref().unwrap().as_ref() {
            Value::Matrix(matrix) => matrix,
            other => panic!("expected matrix, got {other:?}"),
        };
        assert_eq!(matrix.entry(0, 0), Some(1));
        assert_eq!(matrix.entry(1, 0), Some(3));
        assert_eq!(matrix.entry(0, 1), Some(9));
        assert_eq!(matrix.entry(1, 1), Some(19));

        let error = convert_and_run_with("v[5]", &globals).expect_err("vec range");
        assert_eq!(
            error.message,
            "index 5 out of range (0<= . <3) in subscription v[5]"
        );
        let error = convert_and_run_with("M[5]", &globals).expect_err("matrix column range");
        assert_eq!(
            error.message,
            "index 5 out of range (0<= . <2) in matrix column selection M[5]"
        );
        let error = convert_and_run_with("M[0,5]", &globals).expect_err("matrix entry range");
        assert_eq!(
            error.message,
            "final index 5 out of range (0<= . <2) in matrix subscription M[0,5]"
        );
        let error = convert_and_run_with("rv[0] := 1", &globals).expect_err("ratvec read only");
        assert_eq!(
            error.message,
            "Cannot subscript value of type ratvec with index of type int in assignment"
        );
        let error =
            convert_and_run_with("M[0,\"x\"] := 1", &globals).expect_err("matrix entry type");
        assert_eq!(
            error.message,
            "Cannot subscript value of type mat with index of type (int,string) in assignment"
        );
        // Transform range checks fire on the synthetic READ, so the oracle
        // quotes the selection with subscription wording.
        let error = convert_and_run_with("v[5] +:= 1", &globals).expect_err("vec transform range");
        assert_eq!(
            error.message,
            "index 5 out of range (0<= . <3) in subscription v[5]"
        );
        let error =
            convert_and_run_with("M[5] +:= v", &globals).expect_err("column transform range");
        assert_eq!(
            error.message,
            "index 5 out of range (0<= . <2) in matrix column selection M[5]"
        );
        let error =
            convert_and_run_with("M[5,0] +:= 1", &globals).expect_err("entry transform range");
        assert_eq!(
            error.message,
            "initial index 5 out of range (0<= . <2) in matrix subscription M[5,0]"
        );
        // Assignment range messages quote the assignment node, including the
        // conversion tag of a converted right side (range_mess, axis.w:7953).
        let error = convert_and_run_with("M[5] := [1,2]", &globals).expect_err("column range");
        assert_eq!(
            error.message,
            "index 5 out of range (0<= . <2) in matrix column assignment M[5]:=V[I]:[1,2]"
        );
        let error = convert_and_run_with("M[5,0] := 1", &globals).expect_err("entry range");
        assert_eq!(
            error.message,
            "initial index 5 out of range (0<= . <2) in matrix entry assignment M[(5,0)]:=1"
        );
        // A replacement column must match the matrix height.
        let error = convert_and_run_with("M[0] := [9]", &globals).expect_err("column size");
        assert_eq!(
            error.message,
            "Cannot replace column of size 2 by one of size 1"
        );
        // A pair index only subscriptions a matrix.
        let error = convert_and_run_with("v[0,1]", &globals).expect_err("vec pair index");
        assert_eq!(
            error.message,
            "Cannot subscript value of type vec with index of type (int,int)"
        );
    }

    #[test]
    fn alias_name_declares_a_variable() {
        // parser.y lexes a defined type name as TYPE_ID, making `p: Pair`
        // a declaration; with no type-table token the bare-identifier right
        // side naming a known type is re-routed to declaration instead.
        let mut context = TypedContext::new();
        let mut reports = Vec::new();
        let mut values = Vec::new();
        for source in [
            "set_type Pair = (int x, int y)",
            "p: Pair",
            "p := (3,4)",
            "p.x",
            "p.y +:= 10",
            "p",
        ] {
            let events = context.execute(&command(source)).expect(source);
            for event in events {
                match event {
                    TypedCommandEvent::Value { value, .. } => values.push(value.to_string()),
                    TypedCommandEvent::ReportLine { text, .. } => reports.push(text),
                    _ => {}
                }
            }
        }
        assert!(
            reports
                .iter()
                .any(|text| text == "Declaring identifier 'p': (int,int)\n"),
            "declaration report missing: {reports:?}"
        );
        assert_eq!(values, ["(3,4)", "3", "14", "(3,14)"]);
    }

    #[test]
    fn field_assignment_updates_tuple_components() {
        let mut context = TypedContext::new();
        let mut values = Vec::new();
        let mut errors = Vec::new();
        for source in [
            "set_type Pair = (int x, int y)",
            "q: (int,int)",
            "q := (1,2)",
            "q.x := 7",
            "q",
            "q.y +:= 10",
            "q",
            "q.z := 7",
            "q.x /:= 2",
            "q.x +:= \"s\"",
            "set !c = (3,4)",
            "c.x := 9",
            "c.x +:= 1",
            "r: (int,int)",
            "r.x := 5",
        ] {
            match context.execute(&command(source)) {
                Ok(events) => {
                    for event in events {
                        if let TypedCommandEvent::Value { value, .. } = event {
                            values.push(value.to_string());
                        }
                    }
                }
                Err(error) => errors.push(error.message),
            }
        }
        assert_eq!(values, ["(1,2)", "7", "(7,2)", "12", "(7,12)"]);
        assert_eq!(
            errors,
            [
                // No `z` projector exists for (int,int) (axis.w:8252).
                "Improper selection in field assignment",
                // The operator result must match the component type.
                "found rat while int was needed.",
                // The desugared call converts the operator first.
                "Failed to match '+' with argument type (int,string)",
                "Name 'c' is constant in field assignment c.x:=9",
                "Name 'c' is constant in field transform c.x +:= 1",
                "Assigning to field of uninitialized variable r",
            ]
        );
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
            Rc::new(Value::Integer(0.into())),
            Rc::new(Value::Integer(0.into())),
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
            Value::Tuple(vec![
                Rc::new(Value::Integer(20.into())),
                Rc::new(Value::Integer(22.into()))
            ])
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
                Rc::new(Value::Integer(1.into())),
                Rc::new(Value::String("ignored".into())),
                Rc::new(Value::Integer(3.into())),
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
            Value::Tuple(vec![
                Rc::new(Value::Integer(1.into())),
                Rc::new(Value::Tuple(Vec::new()))
            ])
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
            Value::Tuple(vec![
                Rc::new(Value::Integer(1.into())),
                Rc::new(Value::Integer(1.into()))
            ])
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
                Rc::new(Value::Integer(1.into())),
                Rc::new(Value::Integer(2.into())),
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
                Rc::new(Value::List(vec![Rc::new(Value::String("x".into()))])),
                Rc::new(Value::Integer(0.into())),
            ])
        );
        let (row_type, _) = globals.lookup("row").expect("row remains defined");
        assert_eq!(*row_type.borrow(), Type::row(Type::Primitive(Prim::Int)));
        assert_eq!(
            row.borrow().as_deref(),
            Some(&Value::List(vec![Rc::new(Value::String("x".into()))]))
        );
    }

    #[test]
    fn multi_assignment_reports_exact_target_analysis_errors() {
        let x = crate::frames::global_with(Rc::new(Value::Integer(0.into())));
        let constant = crate::frames::global_with(Rc::new(Value::Integer(0.into())));
        let pair = crate::frames::global_with(Rc::new(Value::Tuple(vec![
            Rc::new(Value::Integer(0.into())),
            Rc::new(Value::Integer(0.into())),
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
            Value::List(vec![
                Rc::new(Value::Integer(20.into())),
                Rc::new(Value::Integer(30.into()))
            ])
        );

        let (_, value) = convert_and_run("[10,20,30,40][2:]").expect("open upper slice");
        assert_eq!(
            value,
            Value::List(vec![
                Rc::new(Value::Integer(30.into())),
                Rc::new(Value::Integer(40.into()))
            ])
        );

        let (_, value) = convert_and_run("[10,20,30,40]~[1:3]").expect("reverse subject");
        assert_eq!(
            value,
            Value::List(vec![
                Rc::new(Value::Integer(30.into())),
                Rc::new(Value::Integer(20.into()))
            ])
        );
        let (_, value) = convert_and_run("[10,20,30,40][3~:4]").expect("reverse lower");
        assert_eq!(
            value,
            Value::List(vec![
                Rc::new(Value::Integer(20.into())),
                Rc::new(Value::Integer(30.into())),
                Rc::new(Value::Integer(40.into()))
            ])
        );
        let (_, value) = convert_and_run("[10,20,30,40][0:1~]").expect("reverse upper");
        assert_eq!(
            value,
            Value::List(vec![
                Rc::new(Value::Integer(10.into())),
                Rc::new(Value::Integer(20.into())),
                Rc::new(Value::Integer(30.into()))
            ])
        );

        let (_, value) = convert_and_run("\"abc\"[1:]").expect("string slice");
        assert_eq!(value, Value::String("bc".into()));
        let (_, value) = convert_and_run("\"abc\"~[0:2]").expect("reversed string slice");
        assert_eq!(value, Value::String("cb".into()));
        let error = convert_and_run("\"abc\"[-1:2]").expect_err("string slice range");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(
            error.message,
            "lower bound -1 out of range (should be >=0) in slice \"abc\"[-1:2]"
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
    fn two_index_subscription_parses_then_fails_typing_like_the_oracle() {
        // parser.y:585-598,606-613: `M[i,j]` (and `M~[i,j]`) parses as a
        // subscription whose index is a two-element tuple display; typing
        // always rejects the tuple index (axis.w:4101-4105). Three indices
        // stay a syntax error upstream ("unexpected ',', expecting ']'").
        let error = convert_and_run("[[1,2],[3,4]][0,1]").expect_err("tuple index");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(
            error.message,
            "Cannot subscript value of type [[int]] with index of type (int,int)"
        );

        let error = convert_and_run("[[1,2],[3,4]]~[0,1]").expect_err("reversed tuple index");
        assert_eq!(
            error.message,
            "Cannot subscript value of type [[int]] with index of type (int,int)"
        );

        let cell = crate::frames::global_with(Rc::new(Value::List(vec![
            Rc::new(Value::Integer(1.into())),
            Rc::new(Value::Integer(2.into())),
            Rc::new(Value::Integer(3.into())),
        ])));
        let mut globals = IdTable::new();
        globals.define("a", Type::row(Type::Primitive(Prim::Int)), cell);
        let error =
            convert_and_run_with("a[0,1] := 5", &globals).expect_err("tuple component assignment");
        assert_eq!(
            error.message,
            "Cannot subscript value of type [int] with index of type (int,int) in assignment"
        );
        let error =
            convert_and_run_with("a[0,1] +:= 5", &globals).expect_err("tuple component transform");
        assert_eq!(
            error.message,
            "Cannot assign to component of value of type [int] selected by index of type (int,int) in transforming assignment"
        );

        let error = parse(&SourceText::new("[[1,2],[3,4]][0,1,2]")).expect_err("three indices");
        assert_eq!(error.kind, ErrorKind::Syntax);
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
    fn global_batch3_builtins_match_the_upstream_linear_algebra_surface() {
        let cases = [
            // echelon (global.w:5202, matreduc.h:128-161): zero columns are
            // REMOVED from E, the kernel columns rotate right in C, pivots
            // ascend, flip = sign det(C).
            (
                "echelon(mat: [[1,2],[3,4]])",
                "(\n| 1, 1 |\n| 0, 2 |\n,\n| -2, 1 |\n|  1, 0 |\n,[0,1],-1)",
            ),
            // Rank-deficient: E keeps only the rank column.
            (
                "echelon(mat: [[2,4],[4,8]])",
                "(\n| 2 |\n| 4 |\n,\n| 1, -2 |\n| 0,  1 |\n,[1],1)",
            ),
            ("echelon(null(0,0))", "(The 0x0 matrix,The 0x0 matrix,[],1)"),
            (
                "echelon(null(2,3))",
                "(The 2x0 matrix,\n| 1, 0, 0 |\n| 0, 1, 0 |\n| 0, 0, 1 |\n,[],1)",
            ),
            // kernel (global.w:5206, lattice.cpp:133-140): the basis order
            // is oracle-defined by the echelon recorder rotation.
            ("kernel(mat: [[1,2],[2,4]])", "\n| -2 |\n|  1 |\n"),
            ("kernel(mat: [[1,0],[0,1]])", "The 2x0 matrix"),
            (
                "kernel(null(0,3))",
                "\n| 1, 0, 0 |\n| 0, 1, 0 |\n| 0, 0, 1 |\n",
            ),
            // eigen_lattice (global.w:5207): NO square check; the diagonal
            // touch runs up to min(rows,cols).
            ("eigen_lattice(mat: [[2,1],[1,2]], 1)", "\n| -1 |\n|  1 |\n"),
            (
                "eigen_lattice(mat: [[1,2],[3,4],[5,6]], 1)",
                "\n|  -3 |\n| -10 |\n|   6 |\n",
            ),
            ("eigen_lattice(null(2,3), 5)", "\n| 0 |\n| 0 |\n| 1 |\n"),
            // invert (global.w:5210, matrix.cpp:471-498): (N,d) with
            // N/d = M^-1, d the positive lcm of denominators; a SINGULAR
            // square matrix returns the zero matrix with d=0, no error.
            (
                "invert(mat: [[1,2],[3,4]])",
                "(\n| -4,  3 |\n|  2, -1 |\n,2)",
            ),
            ("invert(mat: [[2,0],[0,3]])", "(\n| 3, 0 |\n| 0, 2 |\n,6)"),
            ("invert(mat: [[1,2],[2,4]])", "(\n| 0, 0 |\n| 0, 0 |\n,0)"),
            ("invert(null(0,0))", "(The 0x0 matrix,1)"),
            // Smith (global.w:5209, matreduc.cpp:359-385): factors positive,
            // divisibility-ordered by the correction loop; a zero matrix
            // gives (identity, []).
            (
                "Smith(mat: [[2,0],[0,3]])",
                "(\n|  4, -1 |\n| -3,  1 |\n,[ 1, 6 ])",
            ),
            ("Smith(mat: [[0]])", "(\n| 1 |\n,[ ])"),
            ("Smith(null(2,3))", "(\n| 1, 0 |\n| 0, 1 |\n,[ ])"),
            // adapted_basis (global.w:5205): the diagonal is NOT
            // divisibility-ordered.
            (
                "adapted_basis(mat: [[2,4],[4,8]])",
                "(\n| 1, 0 |\n| 2, 1 |\n,[ 2 ])",
            ),
            ("adapted_basis(null(0,0))", "(The 0x0 matrix,[ ])"),
            // diagonalize (global.w:5204): (diagonal, row, column) — the
            // diagonal comes FIRST; only its first entry may be negative.
            (
                "diagonalize(mat: [[2,0],[0,3]])",
                "([ 2, 3 ],\n| 1, 0 |\n| 0, 1 |\n,\n| 1, 0 |\n| 0, 1 |\n)",
            ),
            ("diagonalize(mat: [[0-2]])", "([ -2 ],\n| 1 |\n,\n| 1 |\n)"),
            (
                "diagonalize(mat: [[0,2],[2,0]])",
                "([ 2, 2 ],\n| 0, 1 |\n| 1, 0 |\n,\n| 1, 0 |\n| 0, 1 |\n)",
            ),
            (
                "diagonalize(null(0,0))",
                "([ ],The 0x0 matrix,The 0x0 matrix)",
            ),
            // Bezout (global.w:5201): v*C == [d,0,...]; det(C) may be -1.
            (
                "Bezout([6,10,15])",
                "(1,\n|  1,  5, -5 |\n|  1,  0, -3 |\n| -1, -2,  4 |\n)",
            ),
            ("Bezout(null(0))", "(0,The 0x0 matrix)"),
            ("Bezout([0-6,9])", "(3,\n| 1, -3 |\n| 1, -2 |\n)"),
            // Machine-int wrapping is observable in the recorder:
            // 2147483647*1 - (-2)*1073741823 wraps to -2147483647... the
            // recorder's bottom-right entry is the wrapped -2^31+1.
            (
                "Bezout([2147483647,0-2])",
                "(1,\n|          1,          -2 |\n| 1073741823, -2147483647 |\n)",
            ),
            // linear_solve (global.w:5203): the first union-returning
            // builtin. Non-exact division scales the solution by `factor`;
            // inconsistency is CAUGHT into the empty_set variant.
            (
                "linear_solve(mat: [[2,0],[0,4]], vec: [6,4])",
                "([ 3, 1 ],1,The 2x0 matrix).affine_subspace",
            ),
            (
                "linear_solve(mat: [[2,0],[0,4]], vec: [6,3])",
                "([ 12,  3 ],4,The 2x0 matrix).affine_subspace",
            ),
            (
                "linear_solve(mat: [[1,2],[2,4]], vec: [3,5])",
                "().empty_set",
            ),
            (
                "linear_solve(mat: [[1,2],[2,4]], vec: [3,6])",
                "([ 3, 0 ],1,\n| -2 |\n|  1 |\n).affine_subspace",
            ),
            (
                "linear_solve(null(0,3), null(0))",
                "([ 0, 0, 0 ],1,\n| 1, 0, 0 |\n| 0, 1, 0 |\n| 0, 0, 1 |\n).affine_subspace",
            ),
            // row_saturate (global.w:5208, hunger 3): adapted_basis of the
            // transpose, as rows.
            ("row_saturate(mat: [[2,4],[4,8]])", "\n| 1, 2 |\n"),
            ("row_saturate(null(2,3))", "The 0x3 matrix"),
        ];
        for (source, expected) in cases {
            let (_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(value.to_string(), expected, "source: {source}");
        }

        // The result type of linear_solve is the two-variant union
        // (global.w:5203): `|vec,int,mat`.
        let (found, _) = convert_and_run("linear_solve(null(0,0), null(0) )").expect("union type");
        assert_eq!(
            found,
            Type::union_of(vec![
                Type::void(),
                Type::tuple(vec![
                    Type::Primitive(Prim::Vec),
                    Type::Primitive(Prim::Int),
                    Type::Primitive(Prim::Mat),
                ]),
            ])
        );

        // Rejections: only invert (non-square), linear_solve (size
        // mismatch) and eigen_lattice (int narrowing) diagnose, all BEFORE
        // the no-value gate (a for-loop body runs at no-value).
        for (source, expected) in [
            ("invert(null(2,3))", "Cannot invert a 2x3 matrix"),
            (
                "invert(mat: [[1,2,3],[4,5,6]])",
                "Cannot invert a 3x2 matrix",
            ),
            (
                "for i:2 do invert(null(2,3)) od",
                "Cannot invert a 2x3 matrix",
            ),
            (
                "linear_solve(null(2,3), vec: [1,2,3])",
                "Linear system size mismatch 2:3",
            ),
            (
                "for i:2 do linear_solve(null(2,3), vec: [1,2,3]) od",
                "Linear system size mismatch 2:3",
            ),
            (
                "eigen_lattice(mat: [[1]], 2147483648)",
                "Integer value to big for conversion",
            ),
            (
                "for i:2 do eigen_lattice(mat: [[1]], 2147483648) od",
                "Integer value to big for conversion",
            ),
        ] {
            match convert_and_run(source) {
                Err(error) => assert_eq!(error.message, expected, "source: {source}"),
                Ok(value) => panic!("{source} unexpectedly succeeded with {value:?}"),
            }
        }
    }

    #[test]
    fn global_batch4_builtins_match_the_upstream_slicer_and_gf2_surface() {
        // swiss_matrix_knife (global.w:4675-4809, install :5195-5196): the
        // flag-bitfield slicer. M below is mat: [[1,2],[3,4],[5,6]] — the
        // 2x3 matrix | 1, 3, 5 | / | 2, 4, 6 | (mat literals are COLUMNS).
        for (source, expected) in [
            (
                "swiss_matrix_knife(0, mat: [[1,2],[3,4],[5,6]], 0, 2, 0, 3)",
                "\n| 1, 3, 5 |\n| 2, 4, 6 |\n",
            ),
            // bit 6 transposes (dimensions swapped BEFORE the copy).
            (
                "swiss_matrix_knife(64, mat: [[1,2],[3,4],[5,6]], 0, 2, 0, 3)",
                "\n| 1, 2 |\n| 3, 4 |\n| 5, 6 |\n",
            ),
            // bit 7 negates; bit 0 reverses output rows; bit 3 columns.
            (
                "swiss_matrix_knife(128, mat: [[1,2],[3,4],[5,6]], 0, 2, 0, 3)",
                "\n| -1, -3, -5 |\n| -2, -4, -6 |\n",
            ),
            (
                "swiss_matrix_knife(129, mat: [[1,2],[3,4],[5,6]], 0, 2, 0, 3)",
                "\n| -2, -4, -6 |\n| -1, -3, -5 |\n",
            ),
            (
                "swiss_matrix_knife(8, mat: [[1,2],[3,4],[5,6]], 0, 2, 0, 3)",
                "\n| 5, 3, 1 |\n| 6, 4, 2 |\n",
            ),
            // from-end bound bits 1/2/4/5.
            (
                "swiss_matrix_knife(2, mat: [[1,2],[3,4],[5,6]], 1, 2, 0, 3)",
                "\n| 2, 4, 6 |\n",
            ),
            (
                "swiss_matrix_knife(32, mat: [[1,2],[3,4],[5,6]], 0, 2, 0, 1)",
                "\n| 1, 3 |\n| 2, 4 |\n",
            ),
            // inverted ranges clamp to empty, keeping the (swapped) shape.
            (
                "swiss_matrix_knife(0, mat: [[1,2],[3,4],[5,6]], 2, 0, 0, 1)",
                "The 0x1 matrix",
            ),
            (
                "swiss_matrix_knife(64, mat: [[1,2],[3,4],[5,6]], 2, 0, 0, 1)",
                "The 1x0 matrix",
            ),
            // flags truncate mod 256 (BitSet<8>, no range/negativity check):
            // 256 == 0 (identity), -1 sets all bits (from-end bits send the
            // row bounds to m, m-2 and the clamp fires).
            (
                "swiss_matrix_knife(256, mat: [[1,2],[3,4],[5,6]], 0, 2, 0, 3)",
                "\n| 1, 3, 5 |\n| 2, 4, 6 |\n",
            ),
            (
                "swiss_matrix_knife(0-1, mat: [[1,2],[3,4],[5,6]], 0, 2, 0, 3)",
                "The 0x0 matrix",
            ),
            ("swiss_matrix_knife(0, null(0,0), 0, 0, 0, 0)", "The 0x0 matrix"),
            ("swiss_matrix_knife(255, null(2,3), 0, 0, 0, 0)", "The 0x0 matrix"),
            // bit 7 negate wraps i32: -(i32::MIN) is itself.
            (
                "swiss_matrix_knife(128, mat: [[0-2147483648]], 0, 1, 0, 1)",
                "\n| -2147483648 |\n",
            ),
            // mod2_section (global.w:5043-5053, bitvector.cpp:346-405): the
            // GF(2) section, TRANSPOSE-shaped output.
            (
                "mod2_section(mat: [[1,0],[0,1]])",
                "\n| 1, 0 |\n| 0, 1 |\n",
            ),
            (
                "mod2_section(mat: [[1,1],[0,1],[1,0]])",
                "\n| 1, 0 |\n| 1, 1 |\n| 0, 0 |\n",
            ),
            // all-even entries reduce to zero mod 2; negative odd are 1.
            (
                "mod2_section(mat: [[2,4],[6,8]])",
                "\n| 0, 0 |\n| 0, 0 |\n",
            ),
            ("mod2_section(mat: [[0-1]])", "\n| 1 |\n"),
            (
                "mod2_section(mat: [[1,0,1],[0,1,1]])",
                "\n| 1, 0, 0 |\n| 0, 1, 0 |\n",
            ),
            ("mod2_section(null(0,0))", "The 0x0 matrix"),
            (
                "mod2_section(null(2,3))",
                "\n| 0, 0 |\n| 0, 0 |\n| 0, 0 |\n",
            ),
            // subspace_normal (global.w:5062-5174): (basis, combinations,
            // relations, [pivots]) with PIVOT-ASCENDING output order.
            (
                "subspace_normal(mat: [[1,0],[0,1]])",
                "(\n| 1, 0 |\n| 0, 1 |\n,\n| 1, 0 |\n| 0, 1 |\n,The 2x0 matrix,[0,1])",
            ),
            (
                "subspace_normal(mat: [[1,1],[1,0],[0,1]])",
                "(\n| 1, 0 |\n| 0, 1 |\n,\n| 0, 1 |\n| 1, 1 |\n| 0, 0 |\n,\n| 1 |\n| 1 |\n| 1 |\n,[0,1])",
            ),
            (
                "subspace_normal(mat: [[1,1,2],[2,0,2]])",
                "(\n| 1 |\n| 1 |\n| 0 |\n,\n| 1 |\n| 0 |\n,\n| 0 |\n| 1 |\n,[0])",
            ),
            (
                "subspace_normal(mat: [[1,0],[1,0],[0,0]])",
                "(\n| 1 |\n| 0 |\n,\n| 1 |\n| 0 |\n| 0 |\n,\n| 1, 0 |\n| 1, 0 |\n| 0, 1 |\n,[0])",
            ),
            (
                "subspace_normal(mat: [[0,1],[0,1]])",
                "(\n| 0 |\n| 1 |\n,\n| 1 |\n| 0 |\n,\n| 1 |\n| 1 |\n,[1])",
            ),
            (
                "subspace_normal(mat: [[0-1, 0],[0, 0-3]])",
                "(\n| 1, 0 |\n| 0, 1 |\n,\n| 1, 0 |\n| 0, 1 |\n,The 2x0 matrix,[0,1])",
            ),
            (
                "subspace_normal(mat: [[3,5],[7,9]])",
                "(\n| 1 |\n| 1 |\n,\n| 1 |\n| 0 |\n,\n| 1 |\n| 1 |\n,[0])",
            ),
            (
                "subspace_normal(null(0,0))",
                "(The 0x0 matrix,The 0x0 matrix,The 0x0 matrix,[])",
            ),
            (
                "subspace_normal(null(3,0))",
                "(The 3x0 matrix,The 0x0 matrix,The 0x0 matrix,[])",
            ),
            (
                "subspace_normal(null(0,2))",
                "(The 0x0 matrix,The 2x0 matrix,\n| 1, 0 |\n| 0, 1 |\n,[])",
            ),
        ] {
            let (_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(value.to_string(), expected, "source: {source}");
        }

        // The result type of subspace_normal is (mat,mat,mat,[int])
        // (global.w:5212-5213).
        let (found, _) = convert_and_run("subspace_normal(null(0,0))").expect("tuple type");
        assert_eq!(
            found,
            Type::tuple(vec![
                Type::Primitive(Prim::Mat),
                Type::Primitive(Prim::Mat),
                Type::Primitive(Prim::Mat),
                Type::row(Type::Primitive(Prim::Int)),
            ])
        );

        // Rejections: the slicer's bounds diagnostic (RAW bounds, verbatim
        // texts — NO space after "are") and the ulong_val narrowings fire
        // BEFORE the no-value gate, as do subspace_normal's size checks
        // (dim first; no spaces around ">"). mod2_section has NO rejected
        // cases upstream. A for-loop body runs at no-value.
        for (source, expected) in [
            (
                "swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 3, 0, 2)",
                "Range exceeds bounds: upper row bound 3 out of range, actual limits are2, 2",
            ),
            (
                "swiss_matrix_knife(0, mat: [[1,2],[3,4]], 5, 1, 0, 1)",
                "Range exceeds bounds: lower row bound 5 out of range, actual limits are2, 2",
            ),
            (
                "swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 1, 0, 5)",
                "Range exceeds bounds: upper column bound 5 out of range, actual limits are2, 2",
            ),
            (
                "swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 1, 5, 1)",
                "Range exceeds bounds: lower column bound 5 out of range, actual limits are2, 2",
            ),
            (
                "swiss_matrix_knife(0, mat: [[1,2],[3,4]], 5, 9, 3, 7)",
                "Range exceeds bounds: both row bounds 5,9 and both column bounds 3,7 out of range, actual limits are2, 2",
            ),
            (
                "swiss_matrix_knife(0, mat: [[1,2],[3,4]], 5, 9, 0, 7)",
                "Range exceeds bounds: both row bounds 5,9 and upper column bound 7 out of range, actual limits are2, 2",
            ),
            (
                "swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 9, 1, 8)",
                "Range exceeds bounds: upper row bound 9 and upper column bound 8 out of range, actual limits are2, 2",
            ),
            (
                "swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0-1, 2, 0, 2)",
                "Negative integer where unsigned is required",
            ),
            (
                "swiss_matrix_knife(0, mat: [[1]], 0, 99999999999999999999999, 0, 1)",
                "Integer value to big for conversion",
            ),
            (
                "for i:2 do swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 5, 0, 2) od",
                "Range exceeds bounds: upper row bound 5 out of range, actual limits are2, 2",
            ),
            ("subspace_normal(null(65,1))", "Dimension too large: 65>64"),
            ("subspace_normal(null(1,65))", "Too many generators: 65>64"),
            ("subspace_normal(null(65,65))", "Dimension too large: 65>64"),
            (
                "for i:2 do subspace_normal(null(65,1)) od",
                "Dimension too large: 65>64",
            ),
        ] {
            match convert_and_run(source) {
                Err(error) => assert_eq!(error.message, expected, "source: {source}"),
                Ok(value) => panic!("{source} unexpectedly succeeded with {value:?}"),
            }
        }
    }

    #[test]
    fn two_dimensional_slice_and_commabarlist_match_the_oracle() {
        // Two-dimensional slice `M[rlo:rhi, clo:chi]` (parser.y:660-705):
        // the parser packs the four from-end bits (rows 0x2/0x4, columns
        // 0x10/0x20; an absent upper bound sets its bit so the zero-filled
        // bound reads as upb=dim) and the evaluator drives the
        // matreduc::swiss_matrix_knife engine directly — the hidden
        // "matrix slicer" name is never registered. M below is
        // mat: [[1,2,3],[4,5,6]] — the 3x2 matrix | 1, 4 | / | 2, 5 | /
        // | 3, 6 | (mat literals are COLUMNS).
        for (source, expected) in [
            (
                "(mat: [[1,2,3],[4,5,6]])[0:3, 0:2]",
                "\n| 1, 4 |\n| 2, 5 |\n| 3, 6 |\n",
            ),
            (
                "(mat: [[1,2,3],[4,5,6]])[1:, 0:2]",
                "\n| 2, 5 |\n| 3, 6 |\n",
            ),
            ("(mat: [[1,2,3],[4,5,6]])[:, 1:]", "\n| 4 |\n| 5 |\n| 6 |\n"),
            // from-end bounds: `1~` lowers to rows-1, `1~`/`2~` uppers.
            ("(mat: [[1,2,3],[4,5,6]])[1~:3, 0:2]", "\n| 3, 6 |\n"),
            (
                "(mat: [[1,2,3],[4,5,6]])[0:1~, 0:2]",
                "\n| 1, 4 |\n| 2, 5 |\n",
            ),
            ("(mat: [[1,2,3],[4,5,6]])[0:2~, 1~:2~]", "The 1x0 matrix"),
            // inverted ranges clamp to empty, keeping the shape.
            ("(mat: [[1,2,3],[4,5,6]])[2:1, 0:2]", "The 0x2 matrix"),
            ("(mat: [[1,2,3],[4,5,6]])[0:2, 1:1]", "The 2x0 matrix"),
            ("(mat: [[1,2,3],[4,5,6]])[2:0, 1:1]", "The 0x0 matrix"),
            // The base converts against `mat`: a row-of-rows DISPLAY
            // coerces (columns-first), so this is the first column of
            // [[1,3],[2,4]].
            ("[[1,2],[3,4]][0:2, 0:1]", "\n| 1 |\n| 2 |\n"),
        ] {
            let (found, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(found, Type::Primitive(Prim::Mat), "source: {source}");
            assert_eq!(value.to_string(), expected, "source: {source}");
        }

        // commabarlist `[a,b | c,d]` (parser.y:370-376, :402-410): segments
        // become the ROWS of the result (upstream: `transpose (mat: …)` via
        // the hidden "transpose "; here the matrix is built directly, so a
        // user `^`(mat) overload cannot intercept it — pinned by fixture
        // eval/commabarlist.atlas).
        for (source, expected) in [
            ("[1,2 | 3,4]", "\n| 1, 2 |\n| 3, 4 |\n"),
            ("[1,2,3 | 4,5,6]", "\n| 1, 2, 3 |\n| 4, 5, 6 |\n"),
            ("[1 | 2 | 3]", "\n| 1 |\n| 2 |\n| 3 |\n"),
            // `;` inside a segment stays statement sequencing.
            ("[1,2|3,4; 5]", "\n| 1, 2 |\n| 3, 5 |\n"),
            ("[0-1, 2 | 3, 0-4]", "\n| -1,  2 |\n|  3, -4 |\n"),
            // a commabarlist is a mat, so it slices as one.
            ("[1,2 | 3,4][0:1, 0:2]", "\n| 1, 2 |\n"),
        ] {
            let (found, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(found, Type::Primitive(Prim::Mat), "source: {source}");
            assert_eq!(value.to_string(), expected, "source: {source}");
        }

        // Rejections: the slicer's RAW-bounds diagnostic and the ulong
        // narrowings fire before the no-value gate (a for-loop body runs at
        // no-value); a non-mat base is the explicit-cast coercion error; a
        // mistyped bound rejects the whole desugared argument tuple
        // upstream, wording replicated here. Commabarlist entries convert
        // against int; the rectangularity check is a runtime error.
        for (source, expected) in [
            (
                "(mat: [[1,2,3],[4,5,6]])[0:9, 0:2]",
                "Range exceeds bounds: upper row bound 9 out of range, actual limits are3, 2",
            ),
            (
                "(mat: [[1,2,3],[4,5,6]])[0:9, 1:8]",
                "Range exceeds bounds: upper row bound 9 and upper column bound 8 out of range, actual limits are3, 2",
            ),
            (
                "(mat: [[1,2,3],[4,5,6]])[0-1:1, 0:1]",
                "Negative integer where unsigned is required",
            ),
            (
                "for i:2 do (mat: [[1,2,3],[4,5,6]])[0:9, 0:2] od",
                "Range exceeds bounds: upper row bound 9 out of range, actual limits are3, 2",
            ),
            // A row display in base position coerces componentwise to vec:
            // its int entries fail there, not against mat.
            ("[1,2,3][0:1, 0:1]", "found int while vec was needed."),
            (
                "(mat: [[1,2,3],[4,5,6]])[0:\"a\", 0:1]",
                "found (int,mat,int,string,int,int) while (int,mat,int,int,int,int) was needed.",
            ),
            ("\"a\"[0:1, 0:1]", "found string while mat was needed."),
            ("[1,2 | 3]", "Vector sizes differ in conversion to matrix"),
            ("[1, \"a\" | 3, 4]", "found string while int was needed."),
            ("[1,2 | 3/4, 5]", "found rat while int was needed."),
            ("[1,2 | vec: [3,4]]", "found vec while int was needed."),
            (
                "[99999999999999999999999 | 3]",
                "Integer value to big for conversion",
            ),
            (
                "for i:2 do [1,2 | 3] od",
                "Vector sizes differ in conversion to matrix",
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
        let last_value = Value::Tuple(Vec::new());
        let last_type: TypeCell = Rc::new(RefCell::new(Type::void()));
        let analysis = Analysis::new(&table, &globals, &overloads, &last_value, &last_type);
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
    fn generic_row_operators_resolve_like_the_upstream_special_instances() {
        // axis.w:2549-2595: the generic `#`/`##` row instances are recognised
        // from the a-priori type once exact ordinary overloads fail, and win
        // over coercible ones. Results stay row values and print compactly.
        let int_row = Type::row(int_type());
        for (source, printed) in [
            ("[1,2]##[3,4]", "[1,2,3,4]"),
            ("[]##[]", "[]"),
            ("##([[1,2],[3,4]])", "[1,2,3,4]"),
            ("##([[],[]])", "[]"),
            ("[1,2]#3", "[1,2,3]"),
            ("3#[1,2]", "[3,1,2]"),
            ("[1,2]#3#4", "[1,2,3,4]"),
            ("[\"a\",\"b\"]##[\"c\"]", "[\"a\",\"b\",\"c\"]"),
        ] {
            let (type_, value) = convert_and_run(source).expect(source);
            assert_eq!(value.to_string(), printed, "source: {source}");
            assert!(
                matches!(type_, Type::Row(_)),
                "source {source:?} keeps a row type, found {type_:?}"
            );
        }
        let (type_, value) = convert_and_run("[1,2]##[3,4]").expect("bare row join");
        assert_eq!(type_, int_row);
        assert_eq!(value.to_string(), "[1,2,3,4]");

        // Suffix beats prefix when both rows could serve (axis.w:2533-2541).
        let (type_, value) = convert_and_run("[[2]]#[]").expect("ambiguous suffix");
        assert_eq!(value.to_string(), "[[2],[]]");
        assert_eq!(type_, Type::row(Type::row(int_type())));
        let (_, value) = convert_and_run("[]#[[2]]").expect("suffix of an empty row");
        assert_eq!(value.to_string(), "[[[2]]]");
        let (_, value) = convert_and_run("[]#[]").expect("empty row suffixed by itself");
        assert_eq!(value.to_string(), "[[]]");

        // A `*` row component adopts the element type (axis.w:2524-2531).
        for (source, printed) in [("[]#3", "[3]"), ("3#[]", "[3]"), ("[1]#[]", "[[1]]")] {
            let (_, value) = convert_and_run(source).expect(source);
            assert_eq!(value.to_string(), printed, "source: {source}");
        }
        let (type_, _) = convert_and_run("[]#3").expect("empty row suffix adopts int");
        assert_eq!(type_, int_row);

        // Exact ordinary overloads still preempt the generics (axis.w:1565-1573).
        let (type_, value) = convert_and_run("(vec: [1,2]) # 3").expect("exact vec suffix");
        assert_eq!(type_, primitive_type(Prim::Vec));
        assert!(matches!(value, Value::Vector(_)), "vec suffix stays a vec");
        let (_, value) = convert_and_run("\"a\"##\"b\"").expect("exact string concat");
        assert_eq!(value.to_string(), "\"ab\"");
        let (_, value) = convert_and_run("##([vec: [1], vec: [2]])").expect("exact vec row join");
        assert!(
            matches!(value, Value::Vector(_)),
            "vec row join stays a vec"
        );
    }

    #[test]
    fn generic_row_operators_fail_with_the_oracle_wording() {
        for (source, message) in [
            (
                "[1,2]##\"a\"",
                "Failed to match '##' with argument type ([int],string)",
            ),
            (
                "[1]#[\"a\"]",
                "Failed to match '#' with argument type ([int],[string])",
            ),
            ("1#2", "Failed to match '#' with argument type (int,int)"),
            ("#(1,2)", "Failed to match '#' with argument type (int,int)"),
            (
                "[1,2]##(3,4)",
                "Failed to match '##' with argument type ([int],(int,int))",
            ),
            // Unequal row pairs match nothing: `==` upstream is structural,
            // so `[*]` does not join `[int]` (axis.w:2585-2587).
            (
                "[]##[1,2]",
                "Failed to match '##' with argument type ([*],[int])",
            ),
            (
                "[1,2]##[]",
                "Failed to match '##' with argument type ([int],[*])",
            ),
        ] {
            let error = convert_and_run(source).expect_err(source);
            assert_eq!(error.kind, ErrorKind::Type, "source: {source}");
            assert_eq!(error.message, message, "source: {source}");
        }
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
        let last_value = Value::Tuple(Vec::new());
        let last_type: TypeCell = Rc::new(RefCell::new(Type::void()));
        let analysis = Analysis::new(&table, &globals, &overloads, &last_value, &last_type);
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
            Rc::new(Value::Vector(Vec32(vec![1, 0]))),
            Rc::new(Value::Vector(Vec32(vec![0]))),
        ]);
        let error = apply_conversion("M[V]", vector_columns, span)
            .expect_err("vector columns have unequal sizes");
        assert!(matches!(
            error,
            Control::Runtime(Diagnostic { message, .. })
                if message == "Vector sizes differ in conversion to matrix"
        ));

        let integer_columns = Value::List(vec![
            Rc::new(Value::List(vec![
                Rc::new(Value::Integer(1.into())),
                Rc::new(Value::Integer(0.into())),
            ])),
            Rc::new(Value::List(vec![Rc::new(Value::Integer(0.into()))])),
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
            Value::Tuple(vec![
                Rc::new(Value::Integer(20.into())),
                Rc::new(Value::Integer(22.into()))
            ])
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
    fn loop_tildes_reverse_traversal_and_collection() {
        // The frozen oracle captures for for_reversed (3604471) and
        // for_reversed_extra (3604479): the in-part tilde traverses in
        // reverse (the `@` index counts down from n-1), the body-side tilde
        // reverse-collects the row, and two tildes cancel out.
        for (source, expected) in [
            ("for i in [1,2,3]~ do i od", "[3,2,1]"),
            ("for i in [1,2,3] do i~ od", "[3,2,1]"),
            ("for i in [1,2,3]~ do i~ od", "[1,2,3]"),
            ("for i@k in [1,2,3]~ do (k,i) od", "[(2,3),(1,2),(0,1)]"),
            ("for (a,b) in [(1,2),(3,4)]~ do a+b od", "[7,3]"),
            ("for i:3 from 0 do i~ od", "[2,1,0]"),
            ("for i:3 from 0~ do i od", "[2,1,0]"),
            ("for i:3 from 0~ do i~ od", "[0,1,2]"),
            ("for i:3 do i~ od", "[2,1,0]"),
            ("for i:3~ do i od", "[2,1,0]"),
            ("for i:3 downto 0 do i od", "[2,1,0]"),
            // The anonymous counted form admits only the body-side tilde
            // (parser.y:565-566, flags=2*t+4).
            ("for :3 do 7~ od", "[7,7,7]"),
            // The while tilde reverse-collects the row of body values
            // (parser.y:364, flags=2*t).
            (
                "let i = 0 in while i < 3 do begin i := i + 1; i end~ od",
                "[3,2,1]",
            ),
        ] {
            let (_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(value.to_string(), expected, "source: {source}");
        }

        // A break in a reverse-collecting loop keeps the completed
        // iterations, still in reverse (axis.w:5994-6004 left-alignment).
        let (_, value) = convert_and_run("for i in [1,2,3] do if i = 3 then break fi; i * 10~ od")
            .expect("break in a reverse-collecting loop");
        assert_eq!(value.to_string(), "[20,10]");
    }

    #[test]
    fn for_loop_over_variable_observes_the_entry_snapshot() {
        // A loop over a variable borrows the aggregate (no upfront copy):
        // reassignment or a component write through the variable mid-loop
        // copy-on-writes, so the traversal still sees the entry-time value.
        for (source, expected) in [
            ("let v = [1,2,3] in for x in v do v := [9,9]; x od", "[1,2,3]"),
            ("let v = [1,2,3] in for x in v do v[0] := 9; x od", "[1,2,3]"),
            // The write itself still lands on the variable for after the
            // loop (each iteration overwrites, so the last one sticks).
            (
                "let v = [1,2,3] in begin for x in v do v[0] := x * 10 od; v end",
                "[30,2,3]",
            ),
            // Reversed traversal over a shared iterable, and the `@` index
            // counting down.
            ("let v = [7,8,9] in for x@i in v~ do (i,x) od", "[(2,9),(1,8),(0,7)]"),
            // An early break clones only the visited components.
            ("let v = [1,2,3] in for x in v do if x = 2 then break fi; x od", "[1]"),
        ] {
            let (_, value) = convert_and_run(source)
                .unwrap_or_else(|error| panic!("{source} should convert and run: {error:?}"));
            assert_eq!(value.to_string(), expected, "source: {source}");
        }
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

        let error = convert_and_run("for b in true do b od").expect_err("non-aggregate iterable");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "Cannot iterate over value of type bool");

        let error = convert_and_run("while 1 do 2 od").expect_err("non-boolean condition");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "found int while bool was needed.");
    }

    #[test]
    fn break_levels_match_the_oracle() {
        // `break N` needs N+1 lexically enclosing loops (parser.y:385-386,
        // axis.w:673-685 layer::may_break); the check runs during analysis.
        let error = convert_and_run("for i:2 do break 1 od").expect_err("shallow break 1");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(
            error.message,
            "Using 'break 1' requires 2 nested levels of loops"
        );

        let error = convert_and_run("break 2").expect_err("top-level break 2");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(
            error.message,
            "Using 'break 2' requires 3 nested levels of loops"
        );

        // `break 0` is exactly `break`, including the plain-break message.
        let error = convert_and_run("break 0").expect_err("top-level break 0");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "Using 'break' not in the reach of any loop");

        // With enough depth, `break 2` unwinds all three loops; the
        // already-completed i=0 rows survive, the breaking iterations
        // contribute nothing (eval/break_levels fixture).
        let (_, value) = convert_and_run(
            "for i:2 do for j:2 do for k:2 do begin if i=1 then break 2 fi; (i,j,k) end od od od",
        )
        .expect("break 2 inside three loops");
        assert_eq!(value.to_string(), "[[[(0,0,0),(0,0,1)],[(0,1,0),(0,1,1)]]]");

        // `break 1` unwinds both loops; only the completed i=0 row survives.
        let (_, value) =
            convert_and_run("for i:2 do for j:2 do begin if i=1 then break 1 fi; (i,j) end od od")
                .expect("break 1 inside two loops");
        assert_eq!(value.to_string(), "[[(0,0),(0,1)]]");
    }

    #[test]
    fn prints_builtin_matches_the_oracle() {
        // axis.w:8773, 8819-8853: the variadic generic printer emits string
        // components without quotes, other values like `print`, then one
        // newline; a lone tuple argument prints component-wise.
        let mut context = TypedContext::new();
        for (source, expected) in [
            ("prints((0,0,1))", "001\n"),
            ("prints(\"ab\", 4)", "ab4\n"),
            ("prints(5)", "5\n"),
            ("prints(\"x\", (1,2), [3])", "x(1,2)[3]\n"),
        ] {
            let events = context.execute(&command(source)).expect("prints runs");
            match &events[..] {
                [TypedCommandEvent::ReportLine { text, .. }, TypedCommandEvent::Value { value, type_, .. }] =>
                {
                    assert_eq!(text, expected, "source: {source}");
                    assert_eq!(value, &Value::Tuple(Vec::new()), "source: {source}");
                    assert!(type_.is_void(), "source: {source}");
                }
                other => panic!("unexpected events for {source}: {other:?}"),
            }
        }
    }

    #[test]
    fn print_to_string_error_match_the_oracle() {
        // axis.w:8767-8771, 8796-8859: print displays the argument tuple
        // verbatim and returns it unchanged; to_string yields the stripped
        // concatenation without a newline; error raises it as a runtime
        // error (also with zero arguments).
        let mut context = TypedContext::new();
        let events = context
            .execute(&command("print(\"a\", 1)"))
            .expect("print runs");
        match &events[..] {
            [TypedCommandEvent::ReportLine { text, .. }, TypedCommandEvent::Value { value, .. }] => {
                assert_eq!(text, "(\"a\",1)\n");
                assert_eq!(value.to_string(), "(\"a\",1)");
            }
            other => panic!("unexpected events: {other:?}"),
        }
        let events = context.execute(&command("print(5)")).expect("print runs");
        match &events[..] {
            [TypedCommandEvent::ReportLine { text, .. }, TypedCommandEvent::Value { value, .. }] => {
                assert_eq!(text, "5\n");
                assert_eq!(value, &Value::Integer(5.into()));
            }
            other => panic!("unexpected events: {other:?}"),
        }
        for (source, expected) in [
            ("to_string(42)", "\"42\""),
            ("to_string([1,2], 3, \"x\")", "\"[1,2]3x\""),
            ("to_string()", "\"\""),
            ("to_string(1, \"a\", [2,3], (4,5))", "\"1a[2,3](4,5)\""),
        ] {
            let events = context.execute(&command(source)).expect("to_string runs");
            match &events[..] {
                [TypedCommandEvent::Value { value, .. }] => {
                    assert_eq!(value.to_string(), expected, "source: {source}")
                }
                other => panic!("unexpected events for {source}: {other:?}"),
            }
        }
        let error = context
            .execute(&command("error(\"a\", 1, [2])"))
            .expect_err("error raises");
        assert!(error.to_string().contains("a1[2]"), "error: {error}");
        let error = context
            .execute(&command("error()"))
            .expect_err("zero-argument error raises");
        assert!(matches!(error.kind, ErrorKind::Runtime), "error: {error}");
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

    fn completion_values(context: &mut TypedContext, prefix: &str) -> Vec<String> {
        let events = context
            .execute(&command(&format!("readline_completions(\"{prefix}\")")))
            .expect("readline_completions runs");
        match &events[..] {
            [TypedCommandEvent::Value {
                value: Value::List(names),
                ..
            }] => names
                .iter()
                .map(|name| match name.as_ref() {
                    Value::String(name) => name.clone(),
                    other => panic!("non-string completion {other:?}"),
                })
                .collect(),
            other => panic!("unexpected completion events {other:?}"),
        }
    }

    #[test]
    fn startup_completion_names_cover_the_registry() {
        // buffer.w:1175-1192 with the oracle capture: every registered
        // builtin appears in the startup completion list, and from index
        // 55 on (after the 34 keywords and 21 type names) every startup
        // name is a registered builtin except the two deliberately
        // unregistered hidden copies (registry batch-4 comment).
        assert_eq!(STARTUP_COMPLETION_NAMES.len(), 294, "oracle startup count");
        let startup: BTreeSet<&str> = STARTUP_COMPLETION_NAMES.iter().copied().collect();
        for builtin in builtin_registry() {
            // The variadic specials are special operators upstream
            // (axis.w:1798-1816, 2504), never installed into the overload
            // table, so the oracle's startup completions contain none of
            // them (probes: `readline_completions("print")` lists the
            // print_* domain printers but no `print`/`prints`;
            // `readline_completions("to_")` and `("err")` likewise lack
            // `to_string`/`error`).
            if matches!(builtin.name, "print" | "prints" | "to_string" | "error") {
                continue;
            }
            assert!(
                startup.contains(builtin.name) || builtin.name == "prints",
                "registry builtin '{}' missing from STARTUP_COMPLETION_NAMES",
                builtin.name
            );
        }
        let registry: BTreeSet<&str> = builtin_registry()
            .iter()
            .map(|builtin| builtin.name)
            .collect();
        for &name in &STARTUP_COMPLETION_NAMES[55..] {
            assert!(
                registry.contains(name) || name == "transpose " || name == "matrix slicer",
                "startup completion name '{name}' is not a registered builtin"
            );
        }
        // The three system variables are session globals, not static
        // startup names.
        for name in SYSTEM_VARIABLE_NAMES {
            assert!(!startup.contains(name), "'{name}' is not static");
        }
    }

    #[test]
    fn startup_system_variables_are_empty_string_rows() {
        // main.w:408-435: batch mode starts all three as empty [string].
        let mut context = TypedContext::new();
        for name in SYSTEM_VARIABLE_NAMES {
            let events = context.execute(&command(name)).expect("system variable");
            assert!(matches!(
                &events[..],
                [TypedCommandEvent::Value {
                    value: Value::List(items),
                    ..
                }] if items.is_empty()
            ));
            let events = context
                .execute(&command(&format!("whattype {name}")))
                .expect("whattype runs");
            assert!(matches!(
                &events[..],
                [TypedCommandEvent::ReportLine { text, .. }] if text == "Type: [string]\n"
            ));
        }
    }

    /// The current `back_trace` global as plain strings.
    fn back_trace_lines(context: &mut TypedContext) -> Vec<String> {
        let events = context
            .execute(&command("back_trace"))
            .expect("back_trace reads");
        match &events[..] {
            [TypedCommandEvent::Value {
                value: Value::List(items),
                ..
            }] => items
                .iter()
                .map(|item| match item.as_ref() {
                    Value::String(line) => line.clone(),
                    other => panic!("non-string trace line {other:?}"),
                })
                .collect(),
            other => panic!("unexpected back_trace events {other:?}"),
        }
    }

    #[test]
    fn trace_location_renders_upstream_spans() {
        use crate::diagnostic::{SourceId, SourcePosition};
        let mut context = EvaluationContext::new();
        let position = |line, column| SourcePosition { line, column };
        // Single line: 1-based line, 0-based columns, end exclusive
        // (parsetree.w:173-180); the stdin buffer is `<standard input>`.
        let span = SourceSpan::new(SourceId::new(7), 0, 4, position(3, 1), position(3, 5));
        assert_eq!(trace_location(&context, &span), "at <standard input>:3:0-4");
        // Across lines: `LINE:COL--ENDLINE:ENDCOL` (the doubled dash is
        // upstream's unconditional `-` plus the multi-line `-EL:`).
        let span = SourceSpan::new(SourceId::new(7), 0, 10, position(3, 2), position(5, 4));
        assert_eq!(
            trace_location(&context, &span),
            "at <standard input>:3:1--5:3"
        );
        // A registered buffer prints its recorded name (buffer.w:694).
        context.note_source_name(SourceId::new(7), "lib.at".to_owned());
        let span = SourceSpan::new(SourceId::new(7), 0, 4, position(3, 1), position(3, 5));
        assert_eq!(trace_location(&context, &span), "at lib.at:3:0-4");
    }

    #[test]
    fn runtime_error_populates_back_trace() {
        // The fixture chain: a call boundary line (outermost first), then
        // the frame dump, down to the failing builtin (axis.w:2253-2265,
        // 2896-2909).
        let mut context = TypedContext::new();
        context
            .execute(&command("set f(int x)=x%0"))
            .expect("define f");
        context
            .execute(&command("set g(int x)=f(x)+1"))
            .expect("define g");
        let error = context.execute(&command("g(2)")).expect_err("g(2) fails");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(
            error.back_trace,
            vec![
                "In call of g@int at <standard input>:1:0-4, defined at <standard input>:1:4-19.",
                "{ x=2 }",
                "In call of f@int at <standard input>:1:13-17, defined at <standard input>:1:4-16.",
                "{ x=2 }",
                "In call of %@(int,int) at <standard input>:1:13-16, built-in.",
            ]
        );
        assert_eq!(back_trace_lines(&mut context), error.back_trace);

        // A failing top-level builtin call replaces the trace.
        context.execute(&command("1%0")).expect_err("1%0 fails");
        assert_eq!(
            back_trace_lines(&mut context),
            vec!["In call of %@(int,int) at <standard input>:1:0-3, built-in."]
        );

        // A multi-parameter frame dumps every slot in bind order.
        context
            .execute(&command("set h(int a, string s)=a%#s"))
            .expect("define h");
        context.execute(&command("h(3,\"\")")).expect_err("h fails");
        assert_eq!(
            back_trace_lines(&mut context),
            vec![
                "In call of h@(int,string) at <standard input>:1:0-7, defined at <standard input>:1:4-27.",
                "{ a=3, s=\"\" }",
                "In call of %@(int,int) at <standard input>:1:23-27, built-in.",
            ]
        );

        // A parameterless closure traces no frame dump.
        context.execute(&command("set z()=7%0")).expect("define z");
        context.execute(&command("z()")).expect_err("z fails");
        assert_eq!(
            back_trace_lines(&mut context),
            vec![
                "In call of z@void at <standard input>:1:0-3, defined at <standard input>:1:4-11.",
                "In call of %@(int,int) at <standard input>:1:8-11, built-in.",
            ]
        );
    }

    #[test]
    fn back_trace_is_sticky_when_the_trace_is_empty() {
        // set_back_trace no-ops on an empty trace (global.w:1137): `die`
        // crosses no call boundary, so the previous trace survives.
        let mut context = TypedContext::new();
        context.execute(&command("1%0")).expect_err("1%0 fails");
        let trace = back_trace_lines(&mut context);
        context.execute(&command("die")).expect_err("die fails");
        assert_eq!(back_trace_lines(&mut context), trace);

        // A failing `set` initializer also stores the trace
        // (global.w:1127).
        context
            .execute(&command("set r = 1%0"))
            .expect_err("set fails");
        assert_eq!(
            back_trace_lines(&mut context),
            vec!["In call of %@(int,int) at <standard input>:1:8-11, built-in."]
        );
    }

    #[test]
    fn for_loop_error_traces_iteration_and_frame() {
        // axis.w:6124-6132: the iteration line, then the loop-variable
        // frame dump, then the inner call lines.
        let mut context = TypedContext::new();
        context
            .execute(&command("for i in [2,1,0] do 6%i od"))
            .expect_err("loop fails");
        assert_eq!(
            back_trace_lines(&mut context),
            vec![
                "During iteration 2 of the for-loop",
                "{ i=0 }",
                "In call of %@(int,int) at <standard input>:1:20-23, built-in.",
            ]
        );
    }

    #[test]
    fn counted_for_loop_error_traces_the_iteration() {
        // axis.w:6685-6698: a named counted loop reports the iteration
        // count and the counter value by name, with no frame dump;
        // `downto` is the counted REVERSED loop.
        let mut context = TypedContext::new();
        context
            .execute(&command("for i:3 from 0 do 6%(2-i) od"))
            .expect_err("loop fails");
        assert_eq!(
            back_trace_lines(&mut context),
            vec![
                "During iteration 2 (i=2) of the counted for-loop",
                "In call of %@(int,int) at <standard input>:1:18-24, built-in.",
            ]
        );

        context
            .execute(&command("for i:3 downto 0 do 6%i od"))
            .expect_err("loop fails");
        assert_eq!(
            back_trace_lines(&mut context),
            vec![
                "During iteration 2 (i=0) of the counted reversed for-loop",
                "In call of %@(int,int) at <standard input>:1:20-23, built-in.",
            ]
        );
    }

    #[test]
    fn anonymous_counted_for_loop_error_keeps_the_shared_format() {
        // axis.w:6587-6594: the anonymous catch shares one format string,
        // so no counter value and a double space before `counted` survive.
        let mut context = TypedContext::new();
        context
            .execute(&command("for :2 do 1%0 od"))
            .expect_err("loop fails");
        assert_eq!(
            back_trace_lines(&mut context),
            vec![
                "During iteration 0 of the  counted for-loop",
                "In call of %@(int,int) at <standard input>:1:10-13, built-in.",
            ]
        );
    }

    #[test]
    fn frame_dump_reads_the_current_slot_values() {
        // The catch reads the frame at throw time (axis.w:2896-2909), so a
        // reassigned parameter prints its value at the moment of failure.
        let mut context = TypedContext::new();
        context
            .execute(&command("set m(int x)=(x:=1; x%0)"))
            .expect("define m");
        context.execute(&command("m(5)")).expect_err("m fails");
        let trace = back_trace_lines(&mut context);
        assert_eq!(trace[1], "{ x=1 }", "trace: {trace:?}");
    }

    #[test]
    fn let_frame_dumps_its_bindings() {
        // axis.w:2882-2909: a let frame unwound by an error dumps its own
        // bindings between the call frame and the failing builtin.
        let mut context = TypedContext::new();
        context
            .execute(&command("set lf(int x)=let y=x+1 in y%0"))
            .expect("define lf");
        context.execute(&command("lf(3)")).expect_err("lf fails");
        assert_eq!(
            back_trace_lines(&mut context),
            vec![
                "In call of lf@int at <standard input>:1:0-5, defined at <standard input>:1:4-30.",
                "{ x=3 }",
                "{ y=4 }",
                "In call of %@(int,int) at <standard input>:1:27-30, built-in.",
            ]
        );

        // Non-integer values print with the standard value printer
        // (a vec as `[ 3 ]`).
        context
            .execute(&command("set e(int x)=let v=vec:[x+2] in x%0"))
            .expect("define e");
        context.execute(&command("e(1)")).expect_err("e fails");
        let trace = back_trace_lines(&mut context);
        assert_eq!(trace[2], "{ v=[ 3 ] }", "trace: {trace:?}");
    }

    #[test]
    fn closure_values_print_multi_line_in_frame_dumps() {
        // A recursive closure's self slot prints `closure_value::print` for
        // the recursive kind (axis.w:3265-3271): the definition location,
        // then `name = (params): ` and the converted body, whose
        // conditional carries its leading and trailing spaces
        // (axis.w:4754-4766) and whose `n-1` prints as the upstream
        // `pred@int(n)` rewrite (global.w:2977-2983).
        let mut context = TypedContext::new();
        context
            .execute(&command(
                "set bomb = rec_fun b(int n) int: if n=0 then 1%0 else b(n-1) fi",
            ))
            .expect("define bomb");
        context
            .execute(&command("bomb(1)"))
            .expect_err("bomb fails");
        let dump = |n: i64| {
            format!(
                "{{ b=Recursive function defined at <standard input>:1:11-63\nb = (n):  if =@(int,int)(n,0) then %@(int,int)(1,0) else b(pred@int(n)) fi , n={n} }}"
            )
        };
        assert_eq!(
            back_trace_lines(&mut context),
            vec![
                "In call of bomb@int at <standard input>:1:0-7, defined at <standard input>:1:11-63.".to_string(),
                dump(1),
                "In call of b at <standard input>:1:54-60, defined at <standard input>:1:11-63.".to_string(),
                dump(0),
                "In call of %@(int,int) at <standard input>:1:45-48, built-in.".to_string(),
            ]
        );
    }

    #[test]
    fn dynamic_call_trace_line_names_the_definition_site() {
        // A call through a variable prints the callee expression with no
        // `@type` suffix and takes `defined` from the closure's lambda
        // location (axis.w:1911-1913, 3273-3274); a let-bound closure in a
        // frame dump prints the non-recursive multi-line form
        // (axis.w:3260-3263).
        let mut context = TypedContext::new();
        context
            .execute(&command("set h(int x)=let g(int y)=y%0 in g(x)"))
            .expect("define h");
        context.execute(&command("h(5)")).expect_err("h fails");
        assert_eq!(
            back_trace_lines(&mut context),
            vec![
                "In call of h@int at <standard input>:1:0-4, defined at <standard input>:1:4-37.",
                "{ x=5 }",
                "{ g=Function defined at <standard input>:1:17-29\n(y): %@(int,int)(y,0) }",
                "In call of g at <standard input>:1:33-37, defined at <standard input>:1:17-29.",
                "{ y=5 }",
                "In call of %@(int,int) at <standard input>:1:26-29, built-in.",
            ]
        );
    }

    #[test]
    fn readline_completions_tracks_the_session_state() {
        let mut context = TypedContext::new();
        // Fresh session: the 294 startup names plus the three system
        // variables (main.w:408-435), in that order.
        let all = completion_values(&mut context, "");
        assert_eq!(all.len(), 297);
        assert_eq!(&all[..3], &["quit", "set", "let"]);
        assert_eq!(&all[294..], &["input_path", "prelude_log", "back_trace"]);

        // Prefix filtering keeps the startup order; unknown prefixes
        // complete to nothing.
        let print_names = completion_values(&mut context, "pri");
        assert_eq!(print_names.len(), 19);
        assert_eq!(print_names[0], "print_block");
        assert_eq!(print_names[18], "print_W_graph");
        assert!(completion_values(&mut context, "zzz").is_empty());

        // Session definitions append in definition order.
        context
            .execute(&command("set myvar=3"))
            .expect("define myvar");
        context
            .execute(&command("set zfun(int x)=x+1"))
            .expect("define zfun");
        context
            .execute(&command("set apple=\"hi\""))
            .expect("define apple");
        let all = completion_values(&mut context, "");
        assert_eq!(&all[297..], &["myvar", "zfun", "apple"]);
        assert_eq!(completion_values(&mut context, "z"), &["zfun"]);
        assert_eq!(completion_values(&mut context, "my"), &["myvar"]);
        assert_eq!(completion_values(&mut context, "app"), &["apple"]);

        // `forget` drops the name; redefining revives it at its ORIGINAL
        // position (upstream hash codes are never recycled).
        context
            .execute(&command("forget myvar"))
            .expect("forget myvar");
        assert!(completion_values(&mut context, "my").is_empty());
        context
            .execute(&command("set myvar=7"))
            .expect("redefine myvar");
        let all = completion_values(&mut context, "");
        assert_eq!(&all[297..], &["myvar", "zfun", "apple"]);
        assert_eq!(completion_values(&mut context, "my"), &["myvar"]);
    }

    #[test]
    fn overriding_a_constant_reports_the_constant_suffix() {
        // global.w:911-994: the override report notes a constant previous
        // binding with a ` (constant)` suffix after its type.
        let mut context = TypedContext::new();
        context.execute(&command("set !c=1")).expect("const define");
        let events = context.execute(&command("set c=2")).expect("override");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::ReportLine { text, .. }]
                if text == "Variable c: int (overriding previous instance, which had type int (constant))\n"
        ));
    }

    fn last_value_display(context: &mut TypedContext) -> String {
        let events = context.execute(&command("$")).expect("$ evaluates");
        match &events[..] {
            [TypedCommandEvent::Value { value, .. }] => value.to_string(),
            other => panic!("unexpected $ events {other:?}"),
        }
    }

    #[test]
    fn operator_casts_capture_overload_instances() {
        // op_cast.atlas (capture 3604640): `name @ type` captures the
        // exactly matching overload instance as a function value.
        let mut context = TypedContext::new();
        context
            .execute(&command("set f=%@(int,int)"))
            .expect("define f");
        let events = context.execute(&command("f(7,3)")).expect("f(7,3)");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::Value { value, .. }] if value.to_string() == "1"
        ));
        // A builtin cast value prints `{name@argtype}` (the registered
        // argument type; the generic prints instance shows `T`).
        for (source, expected) in [
            ("-@int", "{-@int}"),
            ("#@vec", "{#@vec}"),
            ("prints@string", "{prints@T}"),
        ] {
            let events = context.execute(&command(source)).expect(source);
            assert!(
                matches!(
                    &events[..],
                    [TypedCommandEvent::Value { value, .. }] if value.to_string() == expected
                ),
                "source: {source}"
            );
        }
        // A user `set` variant casts to its closure, callable in place.
        context
            .execute(&command("set u(int x)=2*x"))
            .expect("define u");
        let events = context.execute(&command("(u@int)(3)")).expect("(u@int)(3)");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::Value { value, .. }] if value.to_string() == "6"
        ));
        // No exact instance (unknown name, wrong argument type, or an
        // arity the operator has no overload for) is the oracle wording.
        for (source, message) in [
            ("mod@(int,int)", "No instance for mod@(int,int) found"),
            ("u@string", "No instance for u@string found"),
            ("+@int", "No instance for +@int found"),
        ] {
            let error = context.execute(&command(source)).expect_err(source);
            assert_eq!(error.kind, ErrorKind::Type);
            assert_eq!(error.message, message, "source: {source}");
        }
    }

    #[test]
    fn operator_casts_select_generic_special_instances() {
        let mut context = TypedContext::new();
        for (source, expected) in [
            ("print@int", "{print@T}"),
            ("prints@int", "{prints@T}"),
            ("to_string@int", "{to_string@T}"),
            ("error@int", "{error@T}"),
            ("##@([int],[int])", "{##@([T],[T])}"),
            ("##@[[int]]", "{##@([[T]])}"),
            ("#@[int]", "{#@[T]}"),
        ] {
            let events = context.execute(&command(source)).expect(source);
            assert!(
                matches!(
                    &events[..],
                    [TypedCommandEvent::Value { value, .. }] if value.to_string() == expected
                ),
                "source: {source}"
            );
        }
        for source in ["#@([*],int)", "#@([int],[*])", "#@([*],[*])"] {
            let error = context.execute(&command(source)).expect_err(source);
            assert_eq!(error.kind, ErrorKind::Type, "source: {source}");
            assert_eq!(
                error.message,
                format!("No instance for {source} found"),
                "source: {source}"
            );
        }
    }

    #[test]
    fn dot_label_discrimination_matches_and_rejects_like_the_oracle() {
        // case_dot_label.atlas (capture 3604622): the dot-label form
        // evaluates like `tag(pattern):`, an unknown label and a repeated
        // label carry the oracle's wording (category type).
        let mut context = TypedContext::new();
        context
            .execute(&command("set_type [ mvv = ( void no_vec | vec solution) ]"))
            .expect("define mvv");
        for (source, expected) in [
            ("case solution([4,5]) | (v).solution: #v | else 0 esac", "2"),
            ("case solution([4,5]) | v.solution: #v | else 0 esac", "2"),
            ("case no_vec() | (v).solution: #v | else 0 esac", "0"),
        ] {
            let events = context.execute(&command(source)).expect(source);
            assert!(matches!(
                &events[..],
                [TypedCommandEvent::Value { value, .. }] if value.to_string() == expected
            ));
        }
        let error = context
            .execute(&command(
                "case solution([4,5]) | (x).bogus: 1 | else 0 esac",
            ))
            .expect_err("unknown label");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(
            error.message,
            "Branch has label bogus not associated to any variant of the union type mvv"
        );
        let error = context
            .execute(&command(
                "case solution([4,5]) | (v).solution: 1 | solution(w): 2 | else 0 esac",
            ))
            .expect_err("duplicate label");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "Multiple branches with label solution");
    }

    #[test]
    fn dollar_captures_the_last_value_and_stays_sticky() {
        // axis.w:573-624 (`last_value_computed`): `$` snapshots the last
        // non-void top-level value at analysis time. Frozen by
        // last_value.atlas (capture 3604641).
        let mut context = TypedContext::new();
        // Before any value exists, `$` is the void empty tuple: the
        // command type-checks and yields no printable value.
        let events = context.execute(&command("$")).expect("initial $");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::Value { value: Value::Tuple(elements), type_, .. }]
                if elements.is_empty() && type_.is_void()
        ));
        context.execute(&command("5+5")).expect("5+5");
        assert_eq!(last_value_display(&mut context), "10");
        context.execute(&command("$+1")).expect("$+1");
        assert_eq!(last_value_display(&mut context), "11");
        // A runtime failure leaves the previous value in place.
        context.execute(&command("1%0")).expect_err("modulo zero");
        assert_eq!(last_value_display(&mut context), "11");
        // A type failure does too.
        context
            .execute(&command("1+\"s\""))
            .expect_err("type error");
        assert_eq!(last_value_display(&mut context), "11");
        // A void evaluation (`prints`) is not a value update.
        context.execute(&command("prints(\"x\")")).expect("prints");
        assert_eq!(last_value_display(&mut context), "11");
    }

    #[test]
    fn dollar_inside_a_function_captures_at_definition_time() {
        // axis.w:612-616: the capture happens at analysis time, so a
        // defined function does NOT track later `$` updates.
        let mut context = TypedContext::new();
        context.execute(&command("7")).expect("seed $");
        context
            .execute(&command("set f(int x)=x+$"))
            .expect("define f");
        context.execute(&command("100")).expect("move $ on");
        let events = context.execute(&command("f(1)")).expect("f(1)");
        assert!(matches!(
            &events[..],
            [TypedCommandEvent::Value { value, .. }] if value.to_string() == "8"
        ));
    }
}
