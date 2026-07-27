# Numeric model and overflow policy

## Decision

Atlas-Rust uses Malachite 0.10's pure-Rust `Integer` and `Rational` for the
Atlas language's exact integer and rational values. The workspace minimum Rust
version is therefore 1.90. This replaces `num-bigint` and `num-rational` in
the interpreter value layer.

Malachite is chosen as the single implementation for both exact types: it
avoids a native GMP dependency, provides canonical rational arithmetic, and
keeps small integers compact while scaling to large operands. It is GPL/LGPL
compatible with this GPL-3.0-or-later project.

## Boundary

The exact-language layer and the structural domain layer have different
requirements.

| Data | Representation | Rule |
| --- | --- | --- |
| Atlas `int` and `rat` values | Malachite `Integer` and `Rational` | Exact; no fixed-width overflow. |
| Root, coroot, weight, and coweight coordinates | Checked fixed-width signed integers | Preserve compact linear-algebra storage. |
| Matrix and pairing intermediates | At least `i128`, with checked operations | Narrow only through a checked conversion. |
| Exact integral kernel reduction | Malachite `Integer` with caller budgets | Track unimodular transformations; reject rank, aggregate-entry, operation, and coefficient-bit excesses. |
| Cartan-fiber denominator | Saturated Malachite kernel, then dynamic `F_2` reduction | Use `red_2 ker_Z(I+theta_Y)`, never a raw image or a second transpose. |
| Indices, dimensions, and allocation sizes | `usize` with checked conversions/arithmetic | Reject impossible shapes before allocation. |
| Explicitly modulo Atlas compatibility paths | Documented wrapping operations | Never use wrapping as a default. |

No global rank cap belongs to the mathematical root-datum model. It records
total lattice rank and semisimple rank separately. Atlas-compatible operations
that rely on the C++ `RANK_MAX = 32` representation enforce that local limit;
the Rust model itself remains dynamically sized.

## Atlas C++ evidence and Rust policy

The C++ interpreter's `int_value` is its custom arbitrary-precision
`arithmetic::big_int`, and rational language values use custom `big_rat`.
Narrowing accessors check range. In contrast, many root/weight paths use
`Vector<int>` and unchecked matrix arithmetic, so signed overflow is not
uniformly detected. A few KL polynomial paths check and report overflow, while
`SizeType` deliberately uses modulo arithmetic.

Rust makes this deterministic:

- language exact arithmetic remains unbounded;
- fixed-width domain operations return an explicit arithmetic or conversion
  error instead of panicking, silently wrapping, or relying on undefined C++
  signed overflow;
- wrapping is allowed only for a named Atlas compatibility operation whose
  C++ behavior is intentionally modulo;
- conversion from a language `Integer` to a domain coordinate is checked and
  reports the target range and source operation.

The migration must add boundary tests for minimum signed coordinates,
large language integers, division by zero, matrix accumulation overflow, and
invalid dimensions. Differential tests on the HPC remain the authority for
Atlas-visible accepted and rejected input behavior.

## Verification status

XMU HPC job `3462432` passed the `atlas-real-group` format check, Clippy with
warnings denied, and 67 structural unit tests against Rust 1.96 and Malachite
0.10. It ran from a frozen input snapshot
`a3e8d6472b20ab767ab5e41443ba29e56db0e9677e94fab2c4f8ff44d79e67f9`,
so its report describes the exact source Cargo consumed. It covers the exact
integral-kernel and resource-bound path, but does not establish Atlas language
compatibility. The existing `parse/exact_numerics` and `eval/exact_numerics`
reference metadata remain `pending_hpc_reference`; their Atlas-oracle reports
are still required for a language claim.
