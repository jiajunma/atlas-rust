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
`supported` requires reference corpus, Rust implementation, and an HPC
differential report naming the job.

| Surface | Bootstrap status | Acceptance evidence |
|---|---|---|
| identifiers, reserved words, literals | supported | scalar/pipeline fixtures; differentials `3501467`, `3501643` |
| comments and source locations | supported | span-exact diagnostics across the B-slice rejected fixtures; `3501467` |
| arithmetic, comparison, boolean, assignment | supported | scalar goldens; `3501467` |
| precedence and associativity | supported | B11 corpus; `3501467` |
| declarations and scoped lookup | supported | B3a/B3c/B8 fixtures; `3501467`, `3501643` |
| functions, arguments, returns, closures | supported | B3a/B3b fixtures; `3501467` |
| lists, tuples, maps, records, iteration | supported | containers/subscriptions/slices + B4 loops; `3501467` |
| constructors, overloads, implicit conversions | supported | B8 + b8c/b8d operator overloads and `whattype * ?`; `3501643` |
| exceptions and runtime errors | supported | B12 + rejected companions across all slices; `3501467`, `3501643` |
| Atlas commands and batch files | supported | B7 forget/die, B9 redirect, B10 include, B13 dont, showall, quit; `3501467`, `3501643` |
| interactive input and completion | partial | TTY banner/prompt implemented; readline completion remains pending |
| domain objects and mathematical operations | partial | 11 of 21 frozen domain contracts verified: display `3501467`, root_coroot/kgb_generation `3501555`, real_group `3501779`, kgb_operations/tits_operations `3501870`, grading `3501915`, weyl_element `3502034`, cartan_aggregation `3502126`, seed_x0 `3502176`, involution_table `3502272`; Block/KL layer pending |
| KL and file formats | planned | explicit filekl adapter coupled to the pending Block/KL math layer |

No row moves to `supported` merely because Rust compiles. It needs a reference
corpus, Rust implementation, and HPC differential report.

## Current Language Slice

The typed session pipeline covers the B3a-B13 language slices, including
operator overload declarations, builtin `whattype * ?`, `dont` in the while
`do_expr` position, `showall`, `quit`, batch inclusion/redirection, and the
basic TTY banner/prompt. These surfaces are covered by HPC differential job
`3501643` (40 fixtures, all PASS); `showall`, `quit`, and the prompt are
covered by direct CLI/session checks. The domain layer is partially ported:
RootDatum root/coroot queries, KGB size/status, real-form numbering and
dual forms, KGB decompose/twist, the grading observables, Weyl elements,
the Cartan-class aggregation surface, synthetic KGB seeds, and the
KGB/strong-real printers are verified by differentials `3501555`,
`3501779`, `3501870`, `3501915`, `3502034`, `3502126`, `3502176`, and
`3502272`; adjoint fibers,
real-form labels and block sizes, synthetic real forms, involution
decomposition, blocks, K-types, parameters, and the primitive involution
constructors have frozen contracts awaiting implementation. Readline
completion and KL binary formats remain outside the language-only gate
because they depend on the unfinished Block/KL domain values.

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
