//! Command-line front end for the Rust Atlas interpreter.
//!
//! Reads Atlas source from the files given as arguments, or from standard
//! input when none are given, and prints one line per session event:
//! values via their language `Display`, definitions as their output text,
//! and diagnostics to stderr with their category, position, and the
//! offending source line. Exit status is nonzero when any diagnostic was
//! produced.

use std::io::Read;
use std::process::ExitCode;

use atlas_core::diagnostic::Diagnostic;
use atlas_core::session::{run_source, SessionEvent};
use atlas_core::source::SourceText;

fn report(diagnostic: &Diagnostic, origin: &str, text: &str) {
    let Some(span) = diagnostic.span else {
        eprintln!("{:?} error: {}", diagnostic.kind, diagnostic.message);
        return;
    };
    eprintln!(
        "{:?} error at {origin}:{}:{}: {}",
        diagnostic.kind, span.start.line, span.start.column, diagnostic.message
    );
    let Some(line) = text.lines().nth(span.start.line.saturating_sub(1)) else {
        return;
    };
    eprintln!("  | {line}");
    let caret_width = span.byte_range().len().clamp(
        1,
        line.len()
            .saturating_sub(span.start.column.saturating_sub(1))
            .max(1),
    );
    eprintln!(
        "  | {}{}",
        " ".repeat(span.start.column.saturating_sub(1)),
        "^".repeat(caret_width)
    );
}

fn run_text(origin: &str, text: &str) -> (usize, usize) {
    let source = SourceText::new(text);
    let mut values = 0;
    let mut diagnostics = 0;
    for event in run_source(&source) {
        match event {
            SessionEvent::Value { value, .. } => {
                values += 1;
                println!("{value}");
            }
            SessionEvent::Output { text, .. } => print!("{text}"),
            SessionEvent::Diagnostic(diagnostic) => {
                diagnostics += 1;
                report(&diagnostic, origin, text);
            }
        }
    }
    (values, diagnostics)
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut diagnostics = 0;
    if arguments.is_empty() {
        let mut text = String::new();
        if std::io::stdin().read_to_string(&mut text).is_err() {
            eprintln!("Io error: could not read standard input");
            return ExitCode::from(2);
        }
        diagnostics += run_text("<stdin>", &text).1;
    } else {
        for path in &arguments {
            match std::fs::read_to_string(path) {
                Ok(text) => diagnostics += run_text(path, &text).1,
                Err(error) => {
                    eprintln!("Io error: {path}: {error}");
                    return ExitCode::from(2);
                }
            }
        }
    }
    if diagnostics == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
