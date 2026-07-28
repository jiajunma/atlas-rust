# Axis session frame design: include machinery (language phase A)

Substrate: docs/AXIS_LANGUAGE_TRACE.md (upstream cites live there;
key sources buffer.w, lexer.w:215-231/455-473/553-594/732-740,
parser.y:180-217, main.w:395-420/504-543/627-635).
Reviewed 2026-07-28 by three independent fresh-context checks
(fidelity / internals / API); all corrections folded, [R] marks the
changed decisions.

## Goal and evidence

The corpus histogram (2026-07-28, all 240 upstream .at scripts through
atlas-cli): 224 of 232 parse failures die at the FIRST line on the
include directive `<basic.at`. The include machinery is the gateway to
every later language feature; phase A ports it plus the session frame
it forces into existence (command loop, output redirect, the
top-level-value output surface). Definitions (`set`, `set_type`,
declarations) are phase B; this design only reserves their output
indentation hook.

## Upstream semantics being ported (traced, review-verified)

1. DIRECTIVES — recognized only as the FIRST token of a command:
   - `<file`  : include, skipped if the file was already COMPLETELY
     read before (include-once). [R] Trailing tokens after the
     filename (other than whitespace/comments) are a syntax error.
   - `<<file` : forced re-include.
   - [R] `>file EXPR` / `>>file EXPR`: redirect the output of the ONE
     expression command in the SAME parse unit (truncate / append).
     Upstream's grammar allows only `TOFILE expr`, `TOFILE whattype
     id?`, `TOFILE showall`; the whattype/showall forms defer with
     their features and stay syntax errors — they must NOT silently
     become legal as generic commands. A bare `>file` line is a syntax
     error.
   Elsewhere `<` `>` stay comparison operators.
2. FILENAME SCANNING: [R] skip whitespace AND `{}` comments first
   (upstream skip_space does both); then either a quoted `"..."`
   string, or an unquoted run of alphanumerics plus exactly
   `.-+~_=!?@#$%&|` (NO slash — subdirectory paths need quotes),
   terminated by the first character outside the set. [R] `<` at end
   of line scans an EMPTY name, which then fails to open as `''`.
3. RESOLUTION: try each `--path=DIR` entry in order, cwd LAST; each
   candidate tried as-is, then with `.at` appended (only if the open
   failed and the name does not already end in `.at`). The winning
   RESOLVED path (prefix + name + maybe `.at`) is the file identity,
   and is the spelling printed in the Starting/Completely lines.
4. BOOKKEEPING: `completed` is set only when a file is read to its
   end without error. `<` skips only seen-AND-completed files (an
   aborted file re-reads); [R] the literal-name shortcut — the typed
   name exactly equals an already-completed resolved path — skips
   without opening (observable: no output at all). A file already on
   the ACTIVE include stack is silently skipped (cycle guard).
   Skipping counts as success.
5. ERRORS: open failure prints `failed to open input file 'name'.` to
   stderr — [R] quoting the name AS TYPED, not any candidate — and
   ABORTS the whole include stack (session continues at depth 0). Any
   Syntax/Name/Type/Runtime diagnostic inside an included file
   abandons ALL open includes, printing
   `Abandoning reading of file 'F' at line N` per open file (stderr,
   innermost first, N = physical line reached). At depth 0 an error
   just continues with the next command (upstream stdin loop).
   [R] CLEAN FLAG: neither the open failure nor the abandon cascade
   flips the session's clean flag — upstream sets `clean=false` only
   for syntax/type/runtime errors. Exit status comes from the clean
   flag, NOT from "any diagnostic seen".
6. OUTPUT SURFACE (byte-exact, stdout):
   - `Starting to read from file 'PATH'.` on push (resolved path).
   - `Completely read file 'PATH'.` on successful pop.
   - Neither line is indented; definition reports (phase B) indent by
     2 x include-depth — the frame exposes that depth.
   - Top-level expression of non-void type prints `Value: <display>`;
     VOID-typed top-level expressions print NOTHING. [R] Phase-A void
     test is `value == empty tuple` at the event boundary (the
     existing scalar-type machinery has no Void; this becomes
     type-driven with the phase-B type system).
   - [R] `Bye.` prints to stdout unconditionally at session end (quit
     or end of input) — upstream does this even piped, and the
     differential's upstream side always ends with it.
7. INPUT PREPROCESSING owned by the frame: [R] per line, strip
   trailing whitespace FIRST, then a now-trailing `\` joins to the
   next line (so `foo\ ` still continues); the join inserts nothing
   and is invisible to the lexer.
8. COMMAND BOUNDARIES: [R] a command ends at a newline reached with
   EMPTY NESTING and NO PENDING prevent_termination — nesting counts
   brackets AND the keyword pairs let/in, begin/end, if/fi, while-for/
   od, case/esac; prevent_termination is set by operators, `, ; : .`,
   and/or/not, set, set_type, whattype, let-in. The crate lexer
   ALREADY implements exactly this (lex.rs: NestingKind,
   prevent_termination, the dot-operator exception, nested `{}`
   comments, `""` escapes) and session.rs already cuts commands at its
   Newline tokens. THERE IS NO NEW TEXT-LEVEL SPLITTER — the earlier
   draft's "token-light scanner" is deleted; a second boundary
   implementation would drift from the lexer's.
9. [R] `quit`: a command whose first token is the `quit` keyword
   (rest of command empty) ends the whole session — from ANY include
   depth — with no diagnostic. `Bye.` still prints.

## Port architecture [R — reshaped by all three reviews]

All in `atlas-core` (new module `src/session_frame.rs`, cooperating
with `pub(crate)` internals of session.rs):

- `pub trait FileProvider { fn read(&self, path: &str) -> Option<String>; }`
  (std::fs in atlas-cli with LOSSY utf-8 decoding — upstream is
  byte-oriented, a stray byte must not become an open failure;
  in-memory map in tests). `Option` conflates open/read errors —
  acceptable with eager whole-file reads; `completed` is set at
  successful PROCESSING end, not read end.
  `pub trait FileSink { fn write(&mut self, path: &str, append: bool, text: &str) -> bool; }`
  for redirects (open/truncate happens at directive dispatch).
- `pub struct SessionFrame<...>`: search path list (each entry
  `/`-terminated at construction), `completed: BTreeSet<String>`,
  active-include stack of resolved paths (cycle scan), an owned
  `EvalContext` (persistence across commands and files comes from
  `run_source_with_context`-style stepping — `run_source` itself
  makes a fresh context per call and is NOT the frame's entry), the
  clean flag, and a SOURCE REGISTRY `SourceId -> (origin path,
  rewritten text, rewritten-line -> physical-line map)` filled during
  preprocessing. A fresh `SourceId` per file; spans stay
  file-absolute because each file is lexed in place, never sliced.
- COMMAND LOOP (per file, mirrors session.rs's existing loop): at
  each command start (empty token buffer), peek the RAW text at the
  lexer's current offset after skipping whitespace/comments; if `<`
  or `>`, scan the directive on raw text (the filename charset is
  untokenizable) and dispatch; if the `quit` keyword, end the
  session; otherwise drive `Lexer::next_token` to the Newline/Eof
  boundary and hand the tokens to the existing parse/eval path
  untouched. A command can never straddle a file boundary (each file
  has its own lexer ending in Eof).
- INCLUDE PROCESSING is recursive (each level locally owns its
  `SourceText` + `Lexer`, sidestepping self-referential borrows) with
  a `MAX_INCLUDE_DEPTH = 64` guard; the active-path cycle scan bounds
  real recursion, the guard is defense in depth. [R] The internals
  review preferred an explicit iterative stack; recursion is chosen
  for borrow simplicity — observable behaviour is identical (cycle
  skip, straddle rule, abandon cascade order), recorded here as a
  deliberate deviation.
- EVENTS: [R] NO new SessionEvent variants. `Starting to read...` /
  `Completely read...` are `SessionEvent::Output` text (byte-exact,
  produced only by the frame), so every existing consumer — the CLI
  arm, the corpus stdout comparison, session tests — works unchanged.
  Open failures and the abandon cascade are `Diagnostic`s with kind
  `Io` (stderr-routed); kind Io does NOT flip the clean flag.
- DIAGNOSTIC RENDERING: the CLI stops carrying (origin, text) pairs;
  `SessionFrame::describe(&Diagnostic) -> String` renders
  `path:line:col`, the offending REWRITTEN source line, and the caret
  from the registry, with line numbers mapped to PHYSICAL lines. The
  abandon message's `at line N` maps the current lexer offset the
  same way.
- REDIRECTS: at dispatch, open the sink (truncate for `>`, append for
  `>>`) BEFORE evaluating; stream the command's Value/Output text
  into it as produced; restore stdout routing unconditionally at
  command end (a failing command leaves a created/truncated file with
  partial output — upstream behaviour). Sink-open failure prints
  `Failed to open NAME` (no quotes, no period) to stderr and SKIPS
  evaluating the already-parsed command; clean flag untouched.
- atlas-cli: owns one `SessionFrame` for the whole invocation.
  [R] File arguments are fed as DEPTH-0 COMMAND STREAMS — today's
  semantics: no Starting/Completely framing for the argument files
  themselves, per-command error recovery, quit ends everything,
  `Bye.` at the end. This is a DELIBERATE deviation from upstream
  (whose non-option args are prelude files with captured output);
  the differential feeds upstream via stdin, so stdin-loop semantics
  for CLI args is exactly what makes outputs comparable.
  Prelude-capture mode stays a non-goal until the language needs it.
  Unreadable argument files keep the current hard exit(2).
  `--path=DIR` repeatable option, prefix form only.
  Exit code: 0 iff the frame's clean flag survived.

## Non-goals (deferred with their phase)

- `set`/`set_type`/`forget`/declarations and their report lines
  (phase B — the frame only carries the indentation depth).
- `>file whattype ...?` / `>file showall` redirect forms (with
  whattype/showall themselves).
- Prelude-capture mode, interactive prompt/banner, `quiet`/`verbose`,
  `$` last-value (REPL concerns).
- Making `input_path` a mutable user-space variable.

## Compatibility notes (checked against the trace and reviews)

- Identity is the RESOLVED path string, so the same file reached via
  two different search-path prefixes is two identities — faithfully
  ported, quirk included.
- A skipped include prints NOTHING (no Starting/Completely lines).
- The differential surface: the corpus harness runs upstream with
  `cwd = atlas-scripts`, so `<basic.at` resolves cwd-relative and the
  Starting line prints the RELATIVE spelling `basic.at` — the Rust
  side must be invoked the same way (see gate) rather than via
  `--path`, which would print absolute paths and break byte equality.

## Tests and gate [R — restated to be falsifiable]

- Unit (in-memory provider): filename scanning (quoted, unquoted
  charset, comment-skip before the name, empty name, trailing-token
  error); resolution order (path entries before cwd, `.at` appended
  only on failure and only when absent); include-once vs `<<`;
  aborted-file re-read; cycle silent-skip; literal-name shortcut
  (no output at all); abort cascade text and innermost-first order;
  clean-flag matrix (Io vs Syntax/Type/Runtime); redirect truncate
  vs append, partial output on failing command, sink-open-failure
  skip; continuation join (`foo\ ` included) mid-token; quit from
  inside an include; `Bye.` and `Value: `/void suppression.
- CLI integration: a two-file include chain through a temp dir,
  golden stdout including Starting/Completely framing, `Value: `
  lines, and final `Bye.`.
- [R] HARNESS CHANGE (part of this stage, not an afterthought):
  hpc/script_corpus_diff.py runs the Rust CLI with `cwd=scripts_dir`
  and the absolute script path, mirroring the C++ invocation, so
  includes resolve and path spellings match.
- [R] CORPUS GATE, falsifiable form: after the change, (i) ZERO
  `failed to open input file` diagnostics and ZERO include-directive
  parse errors across all 240 scripts; (ii) the expected single
  dominant bucket IS the finding: every `<basic.at` file now dies at
  basic.at's first real command (`set_type [...]`, basic.at:3) — that
  one line becomes the headline phase-B blocker, recorded in the
  commit message; (iii) the 2 current MATCHes must not regress, and
  `Value:`/`Bye.` alignment should flip the 2 OUTPUT_DIFF entries or
  the diff shrinks to content differences only.

## Review disposition

Fidelity: 7 findings — folded (splitter deleted in favour of the
lexer's boundary state; redirect narrowed to the same-parse-unit
expression form with open-before-eval and `Failed to open NAME`;
clean-flag semantics decoupled from Io diagnostics; harness cwd
change scheduled; `Bye.` added; CLI-args semantics re-justified as a
deliberate stdin-loop deviation; filename-scan minutiae folded).
Internals: 10 findings — folded (lexer-driven boundaries; depth-0
streams for CLI args; harness change; source registry + physical-line
map + `describe`; empty-tuple void test; quit pathway; streaming
redirect capture; lossy decode + Option semantics; recursion
deviation recorded with rationale; `run_source_with_context` named as
the persistence path). API: 5 findings — folded (gate restated
falsifiably around basic.at:3; harness invocation + categorization
documented; deliberate CLI-arg deviation; registry-backed rendering
replaces the CLI's (origin, text) pair; Starting/Completely as Output
events, failures as Io Diagnostics, no new variants).
No unresolved blocking items.
