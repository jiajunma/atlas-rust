# Atlas source and language support

## Reference source tree

The upstream [Atlas repository](https://github.com/jeffreyadams/atlasofliegroups)
is the normative source reference.

| Upstream area | Role | Atlas-Rust treatment |
|---|---|---|
| `sources/interpreter/*.w` | interpreter and evaluator CWEB sources | port behavior into `atlas-core`; do not copy generated files |
| `sources/interpreter/parser.y` | parser grammar reference | preserve grammar and diagnostics in parser fixtures |
| `sources/io/filekl.w` | KL/file input and output | implement as an explicit format adapter |
| `sources/stand-alone/*.w` | independent mathematical utilities | port after values and I/O stabilize |
| `sources/utilities`, `sources/structure`, `sources/gkmod`, `sources/interface` | C++ domain/UI implementation | map to domain crates and CLI in stages |
| `doc`, `messages`, `archive` | documentation, messages, historical/data assets | use as corpus and output references |

The `.w` files are CWEB sources and the interpreter is generated in C++ mode.
“C support” here means Atlas-language behavior, not preservation of C++ layout
or CWEB preprocessing.

## Language surface matrix

`planned` means the design is fixed but no compatibility claim may be made.

| Surface | Bootstrap status | Acceptance evidence |
|---|---|---|
| identifiers, reserved words, literals | planned | token-stream fixtures |
| comments and source locations | planned | lexer fixtures |
| arithmetic, comparison, boolean, assignment | planned | expression goldens |
| precedence and associativity | planned | parser ambiguity corpus |
| declarations and scoped lookup | planned | scope/error goldens |
| functions, arguments, returns, closures | planned | call/effect corpus |
| lists, tuples, maps, records, iteration | planned | value normalization goldens |
| constructors, overloads, implicit conversions | planned | type corpus |
| exceptions and runtime errors | planned | diagnostic/exit tests |
| Atlas commands and batch files | planned | event stream and exit tests |
| interactive input and completion | planned | pseudo-terminal HPC tests |
| domain objects and mathematical operations | planned | per-domain differential suites; `atlas-real-group` provides an initial structural API |
| KL and file formats | planned | byte/semantic comparisons |

No row moves to `supported` merely because Rust compiles. It needs a reference
corpus, Rust implementation, and HPC differential report.

## Source compatibility rules

- Preserve accepted syntax before adding extensions.
- Preserve rejection behavior for malformed or unsupported programs.
- Preserve evaluation order where it affects mutation, output, exceptions, or
  file writes.
- Preserve stable ordering of printed collections and serialized data.
- Preserve command exit status and diagnostic category.
- Keep implementation extensions opt-in until compatibility is complete.

## Corpus layout

```text
tests/fixtures/{lex,parse,eval,commands,domain,negative}/
tests/reference/<fixture>.events.json
tests/reference/<fixture>.meta.json
```

Large reference outputs and logs stay on HPC. Git stores small fixtures,
metadata, checksums, and summarized reports.
