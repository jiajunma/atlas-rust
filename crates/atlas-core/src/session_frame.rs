//! The Atlas session frame: file inclusion, output redirection, and the
//! top-level output surface.
//!
//! Ports the upstream input machinery (buffer.w / main.w): `<file` includes
//! with include-once bookkeeping, `<<file` forced re-reads, `>file` /
//! `>>file` single-command output redirection, the `Starting to read from
//! file ...` / `Completely read file ...` framing, `Value:` printing with
//! void suppression, the abandon cascade on errors inside includes, and the
//! clean flag that decides the session's exit status. Definition reports are
//! indented here from the active include depth, keeping output formatting in
//! one layer.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::diagnostic::{Diagnostic, ErrorKind, SourceId};
use crate::lex::{DirectiveKind, Lexer, Token, TokenKind};
use crate::session::{execute_tokens, SessionEvent};
use crate::source::SourceText;
use crate::typed::TypedContext;

/// Read access to included files. The CLI backs this with the filesystem
/// (lossy UTF-8 — upstream is byte-oriented, a stray byte must not become an
/// open failure); tests use an in-memory map. `None` means the open failed.
pub trait FileProvider {
    fn read(&self, path: &str) -> Option<String>;
}

/// Write access for `>file` / `>>file` redirects. The sink opens (creating
/// or truncating) BEFORE the command evaluates, so a failing command still
/// leaves the file with any partial output — upstream behaviour.
pub trait FileSink {
    fn open(&mut self, path: &str, append: bool) -> bool;
    fn write(&mut self, text: &str);
    fn close(&mut self);
}

/// Upstream nests includes at most this deep before something is wrong; the
/// cycle guard bounds real recursion, this bounds pathological providers.
const MAX_INCLUDE_DEPTH: usize = 64;

struct FileRecord {
    origin: String,
    text: String,
    /// Rewritten line number (1-based index) -> first physical line.
    line_map: Vec<usize>,
}

/// How one command stream ended.
enum Outcome {
    /// End of input reached; every command ran (or recovered at depth 0).
    Finished,
    /// A `quit` command ended the whole session.
    Quit,
    /// An error must unwind the include stack (the enclosing levels print
    /// their abandon lines while propagating).
    Abort,
}

pub struct SessionFrame<P: FileProvider, S: FileSink> {
    provider: P,
    sink: S,
    /// Search prefixes, each already `/`-terminated; the empty prefix (cwd)
    /// is implicit and always tried last.
    search_path: Vec<String>,
    completed: BTreeSet<String>,
    active: Vec<String>,
    context: TypedContext,
    clean: bool,
    quitting: bool,
    next_source_id: u64,
    registry: BTreeMap<u64, FileRecord>,
}

impl<P: FileProvider, S: FileSink> SessionFrame<P, S> {
    pub fn new(provider: P, sink: S, search_path: Vec<String>) -> Self {
        let search_path = search_path
            .into_iter()
            .map(|entry| {
                if entry.ends_with('/') {
                    entry
                } else {
                    format!("{entry}/")
                }
            })
            .collect();
        Self {
            provider,
            sink,
            search_path,
            completed: BTreeSet::new(),
            active: Vec::new(),
            context: TypedContext::new(),
            clean: true,
            quitting: false,
            next_source_id: 1,
            registry: BTreeMap::new(),
        }
    }

    /// Whether any syntax/type/runtime error occurred. Include-open failures
    /// and the abandon cascade deliberately do not flip this (upstream sets
    /// `clean=false` only in `yyerror` and the evaluation catch blocks).
    pub fn is_clean(&self) -> bool {
        self.clean
    }

    /// Whether a `quit` command ended the session.
    pub fn is_quitting(&self) -> bool {
        self.quitting
    }

    /// Feed one top-level input (a CLI file argument or stdin) as a depth-0
    /// command stream: no Starting/Completely framing for the stream itself,
    /// per-command error recovery, includes resolved for `<`/`<<` inside.
    pub fn run_top_level(&mut self, origin: &str, raw_text: &str) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        if !self.quitting {
            self.run_stream(origin.to_owned(), raw_text.to_owned(), &mut events);
        }
        events
    }

    /// Render a diagnostic with its origin path, physical position, and the
    /// offending source line. Diagnostics without a span carry the
    /// `<Kind> error:` header (the oracle prints those lines bare, but the
    /// differential harness's stderr grammar keys every diagnostic off the
    /// header, and the frozen events record them by category).
    pub fn describe(&self, diagnostic: &Diagnostic) -> String {
        let Some(span) = diagnostic.span else {
            return format!("{:?} error: {}", diagnostic.kind, diagnostic.message);
        };
        let Some(record) = self.registry.get(&span.source_id().get()) else {
            return format!("{:?} error: {}", diagnostic.kind, diagnostic.message);
        };
        let line = record
            .text
            .lines()
            .nth(span.start.line.saturating_sub(1))
            .unwrap_or_default();
        let physical = record
            .line_map
            .get(span.start.line.saturating_sub(1))
            .copied()
            .unwrap_or(span.start.line);
        let column = span.start.column;
        let caret_width = span.byte_range().len().clamp(
            1,
            line.len().saturating_sub(column.saturating_sub(1)).max(1),
        );
        format!(
            "{:?} error at {}:{physical}:{column}: {}\n  | {line}\n  | {}{}",
            diagnostic.kind,
            record.origin,
            diagnostic.message,
            " ".repeat(column.saturating_sub(1)),
            "^".repeat(caret_width),
        )
    }

    fn register(&mut self, origin: String, raw_text: &str) -> SourceText {
        let (text, line_map) = preprocess(raw_text);
        let id = self.next_source_id;
        self.next_source_id += 1;
        self.registry.insert(
            id,
            FileRecord {
                origin,
                text: text.clone(),
                line_map,
            },
        );
        SourceText::with_id(SourceId::new(id), text)
    }

    fn run_stream(
        &mut self,
        origin: String,
        raw_text: String,
        events: &mut Vec<SessionEvent>,
    ) -> Outcome {
        let source = self.register(origin, &raw_text);
        let mut lexer = Lexer::new(&source);
        let mut command: Vec<Token> = Vec::new();
        let depth = self.active.len();

        loop {
            let token = match lexer.next_token() {
                Ok(token) => token,
                Err(diagnostic) => {
                    events.push(SessionEvent::Diagnostic(diagnostic));
                    self.clean = false;
                    if depth > 0 {
                        self.abandon(&source, &lexer, events);
                        return Outcome::Abort;
                    }
                    continue;
                }
            };
            match token.kind {
                TokenKind::Newline | TokenKind::Eof => {
                    let ended = token.kind == TokenKind::Eof;
                    if !command.is_empty() {
                        match self.run_command(&mut command, &source, events) {
                            Outcome::Finished => {}
                            Outcome::Quit => return Outcome::Quit,
                            Outcome::Abort => {
                                if depth > 0 {
                                    self.abandon(&source, &lexer, events);
                                    return Outcome::Abort;
                                }
                            }
                        }
                    }
                    if ended {
                        return Outcome::Finished;
                    }
                }
                TokenKind::Directive(kind) if command.is_empty() => {
                    let name = token.value.clone().unwrap_or_default();
                    let outcome = match kind {
                        DirectiveKind::Include | DirectiveKind::ForceInclude => self
                            .run_include_command(
                                &mut lexer,
                                &name,
                                kind == DirectiveKind::ForceInclude,
                                events,
                            ),
                        DirectiveKind::ToFile | DirectiveKind::AddToFile => self
                            .run_redirect_command(
                                &mut lexer,
                                &source,
                                &name,
                                kind == DirectiveKind::AddToFile,
                                events,
                            ),
                    };
                    match outcome {
                        Outcome::Finished => {}
                        Outcome::Quit => return Outcome::Quit,
                        Outcome::Abort => {
                            if depth > 0 {
                                self.abandon(&source, &lexer, events);
                                return Outcome::Abort;
                            }
                        }
                    }
                }
                _ => command.push(token),
            }
        }
    }

    /// `<file` / `<<file`: the directive must be the whole command.
    fn run_include_command(
        &mut self,
        lexer: &mut Lexer<'_>,
        name: &str,
        forced: bool,
        events: &mut Vec<SessionEvent>,
    ) -> Outcome {
        match lexer.next_token() {
            Ok(token) if matches!(token.kind, TokenKind::Newline | TokenKind::Eof) => {}
            Ok(token) => {
                events.push(SessionEvent::Diagnostic(Diagnostic::new(
                    ErrorKind::Syntax,
                    "unexpected token after include directive",
                    Some(token.span),
                )));
                self.clean = false;
                lexer.recover_command();
                return Outcome::Abort;
            }
            Err(diagnostic) => {
                events.push(SessionEvent::Diagnostic(diagnostic));
                self.clean = false;
                lexer.recover_command();
                return Outcome::Abort;
            }
        }
        self.include(name, forced, events)
    }

    fn include(&mut self, name: &str, forced: bool, events: &mut Vec<SessionEvent>) -> Outcome {
        // Literal-name shortcut: the typed name equals an already-completed
        // resolved path — skip without opening, printing nothing.
        if !forced && self.completed.contains(name) {
            return Outcome::Finished;
        }
        let Some((resolved, text)) = self.resolve(name) else {
            events.push(SessionEvent::Diagnostic(Diagnostic::new(
                ErrorKind::Io,
                format!("failed to open input file '{name}'."),
                None,
            )));
            return Outcome::Abort;
        };
        if !forced && self.completed.contains(&resolved) {
            return Outcome::Finished;
        }
        // A file already being read is silently skipped (cycle guard);
        // skipping counts as success.
        if self.active.iter().any(|path| path == &resolved) {
            return Outcome::Finished;
        }
        if self.active.len() >= MAX_INCLUDE_DEPTH {
            events.push(SessionEvent::Diagnostic(Diagnostic::new(
                ErrorKind::Io,
                format!("include depth limit reached at '{resolved}'."),
                None,
            )));
            return Outcome::Abort;
        }
        events.push(output(format!(
            "Starting to read from file '{resolved}'.\n"
        )));
        self.active.push(resolved.clone());
        let outcome = self.run_stream(resolved.clone(), text, events);
        self.active.pop();
        match outcome {
            Outcome::Finished => {
                self.completed.insert(resolved.clone());
                events.push(output(format!("Completely read file '{resolved}'.\n")));
                Outcome::Finished
            }
            other => other,
        }
    }

    /// Search-path resolution: every `--path` prefix in order, the empty
    /// (cwd-relative) prefix last; each candidate tried as typed, then with
    /// `.at` appended when the name does not already carry it.
    fn resolve(&self, name: &str) -> Option<(String, String)> {
        for prefix in self
            .search_path
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(""))
        {
            let plain = format!("{prefix}{name}");
            if let Some(text) = self.provider.read(&plain) {
                return Some((plain, text));
            }
            if !name.ends_with(".at") {
                let extended = format!("{plain}.at");
                if let Some(text) = self.provider.read(&extended) {
                    return Some((extended, text));
                }
            }
        }
        None
    }

    /// `>file EXPR` / `>>file EXPR`: open the sink before evaluation, stream
    /// the command's printable output into it, restore unconditionally.
    fn run_redirect_command(
        &mut self,
        lexer: &mut Lexer<'_>,
        source: &SourceText,
        name: &str,
        append: bool,
        events: &mut Vec<SessionEvent>,
    ) -> Outcome {
        let mut command = Vec::new();
        loop {
            match lexer.next_token() {
                Ok(token) if matches!(token.kind, TokenKind::Newline | TokenKind::Eof) => break,
                Ok(token) => command.push(token),
                Err(diagnostic) => {
                    events.push(SessionEvent::Diagnostic(diagnostic));
                    self.clean = false;
                    lexer.recover_command();
                    return Outcome::Abort;
                }
            }
        }
        if command.is_empty() {
            events.push(SessionEvent::Diagnostic(Diagnostic::new(
                ErrorKind::Syntax,
                "output redirection needs a command",
                None,
            )));
            self.clean = false;
            return Outcome::Abort;
        }
        // Upstream opens (and truncates) before evaluating; a sink that
        // cannot open skips the command with a bare stderr line.
        if !self.sink.open(name, append) {
            events.push(SessionEvent::Diagnostic(Diagnostic::new(
                ErrorKind::Io,
                format!("Failed to open {name}"),
                None,
            )));
            return Outcome::Finished;
        }
        let mut command_events = Vec::new();
        let outcome = self.execute(&mut command, source, &mut command_events);
        for event in command_events {
            match event {
                SessionEvent::Output { text, .. } => self.sink.write(&text),
                other => events.push(other),
            }
        }
        self.sink.close();
        outcome
    }

    fn run_command(
        &mut self,
        command: &mut Vec<Token>,
        source: &SourceText,
        events: &mut Vec<SessionEvent>,
    ) -> Outcome {
        if is_quit(command) {
            command.clear();
            self.quitting = true;
            return Outcome::Quit;
        }
        self.execute(command, source, events)
    }

    /// Run one ordinary command, converting value events into the printed
    /// top-level surface: `Value: <display>` for non-void values, nothing
    /// for void (the empty tuple), definition output verbatim.
    fn execute(
        &mut self,
        command: &mut Vec<Token>,
        source: &SourceText,
        events: &mut Vec<SessionEvent>,
    ) -> Outcome {
        let mut raw_events = Vec::new();
        execute_tokens(command, source, &mut self.context, &mut raw_events);
        let mut failed = false;
        for event in raw_events {
            match event {
                SessionEvent::Value {
                    value,
                    is_void_type,
                    span,
                } => {
                    if !is_void_type {
                        events.push(SessionEvent::Output {
                            text: format!("Value: {value}\n"),
                            span,
                        });
                    }
                }
                SessionEvent::ReportLine { text, span } => {
                    events.push(SessionEvent::Output {
                        text: format!("{}{text}", "  ".repeat(self.active.len())),
                        span,
                    });
                }
                SessionEvent::Diagnostic(diagnostic) => {
                    if diagnostic.kind != ErrorKind::Io {
                        self.clean = false;
                        failed = true;
                    }
                    events.push(SessionEvent::Diagnostic(diagnostic));
                }
                other => events.push(other),
            }
        }
        if failed {
            Outcome::Abort
        } else {
            Outcome::Finished
        }
    }

    /// The abandon line for the CURRENT file; enclosing levels emit their own
    /// while unwinding, so the cascade reads innermost first.
    fn abandon(&mut self, source: &SourceText, lexer: &Lexer<'_>, events: &mut Vec<SessionEvent>) {
        // The offset sits just past the last consumed byte (often the
        // command-ending newline); back up one so the reported line is the
        // one that was being read, not the next.
        let rewritten_line = source.position(lexer.offset().saturating_sub(1)).line;
        let physical = self
            .registry
            .get(&source.source_id().get())
            .and_then(|record| record.line_map.get(rewritten_line.saturating_sub(1)))
            .copied()
            .unwrap_or(rewritten_line);
        let origin = self
            .registry
            .get(&source.source_id().get())
            .map(|record| record.origin.clone())
            .unwrap_or_default();
        events.push(SessionEvent::Diagnostic(Diagnostic::new(
            ErrorKind::Io,
            format!("Abandoning reading of file '{origin}' at line {physical}"),
            None,
        )));
    }
}

fn output(text: String) -> SessionEvent {
    SessionEvent::Output {
        text,
        span: SourceText::new("").span(0, 0),
    }
}

fn is_quit(command: &[Token]) -> bool {
    command.len() == 1 && command[0].kind == TokenKind::Keyword("quit".into())
}

/// Strip trailing whitespace per line FIRST, then join lines whose stripped
/// form ends in `\` (so `foo\ ` still continues); the join inserts nothing.
/// Returns the rewritten text and the rewritten-line -> physical-line map.
fn preprocess(raw: &str) -> (String, Vec<usize>) {
    let mut lines: Vec<&str> = raw.split('\n').collect();
    if raw.ends_with('\n') {
        // `split` yields a spurious final empty segment then.
        lines.pop();
    }
    let mut text = String::with_capacity(raw.len());
    let mut line_map = Vec::new();
    let mut joining = false;
    for (index, line) in lines.into_iter().enumerate() {
        let stripped = line.trim_end();
        if !joining {
            line_map.push(index + 1);
        }
        if let Some(continued) = stripped.strip_suffix('\\') {
            text.push_str(continued);
            joining = true;
        } else {
            text.push_str(stripped);
            text.push('\n');
            joining = false;
        }
    }
    if joining {
        text.push('\n');
    }
    (text, line_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MemoryFiles {
        files: BTreeMap<String, String>,
    }

    impl FileProvider for MemoryFiles {
        fn read(&self, path: &str) -> Option<String> {
            self.files.get(path).cloned()
        }
    }

    /// Sink recording open/write/close calls; `refuse` simulates open failure.
    #[derive(Default)]
    struct MemorySink {
        refuse: bool,
        writes: Vec<(String, bool, String)>,
        open: Option<(String, bool)>,
    }

    impl FileSink for MemorySink {
        fn open(&mut self, path: &str, append: bool) -> bool {
            if self.refuse {
                return false;
            }
            self.open = Some((path.to_owned(), append));
            true
        }

        fn write(&mut self, text: &str) {
            let (path, append) = self.open.clone().expect("write after open");
            self.writes.push((path, append, text.to_owned()));
        }

        fn close(&mut self) {
            self.open = None;
        }
    }

    fn frame(files: &[(&str, &str)], paths: &[&str]) -> SessionFrame<MemoryFiles, MemorySink> {
        let provider = MemoryFiles {
            files: files
                .iter()
                .map(|(name, text)| ((*name).to_owned(), (*text).to_owned()))
                .collect(),
        };
        SessionFrame::new(
            provider,
            MemorySink::default(),
            paths.iter().map(|path| (*path).to_owned()).collect(),
        )
    }

    fn stdout_of(events: &[SessionEvent]) -> String {
        events
            .iter()
            .filter_map(|event| match event {
                SessionEvent::Output { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    fn stderr_of(events: &[SessionEvent]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                SessionEvent::Diagnostic(diagnostic) => Some(diagnostic.message.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn includes_run_with_the_exact_upstream_framing_lines() {
        let mut frame = frame(&[("lib.at", "1+1\n")], &[]);
        let events = frame.run_top_level("<stdin>", "<lib.at\n2+2\n");
        assert_eq!(
            stdout_of(&events),
            "Starting to read from file 'lib.at'.\nValue: 2\nCompletely read file 'lib.at'.\nValue: 4\n"
        );
        assert!(frame.is_clean());
    }

    #[test]
    fn include_once_skips_silently_and_force_reincludes() {
        let mut frame = frame(&[("lib.at", "1\n")], &[]);
        let first = frame.run_top_level("<stdin>", "<lib.at\n<lib.at\n");
        assert_eq!(stdout_of(&first).matches("Starting").count(), 1);
        let forced = frame.run_top_level("<stdin>", "<<lib.at\n");
        assert_eq!(stdout_of(&forced).matches("Starting").count(), 1);
    }

    #[test]
    fn resolution_prefers_path_entries_then_cwd_and_appends_at() {
        let mut frame = frame(
            &[("scripts/lib.at", "1\n"), ("lib.at", "2\n")],
            &["scripts"],
        );
        let events = frame.run_top_level("<stdin>", "<lib\n");
        assert_eq!(
            stdout_of(&events),
            "Starting to read from file 'scripts/lib.at'.\nValue: 1\nCompletely read file 'scripts/lib.at'.\n"
        );
    }

    #[test]
    fn missing_file_aborts_the_include_stack_but_not_the_session() {
        let mut frame = frame(&[("outer.at", "<missing.at\n7\n")], &[]);
        let events = frame.run_top_level("<stdin>", "<outer.at\n5\n");
        let errors = stderr_of(&events);
        assert_eq!(errors[0], "failed to open input file 'missing.at'.");
        assert_eq!(errors[1], "Abandoning reading of file 'outer.at' at line 1");
        // The outer stream continues; 7 (inside outer.at) never ran.
        let stdout = stdout_of(&events);
        assert!(!stdout.contains("Value: 7"));
        assert!(stdout.contains("Value: 5"));
        assert!(stdout.contains("Starting to read from file 'outer.at'."));
        assert!(!stdout.contains("Completely read file 'outer.at'."));
        // Open failures leave the session clean (upstream semantics).
        assert!(frame.is_clean());
    }

    #[test]
    fn an_error_inside_an_include_cascades_innermost_first_and_rereads() {
        let mut frame = frame(
            &[("outer.at", "<inner.at\n"), ("inner.at", "1 +\n)\n")],
            &[],
        );
        let events = frame.run_top_level("<stdin>", "<outer.at\n");
        let errors = stderr_of(&events);
        assert!(errors
            .iter()
            .any(|message| message == "Abandoning reading of file 'inner.at' at line 2"));
        assert!(errors
            .iter()
            .any(|message| message == "Abandoning reading of file 'outer.at' at line 1"));
        let inner_index = errors
            .iter()
            .position(|message| message.contains("'inner.at'"))
            .unwrap();
        let outer_index = errors
            .iter()
            .position(|message| message.contains("'outer.at'"))
            .unwrap();
        assert!(inner_index < outer_index, "innermost abandon first");
        assert!(!frame.is_clean());
        // Neither file completed, so both re-read next time.
        let again = frame.run_top_level("<stdin>", "<outer.at\n");
        assert!(stdout_of(&again).contains("Starting to read from file 'inner.at'."));
    }

    #[test]
    fn cyclic_includes_are_silently_skipped() {
        let mut frame = frame(&[("a.at", "<b.at\n1\n"), ("b.at", "<a.at\n2\n")], &[]);
        let events = frame.run_top_level("<stdin>", "<a.at\n");
        let stdout = stdout_of(&events);
        assert_eq!(stdout.matches("Starting").count(), 2);
        assert!(stdout.contains("Value: 2"));
        assert!(stdout.contains("Value: 1"));
        assert!(frame.is_clean());
    }

    #[test]
    fn quit_inside_an_include_ends_the_whole_session() {
        let mut frame = frame(&[("lib.at", "1\nquit\n2\n")], &[]);
        let events = frame.run_top_level("<stdin>", "<lib.at\n3\n");
        let stdout = stdout_of(&events);
        assert!(stdout.contains("Value: 1"));
        assert!(!stdout.contains("Value: 2"));
        assert!(!stdout.contains("Value: 3"));
        assert!(frame.is_quitting());
        assert!(frame.run_top_level("<stdin>", "4\n").is_empty());
    }

    #[test]
    fn redirects_open_before_evaluation_and_capture_output() {
        let mut frame = frame(&[], &[]);
        let events = frame.run_top_level("<stdin>", ">out.txt 1+2\n5\n");
        assert_eq!(stdout_of(&events), "Value: 5\n");
        assert_eq!(
            frame.sink.writes,
            vec![("out.txt".to_owned(), false, "Value: 3\n".to_owned())]
        );
        let appended = frame.run_top_level("<stdin>", ">>out.txt 7\n");
        assert_eq!(stdout_of(&appended), "");
        assert_eq!(
            frame.sink.writes[1],
            ("out.txt".to_owned(), true, "Value: 7\n".to_owned())
        );
    }

    #[test]
    fn definition_reports_are_indented_by_include_depth_only() {
        let mut frame = frame(
            &[
                ("outer.at", "outer_value: 2\n<inner.at\n"),
                ("inner.at", "inner_value: 3\n"),
            ],
            &[],
        );
        let events = frame.run_top_level("<stdin>", "top_value: 1\n<outer.at\n");
        assert_eq!(
            stdout_of(&events),
            concat!(
                "Variable top_value: int\n",
                "Starting to read from file 'outer.at'.\n",
                "  Variable outer_value: int\n",
                "Starting to read from file 'inner.at'.\n",
                "    Variable inner_value: int\n",
                "Completely read file 'inner.at'.\n",
                "Completely read file 'outer.at'.\n",
            )
        );
    }

    #[test]
    fn statically_void_expressions_do_not_print_values() {
        let mut frame = frame(&[], &[]);
        let events = frame.run_top_level("<stdin>", "if true then 7 fi\n9\n");
        assert_eq!(stdout_of(&events), "Value: 9\n");
        assert!(stderr_of(&events).is_empty());
    }

    #[test]
    fn redirected_definition_reports_keep_include_indentation() {
        let mut frame = frame(&[("lib.at", ">out.txt x: 1\n")], &[]);
        let events = frame.run_top_level("<stdin>", "<lib.at\n");
        assert_eq!(
            stdout_of(&events),
            concat!(
                "Starting to read from file 'lib.at'.\n",
                "Completely read file 'lib.at'.\n",
            )
        );
        assert_eq!(
            frame.sink.writes,
            vec![(
                "out.txt".to_owned(),
                false,
                "  Variable x: int\n".to_owned(),
            )]
        );
    }

    #[test]
    fn failed_redirect_evaluation_closes_sink_and_top_level_recovers() {
        let mut frame = frame(&[], &[]);
        let events = frame.run_top_level("<stdin>", ">bad.txt unknown_name\n>out.txt 3\n5\n");
        assert!(frame.sink.open.is_none(), "redirect sink must be restored");
        assert_eq!(
            frame.sink.writes,
            vec![("out.txt".into(), false, "Value: 3\n".into())]
        );
        assert_eq!(stdout_of(&events), "Value: 5\n");
        assert_eq!(stderr_of(&events).len(), 1);
        assert!(!frame.is_clean());
    }

    #[test]
    fn failed_redirect_inside_include_closes_sink_before_abandoning() {
        let mut frame = frame(&[("lib.at", ">bad.txt unknown_name\n7\n")], &[]);
        let events = frame.run_top_level("<stdin>", "<lib.at\n5\n");
        assert!(frame.sink.open.is_none(), "redirect sink must be restored");
        assert!(frame.sink.writes.is_empty());
        assert_eq!(
            stdout_of(&events),
            "Starting to read from file 'lib.at'.\nValue: 5\n"
        );
        assert!(stderr_of(&events)
            .iter()
            .any(|message| message == "Abandoning reading of file 'lib.at' at line 1"));
        assert!(!frame.is_clean());
    }

    #[test]
    fn failed_redirect_open_skips_the_command_and_stays_clean() {
        let mut frame = frame(&[], &[]);
        frame.sink.refuse = true;
        let events = frame.run_top_level("<stdin>", ">out.txt 1+2\n");
        assert_eq!(stderr_of(&events), vec!["Failed to open out.txt"]);
        assert_eq!(stdout_of(&events), "");
        assert!(frame.is_clean());
    }

    #[test]
    fn continuation_joins_after_stripping_trailing_whitespace() {
        let (text, line_map) = preprocess("1 + \\  \n2\n3\n");
        assert_eq!(text, "1 + 2\n3\n");
        assert_eq!(line_map, vec![1, 3]);
        let mut frame = frame(&[], &[]);
        let events = frame.run_top_level("<stdin>", "1 + \\\n2\n");
        assert_eq!(stdout_of(&events), "Value: 3\n");
    }

    #[test]
    fn directive_with_trailing_tokens_is_a_syntax_error() {
        let mut frame = frame(&[("lib.at", "1\n")], &[]);
        let events = frame.run_top_level("<stdin>", "<lib.at 3\n");
        assert!(!frame.is_clean());
        assert!(stdout_of(&events).is_empty());
    }

    #[test]
    fn describe_reports_physical_lines_through_the_registry() {
        let mut frame = frame(&[], &[]);
        let events = frame.run_top_level("script.at", "1\n2 + \\\n)\n");
        let diagnostic = events
            .iter()
            .find_map(|event| match event {
                SessionEvent::Diagnostic(diagnostic) => Some(diagnostic),
                _ => None,
            })
            .expect("syntax error");
        let rendered = frame.describe(diagnostic);
        assert!(rendered.contains("script.at:2:"), "{rendered}");
        assert!(rendered.contains("2 + )"), "{rendered}");
    }
}
