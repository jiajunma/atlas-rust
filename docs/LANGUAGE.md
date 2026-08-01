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
| arithmetic, comparison, boolean, assignment | supported | scalar goldens + Split dual numbers; `3501467`, `3502718` |
| precedence and associativity | supported | B11 corpus; `3501467` |
| declarations and scoped lookup | supported | B3a/B3c/B8 + declarations/let contracts; `3501467`, `3501643`, `3503356` |
| functions, arguments, returns, closures | supported | B3a/B3b fixtures; `3501467` |
| lists, tuples, maps, records, iteration | supported | containers/subscriptions/slices + B4 loops; `3501467`, `3503356` |
| constructors, overloads, implicit conversions | supported | B8 + b8c/b8d operator overloads and `whattype * ?`; `3501643` |
| exceptions and runtime errors | supported | B12 + rejected companions across all slices; `3501467`, `3501643` |
| Atlas commands and batch files | supported | B7 forget/die, B9 redirect, B10 include, B13 dont, showall, quit; `3501467`, `3501643` |
| interactive input and completion | partial | TTY banner/prompt implemented; readline completion remains pending |
| domain objects and mathematical operations | partial | 77 of 77 frozen domain contracts verified: display `3501467`, root_coroot/kgb_generation `3501555`, real_group `3501779`, kgb_operations/tits_operations `3501870`, grading `3501915`, weyl_element `3502034`, cartan_aggregation `3502126`, seed_x0 `3502176`, involution_table `3502272`, adjoint_fiber `3502318`, real_form_labels `3502375`, relations `3502506`, involution_decomposition `3502550`, weak_real_form `3502697`/`3502969`, strong_real `3502718`/`3502731`/`3502736`, block `3503231`, involution_primitive `3503322`, ktype/param family `3506258` |
| KL and file formats | planned | explicit filekl adapter coupled to the pending Block/KL math layer |

No row moves to `supported` merely because Rust compiles. It needs a reference
corpus, Rust implementation, and HPC differential report.

## Current Language Slice

The typed session pipeline covers the B3a-B13 language slices, including
operator overload declarations, builtin `whattype * ?`, `dont` in the while
`do_expr` position, `showall`, `quit`, batch inclusion/redirection, and the
basic TTY banner/prompt. These surfaces are covered by HPC differential job
`3501643` (40 fixtures, all PASS); `showall`, `quit`, and the prompt are
covered by direct CLI/session checks. The eval family is complete through
`split_basic` (`3502718`), and the 21 legacy command/eval contracts
(declarations, assignments, let, containers, subscriptions, slices, exact
bignum numerics, name/type rejections, error recovery) are verified by
`3503356`. The domain layer is now complete: all 77 frozen domain
contracts are verified, the last six — the K-type/standard-parameter
family (`ktype_basic{,_rejected}`, `param_basic{,_rejected}`,
`ktypepol_basic`, `parampol_basic`) — by differential `3506258` on top of
the Rep_context crate milestone. Two legacy contracts remain frozen with
implementation gaps in flight: `set verbose` (L3) and the
unterminated-string recovery (L4). Readline completion and KL binary
formats remain outside the language-only gate because they depend on the
unfinished Block/KL domain values.

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
