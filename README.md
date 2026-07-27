# atlas-rust

Rust reimplementation of the [Atlas of Lie Groups](https://github.com/jeffreyadams/atlasofliegroups) interpreter.

The primary compatibility target is the Atlas language: existing Atlas source
files must parse, evaluate, and produce equivalent observable output. This
project is not a line-by-line C++ translation. It defines a Rust runtime around
the language and preserves compatibility at the parser, value, error, command,
and file-format boundaries.

## Current status

Project bootstrap. The compatibility contract is in
[`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md), the language support matrix is
in [`docs/LANGUAGE.md`](docs/LANGUAGE.md), the implementation design is in
[`docs/DESIGN.md`](docs/DESIGN.md), and the numeric model is in
[`docs/NUMERICS.md`](docs/NUMERICS.md). Contributor and agent rules are in
[`AGENTS.md`](AGENTS.md). Small focused checks may run locally; differential
tests and resource-heavy verification run on the XMU HPC according to
[`hpc/README.md`](hpc/README.md). Rust 1.90 or newer is required; the
repository follows the installed stable toolchain.

## Scope

- `atlas-core`: lexer, parser, values, evaluator, diagnostics, and compatible
  serialization primitives.
- `atlas-cli`: command-line interface and interactive session behavior.
- `atlas-real-group`: structural root-data, Weyl-action, and real-form APIs;
  differential domain compatibility remains planned; its staged design is in
  [`docs/REAL_GROUP_DESIGN.md`](docs/REAL_GROUP_DESIGN.md).
- `hpc`: reproducible build, test, and oracle-comparison jobs for XMU SLURM.

The original CWEB sources remain the behavioral reference for Atlas behavior.
The Rust implementation is organized by language boundaries, not by CWEB file
names. Generated C/C++ files are inspected and run only on HPC; they are not
copied into this repository as Rust source.

## License

GPL-3.0-or-later.
