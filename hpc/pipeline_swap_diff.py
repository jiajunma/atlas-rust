#!/usr/bin/env python3
"""Compare atlas-cli with the frozen typed-pipeline Atlas event fixtures.

The checked-in event files are the oracle.  This driver deliberately runs
only the Rust interpreter.  Fixture lines whose Rust builtins are not yet
implemented are omitted from the runnable input and recorded as explicit
pending coverage, so a partial port can never be reported as a full pass.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import platform
import re
import resource
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from typing import Any


PINNED_ATLAS_REVISION = "4d3e9449062a07c1c85f4e6df215eb6ccc0eeae9"
COMMIT_TOKEN = re.compile(r"^(?:[0-9a-fA-F]{40}|unversioned)$")
DIRTY_TREE_TOKENS = {"true", "false", "unknown"}


@dataclass(frozen=True)
class PendingCase:
    feature: str
    source_line: int
    reference_event: int
    reason: str


@dataclass(frozen=True)
class FixturePlan:
    name: str
    runnable_lines: tuple[int, ...] | None = None
    runnable_events: tuple[int, ...] | None = None
    pending: tuple[PendingCase, ...] = ()
    # Lines that are executed as part of the runnable input but produce no
    # observable event (for example output redirected to a file).
    silent_lines: tuple[int, ...] = ()


# These overloads are present in the Atlas oracle but are intentionally not
# registered until their owning Rust domain types and semantics are ported.
# They are coverage gaps, not fixture failures.
PENDING_OVERLOADS = ()


FIXTURE_PLANS = (
    FixturePlan(name="eval/pipeline_swap_constructors"),
    # RootDatum/InnerClass/RealForm construction, full domain renderings,
    # KGBElt display, and the equality/inequality relations are all ported.
    FixturePlan(name="eval/pipeline_swap_domain_equality"),
    FixturePlan(name="eval/pipeline_swap_linear_values"),
    FixturePlan(name="eval/pipeline_swap_rejected"),
    FixturePlan(name="eval/pipeline_swap_void_reports"),
    # B3a non-recursive functions: typed lambdas, closure capture, return at
    # the call boundary, and identifier selectors.
    FixturePlan(name="eval/functions_b3"),
    FixturePlan(name="eval/functions_b3_rejected"),
    # B3b recursive functions and let-declaration definition sugar.
    FixturePlan(name="eval/functions_b3b"),
    FixturePlan(name="eval/functions_b3b_rejected"),
    # B3c parameter patterns: tuple destructuring, discard, and const patterns.
    FixturePlan(name="eval/patterns_b3c"),
    FixturePlan(name="eval/patterns_b3c_rejected"),
    # Atlas 0.9.1 multiple assignment: recursive tuple destinations, omitted
    # slots, whole-value targets, mixed global/local writes, and exact target
    # analysis diagnostics (parser.y:264; axis.w:6956-7500).
    FixturePlan(name="eval/multi_assignment"),
    FixturePlan(name="eval/multi_assignment_rejected"),
    # P1 builtin reconciliation: Weyl left/right products, Cartan class from
    # KGB, and unary/list KTypePol/ParamPol operations. The reference events
    # were frozen by HPC oracle capture 3543697 before the Rust port.
    FixturePlan(name="domain/p1_simple_signatures"),
    FixturePlan(name="domain/p1_simple_signatures_rejected"),
    # Twisted full deformation recursion over interval-below common blocks;
    # local oracle capture 2026-08-20 is byte-identical for integral,
    # half-integral, and non-final B2 parameters.
    FixturePlan(name="domain/twisted_full_deform_proper"),
    # Twisted deformation on proper integral subsystems (24fab16, slice 4):
    # pb/pa empty-sum anchors, non-empty q2 terms on a cold pool, and the
    # non-final/overload rejections. Reference frozen by HPC capture 3591165.
    FixturePlan(name="domain/twisted_deform_proper"),
    FixturePlan(name="domain/twisted_deform_proper_terms"),
    FixturePlan(name="domain/twisted_deform_proper_rejected"),
    # full_deform over partial common blocks (466e066, next-wave B): B2
    # integral/half-integral/non-final anchors. Reference frozen by HPC
    # capture 3586752.
    FixturePlan(name="domain/full_deform_proper"),
    # P2 Block W-graph overloads captured by HPC oracle 3543699.  The
    # full-integral A1 `block(Param)` path now uses the shared RepTable lookup;
    # the proper-integral KL_column case is covered by the shared subsystem
    # lookup as well.
    FixturePlan(
        name="domain/p2_block_graph_signatures",
        runnable_lines=(2, 4, 6, 8, 10, 11, 12, 14, 15, 16, 17),
        runnable_events=(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16),
        silent_lines=(1, 3, 5, 7, 9, 13),
    ),
    FixturePlan(
        name="domain/p2_block_graph_signatures_rejected",
        runnable_lines=(2, 4, 6, 7, 8, 9),
        runnable_events=(0, 1, 2, 3, 4, 5, 6, 7, 8),
        silent_lines=(1, 3, 5),
    ),
    # B3d selectors: unit selector and operator selectors.
    FixturePlan(name="eval/selectors_b3d"),
    FixturePlan(name="eval/selectors_b3d_rejected"),
    # B4 loops: while/for value collection, break, and loop rejections.
    FixturePlan(name="eval/loops_b4"),
    FixturePlan(name="eval/loops_b4_rejected"),
    # B5 set_type: user-defined types, union display, and case discrimination.
    FixturePlan(name="eval/settype_b5"),
    FixturePlan(name="eval/settype_b5_rejected"),
    # B6 case and counted for: integer case selection, union case, and
    # from/downto loops.
    FixturePlan(name="eval/casefor_b6"),
    FixturePlan(name="eval/casefor_b6_rejected"),
    # B7 misc commands: forget, die, and coercion after overload removal.
    FixturePlan(name="eval/commands_b7"),
    FixturePlan(name="eval/commands_b7_rejected"),
    # Combined assignment family (80518bd): component/field assignment and
    # op:= transforms on globals, row locals, let bindings, and set_type
    # projectors. Reference frozen by capture 3585649; events.json
    # pre-verified (oracle and CLI diagnostics cross-checked equal).
    FixturePlan(name="eval/combined_assignment"),
    FixturePlan(name="eval/combined_assignment_rejected"),
    # B8 user overloads: definition accumulation, redefinition, listing, and
    # wrong-arity rejection.
    FixturePlan(name="eval/overloads_b8"),
    FixturePlan(name="eval/overloads_b8b"),
    FixturePlan(name="eval/overloads_b8_rejected"),
    FixturePlan(name="eval/overloads_ops_b8c"),
    FixturePlan(name="eval/overloads_ops_b8c_rejected"),
    FixturePlan(name="eval/whattype_ops_b8d"),
    # B13 do-expression termination: `dont` is admitted only after a
    # semicolon in a while condition, not as a plain expression after `do`.
    FixturePlan(name="eval/dont_b13"),
    FixturePlan(name="eval/dont_b13_rejected"),
    # B9 file commands: tofile/addtofile redirection and its rejections.
    # The two redirect lines run but produce no stdout event.
    FixturePlan(
        name="eval/file_commands_b9",
        runnable_lines=(3,),
        runnable_events=(0,),
        silent_lines=(1, 2),
    ),
    FixturePlan(name="eval/file_commands_b9_rejected"),
    # B10 fromfile inclusion errors and quit semantics. The quit line and
    # the unreachable line after it run but produce no event.
    FixturePlan(name="eval/fromfile_b10"),
    # B10 accepted inclusion: line 3 is a silent skip (file already seen).
    FixturePlan(
        name="eval/fromfile_accepted_b10",
        runnable_lines=(1, 2, 4),
        runnable_events=(0, 1, 2),
        silent_lines=(3,),
    ),
    FixturePlan(
        name="eval/quit_b10",
        runnable_lines=(1,),
        runnable_events=(0,),
        silent_lines=(2, 3),
    ),
    # B11 precedence/associativity corpus and B12 runtime-error corpus.
    FixturePlan(name="eval/precedence_b11"),
    FixturePlan(name="eval/runtime_errors_b12"),
    # Legacy command/eval contracts frozen verbatim by capture job 3503334:
    # declarations/assignments/let, container and subscription behavior, exact
    # bignum numerics, undefined-name and wrong-type rejections, and error
    # recovery across lines.
    FixturePlan(name="commands/assignments"),
    FixturePlan(name="commands/assignment_order"),
    FixturePlan(name="commands/container_assignments"),
    FixturePlan(name="commands/declarations"),
    FixturePlan(name="commands/declaration_errors"),
    FixturePlan(name="commands/let_errors"),
    FixturePlan(name="commands/let_error_order"),
    # L1 diagnostic wordings (agent-28): assignment/slice/subscription and
    # container-list messages match the oracle's source-text appending and
    # the `No common type found` wording.
    FixturePlan(name="commands/assignment_errors"),
    FixturePlan(name="commands/slice_errors"),
    FixturePlan(name="commands/subscription_errors"),
    FixturePlan(name="eval/container_errors"),
    # L2 bison syntax messages (agent-29): the `syntax error, unexpected X,
    # expecting Y` wording, with recovery continuing at the next line.
    FixturePlan(name="parse/negative_trailing_token"),
    FixturePlan(name="commands/invalid_token_continues"),
    FixturePlan(name="commands/mismatched_delimiter_continues"),
    FixturePlan(name="commands/nested_invalid_token_continues"),
    # L3 `set quiet`/`set verbose` option commands (parser.y:171-178) with
    # the verbose analysis trace (main.w:495-516, 528-540), and L4's
    # unterminated-string lexical recovery (lexer.w:311-320): a warning
    # diagnostic that does not flip the exit status.
    # The `set verbose` line itself produces no event (silent); line 3's
    # trace + value span events 0-3, and line 2's syntax diagnostic is
    # event 4 — the source/event mapping is not one-to-one.
    FixturePlan(
        name="lex/basic",
        runnable_lines=(2, 3),
        runnable_events=(0, 1, 2, 3, 4),
        silent_lines=(1,),
    ),
    # The recovered string evaluates to a Value and reports the lexical
    # warning in one source line.
    FixturePlan(
        name="negative/unterminated_string",
        runnable_lines=(1,),
        runnable_events=(0, 1),
    ),
    # The dangling `[` line is excluded: the oracle saw the capture-time
    # appended `quit` (`unexpected QUIT, expecting ']'`) where the CLI sees
    # EOF, so that event stays pending; the `4` line after it belongs to the
    # same unclosed command (swallowed by the open bracket) and produced no
    # oracle event of its own, so it is pending on the same event.
    FixturePlan(
        name="commands/container_syntax_errors",
        runnable_lines=(1, 2, 3, 4, 5, 6),
        runnable_events=(0, 1, 2, 3, 4, 5),
        pending=(
            PendingCase(
                feature="dangling open bracket sees capture-time quit",
                source_line=7,
                reference_event=6,
                reason="the oracle parsed the harness-appended `quit` where the CLI sees EOF",
            ),
            PendingCase(
                feature="swallowed line after the dangling open bracket",
                source_line=8,
                reference_event=6,
                reason="the oracle's unclosed `[` swallows this line before quit",
            ),
        ),
    ),
    # The final let command spans lines 12-14; the two continuation lines are
    # part of the runnable input but produce no event of their own.
    FixturePlan(
        name="commands/let",
        runnable_lines=(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12),
        runnable_events=(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11),
        silent_lines=(13, 14),
    ),
    FixturePlan(name="commands/ordered_events"),
    FixturePlan(name="commands/slice_order"),
    FixturePlan(name="commands/subscription_context"),
    FixturePlan(name="commands/subscription_order"),
    FixturePlan(name="eval/containers"),
    # global.w batch 1: rat floor/ceil/frac, string ##/ascii, cardinality #
    # on string/vec/ratvec/mat, matrix shape/row/column/rows/columns
    # (global.w:3249-3251, 4387-4404); reference frozen by capture 3574819.
    FixturePlan(name="eval/global_batch1"),
    FixturePlan(name="eval/global_batch1_rejected"),
    # global.w batch 2: int bit utilities, container relations/arithmetic,
    # selectors/joins, matrix constructors, gcd(vec), elapsed_ms
    # (global.w:2859-2994, 3953-4451, 5183-5245); reference frozen by
    # capture 3574906.
    FixturePlan(name="eval/global_batch2"),
    FixturePlan(name="eval/global_batch2_rejected"),
    # global.w batch 3: exact linear algebra (Bezout, echelon,
    # linear_solve, diagonalize, adapted_basis, kernel, eigen_lattice,
    # row_saturate, Smith, invert); reference frozen by capture 3574944.
    FixturePlan(name="eval/global_batch3"),
    FixturePlan(name="eval/global_batch3_rejected"),
    # global.w batch 4 (final global.w slice): swiss_matrix_knife slicer,
    # mod2_section, subspace_normal; reference frozen by capture 3576078.
    FixturePlan(name="eval/global_batch4"),
    FixturePlan(name="eval/global_batch4_rejected"),
    FixturePlan(name="eval/slices"),
    FixturePlan(name="eval/subscriptions"),
    FixturePlan(name="eval/context"),
    FixturePlan(name="eval/exact_numerics"),
    FixturePlan(name="eval/negative_type"),
    FixturePlan(name="eval/negative_undefined"),
    FixturePlan(name="parse/basic"),
    FixturePlan(name="parse/exact_numerics"),
    # RootDatum root/coroot queries: oracle presentation order, negative-index
    # negation, long/short flags, rank, and the illegal-index rejection.
    FixturePlan(name="domain/root_coroot"),
    FixturePlan(name="domain/root_coroot_rejected"),
    # Batch 1 of the remaining-builtin ledger (docs/REMAINING_BUILTINS.md):
    # simple_roots/simple_coroots matrices, the is_Cartan_matrix predicate,
    # and dual_datum(InnerClass).
    FixturePlan(name="domain/simple_roots"),
    # Batch 2 (KGB Bruhat printers): print_KGB_order (Hasse rows +
    # comparable-pair count) and print_KGB_graph (Graphviz digraph).
    FixturePlan(name="domain/kgb_bruhat"),
    # Batch 3 (root/radical data): root_coradical / coroot_radical print
    # the simple (co)roots plus the kernel basis of the (co)roots.
    FixturePlan(name="domain/radical"),
    # Batch 3 (remainder): components_rank (dual component-group rank) and
    # strong_components (Tarjan + induced quotient graph).
    FixturePlan(name="domain/components"),
    # Batch 4 (KL printers): print_KL_basis / print_prim_KL / print_KL_list
    # over the block's KL table, plus BlockGraph::bruhat_hasse.
    FixturePlan(name="domain/kl_print"),
    # Batch 4 (block printers): print_block / print_blockd / print_blocku.
    FixturePlan(name="domain/block_print"),
    # print_block_words: regression pin for the '*' right-alignment and
    # WeylGroup::word tie-break fixes (2026-08-13). Type ascriptions silent,
    # each `:=` -> Declaring+Value, each print_* call -> one Output per row.
    # 23 lines/77 events; alignment pre-analysed against
    # print_block_words.events.json.
    FixturePlan(
        name="domain/print_block_words",
        runnable_lines=(2, 4, 6, 8, 10, 11, 13, 15, 17, 19, 21, 22, 23),
        silent_lines=(1, 3, 5, 7, 9, 12, 14, 16, 18, 20),
    ),
    # print_block_words_rejected: ascriptions silent, 3 declarations ->
    # Declaring+Value, 3 erroring calls -> 1 Diagnostic each (6 lines/9
    # events).
    FixturePlan(
        name="domain/print_block_words_rejected",
        runnable_lines=(2, 4, 6, 7, 8, 9),
        silent_lines=(1, 3, 5),
    ),
    # prim_kl_order: regression pin for the prim_KL ascending-order and
    # P_{y,y} pad fixes (2026-08-13). Ascriptions silent, `:=` ->
    # Declaring+Value, print_prim_KL -> one Output per line (11
    # lines/111 events).
    FixturePlan(
        name="domain/prim_kl_order",
        runnable_lines=(2, 4, 6, 8, 10, 11),
        silent_lines=(1, 3, 5, 7, 9),
    ),
    # prim_kl_order_rejected: ascription silent, 1 declaration ->
    # Declaring+Value, 2 erroring calls -> 1 Diagnostic each (3 lines/4
    # events).
    FixturePlan(
        name="domain/prim_kl_order_rejected",
        runnable_lines=(2, 3, 4),
        silent_lines=(1,),
    ),
    # Batch 3 completion: components_rank / strong_components.
    FixturePlan(name="domain/components_rank"),
    # Batch 8 (misc): KGB_Hasse matrix.
    FixturePlan(name="domain/kgb_hasse"),
    # Batch 8 (misc): Cartan_info.
    FixturePlan(name="domain/cartan_info"),
    # Batch 8 (misc): orientation_nr.
    FixturePlan(name="domain/orientation_nr"),
    # Batch 8 (misc): block_Hasse.
    FixturePlan(name="domain/block_hasse"),
    FixturePlan(name="domain/block_hasse_param_proper"),
    # Batch 5 (W-graph): W_graph/W_cells over a parameter.
    FixturePlan(name="domain/w_graph_param"),
    FixturePlan(name="domain/w_graph_param_proper"),
    # Batch 5 (KL access): raw_KL.
    FixturePlan(name="domain/raw_kl"),
    # Batch 5 (KL access): KL_sum_at_s.
    FixturePlan(name="domain/kl_sum_at_s"),
    FixturePlan(name="domain/kl_sum_at_s_param_proper"),
    # Batch 1: dual_datum.
    FixturePlan(name="domain/dual_datum"),
    # Batch 1: is_Cartan_matrix.
    FixturePlan(name="domain/is_cartan_matrix"),
    # Batch 6: extend_Lie_type.
    FixturePlan(name="domain/extend_lie_type"),
    # Batch 6: default_extended.
    FixturePlan(name="domain/default_extended"),
    # Batch 6: partial_block.
    FixturePlan(name="domain/partial_block"),
    FixturePlan(name="domain/partial_block_param_proper"),
    FixturePlan(name="domain/partial_block_param_proper_shift"),
    FixturePlan(name="domain/partial_block_param_proper_rejected"),
    # Batch 7: KL_block.
    FixturePlan(name="domain/kl_block"),
    # Batch 7: full_deform.
    FixturePlan(name="domain/full_deform"),
    # Timed full_deform overload: cooperative deadline, cache-sensitive
    # zero/negative timers, no-value validation, and exact union injectors.
    FixturePlan(name="domain/timed_full_deform_signatures"),
    FixturePlan(name="domain/timed_full_deform_cache"),
    FixturePlan(name="domain/timed_full_deform_timeout_zero"),
    FixturePlan(name="domain/timed_full_deform_timeout_negative"),
    # Batch 6: partial_KL_block (first extended-block surface).
    FixturePlan(name="domain/partial_kl_block"),
    # Batch 5: KL_column (partial block), including the A2 proper-integral
    # subsystem case now handled by the subsystem-aware RepTable path.
    FixturePlan(name="domain/kl_column"),
    # Shared Rep_table sequencing: only value-demanded full materializers
    # install the family; no-value calls and direct printers do not warm it.
    FixturePlan(name="domain/rep_table_sequence"),
    FixturePlan(name="domain/rep_table_sequence_rejected"),
    FixturePlan(name="domain/p0_simple_signatures"),
    FixturePlan(name="domain/p0_simple_signatures_rejected"),
    # Cross-block partial merge (RepTable::commit_partial merge port);
    # reference frozen by capture 3575819.
    FixturePlan(name="domain/partial_merge_containment"),
    FixturePlan(name="domain/partial_merge_union"),
    FixturePlan(name="domain/partial_merge_chain"),
    FixturePlan(name="domain/partial_merge_a2"),
    # Locator step 4: consumers transported through the RepTable pool's
    # block modifier — print_c_block_wrapper header/shift, modifier-aware
    # singular flags, and sr(srm,bm,gamma) row reconstruction. References
    # frozen by captures 3574723 (w=<1>), 3574854 (<0.2> permuted), and
    # 3574845 (rank-0, <0.1.0>).
    FixturePlan(name="domain/common_block_locator"),
    FixturePlan(name="domain/common_block_simple_pi"),
    FixturePlan(name="domain/common_block_rank0_locator"),
    # Slice 1B (twisted_ext_proper_workorder.md): extended_block on a
    # proper integral subsystem — B2 split form 2 pb (rank-1 subsystem,
    # 3-element ext block) with A2/C2 integral controls. Reference frozen
    # by capture 3574900; events.json pre-verified.
    FixturePlan(name="domain/ext_block_proper"),
    # Slice 2: raw_ext_KL + partial_extended_KL_block on proper integral
    # subsystems (B2/A2/C2 proper + B2/A1 rank-0). Reference frozen by
    # capture 3585276; events.json pre-verified.
    FixturePlan(name="domain/ext_kl_proper"),
    FixturePlan(name="domain/ext_kl_proper_rejected"),
    # Slice 3 (63e8118): twisted_KL_sum_at_s proper-subsystem overloads —
    # B2 split form 2 and A2 su(2,1) anchors, bare and explicit-twist mat
    # forms, with semifinal/distinguished runtime rejections and
    # argument-shape type errors. Reference frozen by capture 3585649;
    # events.json pre-verified.
    FixturePlan(name="domain/twisted_kl_proper"),
    FixturePlan(name="domain/twisted_kl_proper_rejected"),
    # dual_KL_block rewired onto the located-block machinery
    # (PartialBlock::dual via lookup_full_block); B2 proper-subsystem and
    # A2 split anchors, events.json pre-verified by capture 3574902.
    FixturePlan(name="domain/length_dual_proper"),
    FixturePlan(name="domain/length_dual_proper_a2"),
    # Batch 3 (root data): two_rho / two_rho_check.
    FixturePlan(name="domain/two_rho"),
    FixturePlan(name="domain/cofolded"),
    FixturePlan(name="domain/block_sizes"),
    FixturePlan(name="domain/fundamental"),
    FixturePlan(name="domain/simple_factors"),
    FixturePlan(name="domain/cartan_matrix_type"),
    FixturePlan(name="domain/integrality"),
    # KGB headline observables: per-form KGB sizes and root statuses across
    # the A1/C2/A2 families, plus the inexistent-element and type rejections.
    FixturePlan(name="domain/kgb_generation"),
    FixturePlan(name="domain/kgb_generation_rejected"),
    # Real-form numbering, form names, and dual real-form construction for
    # A1/C2/A2, including the exact illegal external-number diagnostic.
    FixturePlan(name="domain/real_group"),
    FixturePlan(name="domain/real_group_rejected"),
    # KGB element operations: cross/Cayley/status/length, torus_factor,
    # equality, the `%` decompose, the distinguished twist, and the
    # illegal-generator rejection.
    FixturePlan(name="domain/kgb_operations"),
    FixturePlan(name="domain/kgb_operations_rejected"),
    # Tits twists: distinguished twist on KGB elements and the outer twist
    # by a matrix, including the unbased-involution rejection.
    FixturePlan(name="domain/tits_operations"),
    FixturePlan(name="domain/tits_operations_rejected"),
    # Grading slice: base_grading_vector/initial_torus_bits per real form
    # and torus_bits per KGB element, plus the RootDatum-argument rejection.
    FixturePlan(name="domain/grading"),
    FixturePlan(name="domain/grading_rejected"),
    # WeylElt surface: W_elt canonical words (A2/B2 braid anchors), word,
    # length, relations, product/inverse/generator-product, root_datum,
    # plus the illegal-entry and negative-entry rejections.
    FixturePlan(name="domain/weyl_element"),
    FixturePlan(name="domain/weyl_element_rejected"),
    # CartanClass surface: per-class occurrence counts and display,
    # involution, most-split, (dual) real-form sweeps, square classes,
    # fiber partition, per-form numbering, and the illegal-number rejection.
    FixturePlan(name="domain/cartan_aggregation"),
    FixturePlan(name="domain/cartan_aggregation_rejected"),
    # Synthetic KGB seed: KGB_elt(RealForm,mat,ratvec) symmetrizes the torus
    # factor, factors theta as a twisted involution, and looks the Tits
    # element up per form; rejections cover the cocharacter-coset and
    # non-involution diagnostics.
    FixturePlan(name="domain/seed_x0"),
    FixturePlan(name="domain/seed_x0_rejected"),
    # Involution-table printers: print_KGB's full table (statuses, crosses,
    # Cayleys, torus parts, canonical-involution words) and
    # print_strong_real's single-class layout on A1, plus the two-overload
    # match failure on a RootDatum argument.
    FixturePlan(name="domain/involution_table"),
    FixturePlan(name="domain/involution_table_rejected"),
    # Adjoint-fiber stabilizer: central_fiber(RealForm->[vec]) on split
    # SL(2,R), compact SU(2), and quasisplit SU(2,1), plus the InnerClass
    # argument conform rejection.
    FixturePlan(name="domain/adjoint_fiber"),
    FixturePlan(name="domain/adjoint_fiber_rejected"),
    # Real-form label matrices: occurrence/dual_occurrence, block_sizes and
    # block_size, and Cartan_order on the A2 compact inner class, plus the
    # real-form-number out-of-bounds rejection.
    FixturePlan(name="domain/real_form_labels"),
    FixturePlan(name="domain/real_form_labels_rejected"),
    # Synthetic weak real form: real_form(InnerClass,mat,ratvec) projects
    # the torus factor onto its theta-fixed part and classifies the
    # resulting grading; rejections cover the non-involution and
    # torus-factor-size diagnostics.
    FixturePlan(name="domain/weak_real_form"),
    FixturePlan(name="domain/weak_real_form_rejected"),
    # Weak-real probes whose prerequisites have landed: B2 downward descent,
    # validation ordering, and the central-coroot rejection (torus-radical
    # fix 646f897). The a1_t1/a2_noncanonical probes await the custom-seed
    # real_form gap and stay unwired.
    FixturePlan(name="domain/weak_real_form_b2_descent_probe"),
    FixturePlan(name="domain/weak_real_form_central_coroot_rejected_probe"),
    FixturePlan(name="domain/weak_real_form_validation_rejected_probe"),
    # The custom-seed real_form path (8135b89): elected square root,
    # involution-table extension, minimal_torus_part descent, and the
    # default-vs-custom seed branch make both remaining probes verbatim.
    FixturePlan(name="domain/weak_real_form_a1_t1_central_probe"),
    FixturePlan(name="domain/weak_real_form_a2_noncanonical_probe"),
    # Root-datum lattice relations: blockwise Smith bases, unit filtering,
    # annihilators modulo d, generator replacement, and quotient bases with
    # the three frozen validation diagnostics.
    FixturePlan(name="domain/relations"),
    FixturePlan(name="domain/relations_rejected"),
    FixturePlan(name="domain/relations_extended_probe"),
    FixturePlan(name="domain/relations_extended_rejected_probe"),
    # Involution decomposition: lattice classification, distinguished
    # matrices, and (WeylElt, InnerClass) factorization.  The probes freeze
    # zero-rank/rank-one behavior, B2/C2 presentation preferences, exact
    # matrix-coercion diagnostics, and root/coroot preservation failures.
    FixturePlan(name="domain/involution_decomposition"),
    FixturePlan(name="domain/involution_decomposition_rejected"),
    FixturePlan(name="domain/involution_decomposition_classify_edges_probe"),
    FixturePlan(name="domain/involution_decomposition_classify_nonsquare_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_classify_zero_row_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_classify_ragged_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_classify_type_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_distinguished_type_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_rank_one_probe"),
    FixturePlan(name="domain/involution_decomposition_b2_c2_preference_probe"),
    FixturePlan(name="domain/involution_decomposition_twisted_coroot_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_twisted_foreign_c2_datum_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_twisted_foreign_datum_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_twisted_matrix_type_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_twisted_non_root_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_twisted_rank_mismatch_rejected_probe"),
    FixturePlan(name="domain/involution_decomposition_twisted_root_datum_type_rejected_probe"),
    # Block domain: the fibred-product BlockGraph over both sides' full KGB
    # (4167249), with the upstream gate order and renumbered descent status.
    FixturePlan(name="domain/block_basic"),
    FixturePlan(name="domain/block_basic_rejected"),
    # Primitive involution constructors (152f4b8): layout tables per Lie
    # letter with the 's'/'u' collapse rules, and the based on_basis
    # transport with the integrality gate.
    FixturePlan(name="domain/involution_primitive"),
    FixturePlan(name="domain/involution_primitive_rejected"),
    # K-type / standard-parameter family (agent-27 Rep_context crate
    # milestone + the language slice): the six contracts over the
    # KType/Param/KTypePol/ParamPol language values.
    FixturePlan(name="domain/ktype_basic"),
    FixturePlan(name="domain/ktype_basic_rejected"),
    FixturePlan(name="domain/param_basic"),
    FixturePlan(name="domain/param_basic_rejected"),
    FixturePlan(name="domain/ktypepol_basic"),
    FixturePlan(name="domain/parampol_basic"),
    # Non-final KTypePol/ParamPol expansion (finals_for/expand_final,
    # K_repr.cpp:290-396 / repr.cpp:1205-1297): adding a non-final K-type
    # or parameter to a polynomial expands to its final terms.
    FixturePlan(name="domain/ktypepol_nonfinal"),
    FixturePlan(name="domain/parampol_nonfinal"),
    # Direct Param->ParamPol / KType->KTypePol coercions (atlas-types.w
    # :5608-5617 K_type_to_poly, :7710-7717 param_to_poly) including the
    # non-dominant expand_final reflection path (repr.cpp:1283-1287).
    FixturePlan(name="domain/polp_coercion"),
    # KTypePol/ParamPol arithmetic: binary sums/differences, Split-scaled
    # products, the (Split,KType) term-list add, binary =/!=, and
    # truncate_above_height.
    FixturePlan(name="domain/ktypepol_arithmetic"),
    FixturePlan(name="domain/parampol_arithmetic"),
    # The first deform-family surface: KGP_sum of a semifinal K-type
    # (K_repr.cpp:398-464, atlas-types.w:5995-6010).
    FixturePlan(name="domain/kgp_sum"),
    # The K-type formula with height cutoff (K_repr.cpp:549-591,
    # atlas-types.w:6030-6054).
    FixturePlan(name="domain/ktype_formula"),
    # The branch of a KTypePol at a height cutoff (K_repr.cpp:592-622,
    # atlas-types.w:6055-6070).
    FixturePlan(name="domain/branch"),
    # ParamPol/Param operations: K_type_pol(ParamPol), last_term, and the
    # (ParamPol,rat)/(Param,rat) scalings.
    FixturePlan(name="domain/param_pol_ops"),
    # Param predicates, dominant/normal transforms, and equivalence.
    FixturePlan(name="domain/param_transforms"),
    # Param/KGB twist continuation: unary Param twist first makes the source
    # dominant, explicit-matrix Param twist operates on the source as-is, and
    # the language-visible UndefKGB sentinel remains printable without ever
    # becoming a graph index. Reference captures 3543702/3543783/3543792/
    # 3543798 pin the accepted, rejected, nonstandard, and sentinel paths.
    FixturePlan(name="domain/p3_param_twist_signatures"),
    FixturePlan(name="domain/p3_param_twist_signatures_rejected"),
    FixturePlan(name="domain/p3_param_twist_nonstandard_rejected"),
    FixturePlan(name="domain/p3_param_twist_undefined"),
    FixturePlan(name="domain/p3_kgb_twist_undefined"),
    FixturePlan(name="domain/p3_twist_undefined_fields"),
    # Narrow full_deform accumulation repair: distinct KTypes with equal
    # Split coefficients remain distinct; equal KTypes combine and zero out.
    # This does not claim recursive/proper-subsystem/timed deformation.
    FixturePlan(name="domain/full_deform_term_merge"),
    FixturePlan(name="domain/full_deform_term_merge_rejected"),
    # Param W-graph static contract: preserve the nested vertex/edge tuple
    # types exposed by atlas-types.w:7521-7524.  The accepted case also
    # exercises the generic row-cardinality `#` primitive used to inspect
    # those nested lists; capture 3543933 pins both accepted and rejected
    # type behaviour before the Rust implementation is compared.
    FixturePlan(name="domain/param_wgraph_types"),
    FixturePlan(name="domain/param_wgraph_types_rejected"),
    # Arbitrary integral-root Param transforms.  The A2 pair covers simple,
    # non-simple and negative roots plus no-value/diagnostic behavior; the A3
    # pair forces successful integral dominance and its diagnostic priority.
    FixturePlan(name="domain/param_root_transforms"),
    FixturePlan(name="domain/param_root_transforms_rejected"),
    FixturePlan(name="domain/param_root_transforms_dominance"),
    FixturePlan(name="domain/param_root_transforms_dominance_rejected"),
    # Strong-real surface: the base contract plus the probes whose slices
    # have landed — the Cartan numbering adapter (a63dc32) covers the four
    # B2/C2 Cartan enumerations and all four rejected diagnostics, and the
    # RootDatum dual-order surface (cba10ec) covers the four dual-order
    # probes. The full-KGB probe awaits the KGB discovery-order slice and
    # stays unwired.
    FixturePlan(name="domain/strong_real"),
    FixturePlan(name="domain/strong_real_b2_root_cartans_probe"),
    FixturePlan(name="domain/strong_real_b2_coroot_cartans_probe"),
    FixturePlan(name="domain/strong_real_c2_root_cartans_probe"),
    FixturePlan(name="domain/strong_real_c2_coroot_cartans_probe"),
    FixturePlan(name="domain/strong_real_cartan_high_rejected_probe"),
    FixturePlan(name="domain/strong_real_cartan_negative_rejected_probe"),
    FixturePlan(name="domain/strong_real_print_type_rejected_probe"),
    FixturePlan(name="domain/strong_real_square_classes_type_rejected_probe"),
    FixturePlan(name="domain/strong_real_b2_root_dual_order_probe"),
    FixturePlan(name="domain/strong_real_b2_coroot_dual_order_probe"),
    FixturePlan(name="domain/strong_real_c2_root_dual_order_probe"),
    FixturePlan(name="domain/strong_real_c2_coroot_dual_order_probe"),
    # The full B2 KGB print: the parabolic-pieces involution key (1e2a3a5)
    # matches the oracle's element numbering exactly.
    FixturePlan(name="domain/strong_real_b2_full_kgb_probe"),
    FixturePlan(name="domain/strong_real_c2_full_kgb_probe"),
    # Weyl layer (agent-30): Weyl_orbit/Weyl_orbit_ws both argument orders,
    # walls/walls_attitude (alcoves.cpp), and from_dominant rejections.
    FixturePlan(name="domain/weyl_orbit"),
    FixturePlan(name="domain/weyl_orbit_rejected"),
    FixturePlan(name="domain/walls"),
    FixturePlan(name="domain/walls_rejected"),
    # Alcove/FPP layer (agent-31): alcove_center/alcove_root_vertex/
    # FPP_numers/FPP_w_shifts (alcoves.cpp:277-341, 345-408, 945-1075) and
    # their size/alcove rejections.
    FixturePlan(name="domain/alcove_fpp"),
    FixturePlan(name="domain/alcove_fpp_rejected"),
    # Extended-block layer (agent-36): extended_block/raw_ext_KL/
    # partial_extended_KL_block on A2 (ic=c and ic=u) + A1; rejections cover
    # delta not fixing gamma and type mismatches.
    FixturePlan(name="domain/ext_block"),
    FixturePlan(name="domain/ext_block_rejected"),
    # Slice A (agent-40): coroot_queries sweep (poscoroots/simple_coroots/
    # two_rho_check/coroot_radical/mod_central_torus_info/adjoint/
    # semisimple_rank/reducibility_points) and the root numbering family
    # (root/coroot expression+index, root_involution/root_permutation).
    FixturePlan(name="domain/coroot_queries"),
    FixturePlan(name="domain/coroot_queries_rejected"),
    # derived_info(±): derived_info/mod_central_torus_info projector+injector
    # matrices and derived-datum isogeny labels on B2 sc/adjoint, A1.T1, G2 —
    # regression pins for the transpose/orientation/isogeny repair.
    FixturePlan(name="domain/derived_info"),
    FixturePlan(name="domain/derived_info_rejected"),
    # Coverage-gap closure (2026-08-12 sweep): integrality_points return-type/
    # normalisation repair pins; index(Block,KGBElt,KGBElt) and
    # to_canonical_fiber(KType) first dedicated coverage.
    FixturePlan(name="domain/integrality_points"),
    FixturePlan(name="domain/integrality_points_rejected"),
    FixturePlan(name="domain/block_ktype_extras"),
    FixturePlan(name="domain/block_ktype_extras_rejected"),
    # dual_kl_raw(±): dual_KL through the swapped-forms block and
    # blocks::dual_map (atlas-types.w:8640-8674), raw_KL cross-check.
    FixturePlan(name="domain/dual_kl_raw"),
    FixturePlan(name="domain/dual_kl_raw_rejected"),
    FixturePlan(name="domain/root_numbering"),
    FixturePlan(name="domain/root_numbering_rejected"),
    # Orbit/ladder (slice B, agent-43): root_ladder_bottoms/
    # coroot_ladder_bottoms/basic_orbit_ws/affine_orbit_ws.
    FixturePlan(name="domain/orbit_ws"),
    FixturePlan(name="domain/orbit_ws_rejected"),
    # Poly surface (slice B, agent-43): ParamPol/KTypePol skip overloads
    # (null_module/real_form/#/first_term/last_term/truncate_above_height/
    # K_type_pol/W_cells).
    FixturePlan(name="domain/poly_surface"),
    FixturePlan(name="domain/poly_surface_rejected"),
    # Slice C (agent-46): print_gradings/print_real_Weyl/
    # print_blockstabilizer printers.
    # print_gradings: the verified capture folds each print's output into
    # the preceding CartanClass Value display, so only the 14 `:=` lines
    # carry events; type ascriptions and the 9 print calls are silent.
    FixturePlan(
        name="domain/print_gradings",
        runnable_lines=(2, 4, 6, 10, 12, 14, 16, 18, 21, 24, 28, 30, 32, 36),
        silent_lines=(
            1, 3, 5, 7, 8, 9, 11, 13, 15, 17, 19, 20, 22, 23, 25, 26, 27, 29,
            31, 33, 34, 35, 37,
        ),
    ),
    FixturePlan(name="domain/print_gradings_rejected"),
    # real_weyl_print: prints are standalone ReportLine events, but a print
    # whose text duplicates the previous one produces no new event in the
    # verified capture (lines 8, 15, 29, 33, 34); type ascriptions silent.
    FixturePlan(
        name="domain/real_weyl_print",
        runnable_lines=(
            2, 4, 6, 7, 10, 11, 13, 14, 17, 19, 21, 22, 24, 25, 27, 28, 31,
            32, 36, 38, 40, 42, 43, 45, 46,
        ),
        silent_lines=(
            1, 3, 5, 8, 9, 12, 15, 16, 18, 20, 23, 26, 29, 30, 33, 34, 35,
            37, 39, 41, 44,
        ),
    ),
    FixturePlan(name="domain/real_weyl_print_rejected"),
    # print_X (a2979ad): each print_X call emits one standalone ReportLine;
    # the three print texts differ so no dedup gap; type ascriptions silent
    # (alignment pre-analysed against print_x.events.json, 2026-08-12).
    FixturePlan(
        name="domain/print_x",
        runnable_lines=(2, 4, 5, 7, 9, 10, 12, 14, 15),
        silent_lines=(1, 3, 6, 8, 11, 13),
    ),
    FixturePlan(name="domain/print_x_rejected"),
    # dual_KL_block (ced33b8 + f399fc8): ascriptions silent, each `:=` ->
    # Declaring+Value, each bare call -> 1 event (33 lines/33 events, no
    # folding/dedup; alignment pre-analysed against
    # dual_kl_block.events.json, 2026-08-12).
    FixturePlan(
        name="domain/dual_kl_block",
        runnable_lines=(
            2, 4, 6, 8, 10, 11, 13, 15, 17, 19, 21, 22, 24, 26, 28, 30,
            32, 33,
        ),
        silent_lines=(1, 3, 5, 7, 9, 12, 14, 16, 18, 20, 23, 25, 27, 29, 31),
    ),
    FixturePlan(name="domain/dual_kl_block_rejected"),
    # print_common_block + print_block(Param) (ab811fa): 31 lines/30
    # events — print_block(p) at line 19 prints byte-identical text to
    # the preceding print_common_block(p) and is deduped silent (same
    # pattern as real_weyl_print). No rejected fixture by documented
    # design.
    FixturePlan(
        name="domain/print_common_block",
        runnable_lines=(
            2, 4, 6, 8, 9, 11, 12, 14, 15, 17, 18, 21, 23, 25, 27, 28, 30,
            31,
        ),
        silent_lines=(1, 3, 5, 7, 10, 13, 16, 19, 20, 22, 24, 26, 29),
    ),
    FixturePlan(
        name="domain/print_common_block_proper",
        runnable_lines=(2, 4, 6, 8, 9, 10),
        silent_lines=(1, 3, 5, 7),
    ),
    # Non-integral common-block slices 1-2 (commit 31064b1): the shared
    # Rep_table lookup makes the Subset-header cache-hit sequence exact.
    FixturePlan(
        name="domain/print_partial_common_block_seq",
        runnable_lines=(2, 4, 6, 8, 9, 10),
        silent_lines=(1, 3, 5, 7),
    ),
    FixturePlan(
        name="domain/print_partial_block_proper",
        runnable_lines=(2, 4, 6, 8, 10, 11, 12),
        silent_lines=(1, 3, 5, 7, 9),
    ),
    # shift_flip (wrapper gates + shifted_default_extension/is_default;
    # accepted cases all return false — ~1300 oracle probes found no true
    # case). Line alignment pre-analysed 2026-08-12 against
    # shift_flip.events.json: 45 lines/45 events; ascriptions silent,
    # each `:=` -> Declaring+Value, each bare call -> 1 Value event.
    FixturePlan(
        name="domain/shift_flip",
        runnable_lines=(
            2, 4, 6, 8, 9, 10, 12, 13, 15, 17, 19, 21, 22, 23, 25, 27, 29,
            30, 31, 33, 34, 35, 37, 39, 41, 43, 44, 45,
        ),
        silent_lines=(
            1, 3, 5, 7, 11, 14, 16, 18, 20, 24, 26, 28, 32, 36, 38, 40, 42,
        ),
    ),
    FixturePlan(name="domain/shift_flip_rejected"),
    # ext_finalise trio (scale_extended/K_type_pol_extended/
    # finalize_extended; upstream gate order test_final →
    # factor-positive → compatible → is_fixed). Line alignment
    # pre-analysed 2026-08-12: 54 lines/54 events, standard pattern.
    FixturePlan(
        name="domain/ext_finalise",
        runnable_lines=(
            2, 4, 6, 8, 9, 10, 11, 12, 14, 15, 16, 18, 20, 22, 24, 25, 26,
            27, 28, 30, 32, 34, 35, 36, 37, 38, 40, 41, 42, 43, 45, 47, 49,
            51, 52, 53, 54,
        ),
        silent_lines=(
            1, 3, 5, 7, 13, 17, 19, 21, 23, 29, 31, 33, 39, 44, 46, 48, 50,
        ),
    ),
    FixturePlan(name="domain/ext_finalise_rejected"),
    # Twisted family (twisted_deform/twisted_full_deform/twisted_KL_sum_at_s
    # + external (Param,mat) overload; E3 language layer 0cfba0b).
    # Line alignment: 29 lines/29 events, standard pattern.
    FixturePlan(
        name="domain/twisted_family",
        runnable_lines=(
            2, 4, 6, 8, 9, 10, 12, 13, 14, 15, 17, 19, 21, 23, 25, 26, 27,
            28, 29,
        ),
        silent_lines=(1, 3, 5, 7, 11, 16, 18, 20, 22, 24),
    ),
    FixturePlan(name="domain/twisted_family_rejected"),
    # Denominator > 2^rank preprocessing for ordinary and twisted full
    # deformation.  Oracle expectations and benchmarks were frozen by
    # reference-capture job 3546215 before the Rust implementation.
    FixturePlan(name="domain/deform_alcove_shrink"),
    FixturePlan(name="domain/deform_alcove_shrink_rejected"),
    # block_deform (deform-to-height pair; same E3 slice).
    # Line alignment: 15 lines/15 events, standard pattern.
    FixturePlan(
        name="domain/block_deform",
        runnable_lines=(2, 4, 6, 8, 10, 11, 12, 13, 14, 15),
        silent_lines=(1, 3, 5, 7, 9),
    ),
    FixturePlan(name="domain/block_deform_rejected"),
    # dual_block(±): dual(Block->Block) + dual(RealForm->InnerClass)
    # coverage-gap closure. 32 lines/32 events; ascriptions silent.
    FixturePlan(
        name="domain/dual_block",
        runnable_lines=(
            2, 4, 6, 8, 10, 12, 13, 14, 15, 17, 19, 21, 23, 25, 27, 29,
            30, 31, 32,
        ),
        silent_lines=(1, 3, 5, 7, 9, 11, 16, 18, 20, 22, 24, 26, 28),
    ),
    FixturePlan(name="domain/dual_block_rejected"),
    # print_partial_block / print_partial_common_block (interval blocks;
    # language arms 6fb1c30 on crate port f11f48a). Line alignment:
    # 28 lines/24 events — each printer pair on the same param is
    # byte-identical in the oracle capture, so the consecutive identical
    # prints merged into single ReportLines in events.json.
    FixturePlan(
        name="domain/print_partial_block",
        runnable_lines=(
            2, 4, 6, 8, 9, 10, 12, 13, 14, 16, 18, 20, 22, 23, 24, 26, 27,
            28,
        ),
        silent_lines=(1, 3, 5, 7, 11, 15, 17, 19, 21, 25),
    ),
    # Builtin hunger contracts (axis.w:7165-7301): the three same-result
    # products, validation before no_value, simple-assignment pilfering, and
    # the timed twisted_full_deform positive-deadline overload.
    FixturePlan(name="domain/hunger_contract"),
    FixturePlan(name="domain/hunger_contract_rejected"),
    FixturePlan(name="domain/hunger_contract_domain"),
    FixturePlan(name="domain/hunger_assignment_semantics"),
    FixturePlan(name="domain/hunger_assignment_semantics_rejected"),
    FixturePlan(name="domain/hunger_contract_timed_nyi"),
    # Timed twisted_full_deform: isolated completed-result cache and
    # cooperative cancellation contract, including bigint narrowing.
    FixturePlan(name="domain/timed_twisted_full_deform_cache"),
    FixturePlan(name="domain/timed_twisted_full_deform_validation_order_rejected"),
    # Early scalar-era fixtures: verified verbatim locally and included so
    # the HPC differential upgrades their reference metadata.
    # declare_types(±): full type grammar in identifier ascriptions
    # (tuple/row/function/void/wild-row/union) — parser.y:162 parity.
    FixturePlan(name="eval/declare_types"),
    FixturePlan(name="eval/declare_types_rejected"),
    FixturePlan(name="eval/scalars"),
    FixturePlan(name="eval/scalar_overloads"),
    FixturePlan(name="eval/scalar_error_fraction_zero"),
    FixturePlan(name="eval/scalar_error_int_power_large"),
    FixturePlan(name="eval/scalar_error_int_power_negative"),
    FixturePlan(name="eval/scalar_error_rat_divide_zero"),
    FixturePlan(name="eval/scalar_error_rat_modulo_zero"),
    FixturePlan(name="eval/scalar_error_rat_power_negative"),
    FixturePlan(name="eval/scalar_error_rat_quotient_zero"),
    # Split dual-number surface: the (int,int)/int coercions, sign-folded
    # display, componentwise arithmetic, dual product, relations, and the
    # `%` destructure, plus the missing-division rejection.
    FixturePlan(name="eval/split_basic"),
    FixturePlan(name="eval/split_basic_rejected"),
    # Generic axis.w row operators ##/# (suffix/prefix/join/fold) — special
    # generic operator between exact and coercible overloads.
    FixturePlan(name="eval/row_operators"),
    FixturePlan(name="eval/row_operators_rejected"),
    # Parser pair: 2-D matrix slice M[r,c] (parser.y:658-705) and
    # commabarlist [a,b | c,d] (parser.y:370-410, overload-immune).
    FixturePlan(name="eval/matrix_2d_slice"),
    FixturePlan(name="eval/matrix_2d_slice_rejected"),
    FixturePlan(name="eval/commabarlist"),
    FixturePlan(name="eval/commabarlist_rejected"),
)


DIAGNOSTIC_HEADER = re.compile(
    r"^(Lexical|Syntax|Name|Type|Runtime|Io) error(?: at .*?:\d+:\d+)?: (.*)$"
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command_output(command: list[str]) -> str:
    try:
        return subprocess.check_output(
            command, text=True, stderr=subprocess.DEVNULL
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unavailable"


def render_value(value: dict[str, Any]) -> str:
    if "display" in value:
        return str(value["display"])
    value_type = value["type"]
    if value_type == "integer":
        return str(value["value"])
    if value_type == "rational":
        return f"{value['numerator']}/{value['denominator']}"
    if value_type == "boolean":
        return "true" if value["value"] else "false"
    if value_type == "string":
        return f'"{value["value"]}"'
    if value_type == "tuple":
        return "(" + ",".join(render_value(item) for item in value["values"]) + ")"
    raise ValueError(f"cannot render expected value type {value_type!r}")


def expected_cli_observation(events: list[dict[str, Any]]) -> dict[str, Any]:
    stdout_parts: list[str] = []
    diagnostics: list[dict[str, str]] = []
    for event in events:
        kind = event.get("kind")
        if kind == "Value":
            stdout_parts.append(f"Value: {render_value(event['value'])}\n")
        elif kind in ("ReportLine", "Output"):
            stdout_parts.append(event["text"])
        elif kind == "Diagnostic":
            diagnostics.append(
                {
                    "category": event["category"].lower(),
                    "message": event["message"],
                }
            )
        else:
            raise ValueError(f"unsupported expected event kind {kind!r}")
    stdout_parts.append("Bye.\n")
    return {
        "stdout": "".join(stdout_parts),
        "diagnostics": diagnostics,
        "exit_status": 1 if diagnostics else 0,
    }


def parse_cli_diagnostics(stderr: str) -> tuple[list[dict[str, str]], list[str]]:
    diagnostics: list[dict[str, str]] = []
    unparsed: list[str] = []
    for line in stderr.splitlines():
        match = DIAGNOSTIC_HEADER.match(line)
        if match:
            diagnostics.append(
                {"category": match.group(1).lower(), "message": match.group(2)}
            )
        elif line.startswith("  | ") or not line.strip():
            continue
        elif line.startswith("  ") and diagnostics:
            # Multi-line message continuation (e.g. the oracle's three-line
            # "Cannot generate block:\n  non-standard parameter(...)\n
            # Parameter not standard" rendered by the CLI with the same
            # two-space indent). Source excerpts carry the "  | " prefix
            # handled above, so plain indented lines extend the current
            # diagnostic's message verbatim.
            diagnostics[-1]["message"] += "\n" + line[2:]
        else:
            unparsed.append(line)
    return diagnostics, unparsed


def selected_fixture_source(source: str, line_numbers: tuple[int, ...]) -> str:
    lines = source.splitlines()
    selected = [lines[line_number - 1] for line_number in line_numbers]
    return "\n".join(selected) + "\n"


def validate_plan(
    plan: FixturePlan,
    source: str,
    events: list[dict[str, Any]],
) -> tuple[tuple[int, ...], tuple[int, ...], list[str]]:
    errors: list[str] = []
    source_lines = source.splitlines()
    nonempty_lines = tuple(
        index for index, line in enumerate(source_lines, start=1) if line.strip()
    )
    runnable_lines = (
        nonempty_lines if plan.runnable_lines is None else plan.runnable_lines
    )
    runnable_events = (
        tuple(range(len(events)))
        if plan.runnable_events is None
        else plan.runnable_events
    )

    # One runnable line can produce several events (the verbose trace
    # emits three stdout lines for one expression; a lexical warning rides
    # along with the recovered value), so only the degenerate direction —
    # a runnable line with no event at all — is invalid (mark it silent).
    if len(runnable_events) < len(runnable_lines):
        errors.append("runnable source/event selection lengths differ")
    if any(line < 1 or line > len(source_lines) for line in runnable_lines):
        errors.append("runnable source line is outside the fixture")
    if any(index < 0 or index >= len(events) for index in runnable_events):
        errors.append("runnable event index is outside the expectation")
    if tuple(sorted(set(runnable_lines))) != runnable_lines:
        errors.append("runnable source lines are not unique and increasing")
    if tuple(sorted(set(runnable_events))) != runnable_events:
        errors.append("runnable event indices are not unique and increasing")

    pending_lines = tuple(case.source_line for case in plan.pending)
    pending_events = tuple(case.reference_event for case in plan.pending)
    if tuple(sorted(set(pending_lines))) != pending_lines:
        errors.append("pending source lines are not unique and increasing")
    if any(index < 0 or index >= len(events) for index in pending_events):
        errors.append("pending event index is outside the expectation")
    silent_lines = plan.silent_lines
    if tuple(sorted(set(silent_lines))) != silent_lines:
        errors.append("silent source lines are not unique and increasing")
    if any(line < 1 or line > len(source_lines) for line in silent_lines):
        errors.append("silent source line is outside the fixture")
    if set(runnable_lines).intersection(pending_lines):
        errors.append("a source line is both runnable and pending")
    if set(runnable_lines).intersection(silent_lines):
        errors.append("a source line is both runnable and silent")
    if set(silent_lines).intersection(pending_lines):
        errors.append("a source line is both silent and pending")
    if set(runnable_events).intersection(pending_events):
        errors.append("an event is both runnable and pending")
    if tuple(sorted(runnable_lines + silent_lines + pending_lines)) != nonempty_lines:
        errors.append("source selection does not cover every nonempty fixture line")
    if tuple(sorted(set(runnable_events + pending_events))) != tuple(range(len(events))):
        errors.append("event selection does not cover every expected event")
    return runnable_lines, runnable_events, errors


def run_fixture(
    plan: FixturePlan,
    cli_bin: pathlib.Path,
    output_dir: pathlib.Path,
    fixture_root: pathlib.Path,
    reference_root: pathlib.Path,
    workspace_root: pathlib.Path,
    expected_revision: str,
    timeout: int,
) -> tuple[dict[str, Any], bool]:
    fixture = fixture_root / f"{plan.name}.atlas"
    expectation_path = reference_root / f"{plan.name}.events.json"
    metadata_path = reference_root / f"{plan.name}.meta.json"
    source_bytes = fixture.read_bytes()
    source = source_bytes.decode("utf-8")
    expectation = json.loads(expectation_path.read_text(encoding="utf-8"))
    metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    events = expectation.get("events", [])
    runnable_lines, runnable_events, configuration_errors = validate_plan(
        plan, source, events
    )

    fixture_sha = sha256(source_bytes)
    if metadata.get("fixture_sha256") != fixture_sha:
        configuration_errors.append("fixture checksum differs from oracle metadata")
    expected_fixture_name = plan.name
    # Older capture jobs recorded the fixture name with its ".atlas" suffix
    # (for example domain/root_coroot.atlas); the events file and the plan
    # both use the bare name, so normalize only that suffix away.
    metadata_fixture = str(metadata.get("fixture", "")).removesuffix(".atlas")
    if metadata_fixture != expected_fixture_name:
        configuration_errors.append("metadata names a different fixture")
    if expectation.get("fixture") != expected_fixture_name:
        configuration_errors.append("event expectation names a different fixture")
    if metadata.get("reference_status") != "verified_hpc_reference":
        configuration_errors.append("reference metadata is not HPC-verified")
    if expectation.get("status") != "verified_hpc_reference":
        configuration_errors.append("event expectation is not HPC-verified")
    if metadata.get("reference_atlas_revision") != expected_revision:
        configuration_errors.append("reference revision differs from requested revision")
    if metadata.get("oracle") != "atlas":
        configuration_errors.append("metadata does not name Atlas as the oracle")
    if metadata.get("stage") != "typed-pipeline-swap":
        configuration_errors.append("metadata belongs to a different stage")

    artifact_dir = output_dir / plan.name
    artifact_dir.mkdir(parents=True, exist_ok=True)
    selected_lines = tuple(sorted(runnable_lines + plan.silent_lines))
    selected_source = selected_fixture_source(source, selected_lines)
    selected_path = artifact_dir / "runnable.atlas"
    selected_path.write_text(selected_source, encoding="utf-8")
    expected_events = [events[index] for index in runnable_events]
    expected = expected_cli_observation(expected_events)
    if not plan.pending:
        if "oracle_exit_status" not in metadata:
            configuration_errors.append("oracle metadata has no exit status")
        else:
            expected["exit_status"] = metadata["oracle_exit_status"]

    timed_out = False
    maxrss_kb = None
    maxrss_approximate = False
    if configuration_errors:
        stdout = b""
        stderr = b""
        exit_status = None
        elapsed = 0.0
    else:
        stdout, stderr, exit_status, timed_out, elapsed, maxrss_kb, maxrss_approximate = (
            measure_command(
                [str(cli_bin), str(selected_path.resolve())],
                cwd=workspace_root,
                timeout=timeout,
            )
        )
        elapsed = round(elapsed, 3)

    stdout_path = artifact_dir / "rust.stdout"
    stderr_path = artifact_dir / "rust.stderr"
    stdout_path.write_bytes(stdout)
    stderr_path.write_bytes(stderr)
    stdout_text = stdout.decode("utf-8", errors="replace")
    stderr_text = stderr.decode("utf-8", errors="replace")
    actual_diagnostics, unparsed_stderr = parse_cli_diagnostics(stderr_text)
    checks = {
        "configuration_valid": not configuration_errors,
        "completed_before_timeout": not timed_out,
        "stdout_exact": stdout_text == expected["stdout"],
        "diagnostics_exact": actual_diagnostics == expected["diagnostics"],
        "stderr_fully_parsed": not unparsed_stderr,
        "exit_status_exact": exit_status == expected["exit_status"],
    }
    runnable_passed = all(checks.values())
    fixture_status = (
        "FAIL" if not runnable_passed else "PARTIAL" if plan.pending else "PASS"
    )

    def relative(path: pathlib.Path) -> str:
        try:
            return path.resolve().relative_to(workspace_root).as_posix()
        except ValueError:
            return str(path.resolve())

    def artifact_relative(path: pathlib.Path) -> str:
        return path.resolve().relative_to(output_dir.resolve()).as_posix()

    entry = {
        "fixture": relative(fixture),
        "fixture_sha256": fixture_sha,
        "expectation": {
            "path": relative(expectation_path),
            "sha256": sha256(expectation_path.read_bytes()),
            "event_indices": list(runnable_events),
            "stdout": {
                "sha256": sha256(expected["stdout"].encode()),
                "text": expected["stdout"],
            },
            "diagnostics": expected["diagnostics"],
            "exit_status": expected["exit_status"],
        },
        "metadata": {
            "path": relative(metadata_path),
            "sha256": sha256(metadata_path.read_bytes()),
            "reference_job": metadata.get("reference_job"),
            "reference_atlas_revision": metadata.get("reference_atlas_revision"),
            "reference_binary_sha256": metadata.get("reference_binary_sha256"),
        },
        "runnable": {
            "source_lines": list(runnable_lines),
            "silent_source_lines": list(plan.silent_lines),
            "input_path": artifact_relative(selected_path),
            "input_sha256": sha256(selected_source.encode()),
        },
        "pending": [
            {
                "feature": case.feature,
                "source_line": case.source_line,
                "reference_event": case.reference_event,
                "reason": case.reason,
            }
            for case in plan.pending
        ],
        "rust": {
            "stdout": {
                "path": artifact_relative(stdout_path),
                "sha256": sha256(stdout),
                "text": stdout_text,
            },
            "stderr": {
                "path": artifact_relative(stderr_path),
                "sha256": sha256(stderr),
                "text": stderr_text,
            },
            "diagnostics": actual_diagnostics,
            "unparsed_stderr": unparsed_stderr,
            "exit_status": exit_status,
            "timed_out": timed_out,
            "seconds": elapsed,
            "maxrss_kb": maxrss_kb,
            "maxrss_approximate": maxrss_approximate,
        },
        "configuration_errors": configuration_errors,
        "checks": checks,
        "runnable_status": "PASS" if runnable_passed else "FAIL",
        "status": fixture_status,
    }
    return entry, runnable_passed


def parse_dirty_tree(value: str) -> bool | str:
    return {"true": True, "false": False}.get(value.lower(), value)


def _parse_time_metrics(path: pathlib.Path) -> dict[str, Any] | None:
    """Parse GNU time -v output: peak RSS (kbytes) and wall seconds."""
    if not path.exists():
        return None
    text = path.read_text(encoding="utf-8", errors="replace")
    rss = None
    seconds = None
    for line in text.splitlines():
        lowered = line.lower()
        if "maximum resident set size" in lowered:
            parts = lowered.split(":")
            if parts:
                digits = re.sub(r"\D", "", parts[-1])
                if digits:
                    rss = int(digits)
        if "elapsed (wall clock) time" in lowered:
            parts = lowered.split(":")
            if parts:
                digits = re.sub(r"\D", "", parts[-1])
                if digits:
                    seconds = float(digits)
    return {"maxrss_kb": rss, "wall_seconds": seconds}


# /usr/bin/time -v (GNU coreutils) is available on the Linux HPC nodes; the
# mac CI boxes only have the BSD variant, so fall back to the cumulative
# child-process peak from getrusage (labelled approximate).
_TIME_BIN = shutil.which("/usr/bin/time") or "/usr/bin/time"
_USE_GNU_TIME = os.path.exists(_TIME_BIN) and platform.system() != "Darwin"


def measure_command(
    argv: list[str],
    cwd: str | os.PathLike[str],
    timeout: int,
    input_bytes: bytes | None = None,
) -> tuple[bytes, bytes, int | None, bool, float, int | None, bool]:
    """Run a command and report (stdout, stderr, exit, timed_out, seconds,
    maxrss_kb, maxrss_approximate)."""
    timed_out = False
    started = time.monotonic()
    if _USE_GNU_TIME:
        with tempfile.TemporaryDirectory() as directory:
            metric_path = pathlib.Path(directory) / "time.metrics"
            argv = [_TIME_BIN, "-v", "-o", str(metric_path)] + argv
            try:
                completed = subprocess.run(
                    argv,
                    cwd=cwd,
                    input=input_bytes,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    timeout=timeout,
                )
                stdout = completed.stdout
                stderr = completed.stderr
                exit_status = completed.returncode
            except subprocess.TimeoutExpired as error:
                timed_out = True
                stdout = error.stdout or b""
                stderr = error.stderr or b""
                exit_status = None
            metrics = _parse_time_metrics(metric_path)
            maxrss = metrics["maxrss_kb"] if metrics else None
            seconds = (
                metrics["wall_seconds"] if metrics and metrics["wall_seconds"]
                else round(time.monotonic() - started, 3)
            )
            return stdout, stderr, exit_status, timed_out, seconds, maxrss, False
    try:
        completed = subprocess.run(
            argv,
            cwd=cwd,
            input=input_bytes,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
        )
        stdout = completed.stdout
        stderr = completed.stderr
        exit_status = completed.returncode
    except subprocess.TimeoutExpired as error:
        timed_out = True
        stdout = error.stdout or b""
        stderr = error.stderr or b""
        exit_status = None
    seconds = round(time.monotonic() - started, 3)
    try:
        rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
        if sys.platform == "darwin":
            rss //= 1024  # bytes -> KiB
        maxrss = int(rss)
    except (AttributeError, ValueError):
        maxrss = None
    return stdout, stderr, exit_status, timed_out, seconds, maxrss, True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("atlas_cli", type=pathlib.Path)
    parser.add_argument("output_dir", type=pathlib.Path)
    parser.add_argument("--workspace-root", type=pathlib.Path, required=True)
    parser.add_argument("--fixture-root", type=pathlib.Path, required=True)
    parser.add_argument("--reference-root", type=pathlib.Path, required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--dirty-tree", required=True)
    parser.add_argument("--detected-commit", required=True)
    parser.add_argument("--detected-dirty-tree", required=True)
    parser.add_argument("--job-id", required=True)
    parser.add_argument("--source-snapshot-sha256", required=True)
    parser.add_argument("--timeout", type=int, default=30)
    parser.add_argument(
        "--expected-reference-revision", default=PINNED_ATLAS_REVISION
    )
    args = parser.parse_args()

    cli_bin = args.atlas_cli.resolve()
    if not os.access(cli_bin, os.X_OK):
        parser.error(f"atlas-cli is not executable: {cli_bin}")
    workspace_root = args.workspace_root.resolve()
    fixture_root = args.fixture_root.resolve()
    reference_root = args.reference_root.resolve()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    source_state_checks = {
        "declared_commit_valid": bool(COMMIT_TOKEN.fullmatch(args.commit)),
        "detected_commit_valid": bool(COMMIT_TOKEN.fullmatch(args.detected_commit)),
        "commit_exact": args.commit == args.detected_commit,
        "declared_dirty_tree_valid": args.dirty_tree in DIRTY_TREE_TOKENS,
        "detected_dirty_tree_valid": (
            args.detected_dirty_tree in DIRTY_TREE_TOKENS
        ),
        "dirty_tree_exact": args.dirty_tree == args.detected_dirty_tree,
    }
    source_state_verified = all(source_state_checks.values())

    entries = []
    all_runnable_passed = True
    for plan in FIXTURE_PLANS:
        entry, passed = run_fixture(
            plan,
            cli_bin,
            output_dir,
            fixture_root,
            reference_root,
            workspace_root,
            args.expected_reference_revision,
            args.timeout,
        )
        entries.append(entry)
        all_runnable_passed = all_runnable_passed and passed

    pending = [
        {
            "fixture": entry["fixture"],
            **case,
        }
        for entry in entries
        for case in entry["pending"]
    ]
    pending.extend(
        {"scope": "uncovered_overload", **overload}
        for overload in PENDING_OVERLOADS
    )
    status = (
        "FAIL"
        if not source_state_verified or not all_runnable_passed
        else "PARTIAL"
        if pending
        else "PASS"
    )
    report = {
        "schema": "atlas-pipeline-swap-diff-v1",
        "stage": "typed-pipeline-swap-rust-vs-frozen-atlas",
        "status": status,
        "runnable_status": "PASS" if all_runnable_passed else "FAIL",
        "compatibility_claim": status == "PASS",
        "pending_features": pending,
        "commit": args.commit,
        "dirty_tree": parse_dirty_tree(args.dirty_tree),
        "source_state": {
            "declared_commit": args.commit,
            "detected_commit": args.detected_commit,
            "declared_dirty_tree": parse_dirty_tree(args.dirty_tree),
            "detected_dirty_tree": parse_dirty_tree(args.detected_dirty_tree),
            "verified": source_state_verified,
            "checks": source_state_checks,
        },
        "source_snapshot_sha256": args.source_snapshot_sha256,
        "source_snapshot_scope": (
            "provided snapshot (exact scope annotated by the batch job)"
        ),
        "harness_sha256": sha256(pathlib.Path(__file__).read_bytes()),
        "atlas_cli": str(cli_bin),
        "atlas_cli_sha256": sha256(cli_bin.read_bytes()),
        "reference_atlas_revision": args.expected_reference_revision,
        "diagnostic_comparison": {
            "scope": "category and message only",
            "source_path_line_column_caret_compared": False,
            "position_context_lines_ignored": "lines beginning with '  | '",
            "other_stderr_is_failure": True,
        },
        "rustc": command_output(["rustc", "--version"]),
        "cargo": command_output(["cargo", "--version"]),
        "slurm": {
            "job_id": args.job_id,
            "node_list": os.environ.get("SLURM_JOB_NODELIST", "unavailable"),
            "hostname": command_output(["hostname"]),
        },
        "fixtures": entries,
    }
    report_path = output_dir / "pipeline_swap_diff_report.json"
    report_path.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(
        f"pipeline swap: {len(entries)} fixtures, "
        f"{len(pending)} pending cases, {status}"
    )
    print(f"report: {report_path}")
    return 0 if source_state_verified and all_runnable_passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
