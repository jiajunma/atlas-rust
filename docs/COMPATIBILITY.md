# Atlas language compatibility contract

## Goal

An existing Atlas source program is compatible when the Rust implementation
preserves its accepted syntax, evaluation result, observable diagnostics, and
documented command/file behavior. Internal data structures and implementation
language are not compatibility requirements.

## Compatibility layers

1. **Lexical layer**: identifiers, numeric and string literals, comments,
   operators, source locations, and end-of-input behavior.
2. **Parsing layer**: declarations, expressions, commands, type constructors,
   precedence, associativity, and syntax diagnostics.
3. **Value layer**: integers, rational values, strings, lists, tuples, maps,
   functions, algebraic/domain values, and null/void behavior.
4. **Evaluation layer**: name lookup, mutation, overload resolution, implicit
   conversions, exceptions, completion behavior, and deterministic ordering.
5. **I/O layer**: file formats, command output, error text categories, and
   interactive versus batch mode behavior.

## Compatibility oracle

The original Atlas executable is the primary behavior oracle. Every migrated
feature gets a corpus of source snippets and expected structured results. Text
goldens are retained for CLI compatibility, while semantic goldens compare a
normalized event stream (values, diagnostics, file writes, and exit status).

The current official `master` oracle revision is
`7e1b958c7aa9456769cc9cf09ac1542814b4800a` (rechecked2026-10-01). This does
not retroactively refresh old captures:347 reference metadata files still
name the earlier `4d3e9449062a07c1c85f4e6df215eb6ccc0eeae9` executable. They
remain valid historical evidence only at that exact pin. Until their accepted
and rejected streams are replayed on an HPC compute node against `7e1b958c`,
they cannot support a blanket “latest original” compatibility claim and their
revision fields must not be edited in place. New mathematical gates already
bind the newer oracle, and the path-independent parent seal/root-ladder repair
sequence precedes this full language-corpus replay.
UPDATE2026-10-10: upstream `master` moved to
`5ae51193cbb8faa022847701195bba81702b1605` (ten commits ahead of the pin;
verified 2026-10-09). The pin remains `7e1b958c` for all frozen evidence; the
pin-refresh is a separate queued HPC-replay transition, and the delta analysis
(including the upstream fix of the pinned rat/int zero-division guard,
`509f584c`) is recorded in
[`slices/upstream_5ae51193_delta_2026_10_09.md`](slices/upstream_5ae51193_delta_2026_10_09.md).
The official GitHub comparison is substantial rather than clerical: the newer
revision is494 commits ahead and changes257 reported paths, including182
`atlas-scripts` files,71 `sources` files, all principal interpreter CWEB/parser
inputs, and scripts used by unitarity, Hodge, KLV and AV-ann. The bounded
read-only inventory is frozen in
`tests/reference/hpc/upstream_compare_2026_10_01.json`; it proves the need for
replay, not that any observable result changed.

The replay corpus has a precise boundary. There are353 `.atlas` language
fixtures, of which346 have a concrete event file and per-fixture metadata;
seven `relations_*_probe` inputs have no event contract and remain probes. The
347th metadata document is only the aggregate `eval/scalar_errors` record. All
346 concrete records still name `4d3e9449`. The current
`pipeline_swap_diff.py` has337 fixture plans and intentionally runs only Rust
against checked-in events; it hard-codes the old revision and therefore cannot
certify the latest original. The nine event-backed fixtures outside that plan
are `domain/deform`, `eval/back_trace_loops`, `eval/case_dot_label`,
`eval/for_iterable_kinds`, `eval/for_quiet_body`, `eval/for_reversed`,
`eval/for_reversed_extra`, `eval/last_value` and `eval/op_cast`.

`reference_capture.py` can collect current-original raw streams, status,
time/RSS and source/binary/script hashes without mutating expectations. Of the
346 old concrete metadata records,199 retain exact raw stdout/stderr hashes;
those can first be classified by byte equality. The remaining147 retain exit
status and verified events but not both raw hashes, so they require either
recovery of the immutable old raw artifact or explicit normalization and
independent review. A future replay report must preserve these classes and all
changed/rejected cases; neither a successful Rust-only pipeline run nor
absence of a raw hash counts as latest-original agreement.

The frozen outcome boundary for those346 cases is200 exit-zero and146
exit-one. Exit status, timeout/resource state and ordered semantic events must
be classified separately: the successful `eval/fromfile_b10` and
`negative/unterminated_string` controls intentionally emit diagnostics, so an
empty diagnostic stream is neither necessary nor sufficient for acceptance.
The existing harnesses also cannot certify all346 cases unchanged:
`eval/fromfile_accepted_b10.atlas` embeds its helper's legacy absolute checkout
path, while `eval/file_commands_b9.atlas` writes a fixed `/tmp` path that the
capture harness neither isolates nor verifies. A replay must provide a proven
private/path-mapped environment and capture the side effect, or report these as
isolation blockers rather than silently rewriting the fixtures.

After the parent seal and ladder AFTER are FINAL and inspected, use two
sequential jobs, never a prequeued pair. The first is one fail-closed
capture/differential job over all346 concrete cases against current original
and the accepted post-ladder Rust artifact; it retains raw streams, outcome,
ordered events, source/script/binary hashes, wall time and RSS, and labels its
report unreviewed. For the147 cases without both old raw hashes, it must either
recover a pinned `4d3e9449` raw baseline or state the resulting comparison
limitation. Only after that report is inspected may a distinct review job bind
and rehash its exact case set and artifacts, check the200/146 classes and the
two diagnostic controls, and issue a separate machine-readable decision.
Append `7e1b958c` provenance and any changed-case records; never edit or relabel
the old metadata, and claim no acceptance before the independent review.
The frozen counts, four path-sensitive cases, manifest hashes, isolation
requirements and sequential scheduler plan are recorded in
[`slices/language_corpus_transition_2026-10-01.md`](slices/language_corpus_transition_2026-10-01.md).
The local manifest helper remains inventory-only and unexecuted: no manifest
JSON, current-original replay or compatibility decision exists yet.

## What "fully compatible" means

The target is source compatibility at the Atlas language level. A program that
is accepted by the reference must be accepted by Rust Atlas with equivalent
evaluation and observable effects; a reference-rejected program must remain
rejected with the same diagnostic category and exit behavior. Exact prose is
tracked separately from semantic equality.

Compatibility includes batch files, command sequencing, mutation, evaluation
order, deterministic collection/file ordering, domain operation results, and
documented persistent formats. It does not include C++ ABI, object addresses,
allocator order, compiler diagnostics, or undocumented timing.

The complete surface and its current status are maintained in
[`LANGUAGE.md`](LANGUAGE.md). No language row is supported until an HPC
differential report exists.

## Non-goals for the first release

- Preserving C++ object layout or ABI.
- Reproducing undocumented memory addresses, allocation order, or timing.
- Reimplementing CWEB as a runtime dependency.
- Accepting syntax that the reference rejects merely because Rust can parse it.
