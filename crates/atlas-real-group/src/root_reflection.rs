//! Shared ambient-root reflection words.
//!
//! This is the single port of upstream `RootDatum::reflection_word`
//! (`rootdata.cpp:1092-1095`).  Consumers must not grow private copies: the
//! word convention is subtle and is shared by extended parameters and common
//! block packet generation.

use crate::{RepContext, RootId, StructureError};

fn wrapping_dot(left: &[i32], right: &[i32]) -> i32 {
    debug_assert_eq!(left.len(), right.len());
    left.iter()
        .zip(right)
        .fold(0_i32, |sum, (&a, &b)| sum.wrapping_add(a.wrapping_mul(b)))
}

fn add_scaled(accumulator: &mut [i32], vector: &[i32], factor: i32) {
    debug_assert_eq!(accumulator.len(), vector.len());
    for (entry, &coordinate) in accumulator.iter_mut().zip(vector) {
        *entry = entry.wrapping_add(coordinate.wrapping_mul(factor));
    }
}

/// `to_dominant(reflection(alpha, twoRho))`, with the resulting word
/// reversed exactly as upstream does.
pub(crate) fn reflection_word(
    rc: &RepContext<'_>,
    alpha: RootId,
) -> Result<Vec<usize>, StructureError> {
    let system = rc.root_system();
    let alpha_root = system.root(alpha).ok_or(StructureError::IndexOutOfRange {
        index: alpha.index(),
        upper_bound: system.roots().len(),
    })?;
    let alpha_coroot = system
        .coroot(alpha)
        .ok_or(StructureError::IndexOutOfRange {
            index: alpha.index(),
            upper_bound: system.roots().len(),
        })?;
    let factor = wrapping_dot(rc.two_rho().as_slice(), alpha_coroot.as_slice());
    let mut value = rc.two_rho().as_slice().to_vec();
    add_scaled(&mut value, alpha_root.as_slice(), factor.wrapping_neg());

    let mut word = Vec::new();
    loop {
        let mut reflected = false;
        for (generator, &simple) in system.simple_root_ids().iter().enumerate() {
            let simple_root = system.root(simple).ok_or(StructureError::IndexOutOfRange {
                index: simple.index(),
                upper_bound: system.roots().len(),
            })?;
            let simple_coroot = system
                .coroot(simple)
                .ok_or(StructureError::IndexOutOfRange {
                    index: simple.index(),
                    upper_bound: system.roots().len(),
                })?;
            let pairing = wrapping_dot(&value, simple_coroot.as_slice());
            if pairing < 0 {
                word.push(generator);
                add_scaled(&mut value, simple_root.as_slice(), pairing.wrapping_neg());
                reflected = true;
                break;
            }
        }
        if !reflected {
            word.reverse();
            return Ok(word);
        }
    }
}
