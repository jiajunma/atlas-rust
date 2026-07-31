//! The `lietype::involution` lookup behind the primitive
//! `involution(LieType,[int],string)` and `involution(LieType,mat,string)`
//! wrappers: `checked_inner_class_type` with its letter-collapse rules
//! (interpreter/atlas-types.w:742-820), the per-letter involution table of
//! `lietype::involution(const Layout&)` on the simply-connected fundamental
//! weight basis (structure/lietype.cpp:507-605), and the exact-division
//! change of basis `PID_Matrix::on_basis` (utilities/matrix.cpp:289-295).
//!
//! These are pure tables: no root datum, Weyl group, or inner-class pipeline
//! is involved. The `perm` argument is the user-supplied Bourbaki
//! renumbering of the flattened simple factors, validated wrapper-side by
//! `checked_permutation` (atlas-types.w:829-846).

use std::fmt;

use malachite::base::num::arithmetic::traits::Floor;
use malachite::base::num::basic::traits::Zero;
use malachite::Rational;

use crate::real_form_seed::invert_rational;

/// The failures of upstream `checked_inner_class_type`; `Display` is the
/// upstream `runtime_error` wording byte for byte.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InnerClassLetterError {
    /// More symbols than simple factors (atlas-types.w:764).
    TooManySymbols,
    /// Fewer symbols than simple factors (atlas-types.w:752).
    TooFewSymbols,
    /// A symbol outside "Ccesu" (atlas-types.w:762).
    UnknownSymbol(char),
    /// `'C'` without two identical consecutive factors (atlas-types.w:770).
    ComplexPair,
    /// `'u'` where no unequal-rank class exists (atlas-types.w:805).
    MeaninglessUnequalRank { letter: char, rank: usize },
}

impl fmt::Display for InnerClassLetterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManySymbols => write!(formatter, "Too many inner class symbols"),
            Self::TooFewSymbols => write!(formatter, "Too few inner class symbols"),
            Self::UnknownSymbol(symbol) => {
                write!(formatter, "Unknown inner class symbol `{symbol}'")
            }
            Self::ComplexPair => write!(
                formatter,
                "Complex inner class needs two identical consecutive types"
            ),
            Self::MeaninglessUnequalRank { letter, rank } => write!(
                formatter,
                "Unequal rank class is meaningless for type {letter}{rank}"
            ),
        }
    }
}

impl std::error::Error for InnerClassLetterError {}

/// `checked_inner_class_type` (atlas-types.w:742-820): parse the inner-class
/// string against the simple factors, one letter per factor with `'C'`
/// consuming two identical consecutive factors. Punctuation and whitespace
/// are skipped (`skip_punctuation`, atlas-types.w:224-227). `'e'` is a
/// synonym of `'c'`; `'s'` collapses to `'c'` exactly where `-1` lies in the
/// Weyl group (A1, B, C, even-rank D, E7, E8, F, G — atlas-types.w:782-790);
/// `'u'` survives only for even-rank D, collapses to `'s'` for A(n>=2),
/// odd-rank D, E6, and T, and is meaningless elsewhere (atlas-types.w:793-820).
pub fn checked_inner_class_letters(
    symbols: &str,
    factors: &[(char, usize)],
) -> Result<Vec<char>, InnerClassLetterError> {
    let mut result = Vec::new();
    let mut characters = symbols.chars().peekable();
    let mut index = 0_usize; // position in the simple factors of the Lie type
    loop {
        while matches!(characters.peek(), Some(c) if c.is_ascii_punctuation() || c.is_whitespace())
        {
            characters.next();
        }
        let Some(symbol) = characters.next() else {
            break;
        };
        if !"Ccesu".contains(symbol) {
            return Err(InnerClassLetterError::UnknownSymbol(symbol));
        }
        let Some(&(letter, rank)) = factors.get(index) else {
            return Err(InnerClassLetterError::TooManySymbols);
        };
        match symbol {
            'C' => {
                // Complex: two identical consecutive simple factors.
                if factors.get(index + 1) != Some(&(letter, rank)) {
                    return Err(InnerClassLetterError::ComplexPair);
                }
                result.push('C');
                index += 2;
            }
            'c' | 'e' => {
                result.push('c');
                index += 1;
            }
            's' => {
                let survives = (letter == 'A' && rank >= 2)
                    || (letter == 'D' && rank % 2 != 0)
                    || (letter == 'E' && rank == 6)
                    || letter == 'T';
                result.push(if survives { 's' } else { 'c' });
                index += 1;
            }
            'u' => {
                if letter == 'D' {
                    result.push(if rank % 2 == 0 { 'u' } else { 's' });
                } else if (letter == 'A' && rank >= 2)
                    || (letter == 'E' && rank == 6)
                    || letter == 'T'
                {
                    result.push('s');
                } else {
                    return Err(InnerClassLetterError::MeaninglessUnequalRank { letter, rank });
                }
                index += 1;
            }
            _ => unreachable!("membership in \"Ccesu\" was checked above"),
        }
    }
    if index < factors.len() {
        return Err(InnerClassLetterError::TooFewSymbols);
    }
    Ok(result)
}

/// `lietype::involution(const Layout&)` (structure/lietype.cpp:507-605): the
/// involution designated by one letter per inner-class entry, as a row-major
/// matrix on the fundamental-weight basis of the simply connected group.
/// `perm[k]` is the matrix index of the k-th flattened simple root (the
/// upstream `Layout::d_perm`); `'C'` covers the next two factors, which the
/// checked letters guarantee identical.
///
/// Precondition (upstream's, checked wrapper-side): `letters` comes from
/// [`checked_inner_class_letters`] and `perm` is a permutation of
/// `0..rank`.
pub fn layout_involution(
    factors: &[(char, usize)],
    letters: &[char],
    perm: &[usize],
) -> Vec<Vec<i32>> {
    let rank: usize = factors.iter().map(|&(_, factor_rank)| factor_rank).sum();
    debug_assert_eq!(perm.len(), rank);
    let mut result = vec![vec![0_i32; rank]; rank];
    // The block helpers of lietype.cpp:622-658, in matrix-index form.
    let compact = |result: &mut Vec<Vec<i32>>, r: usize, rs: usize| {
        for i in 0..rs {
            result[perm[r + i]][perm[r + i]] = 1;
        }
    };
    let flip_last_two = |result: &mut Vec<Vec<i32>>, r: usize, rs: usize| {
        for i in 0..rs - 2 {
            result[perm[r + i]][perm[r + i]] = 1;
        }
        result[perm[r + rs - 2]][perm[r + rs - 1]] = 1;
        result[perm[r + rs - 1]][perm[r + rs - 2]] = 1;
    };
    let mut r = 0_usize; // position in the flattened diagram; indexes `perm`
    let mut pos = 0_usize; // position in `factors`
    for &letter in letters {
        let &(factor_letter, rs) = &factors[pos];
        match letter {
            'c' => compact(&mut result, r, rs),
            's' => match factor_letter {
                'A' => {
                    // The antidiagonal matrix.
                    for i in 0..rs {
                        result[perm[r + i]][perm[r + rs - 1 - i]] = 1;
                    }
                }
                'D' => {
                    if rs % 2 != 0 {
                        flip_last_two(&mut result, r, rs);
                    } else {
                        compact(&mut result, r, rs);
                    }
                }
                'E' => {
                    if rs == 6 {
                        result[perm[r + 1]][perm[r + 1]] = 1;
                        result[perm[r + 3]][perm[r + 3]] = 1;
                        result[perm[r]][perm[r + 5]] = 1;
                        result[perm[r + 5]][perm[r]] = 1;
                        result[perm[r + 2]][perm[r + 4]] = 1;
                        result[perm[r + 4]][perm[r + 2]] = 1;
                    } else {
                        compact(&mut result, r, rs);
                    }
                }
                'T' => {
                    for i in 0..rs {
                        result[perm[r + i]][perm[r + i]] = -1;
                    }
                }
                // Identity involution for types B, C, E7, E8, F, G.
                _ => compact(&mut result, r, rs),
            },
            'C' => {
                // Parallel interchange of `rs` vertices with the next `rs`.
                for i in 0..rs {
                    result[perm[r + i]][perm[r + rs + i]] = 1;
                    result[perm[r + rs + i]][perm[r + i]] = 1;
                }
                pos += 1;
                r += rs;
            }
            'u' => flip_last_two(&mut result, r, rs),
            _ => unreachable!("checked letters are one of c/s/C/u"),
        }
        pos += 1;
        r += rs;
    }
    debug_assert_eq!(r, rank);
    result
}

/// `PID_Matrix::on_basis` (utilities/matrix.cpp:289-295): the change of
/// basis `basis^-1 * matrix * basis`, where the upstream
/// adjugate-times-exact-division computation throws "Inexact integer
/// division" on any non-integral entry. Returns `None` for a singular
/// basis, a non-integral result, or non-square input — the wrapper relabels
/// every one of these as an incompatible lattice.
pub fn on_basis(matrix: &[Vec<i32>], basis: &[Vec<i32>]) -> Option<Vec<Vec<i32>>> {
    let rank = basis.len();
    if matrix.len() != rank
        || basis.iter().any(|row| row.len() != rank)
        || matrix.iter().any(|row| row.len() != rank)
    {
        return None;
    }
    let rational = |rows: &[Vec<i32>]| {
        rows.iter()
            .map(|row| {
                row.iter()
                    .map(|&value| Rational::from(value))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    let inverse_columns = invert_rational(&rational(basis)).ok()?;
    let inverse: Vec<Vec<Rational>> = (0..rank)
        .map(|row| {
            (0..rank)
                .map(|column| inverse_columns[column][row].clone())
                .collect()
        })
        .collect();
    let product = |left: &[Vec<Rational>], right: &[Vec<Rational>]| {
        let mut result = vec![vec![Rational::ZERO; rank]; rank];
        for (i, result_row) in result.iter_mut().enumerate() {
            for (j, entry) in result_row.iter_mut().enumerate() {
                let mut sum = Rational::ZERO;
                for k in 0..rank {
                    sum += left[i][k].clone() * right[k][j].clone();
                }
                *entry = sum;
            }
        }
        result
    };
    let transported = product(&product(&inverse, &rational(matrix)), &rational(basis));
    transported
        .iter()
        .map(|row| {
            row.iter()
                .map(|value| {
                    let floored = value.clone().floor();
                    if *value != floored {
                        return None;
                    }
                    i32::try_from(&floored).ok()
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn letters(symbols: &str, factors: &[(char, usize)]) -> Vec<char> {
        checked_inner_class_letters(symbols, factors).expect("valid inner class string")
    }

    #[test]
    fn split_collapses_to_compact_exactly_where_minus_one_is_weyl() {
        // -1 in W: A1, B, C, even D, E7, E8, F, G collapse 's' to 'c'.
        for factors in [
            &[('A', 1)][..],
            &[('B', 2)][..],
            &[('C', 3)][..],
            &[('D', 4)][..],
            &[('D', 6)][..],
            &[('E', 7)][..],
            &[('E', 8)][..],
            &[('F', 4)][..],
            &[('G', 2)][..],
        ] {
            assert_eq!(letters("s", factors), vec!['c'], "factors: {factors:?}");
        }
        // A(n>=2), odd D, E6, T keep 's'.
        for factors in [
            &[('A', 2)][..],
            &[('A', 5)][..],
            &[('D', 5)][..],
            &[('E', 6)][..],
            &[('T', 1)][..],
        ] {
            assert_eq!(letters("s", factors), vec!['s'], "factors: {factors:?}");
        }
    }

    #[test]
    fn unequal_rank_survives_only_for_even_rank_d() {
        assert_eq!(letters("u", &[('D', 4)]), vec!['u']);
        assert_eq!(letters("u", &[('D', 6)]), vec!['u']);
        // Everywhere else 'u' means 's'...
        for factors in [
            &[('A', 2)][..],
            &[('D', 5)][..],
            &[('E', 6)][..],
            &[('T', 1)][..],
        ] {
            assert_eq!(letters("u", factors), vec!['s'], "factors: {factors:?}");
        }
        // ...or is meaningless, with the upstream wording.
        for (factors, letter, rank) in [
            (&[('A', 1)][..], 'A', 1_usize),
            (&[('B', 2)][..], 'B', 2),
            (&[('C', 2)][..], 'C', 2),
            (&[('E', 7)][..], 'E', 7),
            (&[('F', 4)][..], 'F', 4),
            (&[('G', 2)][..], 'G', 2),
        ] {
            assert_eq!(
                checked_inner_class_letters("u", factors),
                Err(InnerClassLetterError::MeaninglessUnequalRank { letter, rank }),
                "factors: {factors:?}"
            );
        }
        assert_eq!(
            InnerClassLetterError::MeaninglessUnequalRank {
                letter: 'A',
                rank: 1
            }
            .to_string(),
            "Unequal rank class is meaningless for type A1"
        );
    }

    #[test]
    fn letter_string_diagnostics_have_the_upstream_wording_and_order() {
        // 'e' is a synonym of 'c'; punctuation and whitespace are skipped.
        assert_eq!(letters("e", &[('A', 2)]), vec!['c']);
        assert_eq!(letters(". c ", &[('A', 1)]), vec!['c']);
        // Unknown symbols report before the factor count.
        assert_eq!(
            checked_inner_class_letters("x", &[('A', 1)]),
            Err(InnerClassLetterError::UnknownSymbol('x'))
        );
        assert_eq!(
            InnerClassLetterError::UnknownSymbol('x').to_string(),
            "Unknown inner class symbol `x'"
        );
        // One symbol per factor; 'C' consumes two identical factors.
        assert_eq!(
            checked_inner_class_letters("ec", &[('A', 2)]),
            Err(InnerClassLetterError::TooManySymbols)
        );
        assert_eq!(
            checked_inner_class_letters("c", &[('A', 1), ('A', 1)]),
            Err(InnerClassLetterError::TooFewSymbols)
        );
        assert_eq!(letters("C", &[('A', 1), ('A', 1)]), vec!['C']);
        assert_eq!(
            checked_inner_class_letters("Cs", &[('A', 1), ('A', 2)]),
            Err(InnerClassLetterError::ComplexPair)
        );
        assert_eq!(
            checked_inner_class_letters("C", &[('A', 1)]),
            Err(InnerClassLetterError::ComplexPair)
        );
        assert_eq!(
            InnerClassLetterError::ComplexPair.to_string(),
            "Complex inner class needs two identical consecutive types"
        );
    }

    #[test]
    fn involution_table_pins_the_frozen_fixture_anchors() {
        // A1: both 'c' and the collapsed 's' give the identity.
        assert_eq!(
            layout_involution(&[('A', 1)], &letters("c", &[('A', 1)]), &[0]),
            vec![vec![1]]
        );
        assert_eq!(
            layout_involution(&[('A', 1)], &letters("s", &[('A', 1)]), &[0]),
            vec![vec![1]]
        );
        // A2: 'c' is the identity; 's' and 'u' are the diagram flip.
        assert_eq!(
            layout_involution(&[('A', 2)], &letters("c", &[('A', 2)]), &[0, 1]),
            vec![vec![1, 0], vec![0, 1]]
        );
        assert_eq!(
            layout_involution(&[('A', 2)], &letters("s", &[('A', 2)]), &[0, 1]),
            vec![vec![0, 1], vec![1, 0]]
        );
        assert_eq!(
            layout_involution(&[('A', 2)], &letters("u", &[('A', 2)]), &[0, 1]),
            vec![vec![0, 1], vec![1, 0]]
        );
        // A2 with the transposed Bourbaki numbering: the flip is symmetric.
        assert_eq!(
            layout_involution(&[('A', 2)], &letters("s", &[('A', 2)]), &[1, 0]),
            vec![vec![0, 1], vec![1, 0]]
        );
        // B2 's' collapses to the identity; A1.A1 'C' swaps the factors.
        assert_eq!(
            layout_involution(&[('B', 2)], &letters("s", &[('B', 2)]), &[0, 1]),
            vec![vec![1, 0], vec![0, 1]]
        );
        assert_eq!(
            layout_involution(
                &[('A', 1), ('A', 1)],
                &letters("C", &[('A', 1), ('A', 1)]),
                &[0, 1]
            ),
            vec![vec![0, 1], vec![1, 0]]
        );
    }

    #[test]
    fn involution_table_covers_the_remaining_letters() {
        // D4 'u' and D5 's' flip the fork tips; D4 's' collapses to identity.
        let identity4 = vec![
            vec![1, 0, 0, 0],
            vec![0, 1, 0, 0],
            vec![0, 0, 1, 0],
            vec![0, 0, 0, 1],
        ];
        let flip4 = vec![
            vec![1, 0, 0, 0],
            vec![0, 1, 0, 0],
            vec![0, 0, 0, 1],
            vec![0, 0, 1, 0],
        ];
        let perm4 = [0, 1, 2, 3];
        assert_eq!(
            layout_involution(&[('D', 4)], &letters("u", &[('D', 4)]), &perm4),
            flip4
        );
        assert_eq!(
            layout_involution(&[('D', 4)], &letters("s", &[('D', 4)]), &perm4),
            identity4
        );
        let perm5 = [0, 1, 2, 3, 4];
        let flip5 = vec![
            vec![1, 0, 0, 0, 0],
            vec![0, 1, 0, 0, 0],
            vec![0, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 1],
            vec![0, 0, 0, 1, 0],
        ];
        assert_eq!(
            layout_involution(&[('D', 5)], &letters("s", &[('D', 5)]), &perm5),
            flip5
        );
        // E6 's': fix 1 and 3, swap 0<->5 and 2<->4.
        let perm6 = [0, 1, 2, 3, 4, 5];
        let e6 = vec![
            vec![0, 0, 0, 0, 0, 1],
            vec![0, 1, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 1, 0],
            vec![0, 0, 0, 1, 0, 0],
            vec![0, 0, 1, 0, 0, 0],
            vec![1, 0, 0, 0, 0, 0],
        ];
        assert_eq!(
            layout_involution(&[('E', 6)], &letters("s", &[('E', 6)]), &perm6),
            e6
        );
        // T1 's' is minus the identity.
        assert_eq!(
            layout_involution(&[('T', 1)], &letters("s", &[('T', 1)]), &[0]),
            vec![vec![-1]]
        );
        // The Bourbaki permutation applies to the table output: A1's root
        // sits at index 2 (compact), A2's roots at indices 0 and 1 (flipped).
        assert_eq!(
            layout_involution(
                &[('A', 1), ('A', 2)],
                &letters("cs", &[('A', 1), ('A', 2)]),
                &[2, 0, 1]
            ),
            vec![vec![0, 1, 0], vec![1, 0, 0], vec![0, 0, 1]]
        );
    }

    #[test]
    fn on_basis_transports_with_the_exact_division_check() {
        // The frozen A2 anchor: the diagram flip on the lattice whose
        // columns are (1,1) and (0,1) (Atlas literal [[1,1],[0,1]]).
        let flip = vec![vec![0, 1], vec![1, 0]];
        let basis = vec![vec![1, 0], vec![1, 1]];
        assert_eq!(on_basis(&flip, &basis), Some(vec![vec![1, 1], vec![0, -1]]));
        // A2 's' on [[1,0],[0,2]]: inexact division, hence incompatible.
        let incompatible = vec![vec![1, 0], vec![0, 2]];
        assert_eq!(on_basis(&flip, &incompatible), None);
        // A1 on the even sublattice keeps the identity.
        assert_eq!(on_basis(&[vec![1]], &[vec![2]]), Some(vec![vec![1]]));
        // A singular basis is incompatible as well.
        assert_eq!(on_basis(&flip, &[vec![1, 1], vec![1, 1]]), None);
        // The identity transports on any invertible basis.
        assert_eq!(
            on_basis(&[vec![1, 0], vec![0, 1]], &basis),
            Some(vec![vec![1, 0], vec![0, 1]])
        );
    }
}
