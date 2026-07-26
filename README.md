# atlas-rust

Rust reimplementation of the [Atlas of Lie Groups](https://github.com/jeffreyadams/atlasofliegroups) interpreter.

The primary compatibility target is the Atlas language: existing Atlas source
files must parse, evaluate, and produce equivalent observable output. This
project is not a line-by-line C++ translation. It defines a Rust runtime around
the language and preserves compatibility at the parser, value, error, command,
and file-format boundaries.

## Current status

Project bootstrap. The compatibility contract and migration plan are recorded
in [`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md) and
[`docs/MIGRATION.md`](docs/MIGRATION.md). No local Rust execution is permitted;
builds and tests run on the XMU HPC according to [`hpc/README.md`](hpc/README.md).

## Scope

- `atlas-core`: lexer, parser, values, evaluator, diagnostics, and compatible
  serialization primitives.
- `atlas-cli`: command-line interface and interactive session behavior.
- `hpc`: reproducible build, test, and oracle-comparison jobs for XMU SLURM.

The original CWEB sources remain the behavioral reference. The PyCox and Atlas
outputs are treated as external oracles; generated artifacts are never edited
by hand.

## License

GPL-3.0-or-later.
