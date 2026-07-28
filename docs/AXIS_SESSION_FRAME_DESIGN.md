# Axis session frame design: include machinery (language phase A)

Substrate: docs/AXIS_LANGUAGE_TRACE.md (upstream cites live there;
key sources buffer.w, lexer.w:554-562/732-740, parser.y:180-217,
main.w:395-420/504-543).

## Goal and evidence

The corpus histogram (2026-07-28, all 240 upstream .at scripts through
atlas-cli): 224 of 232 parse failures die at the FIRST line on the
include directive `<basic.at`. The include machinery is the gateway to
every later language feature; phase A ports it plus the session frame
it forces into existence (command loop, output redirect, the
top-level-value output surface). Definitions (`set`, `set_type`,
declarations) are phase B; this design only reserves their output
indentation hook.

## Upstream semantics being ported (traced)

1. DIRECTIVES — recognized only as the FIRST token of a command:
   - `<file`  : include, skipped if the file was already COMPLETELY
     read before (include-once).
   - `<<file` : forced re-include.
   - `>file CMD` / `>>file CMD`: redirect the output of exactly one
     following command to the file (truncate / append), then restore.
   Elsewhere `<` `>` stay comparison operators.
2. FILENAME SCANNING: after optional space, either a quoted `"..."`
   string, or an unquoted run of alphanumerics plus exactly
   `.-+~_=!?@#$%&|` (NO slash — subdirectory paths need quotes),
   terminated by the first character outside the set.
3. RESOLUTION: try each `--path=DIR` entry in order, cwd LAST; each
   candidate tried as-is, then with `.at` appended (only if the open
   failed and the name does not already end in `.at`). The winning
   RESOLVED path is the file's identity.
4. BOOKKEEPING: `seen` = interned resolved paths; `completed` = set
   only when a file is read to its end without error. `<` skips only
   seen-AND-completed files (an aborted file re-reads); an exact
   literal match of the typed name against a seen+completed path skips
   without opening. A file already on the ACTIVE include stack is
   silently skipped (cycle guard). Skipping counts as success.
5. ERRORS: open failure prints `failed to open input file 'name'.` to
   stderr and ABORTS the whole include stack (session continues). Any
   diagnostic inside an included file abandons ALL open includes,
   printing `Abandoning reading of file 'F' at line N` per open file
   (stderr, innermost first).
6. OUTPUT SURFACE (byte-exact, stdout):
   - `Starting to read from file 'PATH'.` on push (resolved path).
   - `Completely read file 'PATH'.` on successful pop.
   - Neither line is indented; definition reports (phase B) indent by
     2 x include-depth — the frame exposes that depth.
   - Top-level expression of non-void type prints `Value: <display>`;
     VOID-typed top-level expressions print NOTHING. (Today the CLI
     prints the bare display and unit values; both change.)
7. INPUT PREPROCESSING owned by the frame: trailing whitespace is
   stripped from every line; a line ending in `\` joins to the next
   (invisible to the lexer, works mid-token).
8. COMMAND BOUNDARIES: a command ends at a newline reached at
   bracket-nesting depth 0 (inside `(..)` `[..]` and `{..}` comments a
   newline is whitespace). `quit` ends the session; end of input in a
   file pops it.

## Port architecture

New module `atlas-core/src/session_frame.rs`:

- `pub struct SessionFrame<F: FileProvider>` owning: the search path
  list, `seen: BTreeMap<String, FileId>`-style interning with a
  `completed` set, the active include stack (resolved path + line
  cursor), and the underlying evaluation session state (the same state
  `run_source` threads today, so values/definitions persist across
  commands and files).
- `pub trait FileProvider { fn read(&self, path: &str) -> Option<String>; }`
  — the filesystem in atlas-cli, an in-memory map in tests. Path
  JOINING stays plain string concatenation exactly like upstream
  (each `--path` entry gets `/` appended once at option parse).
- Event-driven output: the frame yields the SAME `SessionEvent` stream
  the CLI already consumes, extended with
  `SessionEvent::IncludeStarted { path }` /
  `SessionEvent::IncludeFinished { path }` /
  `SessionEvent::IncludeAborted { path, line }` so the CLI (and the
  differential) renders the exact upstream lines and routes them to
  stdout/stderr correctly. The CLI's `Value` rendering gains the
  `Value: ` prefix and the void suppression; the frame tags each value
  event with `is_void` (from the existing scalar-type machinery).
- COMMAND SPLITTER inside the frame: line-continuation join, trailing
  -space strip, then a token-light scanner tracking bracket depth and
  `{}` comment nesting and string literals, cutting command texts at
  depth-0 newlines. Directive commands (`<` `<<` `>` `>>` first
  token) are handled by the frame; every other command text goes to
  the existing `run_source` path with its ORIGIN (path, starting line)
  so diagnostics keep absolute positions.
- Redirects: `>file` / `>>file` capture the Value/Output events of the
  ONE following command into the named file (truncate/append via a
  `FileSink` trait mirroring `FileProvider`; CLI = std::fs, tests =
  memory). Diagnostics are NOT redirected (stderr semantics).
- atlas-cli: replaces its per-file `run_text` loop with one
  `SessionFrame` fed the CLI arguments in order (matching upstream,
  where non-option args are startup scripts), plus `--path=DIR`
  option parsing (repeatable, prefix form only).

## Non-goals (deferred with their phase)

- `set`/`set_type`/`forget`/declarations and their report lines
  (phase B — the frame only carries the indentation depth).
- The prelude-capture mode, interactive prompt/banner, `quiet`/
  `verbose` toggles, `$` last-value (REPL concerns).
- Making `input_path` a mutable user-space variable (needs the
  language's `[string]` globals; the CLI option list is fixed for
  now).
- Lexer-level `<` inside expressions is already a comparison operator
  in the existing grammar; nothing changes there.

## Compatibility notes (checked against the trace)

- Identity is the RESOLVED path string, so the same file reached via
  two different search-path prefixes is two identities — faithfully
  ported, quirk included.
- The literal-name skip shortcut (typed name equals a seen+completed
  path) is ported: it is observable (no `Starting...` line).
- A skipped include prints NOTHING (no Starting/Completely lines).
- `quit` inside an included file ends the whole session (upstream
  behaviour; the frame propagates it).
- Exit status: any diagnostic anywhere makes the session unclean
  (existing CLI behaviour preserved).

## Tests and gate

- Unit (in-memory provider): filename scanning (quoted, unquoted
  charset, terminator); resolution order (path entries before cwd,
  `.at` appended only on failure and only when absent); include-once
  vs `<<`; aborted-file re-read; cycle silent-skip; literal-name
  shortcut; abort cascade messages and their order; redirect capture
  truncate vs append; continuation join mid-token; command splitting
  with nested brackets/comments/strings.
- CLI integration: a two-file include chain runs end to end with
  byte-exact `Starting.../Completely read...` framing and `Value: `
  lines (golden-string test through a temp dir).
- CORPUS RERUN as the stage gate: the 224 `<`-blocked files must all
  progress PAST the include line into their real content; the new
  first-error histogram is recorded in the commit message as the
  phase-B priority list. MATCH count must not regress (currently 2).
- Differential: hpc/script_corpus_diff.py reruns on the HPC after the
  local gate; `Value:`-prefix and void suppression should flip some
  OUTPUT_DIFF entries toward MATCH.

## Three independent design checks

(1) Upstream fidelity — directive recognition/scanning/resolution/
bookkeeping/output lines vs buffer.w+lexer.w+main.w, including the
skip shortcut, cycle silence, and abort cascade; (2) Rust internals —
frame/session-state ownership split, the command splitter vs the
existing lexer (duplication risk), event extension shape, provider
traits; (3) API and consumer fit — SessionEvent additions' blast
radius (session gate tests, kgb_differential driver, corpus script),
CLI argument semantics, diagnostics origin threading. Corrections
fold here before implementation.
