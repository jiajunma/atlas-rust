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

## Non-goals for the first release

- Preserving C++ object layout or ABI.
- Reproducing undocumented memory addresses, allocation order, or timing.
- Reimplementing CWEB as a runtime dependency.
- Accepting syntax that the reference rejects merely because Rust can parse it.
