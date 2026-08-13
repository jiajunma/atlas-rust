//! Alcove geometry used by deformation preprocessing.
//!
//! This module owns the domain-level form of `weyl::alcove_center`
//! (`alcoves.cpp:277-341`).  It keeps a standard parameter's KGB and
//! lambda-rho data and replaces only its infinitesimal character.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use malachite::Rational;

use crate::{RationalWeight, RepContext, RootId, RootSystem, StandardRepr, StructureError};

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
struct RootNumbering {
    npos: usize,
    by_nbr: Vec<RootId>,
}

impl RootNumbering {
    fn new(root_system: &RootSystem) -> Self {
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
                Ordering::Equal
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
        Self { npos, by_nbr }
    }

    fn id(&self, nbr: usize) -> RootId {
        self.by_nbr[nbr]
    }

    fn is_negative(&self, nbr: usize) -> bool {
        nbr < self.npos
    }
}

fn wall_set(
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
        let min_level = levels.iter().map(|(_, level)| *level).min().ok_or(
            StructureError::RootSystemInvariantViolation {
                invariant: "nonempty alcove levels",
            },
        )?;
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
                .ok_or(StructureError::RootSystemInvariantViolation {
                    invariant: "alcove root has no coroot",
                })?
                .as_slice()
                .to_vec();
            let mut kept = Vec::new();
            for item in levels.drain(..) {
                let beta_coroot = root_system
                    .coroot(numbering.id(item.0))
                    .ok_or(StructureError::RootSystemInvariantViolation {
                        invariant: "alcove root has no coroot",
                    })?
                    .as_slice();
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

fn checked_dot(left: &[i64], right: &[i32]) -> Result<i64, StructureError> {
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

fn root_components(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denominator_center_bound_handles_signed_shift_boundary() {
        let rank_62_bound = 1_i64 << 62;
        assert!(!denominator_exceeds_alcove_bound(62, rank_62_bound));
        assert!(denominator_exceeds_alcove_bound(62, rank_62_bound + 1));
        assert!(!denominator_exceeds_alcove_bound(63, i64::MAX));
        assert!(!denominator_exceeds_alcove_bound(64, i64::MAX));
    }

    #[test]
    fn rational_solver_rejects_inconsistent_overdetermined_rows() {
        let rows = vec![(vec![1], 0), (vec![1], 1)];
        assert_eq!(solve_rational_system(&rows, 1), None);
    }
}
