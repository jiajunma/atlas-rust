//! Weyl-group orders from Cartan-matrix type recognition.
//!
//! The on-demand twisted-conjugacy partition (task #9) checks each orbit
//! size against the upstream order-quotient formula
//! `|W| / (|W_im| x |W_re| x |W_cx|)` (cartanclass.cpp:1046-1064). That
//! check needs only the ORDERS of the Weyl groups of recognized Cartan
//! matrices — so B/C orientation is irrelevant (both give `2^n n!`), and
//! recognition reduces to component splitting plus branch-shape analysis.
//! Orders use exact `Integer` arithmetic: component products overflow
//! `u128` well inside the crate's dynamic rank range.

use malachite::base::num::basic::traits::One;
use malachite::Integer;

use crate::grading::try_capacity;
use crate::StructureError;

/// The Weyl-group order of the root system presented by a Cartan matrix
/// (entries `<alpha_i, alpha_j_vee>`; zero rows/columns are torus factors
/// contributing order one).
pub(crate) fn weyl_order_of_cartan(matrix: &[Vec<i32>]) -> Result<Integer, StructureError> {
    let rank = matrix.len();
    for row in matrix {
        if row.len() != rank {
            return Err(StructureError::NonSquareCartan);
        }
    }
    let mut seen = vec![false; rank];
    let mut order = Integer::ONE;
    for start in 0..rank {
        if seen[start] || matrix[start][start] == 0 {
            continue;
        }
        // Connected component by nonzero off-diagonal links.
        let mut component = try_capacity(rank)?;
        component.push(start);
        seen[start] = true;
        let mut cursor = 0;
        while cursor < component.len() {
            let node = component[cursor];
            for other in 0..rank {
                if !seen[other]
                    && matrix[other][other] != 0
                    && matrix[node][other] != 0
                    && node != other
                {
                    seen[other] = true;
                    component.push(other);
                }
            }
            cursor += 1;
        }
        order *= component_order(matrix, &component)?;
    }
    Ok(order)
}

/// Order of one connected component, by branch-shape recognition.
fn component_order(matrix: &[Vec<i32>], nodes: &[usize]) -> Result<Integer, StructureError> {
    let rank = nodes.len();
    if rank == 1 {
        return Ok(Integer::from(2));
    }
    // Edge multiplicities m = C_ij * C_ji within the component.
    let mut max_multiplicity = 1;
    let mut degrees = vec![0_usize; rank];
    for (a, &node_a) in nodes.iter().enumerate() {
        for (b, &node_b) in nodes.iter().enumerate().skip(a + 1) {
            let product = matrix[node_a][node_b]
                .checked_mul(matrix[node_b][node_a])
                .ok_or(StructureError::ArithmeticOverflow)?;
            if product != 0 {
                degrees[a] += 1;
                degrees[b] += 1;
                max_multiplicity = max_multiplicity.max(product);
            }
        }
    }
    let invalid = StructureError::InvalidCartanMatrix;
    match max_multiplicity {
        3 => {
            if rank == 2 {
                Ok(Integer::from(12)) // G2
            } else {
                Err(invalid)
            }
        }
        2 => {
            // B/C chains share 2^n n!; F4 is the rank-4 double-middle chain.
            if degrees.iter().any(|&d| d > 2) {
                return Err(invalid);
            }
            if rank == 4 {
                // Distinguish F4 (double edge between the two INNER nodes)
                // from B4/C4 (double edge at a chain end).
                let inner_double = nodes.iter().enumerate().any(|(a, &node_a)| {
                    nodes.iter().enumerate().any(|(b, &node_b)| {
                        a < b
                            && degrees[a] == 2
                            && degrees[b] == 2
                            && matrix[node_a][node_b] * matrix[node_b][node_a] == 2
                    })
                });
                if inner_double {
                    return Ok(Integer::from(1152)); // F4
                }
            }
            Ok(two_power_factorial(rank))
        }
        1 => {
            let forks = degrees.iter().filter(|&&d| d >= 3).count();
            if degrees.iter().any(|&d| d > 3) || forks > 1 {
                return Err(invalid);
            }
            if forks == 0 {
                return factorial(rank + 1); // A_n
            }
            // One degree-3 node: D or E by sorted branch lengths.
            let mut lengths = branch_lengths(matrix, nodes, &degrees)?;
            lengths.sort_unstable();
            match lengths.as_slice() {
                [1, 1, _] => Ok(two_power_factorial(rank) / Integer::from(2)), // D_n
                [1, 2, 2] => Ok(Integer::from(51_840_i64)),                    // E6
                [1, 2, 3] => Ok(Integer::from(2_903_040_i64)),                 // E7
                [1, 2, 4] => Ok(Integer::from(696_729_600_i64)),               // E8
                _ => Err(invalid),
            }
        }
        _ => Err(invalid),
    }
}

/// Branch lengths from the unique degree-3 node of a simply laced tree.
fn branch_lengths(
    matrix: &[Vec<i32>],
    nodes: &[usize],
    degrees: &[usize],
) -> Result<Vec<usize>, StructureError> {
    let fork = degrees
        .iter()
        .position(|&d| d == 3)
        .ok_or(StructureError::InvalidCartanMatrix)?;
    let mut lengths = try_capacity(3)?;
    for (neighbor, &node) in nodes.iter().enumerate() {
        if neighbor == fork || matrix[nodes[fork]][node] == 0 {
            continue;
        }
        // Walk away from the fork until a leaf.
        let mut length = 1;
        let mut previous = fork;
        let mut current = neighbor;
        loop {
            let next = nodes.iter().enumerate().find(|&(candidate, &c_node)| {
                candidate != previous
                    && candidate != current
                    && matrix[nodes[current]][c_node] != 0
            });
            match next {
                Some((candidate, _)) => {
                    previous = current;
                    current = candidate;
                    length += 1;
                }
                None => break,
            }
        }
        lengths.push(length);
    }
    if lengths.len() != 3 {
        return Err(StructureError::InvalidCartanMatrix);
    }
    Ok(lengths)
}

fn factorial(n: usize) -> Result<Integer, StructureError> {
    let mut value = Integer::ONE;
    for factor in 2..=n {
        value *= Integer::from(factor);
    }
    Ok(value)
}

fn two_power_factorial(rank: usize) -> Integer {
    let mut value = Integer::ONE;
    for factor in 2..=rank {
        value *= Integer::from(factor);
    }
    value << rank
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chain(rank: usize, tail: (i32, i32)) -> Vec<Vec<i32>> {
        let mut matrix = vec![vec![0; rank]; rank];
        for (index, row) in matrix.iter_mut().enumerate() {
            row[index] = 2;
        }
        for i in 0..rank - 1 {
            let (upper, lower) = if i == rank - 2 { tail } else { (-1, -1) };
            matrix[i][i + 1] = upper;
            matrix[i + 1][i] = lower;
        }
        matrix
    }

    #[test]
    fn recognizes_the_classical_series_orders() {
        assert_eq!(
            weyl_order_of_cartan(&chain(4, (-1, -1))).unwrap(),
            Integer::from(120) // A4
        );
        assert_eq!(
            weyl_order_of_cartan(&chain(5, (-2, -1))).unwrap(),
            Integer::from(3_840) // B5
        );
        assert_eq!(
            weyl_order_of_cartan(&chain(5, (-1, -2))).unwrap(),
            Integer::from(3_840) // C5
        );
        // D5: fork at node 2 toward nodes 3 and 4.
        let mut d5 = vec![vec![0; 5]; 5];
        for (i, row) in d5.iter_mut().enumerate() {
            row[i] = 2;
        }
        for (a, b) in [(0, 1), (1, 2), (2, 3), (2, 4)] {
            d5[a][b] = -1;
            d5[b][a] = -1;
        }
        assert_eq!(weyl_order_of_cartan(&d5).unwrap(), Integer::from(1_920));
    }

    #[test]
    fn recognizes_the_exceptional_orders_and_torus_factors() {
        let g2 = vec![vec![2, -1], vec![-3, 2]];
        assert_eq!(weyl_order_of_cartan(&g2).unwrap(), Integer::from(12));
        let f4 = vec![
            vec![2, -1, 0, 0],
            vec![-1, 2, -2, 0],
            vec![0, -1, 2, -1],
            vec![0, 0, -1, 2],
        ];
        assert_eq!(weyl_order_of_cartan(&f4).unwrap(), Integer::from(1_152));
        // E6 in the upstream numbering (0-2 bridge, 1-3 bridge).
        let mut e6 = vec![vec![0; 6]; 6];
        for (i, row) in e6.iter_mut().enumerate() {
            row[i] = 2;
        }
        for (a, b) in [(0, 2), (1, 3), (2, 3), (3, 4), (4, 5)] {
            e6[a][b] = -1;
            e6[b][a] = -1;
        }
        assert_eq!(weyl_order_of_cartan(&e6).unwrap(), Integer::from(51_840));
        // A1 x A1 with an interleaved torus row.
        let mixed = vec![vec![2, 0, 0], vec![0, 0, 0], vec![0, 0, 2]];
        assert_eq!(weyl_order_of_cartan(&mixed).unwrap(), Integer::from(4));
        // Empty system: order one.
        assert_eq!(
            weyl_order_of_cartan(&vec![vec![0; 2]; 2]).unwrap(),
            Integer::ONE
        );
    }
}
