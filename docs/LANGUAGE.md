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
| identifiers, reserved words, literals | supported | scalar/pipeline fixtures + the unterminated-string recovery; `3501467`, `3501643`, `3506272` |
| comments and source locations | supported | span-exact diagnostics across the B-slice rejected fixtures; `3501467` |
| arithmetic, comparison, boolean, assignment | supported | scalar goldens + Split dual numbers; `3501467`, `3502718` |
| precedence and associativity | supported | B11 corpus; `3501467` |
| declarations and scoped lookup | supported | B3a/B3c/B8 + declarations/let contracts; `3501467`, `3501643`, `3503356` |
| functions, arguments, returns, closures | supported | B3a/B3b fixtures; `3501467` |
| lists, tuples, maps, records, iteration | supported | containers/subscriptions/slices + B4 loops; `3501467`, `3503356` |
| constructors, overloads, implicit conversions | supported | B8 + b8c/b8d operator overloads and `whattype * ?`; `3501643` |
| exceptions and runtime errors | supported | B12 + rejected companions across all slices; `3501467`, `3501643` |
| Atlas commands and batch files | supported | B7 forget/die, B9 redirect, B10 include, B13 dont, showall, quit, set quiet/verbose; `3501467`, `3501643`, `3506272` |
| interactive input and completion | partial | TTY banner/prompt implemented; readline completion remains pending |
| domain objects and mathematical operations | supported | 152 of 152 domain contracts verified_hpc (231 of 231 across all categories; from display `3501467` through block/KL/param-transforms/ext/print slices; latest: ext_finalise `3542388`, twisted_family/block_deform `3542417`, print_partial_block `3542430`) |
| KL and file formats | planned | filekl.w is used only by stand-alone utilities; zero interpreter references — no Atlas-language builtin reads/writes KL binary files. Deferred outside the language-only gate pending a user decision (HANDOFF 2026-08-12b) |

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
`3503356`. The 77 frozen domain contracts of the 2026-07-31 checkpoint
are all verified (the last six — the K-type/standard-parameter family —
by differential `3506258`; the two last legacy contracts by `3506272`).
Since then the block/KL/extended-parameter wave landed: block
(`3503231`), strong_real, branch, KGP_sum, K_type_formula,
param_transforms, ext_block/ext_KL family (`3537192`),
print_gradings/real_weyl_print (`3538976`), print_X (`3540739`),
dual_KL_block (`3541634`), print_common_block (`3541690`),
shift_flip (`3541896`), ext_finalise (`3542388`), and
twisted_family/block_deform (`3542417`), and print_partial_block
(`3542430`).
As of 2026-08-13, all 231 fixture contracts across
{commands,domain,eval,lex,negative,parse} are verified_hpc — the
domain surface is fully closed (every upstream install_function name
is live and differential-verified). Readline completion (TTY-only) and
KL binary file formats (no Atlas-language builtin touches them; filekl.w
serves stand-alone utilities only) remain deferred outside the
language-only gate pending a user decision.

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
