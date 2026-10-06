//! Reflection-subgroup orbits, with ambient Weyl witnesses.
//!
//! Ordering follows rootdata.cpp's BitMap basic_orbit/extend_orbit, not the
//! sorted-layer alcove variant. Unlike original 7e1b958c rootdata.h, the
//! initial dominance operation actually uses the supplied generators.
//! See docs/slices/weyl_subgroup_orbits_2026-09-29.md for the retained
//! original counterexample and the independent trivial-subgroup invariant.

use super::{
    arity, as_integer, as_root_datum, build_weyl_context, infer_lie_type,
    internal_root_nbr, reflection_word, runtime, signed_roots, type_error,
    weyl_elt_value, BigInt, Diagnostic, DomainValue, Matrix, RootDatumHandle,
    RootNumbering, RootSystem, SourceSpan, Value, Vec32, WeylElement,
};
use std::collections::BTreeSet;
use std::sync::Arc;

struct Subgroup {
    system: RootSystem,
    numbering: RootNumbering,
    // User order controls dominance and extension. RootNbr order controls BFS.
    generators: Vec<usize>,
    cartan: Vec<Vec<i32>>,
}

fn pairing(left: &[i32], right: &[i32], span: SourceSpan) -> Result<i128, Diagnostic> {
    left.iter().zip(right).try_fold(0_i128, |sum, (&a, &b)| {
        sum.checked_add(i128::from(a) * i128::from(b))
            .ok_or_else(|| runtime(span, "Integer value too big for subgroup pairing"))
    })
}

fn arguments(
    args: &[Value],
    span: SourceSpan,
) -> Result<(&RootDatumHandle, &Value, &Value, bool), Diagnostic> {
    arity("Weyl subgroup orbit", args, 3, span)?;
    let dual = !matches!(&args[0], Value::Domain(DomainValue::RootDatum(_)));
    if dual {
        Ok((as_root_datum(&args[1], span)?, &args[2], &args[0], true))
    } else {
        Ok((as_root_datum(&args[0], span)?, &args[1], &args[2], false))
    }
}

impl Subgroup {
    fn new(handle: &RootDatumHandle, row: &Value, span: SourceSpan) -> Result<Self, Diagnostic> {
        let Value::List(entries) = row else {
            return Err(type_error(span, "expected a row of integers"));
        };
        let (system, numbering) = signed_roots(handle, span)?;
        let mut generators = Vec::with_capacity(entries.len());
        for entry in entries {
            let index = as_integer(entry, span)?;
            // check_W_subsystem narrows before checking each signed root.
            i32::try_from(&index)
                .map_err(|_| runtime(span, "Integer value too big for conversion"))?;
            generators.push(internal_root_nbr(&index, &numbering, false, span)?);
        }
        let mut cartan = Vec::with_capacity(generators.len());
        for &alpha in &generators {
            let mut row = Vec::with_capacity(generators.len());
            for &beta in &generators {
                let p = pairing(
                    system.root(numbering.id(alpha)).expect("valid root").as_slice(),
                    system.coroot(numbering.id(beta)).expect("valid coroot").as_slice(),
                    span,
                )?;
                row.push(i32::try_from(p)
                    .map_err(|_| runtime(span, "Integer value too big for Cartan pairing"))?);
            }
            cartan.push(row);
        }
        if infer_lie_type(&cartan, handle.datum.lattice_rank(), span).is_err() {
            let text = if cartan.len() == 1 {
                format!("[[{}]]", cartan[0][0])
            } else {
                format!("[{}]", cartan.iter().map(|row| {
                    row.iter().map(i32::to_string).collect::<Vec<_>>().join(",")
                }).collect::<Vec<_>>().join("|"))
            };
            return Err(runtime(span, format!(
                "Matrix for root indices is not a Cartan matrix: \n  {text}"
            )));
        }
        Ok(Self { system, numbering, generators, cartan })
    }

    fn root(&self, s: usize) -> &[i32] {
        self.system.root(self.numbering.id(self.generators[s]))
            .expect("validated root").as_slice()
    }

    fn coroot(&self, s: usize) -> &[i32] {
        self.system.coroot(self.numbering.id(self.generators[s]))
            .expect("validated coroot").as_slice()
    }

    fn level(&self, s: usize, v: &[i32], dual: bool, span: SourceSpan) -> Result<i128, Diagnostic> {
        pairing(v, if dual { self.root(s) } else { self.coroot(s) }, span)
    }

    fn reflect(&self, s: usize, v: &[i32], dual: bool, span: SourceSpan) -> Result<Vec<i32>, Diagnostic> {
        let level = self.level(s, v, dual, span)?;
        let step = if dual { self.coroot(s) } else { self.root(s) };
        v.iter().zip(step).map(|(&a, &b)| {
            level.checked_mul(i128::from(b))
                .and_then(|delta| i128::from(a).checked_sub(delta))
                .and_then(|x| i32::try_from(x).ok())
                .ok_or_else(|| runtime(span, "Integer value too big for subgroup orbit coordinate"))
        }).collect()
    }

    /// Return reflection events in application order (for BOTH actions).
    fn dominant(&self, v: &mut Vec<i32>, dual: bool, span: SourceSpan) -> Result<Vec<usize>, Diagnostic> {
        let mut events = Vec::new();
        loop {
            let mut next = None;
            for s in 0..self.generators.len() {
                if self.level(s, v, dual, span)? < 0 {
                    next = Some(s);
                    break;
                }
            }
            let Some(s) = next else { return Ok(events) };
            *v = self.reflect(s, v, dual, span)?;
            events.push(s);
        }
    }

    /// Coset tree in exact simple-(co)root pairing coordinates.
    ///
    /// Original's kernel-selected vector has zero pairings on the old stab
    /// and a positive pairing c on generator i. Our vector has 1 there.
    /// On the enlarged subgroup its pairing vector is therefore scaled by
    /// positive c: every sign, equality, and BFS edge is identical. Invertible
    /// finite Cartan matrices make restriction injective on orbit differences.
    /// Extra coordinates outside the active subgroup cannot change equality.
    /// This avoids arbitrary lattice-kernel elections and fixed-width overflow.
    fn cosets(&self, stab: &mut [bool], i: usize, dual: bool) -> Vec<(usize, usize)> {
        let n = self.generators.len();
        let mut e = vec![BigInt::from(0); n];
        e[i] = BigInt::from(1);
        let reflect = |v: &[BigInt], s: usize| -> Vec<BigInt> {
            (0..n).map(|j| {
                let c = if dual { self.cartan[j][s] } else { self.cartan[s][j] };
                &v[j] - &v[s] * BigInt::from(c)
            }).collect()
        };
        let first = reflect(&e, i);
        let mut vectors = vec![e, first];
        let mut tree = vec![(usize::MAX, usize::MAX), (i, 0)];
        stab[i] = true;
        let mut active: Vec<usize> = (0..n).filter(|&s| stab[s]).collect();
        active.sort_unstable_by_key(|&s| self.generators[s]);
        let (mut start, mut finish) = (1, 2);
        loop {
            // Only deduplicate the new layer, preserving first insertion.
            let mut layer = BTreeSet::new();
            for parent in start..finish {
                for &s in &active {
                    if vectors[parent][s] <= 0 { continue; }
                    let next = reflect(&vectors[parent], s);
                    if layer.insert(next.clone()) {
                        vectors.push(next);
                        tree.push((s, parent));
                    }
                }
            }
            if vectors.len() == finish { return tree; }
            start = finish;
            finish = vectors.len();
        }
    }
}

pub(super) fn validate(args: &[Value], span: SourceSpan) -> Result<(), Diagnostic> {
    let (handle, gens, _, _) = arguments(args, span)?;
    Subgroup::new(handle, gens, span)?;
    Ok(())
}

pub(super) fn call(name: &str, args: &[Value], span: SourceSpan) -> Result<Value, Diagnostic> {
    let (handle, gens, coordinates, dual) = arguments(args, span)?;
    let group = Subgroup::new(handle, gens, span)?;
    let mut weight = match coordinates {
        Value::Vector(Vec32(v)) => v.clone(),
        Value::List(entries) => entries.iter().map(|entry| {
            i32::try_from(&as_integer(entry, span)?)
                .map_err(|_| runtime(span, "Integer value too big for conversion"))
        }).collect::<Result<Vec<_>, _>>()?,
        _ => return Err(type_error(span, "expected a vec")),
    };
    let rank = handle.datum.lattice_rank();
    // Upstream reads beyond short vectors. Do not emulate undefined behavior.
    if weight.len() != rank {
        return Err(runtime(span, "Wrong vector size for Weyl subgroup orbit"));
    }
    let events = group.dominant(&mut weight, dual, span)?;
    let mut stab = (0..group.generators.len()).map(|s| {
        group.level(s, &weight, dual, span).map(|level| level == 0)
    }).collect::<Result<Vec<_>, _>>()?;
    let non_stab: Vec<usize> = (0..stab.len()).filter(|&s| !stab[s]).collect();
    if name == "Weyl_orbit" {
        let mut orbit = vec![weight];
        for i in non_stab {
            let tree = group.cosets(&mut stab, i, dual);
            let mut next = Vec::new();
            for element in orbit {
                let mut segment = vec![element];
                for &(s, parent) in &tree[1..] {
                    segment.push(group.reflect(s, &segment[parent], dual, span)?);
                }
                next.extend(segment);
            }
            orbit = next;
        }
        let data = orbit.iter().flatten().copied().collect();
        return Ok(Value::Matrix(Matrix::from_columns(rank, orbit.len(), data)
            .expect("orbit columns have checked lattice rank")));
    }
    let reflections: Vec<Vec<usize>> = group.generators.iter().map(|&r| {
        reflection_word(&group.system, &group.numbering, r)
    }).collect();
    let mut word = Vec::new();
    // Weights apply right to left, coweights left to right.
    if dual {
        for s in events { word.extend_from_slice(&reflections[s]); }
    } else {
        for s in events.into_iter().rev() { word.extend_from_slice(&reflections[s]); }
    }
    let mut orbit = vec![word];
    for i in non_stab {
        let tree = group.cosets(&mut stab, i, dual);
        let mut next = Vec::new();
        for word in orbit {
            let mut segment = vec![word];
            for &(s, parent) in &tree[1..] {
                let (left, right) = if dual {
                    (&segment[parent], &reflections[s])
                } else {
                    (&reflections[s], &segment[parent])
                };
                let mut extended = left.clone();
                extended.extend_from_slice(right);
                segment.push(extended);
            }
            next.extend(segment);
        }
        orbit = next;
    }
    let context = build_weyl_context(handle, span)?;
    let mut result = Vec::with_capacity(orbit.len());
    for word in orbit {
        let mut element = WeylElement::identity(&context.kernel.system)
            .map_err(|e| runtime(span, e.to_string()))?;
        for s in word {
            element = element.right_multiply_simple(&context.kernel.system, s)
                .map_err(|e| runtime(span, e.to_string()))?.0;
        }
        result.push(weyl_elt_value(Arc::clone(&context), element, span)?);
    }
    Ok(Value::List(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{SourceId, SourcePosition};
    use super::super::call as domain_call;

    fn span() -> SourceSpan {
        SourceSpan::new(SourceId::anonymous(), 0, 0,
            SourcePosition { line: 1, column: 1 }, SourcePosition { line: 1, column: 1 })
    }

    fn datum(label: &str, numbering: bool, adjoint: bool) -> Value {
        let lie = domain_call("Lie_type", &[Value::String(label.into())], span()).unwrap();
        domain_call(if adjoint { "adjoint" } else { "simply_connected" },
            &[lie, Value::Boolean(numbering)], span()).unwrap()
    }

    fn row(gens: &[i32]) -> Value {
        Value::List(gens.iter().map(|&s| Value::Integer(BigInt::from(s))).collect())
    }

    fn args(rd: &Value, gens: &[i32], v: &[i32], dual: bool) -> Vec<Value> {
        let v = Value::Vector(Vec32(v.to_vec()));
        if dual { vec![v, rd.clone(), row(gens)] } else { vec![rd.clone(), row(gens), v] }
    }

    fn matrix(args: &[Value]) -> Matrix {
        let Value::Matrix(m) = domain_call("Weyl_orbit", args, span()).unwrap() else {
            panic!("orbit is not a matrix")
        };
        m
    }

    #[test]
    fn empty_subgroup_fixes_negative_weights_and_returns_ambient_identity() {
        for label in ["A2", "B2", "C2", "D4", "G2", "F4", "E6", "E7", "E8"] {
            for numbering in [true, false] {
                for adjoint in [false, true] {
                    let rd = datum(label, numbering, adjoint);
                    let rank = as_root_datum(&rd, span()).unwrap().datum.lattice_rank();
                    let v: Vec<_> = (0..rank).map(|j| -(j as i32) - 1).collect();
                    for dual in [false, true] {
                        let a = args(&rd, &[], &v, dual);
                        let m = matrix(&a);
                        assert_eq!(m.cols(), 1);
                        assert_eq!(m.column(0).0, v);
                        let Value::List(ws) = domain_call("Weyl_orbit_ws", &a, span()).unwrap() else {
                            panic!("witness list")
                        };
                        assert_eq!(ws.len(), 1);
                        assert_eq!(domain_call("word", &ws, span()).unwrap(), Value::List(vec![]));
                        assert_eq!(domain_call("root_datum", &ws, span()).unwrap(), rd);
                    }
                }
            }
        }
    }

    /// Independent exhaustive graph closure: no dominance or coset tree.
    fn closure(group: &Subgroup, v: &[i32], dual: bool) -> BTreeSet<Vec<i32>> {
        let mut found = BTreeSet::from([v.to_vec()]);
        let mut pending = vec![v.to_vec()];
        while let Some(v) = pending.pop() {
            for s in 0..group.generators.len() {
                let (linear, step) = if dual {
                    (group.root(s), group.coroot(s))
                } else { (group.coroot(s), group.root(s)) };
                let level: BigInt = v.iter().zip(linear)
                    .map(|(&a, &b)| BigInt::from(a) * BigInt::from(b)).sum();
                let image: Vec<_> = v.iter().zip(step).map(|(&a, &b)| {
                    i32::try_from(&(BigInt::from(a) - &level * BigInt::from(b))).unwrap()
                }).collect();
                if found.insert(image.clone()) { pending.push(image); }
            }
        }
        found
    }

    #[test]
    fn coset_orbits_equal_independent_closure_and_every_witness_reconstructs() {
        for label in ["A2", "B2", "C2", "D4", "G2", "F4", "E6", "E7"] {
            for numbering in [true, false] {
                let rd = datum(label, numbering, false);
                let handle = as_root_datum(&rd, span()).unwrap();
                let (_, num) = signed_roots(handle, span()).unwrap();
                for gens in [vec![], vec![-1], vec![num.npos as i32 - 1], vec![0, 1], vec![1, 0]] {
                    let group = Subgroup::new(handle, &row(&gens), span()).unwrap();
                    for sign in [-1, 0, 1] {
                        let v: Vec<_> = (0..handle.datum.lattice_rank())
                            .map(|j| sign * (j as i32 + 1)).collect();
                        for dual in [false, true] {
                            let a = args(&rd, &gens, &v, dual);
                            let m = matrix(&a);
                            let actual: BTreeSet<_> = (0..m.cols()).map(|j| m.column(j).0).collect();
                            assert_eq!(actual.len(), m.cols(), "duplicate columns");
                            assert_eq!(actual, closure(&group, &v, dual), "{label} {gens:?} {v:?} dual={dual}");
                            let Value::List(ws) = domain_call("Weyl_orbit_ws", &a, span()).unwrap() else {
                                panic!("witness list")
                            };
                            assert_eq!(ws.len(), m.cols());
                            for (j, w) in ws.into_iter().enumerate() {
                                let original = Value::Vector(Vec32(v.clone()));
                                let action = if dual { vec![original, w] } else { vec![w, original] };
                                assert_eq!(domain_call("*", &action, span()).unwrap(), Value::Vector(m.column(j)));
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn full_simple_subgroup_keeps_complete_full_group_orbit_order() {
        for label in ["A2", "B2", "C2", "G2", "F4"] {
            for numbering in [true, false] {
                let rd = datum(label, numbering, false);
                let rank = as_root_datum(&rd, span()).unwrap().datum.lattice_rank();
                let gens: Vec<_> = (0..rank as i32).collect();
                let v: Vec<_> = (0..rank as i32).map(|j| j + 1).collect();
                for dual in [false, true] {
                    let original = Value::Vector(Vec32(v.clone()));
                    let full_args = if dual { vec![original, rd.clone()] } else { vec![rd.clone(), original] };
                    for name in ["Weyl_orbit", "Weyl_orbit_ws"] {
                        assert_eq!(domain_call(name, &args(&rd, &gens, &v, dual), span()).unwrap(),
                            domain_call(name, &full_args, span()).unwrap(), "{label} {name} dual={dual}");
                    }
                }
            }
        }
    }

    #[test]
    fn invalid_generators_match_original_diagnostics_before_discarding() {
        let rd = datum("A2", true, false);
        for (gens, message) in [
            (vec![3], "Illegal root index 3"),
            (vec![-4], "Illegal root index -4"),
            (vec![0, 0], "Matrix for root indices is not a Cartan matrix: \n  [2,2|2,2]"),
            (vec![0, -1], "Matrix for root indices is not a Cartan matrix: \n  [2,-2|-2,2]"),
            (vec![0, 2], "Matrix for root indices is not a Cartan matrix: \n  [2,1|1,2]"),
        ] {
            for dual in [false, true] {
                let a = args(&rd, &gens, &[1, 2], dual);
                for name in ["Weyl_orbit", "Weyl_orbit_ws"] {
                    assert_eq!(domain_call(name, &a, span()).unwrap_err().message, message);
                    assert_eq!(super::super::validate(name, &a, span()).unwrap_err().message, message);
                }
            }
        }
        let a = vec![rd, Value::List(vec![Value::Integer(BigInt::from(2147483648_i64))]),
            Value::Vector(Vec32(vec![1, 2]))];
        assert_eq!(validate(&a, span()).unwrap_err().message, "Integer value too big for conversion");
    }

    #[test]
    fn overflow_and_wrong_rank_are_safe_errors_not_wrapped_orbits() {
        let rd = datum("A1", true, false);
        let a = args(&rd, &[0], &[i32::MIN], false);
        assert!(domain_call("Weyl_orbit", &a, span()).unwrap_err().message
            .contains("too big for subgroup orbit coordinate"));
        assert!(validate(&a, span()).is_ok(), "discard validation does not compute the orbit");
        let empty = args(&rd, &[], &[i32::MIN], false);
        assert_eq!(matrix(&empty).column(0).0, vec![i32::MIN]);
        assert_eq!(domain_call("Weyl_orbit", &args(&rd, &[0], &[], false), span())
            .unwrap_err().message, "Wrong vector size for Weyl subgroup orbit");
    }
}
