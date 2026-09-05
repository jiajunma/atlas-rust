//! Alcove geometry used by deformation preprocessing and integral-datum
//! canonicalization.
//!
//! This module owns the domain-level form of `weyl::alcove_center`
//! (`alcoves.cpp:277-341`).  It keeps a standard parameter's KGB and
//! lambda-rho data and replaces only its infinitesimal character.
//! [`root_vertex_of_alcove`] (`alcoves.cpp:414-428`) serves the
//! fundamental-alcove reduction of the locator slice (`locator.rs`).

use std::collections::{BTreeMap, BTreeSet};

use malachite::Rational;

use crate::{RationalWeight, RepContext, RootId, RootSystem, StandardRepr, StructureError, Weight};

/// Return the barycentre of the alcove containing `z.gamma()`.
///
/// The KGB coordinate and `lambda_rho` are preserved; the returned parameter
/// is rebuilt with [`RepContext::sr_gamma`] so its packed torsion data and
/// derived height remain canonical.
pub fn alcove_center(
    rc: &RepContext<'_>,
    z: &StandardRepr,
) -> Result<StandardRepr, StructureError> {
    let datum = rc.datum();
    let root_system = rc.root_system();
    let rank = datum.lattice_rank();
    let gamma = z.gamma();
    let numbering = RootNumbering::new(root_system);
    let (walls, integrals) = wall_set(root_system, &numbering, gamma)?;
    let fracs = barycentre_eq(root_system, &numbering, &walls, &integrals)?;

    let mut rows = Vec::new();
    rows.try_reserve_exact(walls.len() + datum.lattice_rank())
        .map_err(|_| StructureError::AllocationFailed {
            requested: walls.len() + datum.lattice_rank(),
        })?;
    for (index, &nbr) in walls.iter().enumerate() {
        let coroot = root_system.coroot(numbering.id(nbr)).ok_or(
            StructureError::RootSystemInvariantViolation {
                invariant: "alcove wall has no coroot",
            },
        )?;
        let scale = fracs[index].1;
        let coefficients = coroot
            .as_slice()
            .iter()
            .map(|&coordinate| i64::from(coordinate).checked_mul(scale))
            .collect::<Option<Vec<_>>>()
            .ok_or(StructureError::ArithmeticOverflow)?;
        let floor = floor_eval_nbr(root_system, &numbering, nbr, gamma)?;
        let rhs = floor
            .checked_mul(scale)
            .and_then(|value| value.checked_add(fracs[index].0))
            .ok_or(StructureError::ArithmeticOverflow)?;
        rows.push((coefficients, rhs));
    }
    for element in datum.radical_basis()? {
        let coefficients = element
            .as_slice()
            .iter()
            .map(|&coordinate| i64::from(coordinate).checked_mul(gamma.denominator()))
            .collect::<Option<Vec<_>>>()
            .ok_or(StructureError::ArithmeticOverflow)?;
        let rhs = gamma
            .numerator()
            .iter()
            .zip(element.as_slice())
            .try_fold(0_i64, |sum, (&coordinate, &radical)| {
                coordinate
                    .checked_mul(i64::from(radical))
                    .and_then(|product| sum.checked_add(product))
            })
            .ok_or(StructureError::ArithmeticOverflow)?;
        rows.push((coefficients, rhs));
    }

    let solution =
        solve_rational_system(&rows, rank).ok_or(StructureError::RepInvariantViolation {
            invariant: "alcove center equations have no unique solution",
        })?;
    let mut denominator = 1_i64;
    for entry in &solution {
        let entry_denominator = i64::try_from(entry.denominator_ref())
            .map_err(|_| StructureError::ArithmeticOverflow)?;
        denominator = checked_lcm(denominator, entry_denominator)?;
    }
    let mut numerators = Vec::new();
    numerators
        .try_reserve_exact(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    for entry in &solution {
        let scaled = entry * Rational::from(denominator);
        numerators.push(
            i64::try_from(scaled.numerator_ref())
                .map_err(|_| StructureError::ArithmeticOverflow)?,
        );
    }
    let centered_gamma = RationalWeight::new(numerators, denominator)?;

    // alcoves.cpp:317-321: the correction may not leave the -theta-fixed
    // subspace on which the parameter's continuous coordinate lives.
    let theta = rc.theta(z)?;
    let difference = centered_gamma.sub(gamma)?;
    for (row, theta_row) in theta.weight_matrix().iter().enumerate() {
        let total = theta_row
            .iter()
            .zip(difference.numerator())
            .try_fold(difference.numerator()[row], |sum, (&entry, &coordinate)| {
                i64::from(entry)
                    .checked_mul(coordinate)
                    .and_then(|product| sum.checked_add(product))
            })
            .ok_or(StructureError::ArithmeticOverflow)?;
        if total != 0 {
            return Err(StructureError::RepInvariantViolation {
                invariant: "alcove correction lies outside the -theta fixed subspace",
            });
        }
    }

    let lambda_rho = rc.lambda_rho(z)?;
    rc.sr_gamma(z.x(), &lambda_rho, &centered_gamma)
}

/// Whether upstream's deformation denominator guard requests an alcove
/// centre: `denominator > 2^rank`.
///
/// Rational-weight denominators are positive `i64` values. For rank 63 and
/// above no such denominator can exceed the mathematical threshold, which is
/// outside the positive `i64` range; shifting a signed `i64` by 63 would
/// instead produce `i64::MIN` and must not be used as the bound.
pub fn denominator_exceeds_alcove_bound(rank: usize, denominator: i64) -> bool {
    rank < i64::BITS as usize - 1 && denominator > (1_i64 << rank)
}

#[derive(Debug)]
pub(crate) struct RootNumbering {
    npos: usize,
    by_nbr: Vec<RootId>,
}

impl RootNumbering {
    pub(crate) fn new(root_system: &RootSystem) -> Self {
        let total = root_system.roots().len();
        let mut positives: Vec<RootId> = (0..total)
            .map(RootId::from_usize)
            .filter(|&id| root_system.is_positive(id).unwrap_or(false))
            .collect();
        // Upstream order: simple-coordinate height first, then the
        // coordinate tuple compared from the LAST index backward. Heights
        // are precomputed once per root instead of re-summed per
        // comparison (this construction runs per `wall_set` caller, i.e.
        // per `int_item`/`alcove_center` call).
        let levels: Vec<i32> = (0..total)
            .map(|index| {
                root_system
                    .simple_coordinates(RootId::from_usize(index))
                    .unwrap_or(&[])
                    .iter()
                    .sum()
            })
            .collect();
        positives.sort_by(|&left, &right| {
            levels[left.index()]
                .cmp(&levels[right.index()])
                .then_with(|| {
                    root_system
                        .simple_coordinates(left)
                        .unwrap_or(&[])
                        .iter()
                        .rev()
                        .cmp(root_system.simple_coordinates(right).unwrap_or(&[]).iter().rev())
                })
        });
        let npos = positives.len();
        // Position of each positive root in `positives`, by root id: the
        // negative-half lookup below resolves `-id` through the negation
        // table and reads its position directly, replacing the per-call
        // BTreeMap<Vec<i32>> index (a top BTreeMap get/insert source in
        // the heavy-unitary profile, perf-unitary-3681949).
        let mut position_of = vec![usize::MAX; total];
        let mut by_nbr = vec![RootId::from_usize(0); total];
        for (position, &id) in positives.iter().enumerate() {
            position_of[id.index()] = position;
            by_nbr[npos + position] = id;
        }
        for index in 0..total {
            let id = RootId::from_usize(index);
            if root_system.is_positive(id).unwrap_or(false) {
                continue;
            }
            let negated = root_system.negatives()[id.index()];
            let position = position_of[negated.index()];
            debug_assert!(position != usize::MAX, "negation of a negative root");
            by_nbr[npos - 1 - position] = id;
        }
        Self { npos, by_nbr }
    }

    pub(crate) fn id(&self, nbr: usize) -> RootId {
        self.by_nbr[nbr]
    }

    fn is_negative(&self, nbr: usize) -> bool {
        nbr < self.npos
    }
}

pub(crate) fn wall_set(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    gamma: &RationalWeight,
) -> Result<(BTreeSet<usize>, BTreeSet<usize>), StructureError> {
    let num_roots = root_system.roots().len();
    // Coroot slices per numbering index, resolved once: the filtering loop
    // pairs every surviving root with every wall root.
    let mut coroots: Vec<&[i32]> = Vec::with_capacity(num_roots);
    for nbr in 0..num_roots {
        coroots.push(
            root_system
                .coroot(numbering.id(nbr))
                .ok_or(StructureError::RootSystemInvariantViolation {
                    invariant: "alcove root has no coroot",
                })?
                .as_slice(),
        );
    }
    // Membership table for coroot-difference probes. The per-call packed
    // u64 key set replaces the boxed-slice BTreeSet whose O(log n)
    // lexicographic slice compares dominated the heavy-unitary profile
    // (perf-unitary-3681949); systems whose coroots do not pack keep the
    // BTreeSet.
    let coroot_table = CorootTable::new(&coroots);
    let mut levels = (0..num_roots)
        .map(|nbr| Ok((nbr, frac_eval_value(root_system, numbering.id(nbr), gamma)?)))
        .collect::<Result<Vec<_>, StructureError>>()?;
    let mut walls = BTreeSet::new();
    let mut integrals = BTreeSet::new();
    while !levels.is_empty() {
        let min_level = levels.iter().map(|(_, level)| *level).min().ok_or(
            StructureError::RootSystemInvariantViolation {
                invariant: "nonempty alcove levels",
            },
        )?;
        // Upstream's stable front partition: minimal-level roots first.
        let mut mins: Vec<(usize, (i64, i64))> = Vec::new();
        let mut rest: Vec<(usize, (i64, i64))> = Vec::new();
        for item in levels {
            if item.1 == min_level {
                mins.push(item);
            } else {
                rest.push(item);
            }
        }
        // Process the minimal-level roots in order; each accepted wall
        // filters every survivor (unprocessed mins included) by whether its
        // coroot differs from the wall's by a coroot. A filtered-out min
        // leaves the queue without becoming a wall (upstream's `n_min`
        // decrement).
        let mut cursor = 0;
        while cursor < mins.len() {
            let (alpha, level) = mins[cursor];
            cursor += 1;
            if level.0 == 0 {
                integrals.insert(alpha);
            }
            walls.insert(alpha);
            let mut kept_mins = Vec::with_capacity(mins.len() - cursor);
            for &item in &mins[cursor..] {
                if !coroot_table.contains_difference(&coroots, alpha, item.0) {
                    kept_mins.push(item);
                }
            }
            let mut kept_rest = Vec::with_capacity(rest.len());
            for &item in &rest {
                if !coroot_table.contains_difference(&coroots, alpha, item.0) {
                    kept_rest.push(item);
                }
            }
            mins = kept_mins;
            rest = kept_rest;
            cursor = 0;
        }
        levels = rest;
    }
    Ok((walls, integrals))
}

/// Coroot membership table for `wall_set`'s difference probes. Every
/// finite-type coroot packs into `pack_root_key`'s 8-bit lanes (rank <= 8,
/// coordinates in [-128, 127]), so the common case is a `u64` hash set
/// plus the per-root packed keys for SWAR difference probes; systems whose
/// coroots do not pack keep the BTreeSet.
enum CorootTable {
    Packed {
        set: std::collections::HashSet<u64, PackedKeyHasherBuilder>,
        /// Packed key per coroot, aligned with the `coroots` argument order.
        keys: Vec<u64>,
        /// Low `8 * rank` bits set: lanes past the rank stay zero, matching
        /// how `pack_root_key` pads short vectors.
        lane_mask: u64,
    },
    Full(BTreeSet<Vec<i32>>),
}

impl CorootTable {
    fn new(coroots: &[&[i32]]) -> Self {
        let mut packed = std::collections::HashSet::with_capacity_and_hasher(
            coroots.len(),
            PackedKeyHasherBuilder,
        );
        let mut keys = Vec::with_capacity(coroots.len());
        for coroot in coroots {
            let Some(key) = crate::root_system::pack_root_key(coroot) else {
                return Self::Full(coroots.iter().map(|c| c.to_vec()).collect());
            };
            packed.insert(key);
            keys.push(key);
        }
        let rank = coroots.first().map_or(0, |coroot| coroot.len());
        let lane_mask = if rank >= 8 { u64::MAX } else { (1_u64 << (8 * rank)) - 1 };
        Self::Packed {
            set: packed,
            keys,
            lane_mask,
        }
    }

    /// Whether `coroots[alpha] - coroots[beta]` is tabled as a coroot.
    fn contains_difference(&self, coroots: &[&[i32]], alpha: usize, beta: usize) -> bool {
        match self {
            Self::Packed {
                set,
                keys,
                lane_mask,
            } => packed_difference_key(keys[alpha], keys[beta], *lane_mask)
                .is_some_and(|key| set.contains(&key)),
            Self::Full(tree) => {
                let difference = coroots[alpha]
                    .iter()
                    .zip(coroots[beta])
                    .map(|(&a, &b)| a - b)
                    .collect::<Vec<_>>();
                tree.contains(&difference)
            }
        }
    }
}

/// Per-lane biased subtraction of two `pack_root_key` values: with
/// `x_i = a_i + 128` and `y_i = b_i + 128` in lane `i`, the difference key
/// needs `a_i - b_i + 128 = x_i - y_i + 128` per lane, which is packable
/// exactly when `x_i - y_i` lands in [-128, 127]. The high-bit lanes of
/// `x`/`y` split the cases: equal high bits keep the difference inside
/// [-127, 127] (always valid); a set high bit only in `x` is valid iff the
/// low parts differ by at least 1 (then add 128 back); a set high bit only
/// in `y` is valid iff the low parts do not borrow (then subtract 128).
/// Valid lanes never carry or borrow across lane boundaries, so the whole
/// probe is a handful of `u64` ops instead of a per-lane i32 loop plus a
/// re-validation.
fn packed_difference_key(x: u64, y: u64, lane_mask: u64) -> Option<u64> {
    const HIGH: u64 = 0x8080_8080_8080_8080;
    // Lane i: (x_i | 0x80) - (y_i & 0x7f) is in [1, 255], so no inter-lane
    // borrow; it equals (x_i mod 128) - (y_i mod 128) + 128.
    let t = (x | HIGH).wrapping_sub(y & !HIGH);
    let high_x = x & HIGH;
    let high_y = y & HIGH;
    let high_t = t & HIGH;
    let invalid = (high_x & !high_y & high_t) | (!high_x & high_y & !high_t & HIGH);
    if invalid & lane_mask != 0 {
        return None;
    }
    let plus = high_x & !high_y & HIGH;
    let minus = !high_x & high_y & HIGH;
    Some(t.wrapping_add(plus).wrapping_sub(minus) & lane_mask)
}

/// murmur3-fmix64 finalizer over the packed coordinate key: the packing
/// leaves entropy in the low lanes, which hashbrown's low-bit bucket index
/// would use unmixed.
#[derive(Clone, Default)]
struct PackedKeyHasherBuilder;

impl std::hash::BuildHasher for PackedKeyHasherBuilder {
    type Hasher = PackedKeyHasher;
    fn build_hasher(&self) -> PackedKeyHasher {
        PackedKeyHasher(0)
    }
}

struct PackedKeyHasher(u64);

impl std::hash::Hasher for PackedKeyHasher {
    fn finish(&self) -> u64 {
        let mut x = self.0;
        x ^= x >> 33;
        x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
        x ^ (x >> 33)
    }
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = (self.0 << 8) | u64::from(byte);
        }
    }
    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }
}

fn frac_eval_value(
    root_system: &RootSystem,
    id: RootId,
    gamma: &RationalWeight,
) -> Result<(i64, i64), StructureError> {
    let coroot = root_system
        .coroot(id)
        .ok_or(StructureError::RootSystemInvariantViolation {
            invariant: "alcove root has no coroot",
        })?;
    let dot = checked_dot(gamma.numerator(), coroot.as_slice())?;
    let denominator = gamma.denominator();
    let mut remainder = dot.rem_euclid(denominator);
    if remainder == 0 && !root_system.is_positive(id).unwrap_or(false) {
        remainder = denominator;
    }
    Ok((remainder, denominator))
}

fn floor_eval_nbr(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    nbr: usize,
    gamma: &RationalWeight,
) -> Result<i64, StructureError> {
    let coroot = root_system.coroot(numbering.id(nbr)).ok_or(
        StructureError::RootSystemInvariantViolation {
            invariant: "alcove root has no coroot",
        },
    )?;
    let dot = checked_dot(gamma.numerator(), coroot.as_slice())?;
    let denominator = gamma.denominator();
    let floor = dot.div_euclid(denominator);
    if numbering.is_negative(nbr) && dot.rem_euclid(denominator) == 0 {
        floor
            .checked_sub(1)
            .ok_or(StructureError::ArithmeticOverflow)
    } else {
        Ok(floor)
    }
}

pub(crate) fn checked_dot(left: &[i64], right: &[i32]) -> Result<i64, StructureError> {
    if left.len() != right.len() {
        return Err(StructureError::RankMismatch {
            expected: left.len(),
            actual: right.len(),
        });
    }
    left.iter()
        .zip(right)
        .try_fold(0_i64, |sum, (&left, &right)| {
            left.checked_mul(i64::from(right))
                .and_then(|product| sum.checked_add(product))
                .ok_or(StructureError::ArithmeticOverflow)
        })
}

pub(crate) fn root_components(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    walls: &BTreeSet<usize>,
) -> Vec<Vec<usize>> {
    let sorted = walls.iter().copied().collect::<Vec<_>>();
    let mut parent = (0..sorted.len()).collect::<Vec<_>>();
    fn find(parent: &mut [usize], index: usize) -> usize {
        let mut root = index;
        while parent[root] != root {
            root = parent[root];
        }
        let mut current = index;
        while parent[current] != current {
            let next = parent[current];
            parent[current] = root;
            current = next;
        }
        root
    }
    for left in 0..sorted.len() {
        for right in (left + 1)..sorted.len() {
            if root_system
                .bracket(numbering.id(sorted[left]), numbering.id(sorted[right]))
                .unwrap_or(0)
                != 0
            {
                let left_root = find(&mut parent, left);
                let right_root = find(&mut parent, right);
                if left_root != right_root {
                    parent[right_root] = left_root;
                }
            }
        }
    }
    let mut components: Vec<Vec<usize>> = Vec::new();
    let mut root_to_component: BTreeMap<usize, usize> = BTreeMap::new();
    for (index, &nbr) in sorted.iter().enumerate() {
        let root = find(&mut parent, index);
        if let Some(&slot) = root_to_component.get(&root) {
            components[slot].push(nbr);
        } else {
            root_to_component.insert(root, components.len());
            components.push(vec![nbr]);
        }
    }
    components
}

fn barycentre_eq(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    walls: &BTreeSet<usize>,
    integrals: &BTreeSet<usize>,
) -> Result<Vec<(i64, i64)>, StructureError> {
    let wall_list = walls.iter().copied().collect::<Vec<_>>();
    let mut result = vec![(0_i64, 1_i64); wall_list.len()];
    for component in root_components(root_system, numbering, walls) {
        let labels = labels_for_component(root_system, numbering, &component)?;
        let off_walls = component
            .iter()
            .copied()
            .filter(|nbr| !integrals.contains(nbr))
            .collect::<Vec<_>>();
        let n_off =
            i64::try_from(off_walls.len()).map_err(|_| StructureError::ArithmeticOverflow)?;
        for nbr in off_walls {
            let position = component.iter().position(|&entry| entry == nbr).ok_or(
                StructureError::RootSystemInvariantViolation {
                    invariant: "alcove component member",
                },
            )?;
            let slot = wall_list.iter().position(|&entry| entry == nbr).ok_or(
                StructureError::RootSystemInvariantViolation {
                    invariant: "alcove wall member",
                },
            )?;
            result[slot] = (
                1,
                n_off
                    .checked_mul(labels[position])
                    .ok_or(StructureError::ArithmeticOverflow)?,
            );
        }
    }
    Ok(result)
}

fn labels_for_component(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    component: &[usize],
) -> Result<Vec<i64>, StructureError> {
    let columns = component
        .iter()
        .map(|&nbr| {
            root_system
                .coroot(numbering.id(nbr))
                .map(|coroot| coroot.as_slice().to_vec())
                .ok_or(StructureError::RootSystemInvariantViolation {
                    invariant: "alcove wall has no coroot",
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let rows = columns.first().map_or(0, Vec::len);
    let mut matrix = (0..rows)
        .map(|row| {
            columns
                .iter()
                .map(|column| Rational::from(column[row]))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let width = columns.len();
    let mut pivot_of_column = vec![None; width];
    let mut pivot_row = 0;
    for column in 0..width {
        if pivot_row >= rows {
            break;
        }
        let Some(found) = (pivot_row..rows).find(|&row| matrix[row][column] != 0) else {
            continue;
        };
        matrix.swap(pivot_row, found);
        let pivot = matrix[pivot_row][column].clone();
        for entry in &mut matrix[pivot_row] {
            *entry /= &pivot;
        }
        for row in 0..rows {
            if row == pivot_row || matrix[row][column] == 0 {
                continue;
            }
            let factor = matrix[row][column].clone();
            let (pivot_line, target) = if row < pivot_row {
                let (head, tail) = matrix.split_at_mut(pivot_row);
                (&tail[0], &mut head[row])
            } else {
                let (head, tail) = matrix.split_at_mut(row);
                (&head[pivot_row], &mut tail[0])
            };
            for (target_entry, pivot_entry) in target.iter_mut().zip(pivot_line) {
                *target_entry -= pivot_entry.clone() * &factor;
            }
        }
        pivot_of_column[column] = Some(pivot_row);
        pivot_row += 1;
    }
    let free = (0..width)
        .filter(|&column| pivot_of_column[column].is_none())
        .collect::<Vec<_>>();
    if free.len() != 1 {
        return Err(StructureError::RootSystemInvariantViolation {
            invariant: "alcove wall component must have one coroot relation",
        });
    }
    let mut relation = vec![Rational::from(0); width];
    relation[free[0]] = Rational::from(1);
    for (column, pivot) in pivot_of_column.iter().enumerate() {
        if let Some(row) = pivot {
            relation[column] = -matrix[*row][free[0]].clone();
        }
    }
    let mut denominator = 1_i64;
    for entry in &relation {
        denominator = checked_lcm(
            denominator,
            i64::try_from(entry.denominator_ref())
                .map_err(|_| StructureError::ArithmeticOverflow)?,
        )?;
    }
    let mut integral = relation
        .iter()
        .map(|entry| {
            let scaled = entry * Rational::from(denominator);
            i64::try_from(scaled.numerator_ref()).map_err(|_| StructureError::ArithmeticOverflow)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let divisor = integral
        .iter()
        .fold(0_i64, |divisor, &entry| gcd(divisor, entry));
    if divisor > 1 {
        for entry in &mut integral {
            *entry /= divisor;
        }
    }
    if integral.first().is_some_and(|&first| first < 0) {
        for entry in &mut integral {
            *entry = entry
                .checked_neg()
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
    }
    Ok(integral)
}

fn solve_rational_system(rows: &[(Vec<i64>, i64)], columns: usize) -> Option<Vec<Rational>> {
    let row_count = rows.len();
    let mut augmented = rows
        .iter()
        .map(|(coefficients, rhs)| {
            coefficients
                .iter()
                .map(|&entry| Rational::from(entry))
                .chain(std::iter::once(Rational::from(*rhs)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut pivot_rows = Vec::with_capacity(columns);
    let mut pivot_row = 0;
    for column in 0..columns {
        let found = (pivot_row..row_count).find(|&row| augmented[row][column] != 0)?;
        augmented.swap(pivot_row, found);
        let pivot = augmented[pivot_row][column].clone();
        for entry in &mut augmented[pivot_row] {
            *entry /= &pivot;
        }
        for row in 0..row_count {
            if row == pivot_row || augmented[row][column] == 0 {
                continue;
            }
            let factor = augmented[row][column].clone();
            let (pivot_line, target) = if row < pivot_row {
                let (head, tail) = augmented.split_at_mut(pivot_row);
                (&tail[0], &mut head[row])
            } else {
                let (head, tail) = augmented.split_at_mut(row);
                (&head[pivot_row], &mut tail[0])
            };
            for (target_entry, pivot_entry) in target.iter_mut().zip(pivot_line) {
                *target_entry -= pivot_entry.clone() * &factor;
            }
        }
        pivot_rows.push(pivot_row);
        pivot_row += 1;
    }
    if augmented
        .iter()
        .any(|row| row[..columns].iter().all(|coefficient| coefficient == &0) && row[columns] != 0)
    {
        return None;
    }
    let mut solution = vec![Rational::from(0); columns];
    for (column, &row) in pivot_rows.iter().enumerate() {
        solution[column] = augmented[row][columns].clone();
    }
    Some(solution)
}

fn gcd(mut left: i64, mut right: i64) -> i64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.abs()
}

fn checked_lcm(left: i64, right: i64) -> Result<i64, StructureError> {
    if left == 0 || right == 0 {
        return Ok(0);
    }
    left.checked_div(gcd(left, right))
        .and_then(|reduced| reduced.checked_mul(right))
        .ok_or(StructureError::ArithmeticOverflow)
}

/// The root-lattice vertex of the alcove containing `gamma`
/// (`weyl::root_vertex_of_alcove`, alcoves.cpp:414-428): summed over the
/// wall components, so that `gamma - vertex` lies in the Weyl orbit of the
/// fundamental alcove. This is step (a) of upstream's `InnerClass::int_item`
/// (innerclass.cpp:1123).
pub(crate) fn root_vertex_of_alcove(
    root_system: &RootSystem,
    gamma: &RationalWeight,
) -> Result<Weight, StructureError> {
    let numbering = RootNumbering::new(root_system);
    let (walls, _integrals) = wall_set(root_system, &numbering, gamma)?;
    let mut result = vec![0_i64; root_system.lattice_rank()];
    for component in root_components(root_system, &numbering, &walls) {
        let mut ev_floors = Vec::new();
        ev_floors.try_reserve_exact(component.len()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: component.len(),
            }
        })?;
        for &nbr in &component {
            // alcoves.cpp:422-425: the plain rational floor, NOT the
            // negative-root adjusted `floor_eval` (alcoves.cpp:29-32).
            let coroot = root_system.coroot(numbering.id(nbr)).ok_or(
                StructureError::RootSystemInvariantViolation {
                    invariant: "alcove wall has no coroot",
                },
            )?;
            let dot = checked_dot(gamma.numerator(), coroot.as_slice())?;
            ev_floors.push(dot.div_euclid(gamma.denominator()));
        }
        let vertex = root_vertex_simple(root_system, &numbering, &component, &ev_floors)?;
        for (entry, &coordinate) in result.iter_mut().zip(vertex.as_slice()) {
            *entry = entry
                .checked_add(i64::from(coordinate))
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
    }
    let mut coordinates = Vec::new();
    coordinates
        .try_reserve_exact(result.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: result.len(),
        })?;
    for entry in result {
        coordinates.push(i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?);
    }
    Ok(Weight::new(coordinates))
}

/// `weyl::root_vertex_simple` (alcoves.cpp:347-412): the unique vertex of
/// the simplex for one wall component (the projection of the alcove onto
/// the span of its coroots) that is an integer combination of the roots.
///
/// The coroot relation of the component (its primitive kernel vector) has
/// all-positive coefficients; the first wall with coefficient 1 is dropped
/// and the remaining walls pin the vertex via the transposed sub-Cartan
/// inverse. When the unshifted solution is not integral, upstream retries
/// with each further coefficient-1 wall's evaluation bumped by one — the
/// `labels_1` loop of alcoves.cpp:395-408.
fn root_vertex_simple(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    component: &[usize],
    ev_floors: &[i64],
) -> Result<Weight, StructureError> {
    let labels = labels_for_component(root_system, numbering, component)?;
    let Some(chosen) = labels.iter().position(|&label| label == 1) else {
        return Err(StructureError::RootSystemInvariantViolation {
            invariant: "alcove component has no coefficient-1 wall",
        });
    };
    let mut generators = Vec::new();
    let mut floors = Vec::new();
    generators
        .try_reserve_exact(component.len() - 1)
        .map_err(|_| StructureError::AllocationFailed {
            requested: component.len() - 1,
        })?;
    floors.try_reserve_exact(component.len() - 1).map_err(|_| {
        StructureError::AllocationFailed {
            requested: component.len() - 1,
        }
    })?;
    let mut labels_1 = Vec::new();
    for (index, &nbr) in component.iter().enumerate() {
        if index == chosen {
            continue;
        }
        if index > chosen && labels[index] == 1 {
            // `labels_1.push_back(i-1)`: the generator index of this wall.
            labels_1.push(generators.len());
        }
        generators.push(nbr);
        floors.push(ev_floors[index]);
    }

    // Upstream inverts `rd.Cartan_matrix(generators).transposed()` with a
    // denominator (matrix.cpp:471-503 `matrix::inverse`).
    let size = generators.len();
    let mut transposed = vec![vec![0_i64; size]; size];
    for (row, &row_nbr) in generators.iter().enumerate() {
        for (column, &column_nbr) in generators.iter().enumerate() {
            transposed[row][column] =
                i64::from(root_system.bracket(numbering.id(column_nbr), numbering.id(row_nbr))?);
        }
    }
    let (inverse_numerator, denominator) =
        rational_inverse(&transposed)?.ok_or(StructureError::RootSystemInvariantViolation {
            invariant: "alcove generator Cartan matrix is singular",
        })?;

    let base = matrix_vector_product(&inverse_numerator, &floors)?;
    let mut attempts = Vec::new();
    attempts
        .try_reserve_exact(labels_1.len() + 1)
        .map_err(|_| StructureError::AllocationFailed {
            requested: labels_1.len() + 1,
        })?;
    attempts.push(base.clone());
    for &column in &labels_1 {
        let mut shifted = Vec::new();
        shifted
            .try_reserve_exact(size)
            .map_err(|_| StructureError::AllocationFailed { requested: size })?;
        for (row, &entry) in base.iter().enumerate() {
            shifted.push(
                entry
                    .checked_add(inverse_numerator[row][column])
                    .ok_or(StructureError::ArithmeticOverflow)?,
            );
        }
        attempts.push(shifted);
    }
    for numerator in attempts {
        // `vertex_adjoint.normalize().denominator()==1`: the normalized
        // denominator is `d/gcd(numerators, d)`, so integrality is exactly
        // divisibility of every numerator entry by `d`.
        if numerator.iter().all(|&entry| entry % denominator == 0) {
            let mut result = vec![0_i64; root_system.lattice_rank()];
            for (index, &entry) in numerator.iter().enumerate() {
                let coefficient = entry / denominator;
                let root = root_system.root(numbering.id(generators[index])).ok_or(
                    StructureError::RootSystemInvariantViolation {
                        invariant: "alcove generator has no root",
                    },
                )?;
                for (target, &coordinate) in result.iter_mut().zip(root.as_slice()) {
                    let term = coefficient
                        .checked_mul(i64::from(coordinate))
                        .ok_or(StructureError::ArithmeticOverflow)?;
                    *target = target
                        .checked_add(term)
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
            }
            let mut coordinates = Vec::new();
            coordinates.try_reserve_exact(result.len()).map_err(|_| {
                StructureError::AllocationFailed {
                    requested: result.len(),
                }
            })?;
            for entry in result {
                coordinates
                    .push(i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?);
            }
            return Ok(Weight::new(coordinates));
        }
    }
    Err(StructureError::RootSystemInvariantViolation {
        invariant: "alcove vertex lies outside the root lattice",
    })
}

/// `inverse_numerator * vector` with checked arithmetic.
fn matrix_vector_product(matrix: &[Vec<i64>], vector: &[i64]) -> Result<Vec<i64>, StructureError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(matrix.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: matrix.len(),
        })?;
    for row in matrix {
        let mut entry = 0_i64;
        for (&coefficient, &coordinate) in row.iter().zip(vector) {
            let term = coefficient
                .checked_mul(coordinate)
                .ok_or(StructureError::ArithmeticOverflow)?;
            entry = entry
                .checked_add(term)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        result.push(entry);
    }
    Ok(result)
}

/// Exact rational inverse of a small integer matrix as `(numerator, d)`
/// with `inverse = numerator / d` and `d > 0` — the
/// `matrix::inverse(A, d)` contract (utilities/matrix.cpp:471-503).
/// `Ok(None)` reports a singular matrix.
fn rational_inverse(matrix: &[Vec<i64>]) -> Result<Option<(Vec<Vec<i64>>, i64)>, StructureError> {
    let n = matrix.len();
    if matrix.iter().any(|row| row.len() != n) {
        return Ok(None);
    }
    let mut left: Vec<Vec<Rational>> = matrix
        .iter()
        .map(|row| row.iter().map(|&entry| Rational::from(entry)).collect())
        .collect();
    let mut right: Vec<Vec<Rational>> = (0..n)
        .map(|row| {
            (0..n)
                .map(|column| Rational::from(if row == column { 1 } else { 0 }))
                .collect()
        })
        .collect();
    for column in 0..n {
        let Some(found) = (column..n).find(|&row| left[row][column] != 0) else {
            return Ok(None);
        };
        left.swap(column, found);
        right.swap(column, found);
        let pivot = left[column][column].clone();
        for entry in &mut left[column] {
            *entry /= &pivot;
        }
        for entry in &mut right[column] {
            *entry /= &pivot;
        }
        for row in 0..n {
            if row == column || left[row][column] == 0 {
                continue;
            }
            let factor = left[row][column].clone();
            for target in column..n {
                let correction = left[column][target].clone() * &factor;
                left[row][target] -= correction;
            }
            for target in 0..n {
                let correction = right[column][target].clone() * &factor;
                right[row][target] -= correction;
            }
        }
    }
    let mut denominator = 1_i64;
    for row in &right {
        for entry in row {
            let entry_denominator = i64::try_from(entry.denominator_ref())
                .map_err(|_| StructureError::ArithmeticOverflow)?;
            denominator = checked_lcm(denominator, entry_denominator)?;
        }
    }
    let mut numerator = Vec::new();
    numerator
        .try_reserve_exact(n)
        .map_err(|_| StructureError::AllocationFailed { requested: n })?;
    for row in &right {
        let mut numerator_row = Vec::new();
        numerator_row
            .try_reserve_exact(n)
            .map_err(|_| StructureError::AllocationFailed { requested: n })?;
        for entry in row {
            let scaled = entry * Rational::from(denominator);
            numerator_row.push(
                i64::try_from(scaled.numerator_ref())
                    .map_err(|_| StructureError::ArithmeticOverflow)?,
            );
        }
        numerator.push(numerator_row);
    }
    Ok(Some((numerator, denominator)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BasedRootDatum;

    #[test]
    fn denominator_center_bound_handles_signed_shift_boundary() {
        let rank_62_bound = 1_i64 << 62;
        assert!(!denominator_exceeds_alcove_bound(62, rank_62_bound));
        assert!(denominator_exceeds_alcove_bound(62, rank_62_bound + 1));
        assert!(!denominator_exceeds_alcove_bound(63, i64::MAX));
        assert!(!denominator_exceeds_alcove_bound(64, i64::MAX));
    }

    #[test]
    fn packed_difference_key_matches_coordinate_packing() {
        fn reference(x: u64, y: u64, rank: usize) -> Option<u64> {
            let mut difference = Vec::with_capacity(rank);
            for lane in 0..rank {
                let a = ((x >> (8 * lane)) & 0xff) as i32 - 128;
                let b = ((y >> (8 * lane)) & 0xff) as i32 - 128;
                difference.push(a - b);
            }
            crate::root_system::pack_root_key(&difference)
        }

        let corners = [-128_i32, -127, -2, -1, 0, 1, 2, 3, 126, 127];
        let pack = |coordinates: &[i32]| crate::root_system::pack_root_key(coordinates).unwrap();
        for rank in 1..=8_usize {
            let lane_mask = if rank >= 8 { u64::MAX } else { (1_u64 << (8 * rank)) - 1 };
            // Corner pairs in every lane position, other lanes pinned to 0.
            for lane in 0..rank {
                for &a in &corners {
                    for &b in &corners {
                        let mut left = vec![0_i32; rank];
                        let mut right = vec![0_i32; rank];
                        left[lane] = a;
                        right[lane] = b;
                        assert_eq!(
                            packed_difference_key(pack(&left), pack(&right), lane_mask),
                            reference(pack(&left), pack(&right), rank),
                            "rank={rank} lane={lane} a={a} b={b}"
                        );
                    }
                }
            }
            // Deterministic multi-lane fuzz over the full packable range.
            let mut state = 0x9e3779b97f4a7c15_u64;
            let mut next = move || {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state
            };
            for _ in 0..20_000 {
                let left: Vec<i32> = (0..rank).map(|_| (next() % 256) as i32 - 128).collect();
                let right: Vec<i32> = (0..rank).map(|_| (next() % 256) as i32 - 128).collect();
                assert_eq!(
                    packed_difference_key(pack(&left), pack(&right), lane_mask),
                    reference(pack(&left), pack(&right), rank),
                    "rank={rank} left={left:?} right={right:?}"
                );
            }
        }
    }

    #[test]
    fn rational_solver_rejects_inconsistent_overdetermined_rows() {
        let rows = vec![(vec![1], 0), (vec![1], 1)];
        assert_eq!(solve_rational_system(&rows, 1), None);
    }

    /// The pre-partition `wall_set`: per-iteration sort, `remove(0)` queue,
    /// and per-pair heap-allocated coroot differences.
    fn reference_wall_set(
        root_system: &RootSystem,
        numbering: &RootNumbering,
        gamma: &RationalWeight,
    ) -> Result<(BTreeSet<usize>, BTreeSet<usize>), StructureError> {
        let num_roots = root_system.roots().len();
        let coroot_table: BTreeSet<Vec<i32>> = (0..num_roots)
            .filter_map(|index| {
                root_system
                    .coroot(RootId::from_usize(index))
                    .map(|coroot| coroot.as_slice().to_vec())
            })
            .collect();
        let mut levels = (0..num_roots)
            .map(|nbr| Ok((nbr, frac_eval_value(root_system, numbering.id(nbr), gamma)?)))
            .collect::<Result<Vec<_>, StructureError>>()?;
        let mut walls = BTreeSet::new();
        let mut integrals = BTreeSet::new();
        while !levels.is_empty() {
            let min_level = levels.iter().map(|(_, level)| *level).min().unwrap();
            levels.sort_by_key(|(_, level)| (*level != min_level) as u8);
            let mut n_min = levels
                .iter()
                .take_while(|(_, level)| *level == min_level)
                .count();
            while n_min > 0 && !levels.is_empty() {
                let (alpha, level) = levels.remove(0);
                if level.0 == 0 {
                    integrals.insert(alpha);
                }
                walls.insert(alpha);
                n_min -= 1;
                let alpha_coroot = root_system
                    .coroot(numbering.id(alpha))
                    .unwrap()
                    .as_slice()
                    .to_vec();
                let mut kept = Vec::new();
                for item in levels.drain(..) {
                    let beta_coroot = root_system.coroot(numbering.id(item.0)).unwrap().as_slice();
                    let difference = alpha_coroot
                        .iter()
                        .zip(beta_coroot)
                        .map(|(&alpha, &beta)| alpha - beta)
                        .collect::<Vec<_>>();
                    if !coroot_table.contains(&difference) {
                        kept.push(item);
                    } else if item.1 == min_level {
                        n_min = n_min.saturating_sub(1);
                    }
                }
                levels = kept;
            }
        }
        Ok((walls, integrals))
    }

    /// The pre-array `RootNumbering::new`: per-comparison level sums and a
    /// `BTreeMap<Vec<i32>>` positive-root index.
    fn reference_root_numbering(root_system: &RootSystem) -> RootNumbering {
        let total = root_system.roots().len();
        let mut positives: Vec<RootId> = (0..total)
            .map(RootId::from_usize)
            .filter(|&id| root_system.is_positive(id).unwrap_or(false))
            .collect();
        positives.sort_by(|&left, &right| {
            let left_coordinates = root_system.simple_coordinates(left).unwrap_or(&[]);
            let right_coordinates = root_system.simple_coordinates(right).unwrap_or(&[]);
            let left_level: i32 = left_coordinates.iter().sum();
            let right_level: i32 = right_coordinates.iter().sum();
            left_level.cmp(&right_level).then_with(|| {
                for index in (0..left_coordinates.len()).rev() {
                    let difference = left_coordinates[index] - right_coordinates[index];
                    if difference != 0 {
                        return difference.cmp(&0);
                    }
                }
                std::cmp::Ordering::Equal
            })
        });
        let npos = positives.len();
        let mut positive_index = BTreeMap::new();
        let mut by_nbr = vec![RootId::from_usize(0); total];
        for (position, &id) in positives.iter().enumerate() {
            positive_index.insert(
                root_system.simple_coordinates(id).unwrap_or(&[]).to_vec(),
                position,
            );
            by_nbr[npos + position] = id;
        }
        for index in 0..total {
            let id = RootId::from_usize(index);
            if root_system.is_positive(id).unwrap_or(false) {
                continue;
            }
            let negated = root_system
                .simple_coordinates(id)
                .unwrap_or(&[])
                .iter()
                .map(|&coordinate| -coordinate)
                .collect::<Vec<_>>();
            let position = positive_index[&negated];
            by_nbr[npos - 1 - position] = id;
        }
        RootNumbering { npos, by_nbr }
    }

    #[test]
    fn root_numbering_matches_reference() {
        let systems = [
            RootSystem::enumerate(
                &BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap(),
                6,
            )
            .unwrap(),
            RootSystem::enumerate(
                &BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap(),
                8,
            )
            .unwrap(),
            RootSystem::enumerate(
                &BasedRootDatum::standard(vec![vec![2, -1], vec![-3, 2]]).unwrap(),
                12,
            )
            .unwrap(),
        ];
        for system in &systems {
            let expected = reference_root_numbering(system);
            let actual = RootNumbering::new(system);
            assert_eq!(actual.npos, expected.npos);
            assert_eq!(actual.by_nbr, expected.by_nbr);
        }
    }

    #[test]
    fn wall_set_matches_reference_across_gammas() {        let systems = [
            RootSystem::enumerate(
                &BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap(),
                6,
            )
            .unwrap(),
            RootSystem::enumerate(
                &BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap(),
                8,
            )
            .unwrap(),
            RootSystem::enumerate(
                &BasedRootDatum::standard(vec![vec![2, -1], vec![-3, 2]]).unwrap(),
                12,
            )
            .unwrap(),
        ];
        let gammas = [
            RationalWeight::new(vec![1, 0], 1).unwrap(),
            RationalWeight::new(vec![3, 5], 2).unwrap(),
            RationalWeight::new(vec![-2, 7], 3).unwrap(),
            RationalWeight::new(vec![0, 0], 1).unwrap(),
            RationalWeight::new(vec![11, -13], 6).unwrap(),
        ];
        for system in &systems {
            let numbering = RootNumbering::new(system);
            for gamma in &gammas {
                assert_eq!(
                    wall_set(system, &numbering, gamma).unwrap(),
                    reference_wall_set(system, &numbering, gamma).unwrap(),
                    "gamma={gamma:?}"
                );
            }
        }
    }
}
