//! Command-line front end for the Rust Atlas interpreter.
//!
//! Runs one shared session over the file arguments (or standard input when
//! none are given) through the session frame: `<file` includes resolve
//! against the repeatable `--path=DIR` prefixes then the working directory,
//! top-level values print as `Value: ...`, definition and include framing
//! text prints verbatim, diagnostics go to stderr with their origin and
//! source excerpt, and the session ends with `Bye.` File arguments are
//! deliberately fed as plain command streams (upstream feeds its main loop
//! from stdin; its prelude-capture semantics for file arguments is a
//! non-goal here). Exit status reflects the upstream clean flag: syntax,
//! type, and runtime errors make it nonzero; a missing include alone does
//! not.

use std::fs::OpenOptions;
use std::io::{BufRead, IsTerminal, Read, Write};
use std::process::ExitCode;

use atlas_core::session::SessionEvent;
use atlas_core::session_frame::{FileProvider, FileSink, SessionFrame};

struct FsProvider;

impl FileProvider for FsProvider {
    fn read(&self, path: &str) -> Option<String> {
        // Lossy: upstream reads bytes, so a stray non-UTF-8 byte must not
        // turn into an open failure.
        std::fs::read(path)
            .ok()
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }
}

#[derive(Default)]
struct FsSink {
    file: Option<std::fs::File>,
}

impl FileSink for FsSink {
    fn open(&mut self, path: &str, append: bool) -> bool {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(path);
        match file {
            Ok(file) => {
                self.file = Some(file);
                true
            }
            Err(_) => false,
        }
    }

    fn write(&mut self, text: &str) {
        if let Some(file) = self.file.as_mut() {
            let _ = file.write_all(text.as_bytes());
        }
    }

    fn close(&mut self) {
        self.file = None;
    }
}

fn print_events(frame: &SessionFrame<FsProvider, FsSink>, events: &[SessionEvent]) {
    for event in events {
        match event {
            SessionEvent::Output { text, .. } => print!("{text}"),
            SessionEvent::ReportLine { text, .. } => print!("{text}"),
            SessionEvent::PlainReportLine { text, .. } => print!("{text}"),
            SessionEvent::Diagnostic(diagnostic) => eprintln!("{}", frame.describe(diagnostic)),
            // The frame renders values as Output text; none reach here.
            SessionEvent::Value { value, .. } => println!("Value: {value}"),
        }
    }
}

fn main() -> ExitCode {
    // The parallel passes (Weyl enumeration, orbit conjugation, KGB BFS)
    // are iterative, so a modest rayon worker stack keeps RSS down
    // (default 8MiB per worker vs ~1-2MiB actually used).
    let _ = rayon::ThreadPoolBuilder::new()
        .stack_size(2 * 1024 * 1024)
        .build_global();
    let mut search_path = Vec::new();
    let mut files = Vec::new();
    for argument in std::env::args().skip(1) {
        if let Some(directory) = argument.strip_prefix("--path=") {
            search_path.push(directory.to_owned());
        } else {
            files.push(argument);
        }
    }

    let mut frame = SessionFrame::new(FsProvider, FsSink::default(), search_path);
    if files.is_empty() {
        if std::io::stdin().is_terminal() {
            return run_interactive(&mut frame);
        }
        let mut text = String::new();
        if std::io::stdin().read_to_string(&mut text).is_err() {
            eprintln!("Io error: could not read standard input");
            return ExitCode::from(2);
        }
        let events = frame.run_top_level("<stdin>", &text);
        print_events(&frame, &events);
    } else {
        for path in &files {
            let text = match std::fs::read(path) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(error) => {
                    eprintln!("Io error: {path}: {error}");
                    return ExitCode::from(2);
                }
            };
            let events = frame.run_top_level(path, &text);
            print_events(&frame, &events);
            if frame.is_quitting() {
                break;
            }
        }
    }
    println!("Bye.");
    if frame.is_clean() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_interactive(frame: &mut SessionFrame<FsProvider, FsSink>) -> ExitCode {
    println!(
        "This is 'atlas' (version 1.1.1, axis language version 1.1),\n\
         the Atlas of Lie Groups and Representations interpreter,\n\
         compiled with Rust, readline disabled.\n\
         http://www.liegroups.org/"
    );
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut line = String::new();
    loop {
        print!("atlas> ");
        let _ = std::io::stdout().flush();
        line.clear();
        match input.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let events = frame.run_top_level("<stdin>", &line);
                print_events(frame, &events);
                if frame.is_quitting() {
                    break;
                }
            }
        }
    }
    println!("Bye.");
    if frame.is_clean() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
