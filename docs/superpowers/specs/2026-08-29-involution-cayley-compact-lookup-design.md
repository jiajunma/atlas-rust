# InvolutionTable Cayley Compact Lookup Design

## Goal

Remove the remaining production dependency of InvolutionTable::cayley on
InvolutionRecord::legacy_element. The method must resolve s * w from the
record-owned compact WeylElt, while preserving Atlas-visible results and
error ordering.

## Design

cayley(generator, id) first validates the source InvolutionId, then the
generator exactly as today. It copies the record's compact element, applies
CompactWeyl::inner_left_mult, and probes the existing table index with the
result. The packed/full permutation distinction remains encapsulated by the
index; cayley itself performs no root-permutation materialization. The legacy
permutation remains available only for compatibility/oracle consumers and
tests until the later record cleanup.

## Invariants and errors

- Invalid source IDs are reported before generator validation.
- Invalid generators report the table's generator bound.
- A target not yet added to the table returns Ok(None).
- A target already in the table returns the same InvolutionId as the legacy
  multiplication path.
- The method remains safe Rust and allocates no full permutation on the
  compact hot path.

## Verification

The RED test compares every B2 record/generator against the legacy oracle,
including None targets and both invalid-input precedence cases. The GREEN
focused HPC gate runs debug and release Weyl, InvolutionTable, and KGB tests;
the full differential and unipotent benchmark are separate gates.
