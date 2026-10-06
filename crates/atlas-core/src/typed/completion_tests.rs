use super::*;
use crate::session::{run_source_with_context, SessionEvent};
use crate::source::SourceText;

fn run(context: &mut TypedContext, source: &str) {
    let events = run_source_with_context(&SourceText::new(source), context);
    assert!(!events.iter().any(|event| matches!(event, SessionEvent::Diagnostic(_))), "{events:?}");
}

#[test]
fn loading_does_not_materialize_completions() {
    let mut context = TypedContext::new();
    assert!(!context.evaluation.completion_snapshot_is_initialized());
    for i in 0..100 {
        run(&mut context, &format!("set completion_work_{i}={i}\n"));
    }
    assert!(!context.evaluation.completion_snapshot_is_initialized());
    run(&mut context, "readline_completions(\"completion_work_\")\n");
    assert!(context.evaluation.completion_snapshot_is_initialized());
    assert_eq!(context.evaluation.completion_candidates().len(), 412);
    run(&mut context, "set completion_work_new=101\n");
    assert!(!context.evaluation.completion_snapshot_is_initialized());
    assert_eq!(context.evaluation.completion_candidates().len(), 413);
}

#[test]
fn inactive_names_and_value_changes_keep_snapshot() {
    let mut context = TypedContext::new();
    run(&mut context, "set completion_kept=1\nreadline_completions(\"\")\n");
    let pointer = context.evaluation.completion_candidates().as_ptr();
    run(&mut context, "let completion_local=0 in ()\nset completion_kept=2\nforget completion_absent\n");
    assert!(context.evaluation.completion_snapshot_is_initialized());
    assert_eq!(context.evaluation.completion_candidates().as_ptr(), pointer);
    run(&mut context, "forget completion_kept\n");
    assert!(!context.evaluation.completion_snapshot_is_initialized());
    assert!(!context.evaluation.completion_candidates().iter().any(|name| name == "completion_kept"));
    run(&mut context, "set completion_local=3\n");
    assert!(!context.evaluation.completion_snapshot_is_initialized());
    assert_eq!(context.evaluation.completion_candidates().last().unwrap(), "completion_local");
}
