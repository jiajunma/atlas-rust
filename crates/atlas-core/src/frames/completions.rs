//! Session-owned completion order, separate from current name visibility.
//!
//! Original buffer.w walks its intern table only on a query. Keep names once,
//! share their allocation between the index and ordered entries, and lazily
//! provide the existing borrowed-slice API. No global cache or runtime tables
//! are captured here; only command publication changes visibility.

use std::cell::OnceCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Default)]
pub(super) struct CompletionCandidates {
    names: Vec<(Rc<str>, bool)>,
    indices: BTreeMap<Rc<str>, usize>,
    snapshot: OnceCell<Vec<String>>,
}

impl CompletionCandidates {
    pub(super) fn intern(&mut self, name: &str) -> usize {
        if let Some(&index) = self.indices.get(name) {
            return index;
        }
        let index = self.names.len();
        let name: Rc<str> = Rc::from(name);
        self.indices.insert(Rc::clone(&name), index);
        self.names.push((name, false));
        // An inactive/local name does not change an existing snapshot.
        index
    }

    pub(super) fn set_active(&mut self, name: &str, active: bool) {
        let index = self.intern(name);
        if self.names[index].1 != active {
            self.names[index].1 = active;
            self.snapshot.take();
        }
    }

    pub(super) fn snapshot(&self) -> &[String] {
        self.snapshot.get_or_init(|| self.names.iter()
            .filter(|(_, active)| *active)
            .map(|(name, _)| name.to_string())
            .collect())
    }

    pub(super) fn replace(&mut self, candidates: Vec<String>) {
        *self = Self::default();
        for name in &candidates {
            self.set_active(name, true);
        }
        // Preserve this public API's supplied order and even duplicate names.
        self.snapshot = OnceCell::from(candidates);
    }

    #[cfg(test)]
    pub(super) fn is_initialized(&self) -> bool {
        self.snapshot.get().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_snapshot_until_queried() {
        let mut names = CompletionCandidates::default();
        for i in 0..1000 {
            names.set_active(&format!("name_{i}"), true);
        }
        assert!(!names.is_initialized());
        assert_eq!(names.snapshot().len(), 1000);
        let pointer = names.snapshot().as_ptr();
        names.intern("local_only");
        names.set_active("name_500", true);
        assert!(names.is_initialized());
        assert_eq!(names.snapshot().as_ptr(), pointer);
        names.set_active("name_500", false);
        assert!(!names.is_initialized());
        assert_eq!(names.snapshot().len(), 999);
    }

    #[test]
    fn visibility_and_revival_order() {
        let mut names = CompletionCandidates::default();
        names.intern("early");
        names.set_active("late", true);
        assert_eq!(names.snapshot(), &["late"]);
        names.set_active("early", true);
        assert_eq!(names.snapshot(), &["early", "late"]);
        names.set_active("early", false);
        assert_eq!(names.snapshot(), &["late"]);
        names.set_active("early", true);
        assert_eq!(names.snapshot(), &["early", "late"]);
        assert_eq!(names.names.len(), 2);
        assert_eq!(Rc::strong_count(&names.names[0].0), 2);
    }

    #[test]
    fn legacy_snapshot_replacement() {
        let mut names = CompletionCandidates::default();
        names.replace(vec!["b".into(), "a".into(), "b".into()]);
        assert_eq!(names.snapshot(), &["b", "a", "b"]);
        names.replace(vec!["new".into()]);
        assert_eq!(names.snapshot(), &["new"]);
        assert_eq!(names.names.len(), 1);
        assert!(!names.indices.contains_key("b"));
        names.replace(Vec::new());
        assert!(names.snapshot().is_empty());
    }
}
