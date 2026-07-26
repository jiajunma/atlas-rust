# Atlas-Rust design

## 1. Compatibility boundary

The project preserves behavior at the Atlas language boundary:

```text
Atlas source -> tokens -> AST -> name/type resolution -> evaluation
                                      |
                         domain objects and operations
                                      |
                 events -> text/JSON/files -> CLI process
```

The CWEB layout is an implementation reference, not the Rust module layout.
The upstream repository contains CWEB interpreter sources and ordinary C++
domain libraries. Atlas-Rust reproduces the language visible above those
layers.

## 2. Crate boundaries

### `atlas-core`

`atlas-core` has no terminal, readline, or process-exit policy. It contains:

- `source`: UTF-8 input, source IDs, one-based spans, and line mapping;
- `lex`: tokens, literals, comments, operators, and lexical diagnostics;
- `syntax`: parser and lossless-enough AST;
- `resolve`: scopes, declarations, overload sets, and name diagnostics;
- `value`: runtime values, containers, functions, domain handles, and void;
- `eval`: interpreter context, mutation, conversions, calls, errors, and events;
- `domain`: traits for Coxeter/root/KGB/block/representation objects;
- `format`: stable text and machine-readable event/file encodings.

The core must make global mutable state explicit in an `InterpreterContext`.

### `atlas-cli`

The CLI owns batch files, stdin, interactive prompts, history/readline
integration, output formatting, exit status, and signal handling. It translates
core events into reference-compatible presentation; the evaluator never prints.

### Optional domain crates

Domain implementations may be separated when dependencies differ:

- `atlas-coxeter`: Cartan matrices, roots, Coxeter elements, Bruhat/KL data;
- `atlas-real-group`: real reductive group and KGB structures;
- `atlas-io`: Atlas-specific persistent data and lookup tables.

These are design slots, not claims that the crates already exist.

## 3. Rustcox integration boundary

`rustcox-core` is a candidate source for the Coxeter/KL layer. An adapter must
translate, rather than expose, its internal indices and canonical JSON:

```text
Atlas value/domain operation -> AtlasDomain trait -> adapter -> rustcox-core
```

The adapter must pin ordering, weight conventions, errors, and serialization
with differential tests before it becomes a dependency. A future shared crate
is preferable to copying code, but only after both projects have stable APIs.

## 4. Events and errors

Evaluation produces an ordered event stream rather than writing to stdout:

- `Value`: a command or expression result;
- `Output`: user-visible text output;
- `Diagnostic`: lexical, syntax, name, type, runtime, or I/O error;
- `FileWrite`: path plus content hash and optional canonical bytes;
- `Exit`: reference exit status and termination reason.

The stream is the differential-test format. CLI text is a separate rendering
layer, so semantic equality can be checked independently of wording.

## 5. Memory and ownership

The runtime owns values through explicit handles or enums. Borrowed source text
is used only during lexing/parsing; AST and runtime strings own their data.
Cycles use an arena/handle design rather than pervasive interior mutability.

## 6. Build and generated-source policy

Atlas-Rust does not need CWEB at runtime. CWEB is used on HPC to inspect and
compare the reference implementation. Rust source is authoritative; generated
C/C++ files are never copied into the Rust crate as an unreviewed source dump.

## 7. Definition of done

A language feature is complete only when positive and negative fixtures exist,
the reference and Rust event streams agree on HPC, CLI/file behavior is checked
where applicable, and the report records the commit and toolchain.
