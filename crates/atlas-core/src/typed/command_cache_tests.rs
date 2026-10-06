use super::*;
use crate::session::{run_source_with_context, SessionEvent};
use crate::source::SourceText;

fn run(context: &mut TypedContext, source: &str) {
    let events = run_source_with_context(&SourceText::new(source), context);
    assert!(!events.iter().any(|event| matches!(event, SessionEvent::Diagnostic(_))), "{events:?}");
}

#[test]
fn name_local_invalidation_keeps_other_views() {
    let mut context = TypedContext::new();
    let succ = context.overloads.merged_view("succ", &context.types);
    let minus = context.overloads.merged_view("-", &context.types);
    run(&mut context, "set succ(int n)=n>0\n");
    let replacement = context.overloads.merged_view("succ", &context.types);
    assert!(!Rc::ptr_eq(&succ, &replacement));
    assert!(replacement.iter().any(|v| v.arg_type == int_type() && v.result_type == bool_type()));
    assert!(Rc::ptr_eq(&minus, &context.overloads.merged_view("-", &context.types)));
    run(&mut context, "forget succ @ int\n");
    assert!(context.overloads.merged_view("succ", &context.types).is_empty());
    assert!(Rc::ptr_eq(&minus, &context.overloads.merged_view("-", &context.types)));
    run(&mut context, "set succ(int n)=n+20\n");
    assert!(context.overloads.merged_view("succ", &context.types).iter()
        .any(|v| v.arg_type == int_type() && v.result_type == int_type()));
}

#[test]
fn immutable_views_survive_replacement() {
    let mut context = TypedContext::new();
    run(&mut context, "set cached_command(int n)=n+10\n");
    let old = context.overloads.merged_view("cached_command", &context.types);
    run(&mut context, "set cached_command(int n)=n>0\n");
    let new = context.overloads.merged_view("cached_command", &context.types);
    assert!(!Rc::ptr_eq(&old, &new));
    assert_eq!(old.len(), 1);
    assert_eq!(old[0].result_type, int_type());
    assert_eq!(new[0].result_type, bool_type());
    let cloned = context.overloads.clone();
    assert!(cloned.views.borrow().variants.is_empty(), "a transactional clone needs no copied cache");
    assert!(context.overloads.views.borrow().variants.contains_key("cached_command"));
}

#[test]
fn only_current_type_revision_retained() {
    let mut context = TypedContext::new();
    let old = Arc::downgrade(context.types.revision());
    let first = context.overloads.merged_view("succ", &context.types);
    context.types.add_alias("CacheRevision", bool_type());
    assert!(old.upgrade().is_some(), "cache owns old identity, preventing address reuse");
    let next = context.overloads.merged_view("succ", &context.types);
    assert!(!Rc::ptr_eq(&first, &next));
    assert!(old.upgrade().is_none(), "old revisions must not accumulate after invalidation");
    let mut types = context.types.clone();
    let retained = Arc::downgrade(types.revision());
    for _ in 0..10 {
        let number = types.lookup("CacheRevision").unwrap();
        types.update(number, int_type(), vec![]);
        context.overloads.merged_view("succ", &types);
        assert_eq!(context.overloads.views.borrow().variants.len(), 1);
    }
    // The live context, not a cache revision chain, still owns this snapshot.
    assert!(retained.upgrade().is_some());
    drop(context);
    assert!(retained.upgrade().is_none());
}
