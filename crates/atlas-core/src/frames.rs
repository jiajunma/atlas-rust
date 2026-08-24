//! Evaluation frames for the typed pipeline (phase B).
//!
//! Ports the upstream shape (axis-types.w:2370-2400, 2830-2848): a heap
//! Rc-linked list of frames whose slots closures keep alive, locals
//! addressed as `(depth, offset)` fixed at analysis, and globals as shared
//! cells captured at analysis time. Slot mutation goes through a `RefCell`
//! per frame (local assignment writes through the shared chain). Where
//! upstream relies on C++ RAII to restore the current context across
//! exceptions, this port routes control flow through `Result` values and
//! restores in scope functions (`with_frame` / `with_context`), which run
//! on every non-panicking exit — including `?` propagation of breaks,
//! returns, and runtime errors.
//!
//! Borrow discipline (design invariants): reads CLONE the slot under a
//! short borrow; writes take a short borrow after the right-hand side has
//! fully evaluated; no borrow is ever held across a nested evaluation.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::diagnostic::SourceId;
use crate::value::Value;

/// A shared runtime value (upstream `shared_value`).
pub type SharedValue = Rc<Value>;

/// One layer of bindings; closures may share tails of the chain.
pub struct Frame {
    next: Option<Rc<Frame>>,
    slots: RefCell<Vec<Option<SharedValue>>>,
}

impl Frame {
    /// Snapshot of the slot values for the error-time local-variable trace
    /// (axis.w:2896-2909): read under a short borrow, after unwinding, so a
    /// slot reassigned before the error prints its CURRENT value.
    pub fn slot_snapshot(&self) -> Vec<Option<SharedValue>> {
        self.slots.borrow().clone()
    }
}

/// The evaluation context: the current head of the frame chain. Empty
/// binding layers get NO frame (the analyser skips them when computing
/// depths), so every frame here holds at least one slot.
#[derive(Default)]
pub struct EvaluationContext {
    current: Option<Rc<Frame>>,
    /// Text produced by printer builtins (upstream writes to
    /// `*output_stream` mid-evaluation); the command layer drains it into
    /// report events after each top-level evaluation.
    printed: Vec<String>,
    /// Names the `readline_completions` builtin completes from
    /// (buffer.w:1175-1192). The command layer refreshes this snapshot at
    /// each command boundary, so a call sees post-previous-command state.
    completion_candidates: Vec<String>,
    /// Display names of source buffers for back-trace locations
    /// (buffer.w:694): the top-level stream is `<standard input>`, include
    /// files their resolved path. The session frame records each buffer as
    /// it registers it; unknown ids fall back to `<standard input>`.
    source_names: BTreeMap<u64, String>,
    /// Upstream's global `while_condition_result` (axis.w:5553-5580): a
    /// do_expr sets it AFTER its body ran (so a nested loop cannot clobber
    /// it), and a while loop reads it after each body evaluation —
    /// `false` ends the loop without collecting that iteration's value.
    while_condition_result: std::cell::Cell<bool>,
}

impl EvaluationContext {
    /// Set the while-condition flag (do_expr/dont evaluation).
    pub fn set_while_condition_result(&self, value: bool) {
        self.while_condition_result.set(value);
    }

    /// Read the while-condition flag (while loop, after each body
    /// evaluation).
    pub fn while_condition_result(&self) -> bool {
        self.while_condition_result.get()
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Append one printer builtin's output (upstream's unconditional
    /// `*output_stream` writes, e.g. atlas-types.w:8944-8957).
    pub fn print_text(&mut self, text: String) {
        self.printed.push(text);
    }

    /// Drain the buffered printer output in production order.
    pub fn take_printed(&mut self) -> Vec<String> {
        std::mem::take(&mut self.printed)
    }

    /// Direct buffer access for domain builtins that both print and throw
    /// mid-evaluation (ext_kl.cpp:945-948 prints `Delta does not fix
    /// gamma=...` before raising `No valid extended block`).
    pub fn printed_buffer(&mut self) -> &mut Vec<String> {
        &mut self.printed
    }

    /// Replace the completion candidate snapshot (command layer, once per
    /// command).
    pub fn set_completion_candidates(&mut self, candidates: Vec<String>) {
        self.completion_candidates = candidates;
    }

    /// Append one freshly defined live name to the snapshot (the
    /// append-only fast path; order matches the completion order).
    pub fn push_completion_candidate(&mut self, name: String) {
        self.completion_candidates.push(name);
    }

    /// The current completion candidate snapshot, in upstream hash order.
    pub fn completion_candidates(&self) -> &[String] {
        &self.completion_candidates
    }

    /// Record a source buffer's trace display name (session frame, once per
    /// registered buffer).
    pub fn note_source_name(&mut self, id: SourceId, name: String) {
        self.source_names.insert(id.get(), name);
    }

    /// The trace display name of a source buffer (buffer.w:694): buffers
    /// the session frame did not name print as `<standard input>`.
    pub fn source_name(&self, id: SourceId) -> &str {
        self.source_names
            .get(&id.get())
            .map(String::as_str)
            .unwrap_or("<standard input>")
    }

    /// The current chain head, for capture into a closure value.
    pub fn capture(&self) -> Option<Rc<Frame>> {
        self.current.clone()
    }

    /// Run `body` with a fresh frame of `slots` pushed; the previous chain
    /// is restored on every non-panicking exit.
    pub fn with_frame<R>(
        &mut self,
        slots: Vec<SharedValue>,
        body: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.with_frame_traced(slots, body).0
    }

    /// Like [`Self::with_frame`], but also hands back the pushed frame so an
    /// error unwinding through the call can dump its slots for the
    /// local-variable back-trace line (axis.w:2896-2909).
    pub fn with_frame_traced<R>(
        &mut self,
        slots: Vec<SharedValue>,
        body: impl FnOnce(&mut Self) -> R,
    ) -> (R, Rc<Frame>) {
        debug_assert!(!slots.is_empty(), "empty layers get no frame");
        let saved = self.current.take();
        let frame = Rc::new(Frame {
            next: saved.clone(),
            slots: RefCell::new(slots.into_iter().map(Some).collect()),
        });
        self.current = Some(frame.clone());
        let result = body(self);
        self.current = saved;
        (result, frame)
    }

    /// Run `body` with the context swapped to a closure's captured chain
    /// (upstream closure apply); the caller's chain is restored on every
    /// non-panicking exit.
    pub fn with_context<R>(
        &mut self,
        captured: Option<Rc<Frame>>,
        body: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let saved = std::mem::replace(&mut self.current, captured);
        let result = body(self);
        self.current = saved;
        result
    }

    fn frame_at(&self, depth: usize) -> Option<&Rc<Frame>> {
        let mut frame = self.current.as_ref()?;
        for _ in 0..depth {
            frame = frame.next.as_ref()?;
        }
        Some(frame)
    }

    /// Read the local at `(depth, offset)`, cloning the shared value out
    /// under a short borrow.
    pub fn local(&self, depth: usize, offset: usize) -> Option<SharedValue> {
        let frame = self.frame_at(depth)?;
        let slots = frame.slots.borrow();
        slots.get(offset).and_then(Clone::clone)
    }

    /// Move the local at `(depth, offset)` out of its slot, leaving that
    /// variable uninitialized until a later assignment. This is the safe
    /// Rust counterpart of upstream's pilfering local identifier.
    pub fn take_local(&self, depth: usize, offset: usize) -> Option<SharedValue> {
        let frame = self.frame_at(depth)?;
        let mut slots = frame.slots.borrow_mut();
        slots.get_mut(offset)?.take()
    }

    /// Write the local at `(depth, offset)`; call only after the value has
    /// fully evaluated. Returns whether the slot existed.
    pub fn set_local(&self, depth: usize, offset: usize, value: SharedValue) -> bool {
        let Some(frame) = self.frame_at(depth) else {
            return false;
        };
        let mut slots = frame.slots.borrow_mut();
        match slots.get_mut(offset) {
            Some(slot) => {
                *slot = Some(value);
                true
            }
            None => false,
        }
    }
}

/// A global's storage cell. Every `set`-style definition allocates a FRESH
/// cell unconditionally (converted code keeps the cell it captured at
/// analysis time); only `:=` assignment writes through an existing cell.
/// `None` marks a declared-but-unset global, a runtime error to read.
pub type GlobalCell = Rc<RefCell<Option<SharedValue>>>;

/// A fresh, unset global cell (for `IDENT : type` declarations).
pub fn unset_global() -> GlobalCell {
    Rc::new(RefCell::new(None))
}

/// A fresh global cell holding `value` (for definitions).
pub fn global_with(value: SharedValue) -> GlobalCell {
    Rc::new(RefCell::new(Some(value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared(value: i64) -> SharedValue {
        Rc::new(Value::Integer(value.into()))
    }

    #[test]
    fn locals_read_and_write_through_the_depth_walk() {
        let mut context = EvaluationContext::new();
        let observed = context.with_frame(vec![shared(1), shared(2)], |context| {
            context.with_frame(vec![shared(10)], |context| {
                assert_eq!(context.local(0, 0), Some(shared(10)));
                assert_eq!(context.local(1, 1), Some(shared(2)));
                assert!(context.set_local(1, 0, shared(7)));
                assert!(!context.set_local(0, 5, shared(0)), "bad offset");
                assert!(context.local(2, 0).is_none(), "no such depth");
                context.local(1, 0)
            })
        });
        assert_eq!(observed, Some(shared(7)));
    }

    #[test]
    fn captured_chains_survive_the_pop_and_share_mutation() {
        let mut context = EvaluationContext::new();
        let captured = context.with_frame(vec![shared(1)], |context| context.capture());
        // The frame is popped, but the capture keeps it alive.
        assert!(context.capture().is_none());
        context.with_context(captured.clone(), |context| {
            assert_eq!(context.local(0, 0), Some(shared(1)));
            assert!(context.set_local(0, 0, shared(9)));
        });
        // Mutation through the swapped-in context is visible to any other
        // holder of the same chain (upstream shared-tail semantics).
        context.with_context(captured, |context| {
            assert_eq!(context.local(0, 0), Some(shared(9)));
        });
    }

    #[test]
    fn frames_are_restored_when_the_body_returns_an_error() {
        let mut context = EvaluationContext::new();
        let result: Result<(), &str> = context.with_frame(vec![shared(1)], |context| {
            context.with_frame(vec![shared(2)], |_context| Err("break"))?;
            unreachable!("the error propagates");
        });
        assert_eq!(result, Err("break"));
        // Both frames unwound despite the ? propagation.
        assert!(context.capture().is_none());
    }

    #[test]
    fn global_cells_distinguish_unset_from_set() {
        let unset = unset_global();
        assert!(unset.borrow().is_none());
        let cell = global_with(shared(4));
        assert_eq!(cell.borrow().clone(), Some(shared(4)));
        *cell.borrow_mut() = Some(shared(5));
        assert_eq!(cell.borrow().clone(), Some(shared(5)));
    }

    #[test]
    fn taking_a_local_leaves_the_slot_uninitialized() {
        let mut context = EvaluationContext::new();
        context.with_frame(vec![shared(7)], |context| {
            assert_eq!(context.take_local(0, 0), Some(shared(7)));
            assert_eq!(context.local(0, 0), None);
            assert_eq!(context.take_local(0, 0), None);
            assert_eq!(context.take_local(1, 0), None);
            assert_eq!(context.take_local(0, 1), None);
            assert!(context.set_local(0, 0, shared(9)));
            assert_eq!(context.local(0, 0), Some(shared(9)));
        });
    }
}
