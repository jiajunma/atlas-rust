//! Command-line front end for the Rust Atlas interpreter.
//!
//! Reads Atlas source from the files given as arguments, or from standard
//! input when none are given, and prints one line per session event:
//! values via their language `Display`, definitions as their output text,
//! and diagnostics to stderr prefixed by their category. Exit status is
//! nonzero when any diagnostic was produced.

use std::io::Read;
use std::process::ExitCode;

use atlas_core::session::{run_source, SessionEvent};
use atlas_core::source::SourceText;

fn run_text(text: &str) -> (usize, usize) {
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
                eprintln!("{:?} error: {}", diagnostic.kind, diagnostic.message);
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
        diagnostics += run_text(&text).1;
    } else {
        for path in &arguments {
            match std::fs::read_to_string(path) {
                Ok(text) => diagnostics += run_text(&text).1,
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
