//! Port of upstream `DynkinDiagram` (structure/dynkin.cpp:28-220): connected
//! components of a finite Cartan matrix, each classified with its Bourbaki
//! vertex order. The component order and per-component `position` vectors
//! reproduce upstream `type()`/`perm()` exactly, including its tie choices
//! (rank-two B/C decided by the given order, any-end starts for A and D4,
//! the E-type long-arm swap).

use std::collections::BTreeSet;

use crate::StructureError;

/// One connected component of a Dynkin diagram.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DynkinComponent {
    /// The simple type letter: A, B, C, D, E, F, or G.
    pub(crate) letter: char,
    /// Datum vertex indices of this component.
    pub(crate) support: BTreeSet<usize>,
    /// The component's vertices in Bourbaki order for its type.
    pub(crate) position: Vec<usize>,
}

impl DynkinComponent {
    /// The lowest datum vertex of this component (upstream `offset`).
    pub(crate) fn offset(&self) -> usize {
        *self.support.iter().next().expect("nonempty Dynkin support")
    }
}

fn first(set: &BTreeSet<usize>) -> usize {
    *set.iter()
        .next()
        .expect("Dynkin vertex sets are nonempty by construction")
}

/// Classify a validated Cartan matrix into connected typed components.
///
/// The input must satisfy the based-datum Cartan invariants (2 on the
/// diagonal, non-positive symmetric adjacency); violations are reported as
/// layout invariant errors because callers construct from checked data.
pub(crate) fn classify(cartan: &[Vec<i32>]) -> Result<Vec<DynkinComponent>, StructureError> {
    let rank = cartan.len();
    if cartan.iter().any(|row| row.len() != rank) {
        return Err(StructureError::NonSquareCartan);
    }

    // Adjacency and labelled (multiple) edges, mirroring the upstream ctor.
    let mut star: Vec<BTreeSet<usize>> = (0..rank).map(|_| BTreeSet::new()).collect();
    let mut down_edges = Vec::new();
    for (i, row) in cartan.iter().enumerate() {
        for (j, &entry) in row.iter().enumerate() {
            if i == j {
                if entry != 2 {
                    return Err(StructureError::LayoutInvariantViolation {
                        invariant: "Cartan diagonal",
                    });
                }
                continue;
            }
            if !(-3..=0).contains(&entry) {
                return Err(StructureError::LayoutInvariantViolation {
                    invariant: "Cartan off-diagonal",
                });
            }
            if entry != 0 {
                if cartan[j][i] == 0 {
                    return Err(StructureError::LayoutInvariantViolation {
                        invariant: "Cartan adjacency symmetry",
                    });
                }
                star[j].insert(i);
                if entry < -1 {
                    down_edges.push((i, j, -entry));
                }
            }
        }
    }

    // Components, in upstream first-fresh-vertex order with first-match merging.
    let mut comps: Vec<BTreeSet<usize>> = Vec::new();
    for (i, neighbours) in star.iter().enumerate() {
        if neighbours.iter().all(|&j| j > i) {
            comps.push(BTreeSet::from([i]));
            continue;
        }
        let first_match = comps
            .iter()
            .position(|support| !support.is_disjoint(neighbours))
            .ok_or(StructureError::LayoutInvariantViolation {
                invariant: "component merge",
            })?;
        comps[first_match].insert(i);
        let mut candidate = first_match + 1;
        while candidate < comps.len() {
            if !comps[candidate].is_disjoint(neighbours) {
                let merged = comps.remove(candidate);
                comps[first_match].extend(merged);
            } else {
                candidate += 1;
            }
        }
    }

    comps
        .iter()
        .map(|support| classify_component(cartan, &star, &down_edges, support))
        .collect()
}

/// Per-component port of `DynkinDiagram::classify(Cartan, comp)`.
fn classify_component(
    cartan: &[Vec<i32>],
    star: &[BTreeSet<usize>],
    down_edges: &[(usize, usize, i32)],
    support: &BTreeSet<usize>,
) -> Result<DynkinComponent, StructureError> {
    let comp_rank = support.len();
    let invalid = |invariant: &'static str| StructureError::LayoutInvariantViolation { invariant };

    if comp_rank <= 2 {
        if comp_rank == 1 {
            return Ok(DynkinComponent {
                letter: 'A',
                support: support.clone(),
                position: vec![first(support)],
            });
        }
        let mut vertices = support.iter().copied();
        let (i, j) = (
            vertices.next().expect("rank two"),
            vertices.next().expect("rank two"),
        );
        let mut position = vec![i, j];
        let letter = match cartan[i][j] * cartan[j][i] {
            1 => 'A',
            // Exceptionally the given order decides the type (dynkin.cpp:113).
            2 => {
                if cartan[i][j] == -1 {
                    'C'
                } else {
                    'B'
                }
            }
            3 => {
                if cartan[i][j] != -1 {
                    position.swap(0, 1);
                }
                'G'
            }
            _ => return Err(invalid("rank-two Cartan product")),
        };
        return Ok(DynkinComponent {
            letter,
            support: support.clone(),
            position,
        });
    }

    let mut fork = None;
    let mut lower = None;
    let mut upper = None;
    let mut extremities = BTreeSet::new();
    for &i in support {
        match star[i].len() {
            0 | 1 => {
                extremities.insert(i);
            }
            2 => {}
            3 => {
                if fork.replace(i).is_some() {
                    return Err(invalid("multiple fork nodes"));
                }
            }
            _ => return Err(invalid("node degree above three")),
        }
    }
    if extremities.len() < 2 {
        return Err(invalid("diagram loop"));
    }
    for &(i, j, label) in down_edges {
        if !support.contains(&i) {
            continue;
        }
        if label == 3 {
            return Err(invalid("oversized type G diagram"));
        }
        if lower.is_none() {
            upper = Some(i);
            lower = Some(j);
        } else {
            return Err(invalid("multiple labelled edges"));
        }
    }

    let letter = if let Some(upper_node) = upper {
        if fork.is_some() {
            return Err(invalid("fork and labelled edge"));
        }
        let lower_node = lower.expect("labelled edge");
        if extremities.contains(&lower_node) {
            'B'
        } else if extremities.contains(&upper_node) {
            'C'
        } else {
            'F'
        }
    } else {
        match fork {
            None => 'A',
            Some(fork_node) => {
                if star[fork_node].intersection(&extremities).count() == 1 {
                    'E'
                } else {
                    'D'
                }
            }
        }
    };

    let mut remain = support.clone();
    let mut position = Vec::with_capacity(comp_rank);
    let mut start = match letter {
        'A' => first(&extremities),
        'B' => {
            extremities.remove(&lower.expect("type B edge"));
            first(&extremities)
        }
        'C' => {
            extremities.remove(&upper.expect("type C edge"));
            first(&extremities)
        }
        'D' => {
            if comp_rank == 4 {
                first(&extremities)
            } else {
                for node in star[fork.expect("type D fork")].clone() {
                    extremities.remove(&node);
                }
                if extremities.len() != 1 {
                    return Err(invalid("fork node without adjacent extremities"));
                }
                first(&extremities)
            }
        }
        'E' => {
            if comp_rank > 8 {
                return Err(invalid("oversized type E diagram"));
            }
            let fork_node = fork.expect("type E fork");
            let short_arm: BTreeSet<usize> = extremities
                .intersection(&star[fork_node])
                .copied()
                .collect();
            if short_arm.len() != 1 {
                return Err(invalid("type E short arm"));
            }
            let mut arm = *extremities
                .difference(&short_arm)
                .next()
                .expect("a longer arm exists");
            if star[arm].is_disjoint(&star[fork_node]) {
                // This is the longest arm; swap for the orthogonal long arm.
                extremities.remove(&arm);
                arm = first(&extremities);
            }
            if star[arm].is_disjoint(&star[fork_node]) {
                return Err(invalid("fork node with two too long arms"));
            }
            position.push(arm);
            remain.remove(&arm);
            position.push(first(&short_arm));
            for node in &short_arm {
                remain.remove(node);
            }
            *star[arm]
                .intersection(&star[fork_node])
                .next()
                .expect("type E common neighbour")
        }
        'F' => {
            if comp_rank > 4 {
                return Err(invalid("oversized type F diagram"));
            }
            for node in star[lower.expect("type F edge")].clone() {
                extremities.remove(&node);
            }
            first(&extremities)
        }
        _ => return Err(invalid("component letter")),
    };

    // Traverse the remainder of the diagram starting from |start|.
    loop {
        position.push(start);
        remain.remove(&start);
        if remain.is_empty() {
            break;
        }
        let candidate: BTreeSet<usize> = star[start].intersection(&remain).copied().collect();
        if !candidate.is_empty() {
            start = first(&candidate);
        } else if letter == 'D' {
            // Only a short arm of the fork can remain.
            if !remain.is_subset(&star[fork.expect("type D fork")]) {
                return Err(invalid("type D traversal"));
            }
            start = first(&remain);
        } else {
            return Err(invalid("component traversal"));
        }
    }
    if position.len() != comp_rank {
        return Err(invalid("component position count"));
    }

    Ok(DynkinComponent {
        letter,
        support: support.clone(),
        position,
    })
}

/// The Cartan matrix of the diagram folded by a `delta`-orbit list of
/// simple generators (upstream `DynkinDiagram::folded`,
/// structure/dynkin.cpp:222-261, computed here via the `cofold` Cartan
/// formulas of structure/rootdata.cpp:1578-1604, which determine the same
/// edge multiplicities). The folded simple root of an orbit is the sum of
/// its members; the folded simple coroot is the first member's coroot for
/// orbits of commuting members (length 2), the sum of both coroots for
/// non-commuting ones (length 3). `cartan[i][j]` is `<alpha_i, alpha_j^v>`
/// (the crate convention), so the folded entry `C(i,j)` sums
/// `cartan[a][b]` over folded roots `a` of orbit `j` and folded coroots
/// `b` of orbit `i`.
pub(crate) fn folded_cartan(
    cartan: &[Vec<i32>],
    orbits: &[crate::ext_block::ExtGen],
) -> Result<Vec<Vec<i32>>, StructureError> {
    let rank = cartan.len();
    let mut folded = vec![vec![0i32; orbits.len()]; orbits.len()];
    for (i, orbit_i) in orbits.iter().enumerate() {
        // Folded coroot support of orbit i.
        let mut coroots = vec![orbit_i.s0];
        if orbit_i.kind == crate::ext_block::ExtGenKind::Three {
            coroots.push(orbit_i.s1);
        }
        // Folded root support of each orbit j.
        for (j, orbit_j) in orbits.iter().enumerate() {
            let mut roots = vec![orbit_j.s0];
            if orbit_j.kind != crate::ext_block::ExtGenKind::One {
                roots.push(orbit_j.s1);
            }
            let mut entry = 0i32;
            for &a in &roots {
                for &b in &coroots {
                    if a >= rank || b >= rank {
                        return Err(StructureError::IndexOutOfRange {
                            index: a.max(b),
                            upper_bound: rank,
                        });
                    }
                    entry += cartan[a][b];
                }
            }
            folded[i][j] = entry;
        }
    }
    Ok(folded)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn letters(comps: &[DynkinComponent]) -> String {
        comps.iter().map(|comp| comp.letter).collect()
    }

    fn positions(comps: &[DynkinComponent]) -> Vec<usize> {
        comps
            .iter()
            .flat_map(|comp| comp.position.iter().copied())
            .collect()
    }

    #[test]
    fn classifies_single_factors_in_bourbaki_order() {
        let a1 = classify(&[vec![2]]).unwrap();
        assert_eq!(letters(&a1), "A");
        assert_eq!(positions(&a1), vec![0]);

        let a2 = classify(&[vec![2, -1], vec![-1, 2]]).unwrap();
        assert_eq!(letters(&a2), "A");
        assert_eq!(positions(&a2), vec![0, 1]);

        // B2: Cartan(i,j) == -2 with i < j stays type B in the given order.
        let b2 = classify(&[vec![2, -2], vec![-1, 2]]).unwrap();
        assert_eq!(letters(&b2), "B");
        assert_eq!(positions(&b2), vec![0, 1]);

        let c2 = classify(&[vec![2, -1], vec![-2, 2]]).unwrap();
        assert_eq!(letters(&c2), "C");
        assert_eq!(positions(&c2), vec![0, 1]);

        // G2 swaps to put the short root first.
        let g2 = classify(&[vec![2, -3], vec![-1, 2]]).unwrap();
        assert_eq!(letters(&g2), "G");
        assert_eq!(positions(&g2), vec![1, 0]);

        let g2_ordered = classify(&[vec![2, -1], vec![-3, 2]]).unwrap();
        assert_eq!(letters(&g2_ordered), "G");
        assert_eq!(positions(&g2_ordered), vec![0, 1]);
    }

    #[test]
    fn classifies_forked_and_exceptional_diagrams() {
        let d4 = classify(&[
            vec![2, -1, 0, 0],
            vec![-1, 2, -1, -1],
            vec![0, -1, 2, 0],
            vec![0, -1, 0, 2],
        ])
        .unwrap();
        assert_eq!(letters(&d4), "D");
        assert_eq!(positions(&d4), vec![0, 1, 2, 3]);

        // E6 in Bourbaki order: 1-3-4-5-6 chain, 2 attached to 4.
        let mut e6 = vec![vec![0; 6]; 6];
        for (index, row) in e6.iter_mut().enumerate() {
            row[index] = 2;
        }
        for (i, j) in [(0, 2), (2, 3), (3, 4), (4, 5), (1, 3)] {
            e6[i][j] = -1;
            e6[j][i] = -1;
        }
        let comps = classify(&e6).unwrap();
        assert_eq!(letters(&comps), "E");
        assert_eq!(positions(&comps), vec![0, 1, 2, 3, 4, 5]);

        // F4 in Bourbaki order: double edge between vertices 1 and 2.
        let f4 = classify(&[
            vec![2, -1, 0, 0],
            vec![-1, 2, -2, 0],
            vec![0, -1, 2, -1],
            vec![0, 0, -1, 2],
        ])
        .unwrap();
        assert_eq!(letters(&f4), "F");
        assert_eq!(positions(&f4), vec![0, 1, 2, 3]);
    }

    #[test]
    fn straightens_permuted_components() {
        // B3 relabeled by the permutation [2, 0, 1] of the canonical matrix.
        let canonical = [[2, -1, 0], [-1, 2, -2], [0, -1, 2]];
        let permutation = [2, 0, 1];
        let relabeled: Vec<Vec<i32>> = permutation
            .iter()
            .map(|&row| {
                permutation
                    .iter()
                    .map(|&column| canonical[row][column])
                    .collect()
            })
            .collect();
        let comps = classify(&relabeled).unwrap();
        assert_eq!(letters(&comps), "B");
        // The Bourbaki permutation reconstructs the canonical matrix.
        let pi = positions(&comps);
        for i in 0..3 {
            for j in 0..3 {
                assert_eq!(relabeled[pi[i]][pi[j]], canonical[i][j]);
            }
        }
    }

    #[test]
    fn splits_disconnected_components_in_first_vertex_order() {
        let cartan = vec![vec![2, 0, 0], vec![0, 2, -1], vec![0, -1, 2]];
        let comps = classify(&cartan).unwrap();
        assert_eq!(letters(&comps), "AA");
        assert_eq!(positions(&comps), vec![0, 1, 2]);
    }
}
