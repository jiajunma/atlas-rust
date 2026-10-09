# Upstream delta 7e1b958c → 5ae51193: analysis — 2026-10-09

Read-only analysis of the 10-commit upstream delta discovered 2026-10-09
(receipt `tests/reference/hpc/upstream_head_2026_10_09.json`, inventory
`upstream_compare_2026_10_09.json`).  The oracle pin stays `7e1b958c`; this
note records what a future refresh would change and one confirmed
original-side defect at the pin.

## Confirmed original-side defect at the pin (fixed upstream)

Commit `509f584c` ("Fix tests for integer 0 in wrapper functions that forgot
to call |is_zero|") fixes a genuine bug class in `global.w`:

- `rat_divide_int_wrapper`: `if (i==0)` compared the **`shared_ptr`** to 0
  (always false) instead of the value — the `Rational division by zero`
  guard NEVER fires at the pin.  Fixed to `i->val.is_zero()`.
- `rat_modulo_int_wrapper`: same bug — `Rational modulo zero` never fires.
- (`int_inverse_wrapper`'s `i->val==0` → `is_zero()` is cosmetic; the value
  comparison already worked.)

So at the pin, rat/int division or modulo by zero falls through the wrapper
into the arithmetic library with unknown behavior (deeper error, wrong
value, or signal) — a labeled original-defect candidate in the sense of the
named-update UB precedent (the oracle defines behavior, but known original
defects are recorded as exceptions, never ported as goldens).  Probe fixture
drafted: `tests/math/generics/rat_int_zero_division.atlas` (sha
`7380908e0f9c9d1b2536bcf3677e5d07f6ea9bdefbe9881cc62870a86ae80a65`, prefix
`RZ_`, recovery 739; nonconstant zero via `rz_n+:=0` defeats the static
fold — the constant-folded `(literal)/0` rejects statically per the
void-boundary lessons; the known-clean int/int control `1/rz_n` runs FIRST
so it survives in the capture even if a probe signals).  Provisional and
unwired until an HPC capture shows the pinned oracle's actual behavior; the
refresh's fixed behavior is known from the diff.

## Other behavior-relevant delta items

- `repr.cpp` (85+/35-): `reducibility_points` substantially restructured
  (new documentation plus algorithmic reorganization of the real/complex
  progression logic).  Pinned goldens involving reducibility points (the
  unitarity lane; cf. the retained GL2R "Inexact halving" failure) may
  differ after a refresh — HPC replay will show it.
- `basic.at` (2+/2-): `status(vec,KGBElt)` now computes
  `root_index(x.root_datum,alpha)` instead of `root_index(real_form(x),…)`
  (companion to 255eeab4: built-in `status@(int,KGBElt)` clarified to work
  with arbitrary roots).  Script-visible after a refresh.
- `atlas-types.w`: `root_coradical`/`coroot_radical` switch to
  `sl_list<int_Vector_cref>` (0da00694) — output-invariant container change;
  KGB_cross/KGB_Cayley/KGB_status hoist the integer read (509f584c family).
  None of the G2 pre-registration's cited sections is touched.
- `rootdata.cpp`: const qualifier only.  `kgb.cpp/h`: comment-only status
  notes.  `matrix.cpp/h`: empty-vector UB robustness (71fc4c4c) — same
  theme as the project's named-update UB lesson.  `main.w`, `error.cpp/h`:
  macOS/readline portability.  New script `cell_graph.at` (75 lines);
  `Levi_subgroups.at` (41+/1-) adds `from_no_Cplus`/`minus` friend versions.
- Negative record: `axis.w` is NOT in the delta, so the documented
  field-assignment out-of-bounds UB (checked-build abort at
  `field_assignment::assign`, index 315 into a two-field tuple; see the
  named-update slice in AGENTS.md) persists at `5ae51193`.  The pin's
  labeled original defects stay labeled; none is silently fixed by this
  delta except the rat/int zero-division guards above.

## Refresh discipline (unchanged)

Historical pins and goldens stay bound to their capture pin; the refresh to
`5ae51193` (or later) is a separate reviewed transition requiring a complete
HPC replay against current master, expected to show deltas at least at the
three items above.  Never relabel existing evidence.
