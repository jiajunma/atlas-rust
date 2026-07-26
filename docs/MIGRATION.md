# Migration plan

## Architecture

The implementation is split at language boundaries rather than copied by
original CWEB file:

```text
source -> lexer -> parser -> typed AST -> evaluator -> event sink -> CLI/files
                                  |
                             domain values
```

`atlas-core` owns the language and domain-independent runtime. Domain modules
are added behind explicit traits so the parser and evaluator do not depend on
the CLI. `atlas-cli` owns readline, batch execution, output formatting, and
process exit behavior.

## Stages

1. Freeze a reference corpus and event schema on HPC.
2. Implement lexer and source spans; differential-test token streams.
3. Implement parser and diagnostics; differential-test accepted/rejected
   programs.
4. Implement scalar/container values and the evaluator for a small language
   slice.
5. Port domain values and file formats, preserving canonical serialization.
6. Add interactive commands and compatibility output.
7. Expand the corpus until the complete Atlas language surface is covered.
8. Run full regression and large-domain jobs on HPC; publish artifacts and
   compatibility reports.

## Risk register

- Dynamic C++ object graphs and casts must become explicit Rust enums/traits.
- Existing global state and mutation must be isolated behind an interpreter
  context so borrow checking reflects actual lifetime rules.
- C++ exceptions need a stable Rust error/event model without changing command
  behavior.
- Templates and generated CWEB sections encode domain invariants that are not
  visible in the parser alone.
- Readline and platform-specific I/O must stay outside the core crate.

## Acceptance gates

No stage is complete on compilation alone. Each stage requires differential
tests against the reference executable, deterministic output, and a recorded
HPC job result. The final gate is a corpus-wide zero-difference report for
accepted programs plus matching diagnostics for rejected programs.
