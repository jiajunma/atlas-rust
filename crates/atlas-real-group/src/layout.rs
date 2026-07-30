//! The inner-class layout: Lie type, inner-class letters, and the Bourbaki
//! permutation of the simple roots, exactly as upstream `check_involution`
//! computes them into a `lietype::Layout` (interpreter/atlas-types.w:2829-3010).
//!
//! The twist permutation of the distinguished involution classifies each
//! simple factor as compact (`'c'`), split/unequal-rank (`'s'`, or `'u'` for
//! even-rank D), or Complex (`'C'`, absorbing a pair of isomorphic factors
//! that are reordered to become adjacent, with the permutation rewritten to
//! the twist images). A central torus contributes one `T1` type factor per
//! dimension and letters from `tori::classify` of the involution's action on
//! the quotient of the weight lattice by the rational root span
//! (atlas-types.w:2977-3010, structure/tori.cpp:189-197).

use malachite::Integer;

use crate::dynkin::DynkinComponent;
use crate::integer_lattice::adapted_basis;
use crate::involution_classification::classify_plus_identity;
use crate::{InnerClass, IntegerLatticeBudget, StructureError};

/// The `lietype::Layout` of one inner class: its Lie type (semisimple
/// factors in print order, Complex pairs adjacent, then one `T1` per central
/// torus dimension), one inner-class letter per entry (`'C'` covering two
/// type factors), and the Bourbaki permutation of the datum's simple roots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InnerClassLayout {
    /// `(letter, rank)` per type entry; torus entries are `('T', 1)`.
    factors: Vec<(char, usize)>,
    /// One letter per inner-class entry; `'C'` consumes two `factors`.
    letters: Vec<char>,
    /// Bourbaki permutation: `perm[k]` is the datum simple-root index of the
    /// k-th simple root in normalized order (upstream `Layout::d_perm`).
    perm: Vec<usize>,
}

impl InnerClassLayout {
    /// Compute the layout of a validated inner class.
    ///
    /// `budget` bounds the Smith-basis computation behind the central-torus
    /// letters; it is unused for semisimple data.
    pub fn build(
        inner_class: &InnerClass,
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, StructureError> {
        let datum = inner_class.datum();
        let semisimple_rank = datum.semisimple_rank();
        let lattice_rank = datum.lattice_rank();

        let twist = twist_permutation(inner_class)?;
        let components = crate::dynkin::classify(datum.cartan_matrix())?;

        let (mut factors, mut letters, perm) =
            inner_class_letters(&twist, &components, semisimple_rank)?;

        if lattice_rank > semisimple_rank {
            let (compact, complex, split) = torus_ranks(inner_class, budget)?;
            for _ in 0..lattice_rank - semisimple_rank {
                factors.push(('T', 1));
            }
            letters.extend(std::iter::repeat_n('c', compact));
            letters.extend(std::iter::repeat_n('C', complex));
            letters.extend(std::iter::repeat_n('s', split));
        }

        Ok(Self {
            factors,
            letters,
            perm,
        })
    }

    /// The type entries: `(letter, rank)` per factor, torus as `('T', 1)`.
    pub fn factors(&self) -> &[(char, usize)] {
        &self.factors
    }

    /// The inner-class letters, one per entry; `'C'` covers two factors.
    pub fn letters(&self) -> &[char] {
        &self.letters
    }

    /// The Bourbaki permutation of the datum simple roots.
    pub fn perm(&self) -> &[usize] {
        &self.perm
    }

    /// Upstream `LieType` printing: `letter+rank` joined by `.`
    /// (io/basic_io.cpp:83-86).
    pub fn lie_type_string(&self) -> String {
        self.factors
            .iter()
            .map(|(letter, rank)| format!("{letter}{rank}"))
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Upstream `InnerClassType` printing: the letters concatenated with no
    /// separator (io/basic_io.cpp:88-91).
    pub fn inner_class_string(&self) -> String {
        self.letters.iter().collect()
    }
}

/// The permutation of the datum's simple roots induced by the distinguished
/// involution (the `weyl::Twist` of upstream `check_involution`).
fn twist_permutation(inner_class: &InnerClass) -> Result<Vec<usize>, StructureError> {
    let datum = inner_class.datum();
    let root_system = inner_class.root_system();
    let delta = inner_class.distinguished_involution();
    let semisimple_rank = datum.semisimple_rank();

    let mut twist = Vec::with_capacity(semisimple_rank);
    let mut seen = vec![false; semisimple_rank];
    for simple in datum.simple_roots().iter() {
        let id = root_system
            .id_of(simple)
            .ok_or(StructureError::LayoutInvariantViolation {
                invariant: "simple-root membership",
            })?;
        let image = delta
            .image(id)
            .ok_or(StructureError::LayoutInvariantViolation {
                invariant: "simple-root image",
            })?;
        let image_weight =
            root_system
                .root(image)
                .ok_or(StructureError::LayoutInvariantViolation {
                    invariant: "simple-root image",
                })?;
        let image_generator = datum
            .simple_roots()
            .iter()
            .position(|candidate| candidate == image_weight)
            .ok_or(StructureError::LayoutInvariantViolation {
                invariant: "distinguished twist is not a simple permutation",
            })?;
        if seen[image_generator] {
            return Err(StructureError::LayoutInvariantViolation {
                invariant: "distinguished twist is not a permutation",
            });
        }
        seen[image_generator] = true;
        twist.push(image_generator);
    }
    Ok(twist)
}

/// The per-factor inner-class letters with Complex-factor reordering
/// (atlas-types.w:2868-2941): compact factors fix every root; unequal-rank
/// factors keep the twist inside the component (`'u'` for even-rank D, else
/// `'s'`); a twist leaving the component marks a Complex pair, which is
/// rotated adjacently while `type` and `pi` are shifted to match.
///
/// The upstream shift order is replicated EXACTLY, including that the shift
/// width `j` is summed over the already-shifted `type` slice — observable
/// only when a Complex pair spans more than one intervening factor of a
/// different rank, a case no current constructor fixture exercises.
/// The (factors, letters, permutation) triple produced by
/// [`inner_class_letters`].
type LetterLayout = (Vec<(char, usize)>, Vec<char>, Vec<usize>);

fn inner_class_letters(
    twist: &[usize],
    components: &[DynkinComponent],
    semisimple_rank: usize,
) -> Result<LetterLayout, StructureError> {
    let complex_pairing = || StructureError::LayoutInvariantViolation {
        invariant: "non-matching Complex factor",
    };

    let mut comps: Vec<DynkinComponent> = components.to_vec();
    let mut factors: Vec<(char, usize)> = components
        .iter()
        .map(|component| (component.letter, component.position.len()))
        .collect();
    let mut perm: Vec<usize> = components
        .iter()
        .flat_map(|component| component.position.iter().copied())
        .collect();
    let mut letters = Vec::with_capacity(components.len());

    let mut offset = 0_usize;
    let mut index = 0_usize;
    while index < comps.len() {
        let comp_rank = factors[index].1;
        let equal_rank = (offset..offset + comp_rank).all(|k| twist[perm[k]] == perm[k]);
        if equal_rank {
            letters.push('c');
        } else if comps[index].support.contains(&twist[perm[offset]]) {
            let (letter, _) = factors[index];
            letters.push(if letter == 'D' && comp_rank.is_multiple_of(2) {
                'u'
            } else {
                's'
            });
        } else {
            letters.push('C');
            let beta = twist[perm[offset]];
            let Some(pair) = ((index + 1)..comps.len())
                .find(|&candidate| comps[candidate].support.contains(&beta))
            else {
                return Err(complex_pairing());
            };
            if pair > index + 1 {
                let component = comps.remove(pair);
                comps.insert(index + 1, component);
                // Upstream shifts `type` one slot up FIRST (leaving the
                // stale entry at `index + 1`) and only then sums the shift
                // width over the shifted slice; replicate that order exactly.
                factors.copy_within(index + 1..pair, index + 2);
                let shift_end = offset
                    + comp_rank
                    + factors[index + 1..pair]
                        .iter()
                        .map(|&(_, rank)| rank)
                        .sum::<usize>();
                perm.copy_within(offset + comp_rank..shift_end, offset + 2 * comp_rank);
            }
            factors[index + 1] = factors[index];
            for k in offset..offset + comp_rank {
                perm[k + comp_rank] = twist[perm[k]];
            }
            offset += comp_rank;
            index += 1;
        }
        offset += comp_rank;
        index += 1;
    }
    debug_assert_eq!(offset, semisimple_rank);
    Ok((factors, letters, perm))
}

/// The `(compact, complex, split)` ranks of the involution's action on the
/// central-torus quotient lattice (`tori::classify`, tori.cpp:189-197): the
/// quotient involution is read off a Smith basis of the root lattice
/// (atlas-types.w:2993-3010), then its `+1`-eigenrank minus the mod-two
/// image rank gives the compact rank, the mod-two image rank is the complex
/// rank, and the remainder is split.
fn torus_ranks(
    inner_class: &InnerClass,
    budget: &IntegerLatticeBudget,
) -> Result<(usize, usize, usize), StructureError> {
    let datum = inner_class.datum();
    let semisimple_rank = datum.semisimple_rank();
    let lattice_rank = datum.lattice_rank();
    let torus_rank = lattice_rank - semisimple_rank;

    // The root lattice as an r-by-s matrix with simple roots as COLUMNS.
    let root_lattice: Vec<Vec<i32>> = (0..lattice_rank)
        .map(|row| {
            datum
                .simple_roots()
                .iter()
                .map(|root| root.as_slice()[row])
                .collect()
        })
        .collect();
    let adapted = adapted_basis(&root_lattice, budget)?;
    if adapted.diagonal.len() != semisimple_rank {
        return Err(StructureError::LayoutInvariantViolation {
            invariant: "root-lattice Smith rank",
        });
    }

    // inv = inverse.block(s,0,r,r) * delta * basis.block(0,s,r,r).
    let delta = inner_class
        .distinguished_involution()
        .involution()
        .weight_matrix();
    let mut involution = vec![vec![Integer::from(0); torus_rank]; torus_rank];
    for (a, inv_row) in involution.iter_mut().enumerate() {
        for (b, entry) in inv_row.iter_mut().enumerate() {
            let mut sum = Integer::from(0);
            for (i, delta_row) in delta.iter().enumerate() {
                let left = adapted.inverse.entry(semisimple_rank + a, i);
                if *left == 0 {
                    continue;
                }
                for (j, &coefficient) in delta_row.iter().enumerate() {
                    if coefficient == 0 {
                        continue;
                    }
                    let right = adapted.basis.entry(j, semisimple_rank + b);
                    if *right == 0 {
                        continue;
                    }
                    sum += left * Integer::from(coefficient) * right;
                }
            }
            *entry = sum;
        }
    }

    // tori::classify: tau1 = inv + 1.
    let mut tau1 = Vec::with_capacity(torus_rank);
    for (a, row) in involution.iter().enumerate() {
        let mut converted = Vec::with_capacity(torus_rank);
        for (b, value) in row.iter().enumerate() {
            let adjusted = value + Integer::from(usize::from(a == b));
            converted
                .push(i32::try_from(&adjusted).map_err(|_| StructureError::ArithmeticOverflow)?);
        }
        tau1.push(converted);
    }

    Ok(classify_plus_identity(&tau1, budget)?.as_tuple())
}

#[cfg(test)]
mod tests {
    use crate::{
        BasedRootDatum, Coweight, InnerClass, IntegerLatticeBudget, LatticeInvolution, Weight,
    };

    use super::*;

    fn budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    fn layout(datum: &BasedRootDatum, involution: LatticeInvolution) -> InnerClassLayout {
        let inner_class = InnerClass::new(datum.clone(), involution, 64).unwrap();
        InnerClassLayout::build(&inner_class, &budget()).unwrap()
    }

    #[test]
    fn a1_compact_inner_class_layout() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let layout = layout(&datum, involution);
        assert_eq!(layout.factors(), &[('A', 1)]);
        assert_eq!(layout.letters(), &['c']);
        assert_eq!(layout.perm(), &[0]);
        assert_eq!(layout.lie_type_string(), "A1");
        assert_eq!(layout.inner_class_string(), "c");
    }

    #[test]
    fn twisted_a2_is_unequal_rank_split_letter() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let layout = layout(&datum, twist);
        assert_eq!(layout.factors(), &[('A', 2)]);
        assert_eq!(layout.letters(), &['s']);
        assert_eq!(layout.perm(), &[0, 1]);
    }

    #[test]
    fn a1_a1_swap_is_a_complex_factor() {
        let datum = BasedRootDatum::standard(vec![vec![2, 0], vec![0, 2]]).unwrap();
        let swap = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let layout = layout(&datum, swap);
        // The Complex letter consumes two A1 factors; the second factor's
        // Bourbaki entries are the twist images of the first.
        assert_eq!(layout.factors(), &[('A', 1), ('A', 1)]);
        assert_eq!(layout.letters(), &['C']);
        assert_eq!(layout.perm(), &[0, 1]);
        assert_eq!(layout.lie_type_string(), "A1.A1");
        assert_eq!(layout.inner_class_string(), "C");
    }

    #[test]
    fn d4_triality_like_twist_is_u_letter() {
        // D4 with the fork-tip swap (2 <-> 3): an unequal-rank twist of an
        // even-rank D factor, letter 'u'.
        let cartan = vec![
            vec![2, -1, 0, 0],
            vec![-1, 2, -1, -1],
            vec![0, -1, 2, 0],
            vec![0, -1, 0, 2],
        ];
        let datum = BasedRootDatum::standard(cartan).unwrap();
        let mut matrix = vec![vec![0; 4]; 4];
        matrix[0][0] = 1;
        matrix[1][1] = 1;
        matrix[2][3] = 1;
        matrix[3][2] = 1;
        let twist = LatticeInvolution::new(&datum, matrix.clone(), matrix).unwrap();
        let layout = layout(&datum, twist);
        assert_eq!(layout.factors(), &[('D', 4)]);
        assert_eq!(layout.letters(), &['u']);
    }

    #[test]
    fn central_torus_letters_follow_the_quotient_involution() {
        // GL(2)-like datum: A1 plus a one-dimensional central torus, with the
        // involution acting as -1 on the torus quotient: one split T1.
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![2, 0])],
            vec![Coweight::new(vec![1, 0])],
        )
        .unwrap();
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![1, 0], vec![0, -1]],
            vec![vec![1, 0], vec![0, -1]],
        )
        .unwrap();
        let layout = layout(&datum, involution);
        assert_eq!(layout.factors(), &[('A', 1), ('T', 1)]);
        assert_eq!(layout.letters(), &['c', 's']);
        assert_eq!(layout.lie_type_string(), "A1.T1");
        assert_eq!(layout.inner_class_string(), "cs");
    }

    #[test]
    fn complex_factor_rotation_reorders_intervening_factors() {
        // A1.B2.A1 with the twist swapping the two A1 factors: the Complex
        // pair rotates adjacently, type and perm follow.
        let cartan = vec![
            vec![2, 0, 0, 0],
            vec![0, 2, -2, 0],
            vec![0, -1, 2, 0],
            vec![0, 0, 0, 2],
        ];
        let datum = BasedRootDatum::standard(cartan).unwrap();
        let mut matrix = vec![vec![0; 4]; 4];
        matrix[0][3] = 1;
        matrix[3][0] = 1;
        matrix[1][1] = 1;
        matrix[2][2] = 1;
        let twist = LatticeInvolution::new(&datum, matrix.clone(), matrix).unwrap();
        let layout = layout(&datum, twist);
        assert_eq!(layout.factors(), &[('A', 1), ('A', 1), ('B', 2)]);
        assert_eq!(layout.letters(), &['C', 'c']);
        // The Complex pair occupies the first two Bourbaki slots: datum
        // vertex 0 and its twist image 3, then the B2 component.
        assert_eq!(layout.perm(), &[0, 3, 1, 2]);
    }
}
