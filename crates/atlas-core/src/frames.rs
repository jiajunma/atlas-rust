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
use std::rc::Rc;

use crate::value::Value;

/// A shared runtime value (upstream `shared_value`).
pub type SharedValue = Rc<Value>;

/// One layer of bindings; closures may share tails of the chain.
pub struct Frame {
    next: Option<Rc<Frame>>,
    slots: RefCell<Vec<SharedValue>>,
}

/// The evaluation context: the current head of the frame chain. Empty
/// binding layers get NO frame (the analyser skips them when computing
/// depths), so every frame here holds at least one slot.
#[derive(Default)]
pub struct EvaluationContext {
    current: Option<Rc<Frame>>,
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self::default()
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
        debug_assert!(!slots.is_empty(), "empty layers get no frame");
        let saved = self.current.take();
        self.current = Some(Rc::new(Frame {
            next: saved.clone(),
            slots: RefCell::new(slots),
        }));
        let result = body(self);
        self.current = saved;
        result
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
        slots.get(offset).cloned()
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
                *slot = value;
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
}
