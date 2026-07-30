//! Lie-algebra names of real forms: a faithful port of upstream
//! `output::printType` and its helpers `print_real_form_name`,
//! `printComplexType`, and `split` (io/output.cpp:556-782).
//!
//! The input grading is the `specialGrading` partition overload: a bitset
//! over the datum's simple roots with 1 = noncompact imaginary. It is
//! adapted to the layout's Bourbaki ordering by the pull-back
//! `pulled[k] = grading[perm[k]]` (permutations.cpp:66-76) and consumed one
//! simple factor at a time, exactly like the upstream `gr >>= rank` loop.

use crate::error::StructureError;
use crate::layout::InnerClassLayout;

/// The upstream `split` helper: `(n)` when `m == 0`, else `(n-m,m)` with
/// the entries weakly decreasing.
fn split(name: &str, n: usize, m: usize) -> String {
    if m == 0 {
        return format!("{name}({n})");
    }
    let minority = if 2 * m > n { n - m } else { m };
    format!("{name}({},{})", n - minority, minority)
}

/// `printComplexType`: the complex Lie algebra of a `'C'` inner-class entry
/// (covering two isomorphic factors, of which the first is passed).
fn complex_name(letter: char, rank: usize) -> Result<String, StructureError> {
    let name = match letter {
        'A' => format!("sl({},C)", rank + 1),
        'B' => format!("so({},C)", 2 * rank + 1),
        'C' => format!("sp({},C)", 2 * rank),
        'D' => format!("so({},C)", 2 * rank),
        'E' => format!("e{rank}(C)"),
        'F' => "f4(C)".to_string(),
        'G' => "g2(C)".to_string(),
        'T' => "gl(1,C)".to_string(),
        _ => {
            return Err(StructureError::LayoutInvariantViolation {
                invariant: "complex factor letter",
            })
        }
    };
    Ok(name)
}

/// `print_real_form_name`: the real Lie algebra of one simple (or torus)
/// factor from its special grading slice and inner-class letter.
///
/// The folded lowercase letters `'f'`/`'g'` occur only in dual-side naming
/// of certain dual real forms; this port covers the non-dual letters that
/// the crate's layout can produce.
fn factor_name(bits: u32, letter: char, rank: usize, ic: char) -> Result<String, StructureError> {
    let trivial = bits == 0;
    // Upstream `m = gr.firstBit() + 1` (0 for a trivial grading).
    let m = if trivial {
        0
    } else {
        bits.trailing_zeros() as usize + 1
    };
    let name = match letter {
        'A' => {
            if rank == 1 {
                if trivial {
                    "su(2)".to_string()
                } else {
                    "sl(2,R)".to_string()
                }
            } else {
                let n = rank + 1;
                if ic == 'c' {
                    split("su", n, m)
                } else if !rank.is_multiple_of(2) && trivial {
                    // Unequal-rank A: both conditions are needed upstream.
                    format!("sl({},H)", n / 2)
                } else {
                    format!("sl({n},R)")
                }
            }
        }
        'B' => split("so", 2 * rank + 1, 2 * m),
        'C' => {
            if m == rank {
                format!("sp({},R)", 2 * rank)
            } else {
                split("sp", rank, m)
            }
        }
        'D' => {
            let n = 2 * rank;
            if ic == 'c' || (rank.is_multiple_of(2) && ic == 's') {
                if m < rank - 1 {
                    split("so", n, 2 * m)
                } else if !rank.is_multiple_of(2) {
                    format!("so*({n})")
                } else if rank.is_multiple_of(4) == (m == rank) {
                    format!("so*({n})[1,0]")
                } else {
                    format!("so*({n})[0,1]")
                }
            } else if rank > 4 {
                split("so", n, 2 * m + 1)
            } else if trivial {
                // Unequal-rank D4: any nonzero grading means so(5,3).
                "so(7,1)".to_string()
            } else {
                "so(5,3)".to_string()
            }
        }
        'E' => {
            let mut name = format!("e{rank}");
            if trivial && (ic == 'c' || rank > 6) {
                return Ok(name);
            }
            name.push('(');
            if rank == 6 {
                if ic == 'c' {
                    name.push_str(if m == 1 || m == 6 {
                        "so(10).u(1)"
                    } else {
                        "su(6).su(2)"
                    });
                } else {
                    name.push_str(if trivial { "f4" } else { "R" });
                }
            } else if rank == 7 {
                name.push_str(if m == 7 {
                    "e6.u(1)"
                } else if m == 2 || m == 5 {
                    "R"
                } else {
                    "so(12).su(2)"
                });
            } else {
                // E8: e8(e7.su(2)) has noncompact positions {2,3,6,7} (0xCC).
                name.push_str(if bits & 0xCC != 0 { "e7.su(2)" } else { "R" });
            }
            name.push(')');
            name
        }
        'F' => {
            if trivial {
                "f4".to_string()
            } else if m >= 3 {
                "f4(so(9))".to_string()
            } else {
                "f4(R)".to_string()
            }
        }
        'G' => {
            if trivial {
                "g2".to_string()
            } else {
                "g2(R)".to_string()
            }
        }
        'T' => {
            if ic == 'c' {
                "u(1)".to_string()
            } else {
                "gl(1,R)".to_string()
            }
        }
        _ => {
            return Err(StructureError::LayoutInvariantViolation {
                invariant: "real form factor letter",
            })
        }
    };
    Ok(name)
}

/// `output::printType`: the dotted Lie-algebra name of the real form whose
/// special grading is `grading` (bits indexed by datum simple roots).
pub fn form_type_name(layout: &InnerClassLayout, grading: u128) -> Result<String, StructureError> {
    let factors = layout.factors();
    let letters = layout.letters();
    let perm = layout.perm();
    if perm.iter().any(|&position| position >= 128) {
        return Err(StructureError::LayoutInvariantViolation {
            invariant: "grading key width",
        });
    }

    let mut name = String::new();
    let mut factor = 0_usize;
    let mut shift = 0_usize;
    for (entry, &ic) in letters.iter().enumerate() {
        if entry > 0 {
            name.push('.');
        }
        let &(letter, rank) =
            factors
                .get(factor)
                .ok_or(StructureError::LayoutInvariantViolation {
                    invariant: "letter/factor alignment",
                })?;
        // Torus factors have semisimple rank zero: they consume no grading
        // bits (upstream `gr >>= lt[i].semisimple_rank()`).
        let slice_rank = if letter == 'T' { 0 } else { rank };
        if ic == 'C' {
            name.push_str(&complex_name(letter, rank)?);
            // A Complex entry consumes two factors and two rank slices.
            shift += slice_rank;
            factor += 1;
            let &(partner_letter, partner_rank) =
                factors
                    .get(factor)
                    .ok_or(StructureError::LayoutInvariantViolation {
                        invariant: "letter/factor alignment",
                    })?;
            shift += if partner_letter == 'T' {
                0
            } else {
                partner_rank
            };
            factor += 1;
        } else {
            // Pull the grading back to Bourbaki order and slice this factor:
            // bit t of the slice is grading bit `perm[shift + t]`.
            let mut bits = 0_u32;
            for t in 0..slice_rank {
                let position =
                    perm.get(shift + t)
                        .ok_or(StructureError::LayoutInvariantViolation {
                            invariant: "permutation width",
                        })?;
                if (grading >> position) & 1 != 0 {
                    bits |= 1_u32 << t;
                }
            }
            name.push_str(&factor_name(bits, letter, rank, ic)?);
            shift += slice_rank;
            factor += 1;
        }
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use crate::layout::InnerClassLayout;
    use crate::{
        BasedRootDatum, Coweight, InnerClass, IntegerLatticeBudget, LatticeInvolution, Weight,
    };

    use super::*;

    fn budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    fn layout_of(datum: &BasedRootDatum, involution: LatticeInvolution) -> InnerClassLayout {
        let inner_class = InnerClass::new(datum.clone(), involution, 256).unwrap();
        InnerClassLayout::build(&inner_class, &budget()).unwrap()
    }

    #[test]
    fn a1_compact_and_split_names() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let layout = layout_of(&datum, LatticeInvolution::identity(&datum).unwrap());
        assert_eq!(form_type_name(&layout, 0).unwrap(), "su(2)");
        assert_eq!(form_type_name(&layout, 1).unwrap(), "sl(2,R)");
    }

    #[test]
    fn a2_names_follow_the_inner_class_letter() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let compact = layout_of(&datum, LatticeInvolution::identity(&datum).unwrap());
        assert_eq!(form_type_name(&compact, 0).unwrap(), "su(3)");
        assert_eq!(form_type_name(&compact, 1).unwrap(), "su(2,1)");
        // Weakly decreasing entries: su(1,2) prints as su(2,1).
        assert_eq!(form_type_name(&compact, 2).unwrap(), "su(2,1)");

        let twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let unequal = layout_of(&datum, twist);
        // Unequal-rank A2 has only sl(3,R), regardless of the grading.
        assert_eq!(form_type_name(&unequal, 0).unwrap(), "sl(3,R)");
    }

    #[test]
    fn a3_unequal_rank_names_sl2h_and_sl4r() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1, 0], vec![-1, 2, -1], vec![0, -1, 2]])
            .unwrap();
        let twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 0, 1], vec![0, 1, 0], vec![1, 0, 0]],
            vec![vec![0, 0, 1], vec![0, 1, 0], vec![1, 0, 0]],
        )
        .unwrap();
        let layout = layout_of(&datum, twist);
        assert_eq!(form_type_name(&layout, 0).unwrap(), "sl(2,H)");
        assert_eq!(form_type_name(&layout, 1).unwrap(), "sl(4,R)");
    }

    #[test]
    fn complex_pair_name_consumes_two_factors() {
        let datum = BasedRootDatum::standard(vec![vec![2, 0], vec![0, 2]]).unwrap();
        let swap = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let layout = layout_of(&datum, swap);
        assert_eq!(form_type_name(&layout, 0).unwrap(), "sl(2,C)");
    }

    #[test]
    fn b2_names_cover_sp11_and_sp4r() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let layout = layout_of(&datum, LatticeInvolution::identity(&datum).unwrap());
        assert_eq!(form_type_name(&layout, 0).unwrap(), "so(5)");
        // B2: bit 0 (long root first in datum order) -> so(3,2).
        assert_eq!(form_type_name(&layout, 1).unwrap(), "so(3,2)");
        assert_eq!(form_type_name(&layout, 2).unwrap(), "so(4,1)");
    }

    #[test]
    fn d4_unequal_rank_names() {
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
        let layout = layout_of(&datum, twist);
        assert_eq!(form_type_name(&layout, 0).unwrap(), "so(7,1)");
        assert_eq!(form_type_name(&layout, 4).unwrap(), "so(5,3)");
    }

    #[test]
    fn torus_letters_name_u1_and_gl1r() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![2, 0])],
            vec![Coweight::new(vec![1, 0])],
        )
        .unwrap();
        let split_torus = LatticeInvolution::new(
            &datum,
            vec![vec![1, 0], vec![0, -1]],
            vec![vec![1, 0], vec![0, -1]],
        )
        .unwrap();
        let layout = layout_of(&datum, split_torus);
        // 'c' on A1 with compact grading 0, 's' on T1: su(2).gl(1,R).
        assert_eq!(form_type_name(&layout, 0).unwrap(), "su(2).gl(1,R)");
        assert_eq!(form_type_name(&layout, 1).unwrap(), "sl(2,R).gl(1,R)");
    }
}
