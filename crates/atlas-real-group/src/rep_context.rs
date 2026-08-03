//! The `Rep_context` subset for `StandardRepr` and `KType` values.
//!
//! This module ports the parameter-layer mathematics of upstream
//! `gkmod/repr.cpp` and the `K_type` normalization of
//! `structure/involutions.cpp`: a [`StandardRepr`] is the
//! `(x, y, gamma, height)` quadruple of `repr.h:76-110`, built only through
//! [`RepContext::sr_gamma`] / [`RepContext::sr`], and the per-involution
//! `(1-theta)X^*` image-basis pair (`lift_mat`, `M_real`) that upstream
//! stores in its `InvolutionTable::record` (involutions.h:104-105) is
//! derived here on construction, per the crate's documented divergence of
//! transporting `M_real`/`lift_mat` in the involution table itself (see
//! `involution_table.rs`). The echelon reduction reproduces upstream's
//! `matreduc::column_echelon` (matreduc.h:129) and its gcd sweep
//! (matreduc.h:70) operation-for-operation, because the elected
//! `lambda-rho` representative depends on the exact image basis.

use std::collections::BTreeMap;

use crate::grading::try_capacity;
use crate::lattice::{checked_add_weights, checked_sub_weights, pair, RationalWeight};
use crate::{
    BlockGraph, Coweight, InnerClass, InvolutionId, InvolutionTable, KType, KgbGraph, KgbId,
    KgbStatus, LatticeInvolution, ModTwoVector, RootId, RootKind, StructureError, Weight,
};

/// `<coweight, weight>` for an i64 numerator vector (the sign evaluation
/// of a rational weight's raw numerator, repr.cpp:1224).
fn pair_i64(weight: &[i64], coweight: &Coweight) -> Result<i64, StructureError> {
    if weight.len() != coweight.rank() {
        return Err(StructureError::RankMismatch {
            expected: coweight.rank(),
            actual: weight.len(),
        });
    }
    let mut result = 0_i64;
    for (&coordinate, &entry) in coweight.as_slice().iter().zip(weight) {
        result = result
            .checked_add(
                i64::from(coordinate)
                    .checked_mul(entry)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            )
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    Ok(result)
}

/// Insert or merge one (KType, integer multiplicity) term; a zero
/// coefficient removes the term (the `K_type_pol::add_term` semantics of
/// the K_type_formula accumulation).
fn merge_ktype_terms(terms: &mut Vec<(KType, i32)>, term: KType, coefficient: i32) {
    if coefficient == 0 {
        return;
    }
    if let Some(index) = terms.iter().position(|(existing, _)| *existing == term) {
        let updated = terms[index]
            .1
            .checked_add(coefficient)
            .expect("K-type formula coefficients stay in range");
        if updated == 0 {
            terms.remove(index);
        } else {
            terms[index].1 = updated;
        }
    } else {
        terms.push((term, coefficient));
    }
}

/// The pairing of a coweight with a rational numerator, as an i64.
fn dot_numerator(coweight: &Coweight, numerator: &[i64]) -> i64 {
    let mut total = 0_i64;
    for (index, &coordinate) in coweight.as_slice().iter().enumerate() {
        if coordinate != 0 {
            if let Some(&entry) = numerator.get(index) {
                total += i64::from(coordinate) * entry;
            }
        }
    }
    total
}

/// Insert a term into the work queue ordered by decreasing height
/// (repr.cpp:1300 `insert_into`).
fn insert_into_by_height(
    to_do: &mut Vec<(StandardRepr, i32)>,
    term: StandardRepr,
    coef: i32,
) -> Result<(), StructureError> {
    let position = to_do
        .iter()
        .position(|(candidate, _)| candidate.height() < term.height());
    match position {
        Some(index) => to_do.insert(index, (term, coef)),
        None => to_do.push((term, coef)),
    }
    Ok(())
}

/// A standard-module parameter: upstream `repr::StandardRepr`
/// (gkmod/repr.h:76-110).
///
/// `y_bits` is the torsion part of `lambda`, packed as the mod-2
/// coordinates of `(1-theta_x)(lambda-rho)` in the involution's `(1-theta)`
/// image basis (upstream `TorusPart`, involutions.h:211). `gamma` is the
/// (representative of the) infinitesimal character. `height` is derived
/// from the other fields at construction and is therefore excluded from
/// equality, exactly like upstream's `StandardRepr::operator==`
/// (repr.cpp:36-40).
#[derive(Clone, Debug)]
pub struct StandardRepr {
    x: KgbId,
    y_bits: ModTwoVector,
    gamma: RationalWeight,
    height: u32,
}

impl StandardRepr {
    pub fn x(&self) -> KgbId {
        self.x
    }

    /// The packed torsion part of `lambda` (upstream `y()`).
    pub fn y_bits(&self) -> &ModTwoVector {
        &self.y_bits
    }

    /// The infinitesimal character, gcd-normalized (upstream `gamma()`).
    pub fn gamma(&self) -> &RationalWeight {
        &self.gamma
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

/// Upstream `StandardRepr::operator==` (gkmod/repr.cpp:36-40): `x`, the
/// packed torsion part, and `gamma`; the derived `height` is not compared.
impl PartialEq for StandardRepr {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y_bits == other.y_bits && self.gamma == other.gamma
    }
}

/// The per-involution `(1-theta)X^*` image data: upstream's
/// `InvolutionTable::record` pair (involutions.h:104-105).
///
/// `lift_mat` is the column-echelon basis of the image of `1-theta`
/// (rank `n x r`); `m_real` (rank `r x n`) expresses an image element in
/// that basis. The pair satisfies `lift_mat * m_real == 1 - theta`
/// (involutions.h:105), which construction verifies.
#[derive(Clone, Debug, Eq, PartialEq)]
struct RealProjection {
    lift_mat: Vec<Vec<i64>>,
    m_real: Vec<Vec<i64>>,
}

impl RealProjection {
    /// Port of `matreduc::column_echelon` (matreduc.h:129-161) applied to
    /// `1-theta`, tracking the column-operation matrix and its inverse
    /// incrementally; upstream's `InvolutionTable::add_involution`
    /// (involutions.cpp:196-208) then takes `M_real` as the first `r`
    /// rows of the inverse.
    fn build(theta: &LatticeInvolution) -> Result<Self, StructureError> {
        let matrix = theta.weight_matrix();
        let rank = matrix.len();
        // `a` starts as the integer matrix `1 - theta` (involutions.cpp:196).
        let mut a: Vec<Vec<i64>> = Vec::new();
        a.try_reserve_exact(rank)
            .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
        for (row_index, row) in matrix.iter().enumerate() {
            let mut converted = Vec::new();
            converted
                .try_reserve_exact(rank)
                .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
            for (column_index, &entry) in row.iter().enumerate() {
                let diagonal = i64::from(row_index == column_index);
                converted.push(
                    diagonal
                        .checked_sub(i64::from(entry))
                        .ok_or(StructureError::ArithmeticOverflow)?,
                );
            }
            a.push(converted);
        }
        let mut col = identity_matrix(rank)?;
        let mut col_inverse = identity_matrix(rank)?;

        // Row sweep, bottom row first; the pivot of each processed row
        // lands at column `limit - 1` (matreduc.h:136-148).
        let mut limit = rank;
        for row in (0..rank).rev() {
            let pivot = gcd_sweep(&mut a, row, limit, &mut col, &mut col_inverse)?;
            if pivot == 0 {
                continue; // partial row already zero: no pivot in this row
            }
            limit -= 1; // pivot now at column `limit`
        }

        // Erase the `limit` zero columns, rotating the corresponding
        // kernel columns of `col` towards the right end one at a time
        // (matreduc.h:150-158): columns already parked at the right are
        // not touched again. `col_inverse` rows get the mirrored cyclic
        // shift, keeping `col * col_inverse == id`.
        let zero_columns = limit;
        let mut erased = 0_usize;
        while limit > 0 {
            limit -= 1;
            for a_row in a.iter_mut() {
                a_row.remove(limit);
            }
            let m_columns = rank - erased - 1; // column count of `a` now
            let cc: Vec<i64> = (0..rank).map(|k| col[k][limit]).collect();
            for col_row in col.iter_mut() {
                for j in limit..m_columns {
                    col_row[j] = col_row[j + 1];
                }
            }
            for (k, col_row) in col.iter_mut().enumerate() {
                col_row[m_columns] = cc[k];
            }
            let rotated_row = col_inverse.remove(limit);
            col_inverse.insert(m_columns, rotated_row);
            erased += 1;
        }
        debug_assert_eq!(erased, zero_columns);

        let image_rank = rank - zero_columns;
        let m_real: Vec<Vec<i64>> = col_inverse[..image_rank].to_vec();
        let projection = Self {
            lift_mat: a,
            m_real,
        };
        projection.check_against(theta)?;
        Ok(projection)
    }

    /// `lift_mat * m_real == 1 - theta` (involutions.h:105).
    fn check_against(&self, theta: &LatticeInvolution) -> Result<(), StructureError> {
        let matrix = theta.weight_matrix();
        for (row_index, row) in matrix.iter().enumerate() {
            for (column_index, &entry) in row.iter().enumerate() {
                let mut product = 0_i64;
                for (basis_index, basis_row) in self.m_real.iter().enumerate() {
                    product = product
                        .checked_add(
                            self.lift_mat[row_index][basis_index]
                                .checked_mul(basis_row[column_index])
                                .ok_or(StructureError::ArithmeticOverflow)?,
                        )
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
                let expected = i64::from(row_index == column_index) - i64::from(entry);
                if product != expected {
                    return Err(StructureError::RepInvariantViolation {
                        invariant: "image basis factorization",
                    });
                }
            }
        }
        Ok(())
    }

    /// `(1-theta)*v` in image-basis coordinates: `M_real * v`
    /// (involutions.h:211).
    fn coordinates(&self, weight: &Weight) -> Result<Vec<i64>, StructureError> {
        let mut result = Vec::new();
        result.try_reserve_exact(self.m_real.len()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: self.m_real.len(),
            }
        })?;
        for row in &self.m_real {
            let mut entry = 0_i64;
            for (&coefficient, &coordinate) in row.iter().zip(weight.as_slice()) {
                let product = coefficient
                    .checked_mul(i64::from(coordinate))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                entry = entry
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            result.push(entry);
        }
        Ok(result)
    }

    /// `lift_mat * coordinates` back in `X^*` (involutions.cpp:346-356).
    fn lift(&self, coordinates: &[i64]) -> Result<Vec<i64>, StructureError> {
        let mut result = vec![0_i64; self.lift_mat.len()];
        for (basis_index, &coordinate) in coordinates.iter().enumerate() {
            for (row, entry) in result.iter_mut().enumerate() {
                let product = self.lift_mat[row][basis_index]
                    .checked_mul(coordinate)
                    .ok_or(StructureError::ArithmeticOverflow)?;
                *entry = entry
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
        }
        Ok(result)
    }
}

fn identity_matrix(rank: usize) -> Result<Vec<Vec<i64>>, StructureError> {
    let mut matrix = Vec::new();
    matrix
        .try_reserve_exact(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    for row in 0..rank {
        let mut values = Vec::new();
        values
            .try_reserve_exact(rank)
            .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
        values.resize(rank, 0);
        values[row] = 1;
        matrix.push(values);
    }
    Ok(matrix)
}

/// One elementary column operation `column_j += c * column_k` on `a` and
/// `col`, mirrored by the inverse row operation `row_k -= c * row_j` on
/// `col_inverse`, keeping `col * col_inverse == id` throughout.
fn column_operation(
    a: &mut [Vec<i64>],
    col: &mut [Vec<i64>],
    col_inverse: &mut [Vec<i64>],
    j: usize,
    k: usize,
    c: i64,
) -> Result<(), StructureError> {
    if c == 0 {
        return Ok(());
    }
    for row in 0..a.len() {
        a[row][j] = a[row][j]
            .checked_add(
                c.checked_mul(a[row][k])
                    .ok_or(StructureError::ArithmeticOverflow)?,
            )
            .ok_or(StructureError::ArithmeticOverflow)?;
        col[row][j] = col[row][j]
            .checked_add(
                c.checked_mul(col[row][k])
                    .ok_or(StructureError::ArithmeticOverflow)?,
            )
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    let source_row = col_inverse[j].clone();
    for (inverse_entry, &source_entry) in col_inverse[k].iter_mut().zip(&source_row) {
        *inverse_entry = inverse_entry
            .checked_sub(
                c.checked_mul(source_entry)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            )
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    Ok(())
}

fn swap_columns(
    a: &mut [Vec<i64>],
    col: &mut [Vec<i64>],
    col_inverse: &mut [Vec<i64>],
    left: usize,
    right: usize,
) {
    if left == right {
        return;
    }
    for row in 0..a.len() {
        a[row].swap(left, right);
        col[row].swap(left, right);
    }
    col_inverse.swap(left, right);
}

/// The gcd sweep of `matreduc::gcd` (matreduc.h:70-122) on the partial
/// row `a[row][0..limit]`, recording the column operations into `col` and
/// their inverses into `col_inverse`. Returns the (positive) pivot, or 0
/// when the partial row is zero. The pivot is left at column `dest =
/// limit - 1`, matching the `column_echelon` call site (matreduc.h:139).
fn gcd_sweep(
    a: &mut [Vec<i64>],
    row: usize,
    limit: usize,
    col: &mut [Vec<i64>],
    col_inverse: &mut [Vec<i64>],
) -> Result<i64, StructureError> {
    let dest = limit
        .checked_sub(1)
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "echelon pivot column",
        })?;
    // Upstream `gcd` reduces a COPY of the partial row (`row` by value,
    // matreduc.h:70-122); the elementary column operations are recorded in
    // the ops matrix and applied to the working matrix M only afterwards.
    // The oracle build therefore negates ONLY the local pivot entry (the
    // pivot sign is never recorded in ops): M keeps the un-negated pivot
    // column, which fixes the sign of the elected lambda-rho representative
    // (the image-basis identity `lift_mat*M_real == 1-theta` holds either
    // way). The local copy also keeps the Euclidean reductions off the
    // working matrix, which is what terminates the sweep.
    let mut local_row = a[row][..limit].to_vec();
    let mut active: Vec<usize> = Vec::new();
    let mut min = 0_i64;
    let mut mindex = 0_usize;
    for (column, &entry) in local_row.iter().enumerate() {
        if entry != 0 {
            active.push(column);
            let magnitude = entry.abs();
            if min == 0 || magnitude < min {
                min = magnitude;
                mindex = column;
            }
        }
    }
    if active.is_empty() {
        return Ok(0);
    }
    if local_row[mindex] < 0 {
        // Force a positive LOCAL pivot (matreduc.h:93-98): upstream flips
        // the sign of the copied row entry only; the release oracle does
        // not record `col(mindex,mindex) = -1`, so the matrices keep the
        // un-negated pivot column.
        local_row[mindex] = -local_row[mindex];
    }

    while active.len() > 1 {
        let current = mindex;
        let pivot = local_row[current];
        let mut survivors = Vec::new();
        survivors.try_reserve_exact(active.len()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: active.len(),
            }
        })?;
        for &j in &active {
            if j == current {
                survivors.push(j);
                continue;
            }
            let quotient = local_row[j].div_euclid(pivot);
            column_operation(a, col, col_inverse, j, current, -quotient)?;
            local_row[j] = local_row[j]
                .checked_sub(
                    pivot
                        .checked_mul(quotient)
                        .ok_or(StructureError::ArithmeticOverflow)?,
                )
                .ok_or(StructureError::ArithmeticOverflow)?;
            if local_row[j] != 0 {
                survivors.push(j);
                if local_row[j] < min {
                    min = local_row[j];
                    mindex = j;
                }
            }
        }
        active = survivors;
    }

    swap_columns(a, col, col_inverse, dest, mindex);
    Ok(min)
}

/// The representation context of one real form: upstream `Rep_context`
/// (gkmod/repr.h:203-411) restricted to the KType/StandardRepr surface.
///
/// The struct borrows the inner class, the involution table, and the
/// form's KGB graph — the same substrate triple the graph was built
/// against — and derives the root-datum constants (`2rho`, `2rho^v`,
/// `rho`) plus every present involution's [`RealProjection`] once at
/// construction.
pub struct RepContext<'a> {
    inner_class: &'a InnerClass,
    table: &'a InvolutionTable,
    graph: &'a KgbGraph,
    two_rho: Weight,
    dual_two_rho: Coweight,
    rho: RationalWeight,
    projections: BTreeMap<usize, RealProjection>,
}

impl<'a> RepContext<'a> {
    /// Bind the context, deriving the datum constants and the
    /// `(1-theta)` image bases of the graph's involutions. The gate is the
    /// same full inner-class equality as the Tits coset's.
    pub fn new(
        inner_class: &'a InnerClass,
        table: &'a InvolutionTable,
        graph: &'a KgbGraph,
    ) -> Result<Self, StructureError> {
        if table.inner_class() != inner_class {
            return Err(StructureError::DatumMismatch);
        }
        let system = inner_class.root_system();
        let lattice_rank = system.lattice_rank();
        let mut two_rho = try_capacity(lattice_rank)?;
        two_rho.resize(lattice_rank, 0_i32);
        let mut dual_two_rho = try_capacity(lattice_rank)?;
        dual_two_rho.resize(lattice_rank, 0_i32);
        for (id, root, coroot) in system.entries() {
            if !system
                .is_positive(id)
                .ok_or(StructureError::IndexOutOfRange {
                    index: id.0,
                    upper_bound: system.roots().len(),
                })?
            {
                continue;
            }
            for (sum, &coordinate) in two_rho.iter_mut().zip(root.as_slice()) {
                *sum = sum
                    .checked_add(coordinate)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            for (sum, &coordinate) in dual_two_rho.iter_mut().zip(coroot.as_slice()) {
                *sum = sum
                    .checked_add(coordinate)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
        }
        let two_rho = Weight::new(two_rho);
        let rho = RationalWeight::new(
            two_rho.as_slice().iter().map(|&c| i64::from(c)).collect(),
            2,
        )?;
        let mut projections = BTreeMap::new();
        for position in 0..graph.packet_count() {
            let involution =
                graph
                    .packet_involution(position)
                    .ok_or(StructureError::IndexOutOfRange {
                        index: position,
                        upper_bound: graph.packet_count(),
                    })?;
            if projections.contains_key(&involution.0) {
                continue;
            }
            let theta = table
                .record(involution)
                .ok_or(StructureError::IndexOutOfRange {
                    index: involution.0,
                    upper_bound: table.involution_count(),
                })?
                .theta()
                .clone();
            projections.insert(involution.0, RealProjection::build(&theta)?);
        }
        Ok(Self {
            inner_class,
            table,
            graph,
            two_rho: two_rho.clone(),
            dual_two_rho: Coweight::new(dual_two_rho),
            rho,
            projections,
        })
    }

    /// The lattice rank of the underlying root datum (repr.h:215).
    pub fn rank(&self) -> usize {
        self.inner_class.datum().lattice_rank()
    }

    /// The based root datum.
    pub fn datum(&self) -> &crate::BasedRootDatum {
        self.inner_class.datum()
    }

    /// The enumerated root system.
    pub fn root_system(&self) -> &crate::RootSystem {
        self.inner_class.root_system()
    }

    pub fn graph(&self) -> &KgbGraph {
        self.graph
    }

    pub fn table(&self) -> &InvolutionTable {
        self.table
    }

    pub fn inner_class(&self) -> &InnerClass {
        self.inner_class
    }

    /// The real form this context belongs to (the graph's weak form).
    pub fn real_form(&self) -> crate::WeakRealFormId {
        self.graph.form()
    }

    fn projection(&self, involution: InvolutionId) -> Result<&RealProjection, StructureError> {
        self.projections
            .get(&involution.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: self.table.involution_count(),
            })
    }

    pub fn involution_of(&self, x: KgbId) -> Result<InvolutionId, StructureError> {
        self.graph
            .involution_of(x)
            .ok_or(StructureError::IndexOutOfRange {
                index: x.index(),
                upper_bound: self.graph.size(),
            })
    }

    pub(crate) fn theta_at(&self, x: KgbId) -> Result<&LatticeInvolution, StructureError> {
        let involution = self.involution_of(x)?;
        Ok(self
            .table
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: self.table.involution_count(),
            })?
            .theta())
    }

    pub(crate) fn theta_plus_one_rho_at(&self, x: KgbId) -> Result<Weight, StructureError> {
        let involution = self.involution_of(x)?;
        Ok(self
            .table
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: self.table.involution_count(),
            })?
            .theta_plus_one_rho()
            .clone())
    }

    /// Half the sum of the positive roots (rootdata.cpp:1260).
    pub fn rho(&self) -> &RationalWeight {
        &self.rho
    }

    /// The sum of the positive roots (rootdata.h:568).
    pub(crate) fn two_rho(&self) -> &Weight {
        &self.two_rho
    }

    /// The sum of the positive roots in `roots` (rootdata.h:746).
    pub(crate) fn two_rho_of(&self, roots: &[RootId]) -> Result<Weight, StructureError> {
        let system = self.inner_class.root_system();
        let mut sum = vec![0_i32; system.lattice_rank()];
        for &id in roots {
            let root = system.root(id).ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: system.roots().len(),
            })?;
            for (total, &coordinate) in sum.iter_mut().zip(root.as_slice()) {
                *total = total
                    .checked_add(coordinate)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
        }
        Ok(Weight::new(sum))
    }

    /// `coroot(alpha).(twoRho())/2` (rootdata.cpp:389-401, which upstream
    /// implements as the simple-coroot coordinate sum; rootdata.h:225
    /// documents the pairing form used here).
    pub(crate) fn colevel(&self, alpha: RootId) -> Result<i32, StructureError> {
        let system = self.inner_class.root_system();
        let coroot = system
            .coroot(alpha)
            .ok_or(StructureError::IndexOutOfRange {
                index: alpha.0,
                upper_bound: system.roots().len(),
            })?;
        let pairing = pair(&self.two_rho, coroot)?;
        if pairing % 2 != 0 {
            return Err(StructureError::RepInvariantViolation {
                invariant: "coroot level parity",
            });
        }
        Ok(pairing / 2)
    }

    /// `lambda_unique` (involutions.cpp:322-332): the elected coset
    /// representative of `lam_rho` modulo `(1-theta)X^*`, subtracting
    /// `lift_mat` times the Euclidean halves of its `M_real` coordinates.
    pub fn lambda_unique(
        &self,
        involution: InvolutionId,
        lam_rho: &Weight,
    ) -> Result<Weight, StructureError> {
        if lam_rho.rank() != self.rank() {
            return Err(StructureError::RankMismatch {
                expected: self.rank(),
                actual: lam_rho.rank(),
            });
        }
        let projection = self.projection(involution)?;
        let coordinates = projection.coordinates(lam_rho)?;
        let halves: Vec<i64> = coordinates.iter().map(|&v| v.div_euclid(2)).collect();
        let correction = projection.lift(&halves)?;
        let mut normalized = Vec::new();
        normalized.try_reserve_exact(lam_rho.rank()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: lam_rho.rank(),
            }
        })?;
        for (&coordinate, &shift) in lam_rho.as_slice().iter().zip(&correction) {
            let shifted = i64::from(coordinate)
                .checked_sub(shift)
                .ok_or(StructureError::ArithmeticOverflow)?;
            normalized
                .push(i32::try_from(shifted).map_err(|_| StructureError::ArithmeticOverflow)?);
        }
        Ok(Weight::new(normalized))
    }

    /// `y_pack` (involutions.h:211-213): the `M_real` coordinates of
    /// `lambda_rho` reduced mod 2, as a [`ModTwoVector`] over the image
    /// basis (upstream `TorusPart`).
    pub fn y_pack(
        &self,
        involution: InvolutionId,
        lam_rho: &Weight,
    ) -> Result<ModTwoVector, StructureError> {
        if lam_rho.rank() != self.rank() {
            return Err(StructureError::RankMismatch {
                expected: self.rank(),
                actual: lam_rho.rank(),
            });
        }
        let projection = self.projection(involution)?;
        let coordinates = projection.coordinates(lam_rho)?;
        let ones: Vec<usize> = coordinates
            .iter()
            .enumerate()
            .filter_map(|(index, &v)| (v.rem_euclid(2) != 0).then_some(index))
            .collect();
        ModTwoVector::from_ones(coordinates.len(), ones)
    }

    /// `y_lift` (involutions.cpp:346-356): `(1-theta)*lam_rho` for a
    /// `lam_rho` with the given packed torsion part.
    pub fn y_lift(
        &self,
        involution: InvolutionId,
        y_bits: &ModTwoVector,
    ) -> Result<Weight, StructureError> {
        let projection = self.projection(involution)?;
        if y_bits.dimension() != projection.m_real.len() {
            return Err(StructureError::RankMismatch {
                expected: projection.m_real.len(),
                actual: y_bits.dimension(),
            });
        }
        let mut coordinates = Vec::new();
        coordinates
            .try_reserve_exact(y_bits.dimension())
            .map_err(|_| StructureError::AllocationFailed {
                requested: y_bits.dimension(),
            })?;
        for index in 0..y_bits.dimension() {
            let bit = y_bits.bit(index).ok_or(StructureError::IndexOutOfRange {
                index,
                upper_bound: y_bits.dimension(),
            })?;
            coordinates.push(i64::from(bit));
        }
        let lifted = projection.lift(&coordinates)?;
        let mut weight = Vec::new();
        weight
            .try_reserve_exact(lifted.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: lifted.len(),
            })?;
        for entry in lifted {
            weight.push(i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?);
        }
        Ok(Weight::new(weight))
    }

    /// `height` (repr.cpp:160-166): `<2rho^v, w>/2` with `w` made
    /// dominant, where `w = (1+theta)*gamma` (an integral weight).
    pub fn height(&self, theta_plus_1_gamma: &Weight) -> Result<u32, StructureError> {
        let dominant = self.make_dominant_weight(theta_plus_1_gamma)?;
        let pairing = pair(&dominant, &self.dual_two_rho)?;
        if pairing < 0 || pairing % 2 != 0 {
            return Err(StructureError::RepInvariantViolation {
                invariant: "height parity",
            });
        }
        u32::try_from(pairing / 2).map_err(|_| StructureError::ArithmeticOverflow)
    }

    /// `RootDatum::make_dominant` (rootdata.h:638-640): reflect at simple
    /// roots with negative pairing until dominant. The termination bound
    /// is exact: `D(w) = sum over positive coroots of max(0, -<w, beta>)`
    /// strictly decreases at every step.
    pub(crate) fn make_dominant_weight(&self, weight: &Weight) -> Result<Weight, StructureError> {
        let datum = self.inner_class.datum();
        let system = self.inner_class.root_system();
        let mut defect = 0_i64;
        for (id, _, coroot) in system.entries() {
            if system
                .is_positive(id)
                .ok_or(StructureError::IndexOutOfRange {
                    index: id.0,
                    upper_bound: system.roots().len(),
                })?
            {
                let pairing = pair(weight, coroot)?;
                if pairing < 0 {
                    defect += i64::from(-pairing);
                }
            }
        }
        let mut result = weight.clone();
        loop {
            let mut reflected = false;
            for (generator, coroot) in datum.simple_coroots().iter().enumerate() {
                if pair(&result, coroot)? < 0 {
                    result = datum.reflect_weight(generator, &result)?;
                    defect -= 1;
                    reflected = true;
                    break;
                }
            }
            if !reflected {
                return Ok(result);
            }
            if defect < 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "dominance termination",
                });
            }
        }
    }

    /// `Rep_context::gamma` (repr.cpp:168-177): the representative
    /// infinitesimal character `(lambda + nu + theta(lambda - nu))/2`
    /// with `lambda = rho + lambda_rho`.
    pub fn gamma(
        &self,
        x: KgbId,
        lambda_rho: &Weight,
        nu: &RationalWeight,
    ) -> Result<RationalWeight, StructureError> {
        if lambda_rho.rank() != self.rank() || nu.rank() != self.rank() {
            return Err(StructureError::RankMismatch {
                expected: self.rank(),
                actual: lambda_rho.rank(),
            });
        }
        let theta = self.theta_at(x)?;
        let lambda = self.rho.add(&RationalWeight::from_weight(lambda_rho)?)?;
        let difference = lambda.sub(nu)?;
        let theta_difference = difference.apply_matrix(theta.weight_matrix())?;
        lambda
            .add(nu)?
            .add(&theta_difference)?
            .halve()?
            .normalized()
    }

    /// `Rep_context::lambda_rho` (repr.cpp:182-204): recover `lambda-rho`
    /// from the doubled `(1+theta)` projection of `gamma - rho` and the
    /// packed torsion part: `((1+theta)(gamma-rho) + y_lift(y))/2`, both
    /// divisions exact.
    pub fn lambda_rho(&self, z: &StandardRepr) -> Result<Weight, StructureError> {
        let involution = self.involution_of(z.x)?;
        let theta = self.theta_at(z.x)?;
        let gamma_minus_rho = z.gamma.sub(&self.rho)?;
        let theta_image = gamma_minus_rho.apply_matrix(theta.weight_matrix())?;
        let doubled = gamma_minus_rho.add(&theta_image)?;
        let projection_weight = doubled.integral_coordinates()?;
        let y_lift = self.y_lift(involution, &z.y_bits)?;
        let mut sum = Vec::new();
        sum.try_reserve_exact(self.rank())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.rank(),
            })?;
        for (&projected, &lifted) in projection_weight.iter().zip(y_lift.as_slice()) {
            let total = projected
                .checked_add(i64::from(lifted))
                .ok_or(StructureError::ArithmeticOverflow)?;
            if total % 2 != 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "lambda-rho halving",
                });
            }
            sum.push(i32::try_from(total / 2).map_err(|_| StructureError::ArithmeticOverflow)?);
        }
        Ok(Weight::new(sum))
    }

    /// `Rep_context::lambda` (repr.h:304): `rho + lambda_rho`, the
    /// half-integral weight the interpreter prints as `lambda=[..]/d`
    /// (basic_io.cpp print_stdrep/print_K_type).
    pub fn lambda(&self, z: &StandardRepr) -> Result<RationalWeight, StructureError> {
        self.rho
            .add(&RationalWeight::from_weight(&self.lambda_rho(z)?)?)
    }

    /// `rho + lambda_rho` of a [`KType`] (basic_io.cpp:158-163).
    pub fn lambda_of_ktype(&self, t: &KType) -> Result<RationalWeight, StructureError> {
        self.rho.add(&RationalWeight::from_weight(t.lambda_rho())?)
    }

    /// `Rep_context::nu` (repr.cpp:239-245): `(gamma - theta*gamma)/2`,
    /// the `-theta`-fixed projection printed as `nu=[..]/d`.
    pub fn nu(&self, z: &StandardRepr) -> Result<RationalWeight, StructureError> {
        let theta = self.theta_at(z.x)?;
        let theta_gamma = z.gamma.apply_matrix(theta.weight_matrix())?;
        z.gamma.sub(&theta_gamma)?.halve()?.normalized()
    }

    /// `Rep_context::sr_gamma` (repr.cpp:756-784): pack the torsion part
    /// of `lambda_rho` and store `gamma` with the height of
    /// `(1+theta)*gamma`.
    /// `Rep_context::is_parity` (repr.cpp:247-270): whether the real
    /// generator `s` at `x` is a parity descent for the parameter with
    /// `lambda_rho` and `gamma`.
    pub fn is_parity(
        &self,
        s: usize,
        x: KgbId,
        lambda_rho: &Weight,
        gamma: &RationalWeight,
    ) -> Result<bool, StructureError> {
        let involution = self.involution_of(x)?;
        let y_bits = self.y_pack(involution, lambda_rho)?;
        let theta_1_lamrho = self.y_lift(involution, &y_bits)?;
        // Non-real positive roots at x (repr.cpp:249 complement).
        let real = self.positive_real_roots_at(x)?;
        let system = self.inner_class.root_system();
        let mut non_real = Vec::new();
        for index in 0..system.roots().len() {
            let id = RootId::from_usize(index);
            if system.is_positive(id) == Some(true) && !real.contains(&id) {
                non_real.push(id);
            }
        }
        let two_rho_non_real = self.two_rho_of(&non_real)?;
        let mut sum_coordinates = Vec::new();
        for (left, &right) in theta_1_lamrho
            .as_slice()
            .iter()
            .zip(two_rho_non_real.as_slice())
        {
            sum_coordinates.push(left + right);
        }
        let sum = Weight::new(sum_coordinates);
        let coroot =
            system
                .coroot(RootId::from_usize(s))
                .ok_or(StructureError::IndexOutOfRange {
                    index: s,
                    upper_bound: system.roots().len(),
                })?;
        let eval = pair(&sum, coroot)?;
        let parity_at_0 = eval % 4 != 0;
        // eval2 = <gamma, coroot(s)>; integral because s is real.
        let numerator = gamma.numerator();
        let denominator = gamma.denominator();
        let mut eval2_numerator: i64 = 0;
        for (index, &coordinate) in coroot.as_slice().iter().enumerate() {
            let entry = numerator.get(index).ok_or(StructureError::RankMismatch {
                expected: gamma.rank(),
                actual: coroot.as_slice().len(),
            })?;
            eval2_numerator += i64::from(coordinate) * *entry;
        }
        let eval2 = if denominator == 0 {
            return Err(StructureError::RankMismatch {
                expected: 1,
                actual: 0,
            });
        } else {
            eval2_numerator / denominator
        };
        Ok(parity_at_0 == (eval2 % 2 == 0))
    }

    /// The packed torsion part (dimension = the involution's `(1-theta)`
    /// image rank) of a KGB element's Tits torus: `M_real * (torus mod 2)`
    /// (involutions.h:211 `y_pack`), used by the common-block matching.
    pub fn torus_part(
        &self,
        x: KgbId,
        kgb_torus: &ModTwoVector,
    ) -> Result<ModTwoVector, StructureError> {
        let involution = self.involution_of(x)?;
        let projection = self.projection(involution)?;
        let mut ones = Vec::new();
        for (row_index, row) in projection.m_real.iter().enumerate() {
            let mut parity = 0_i64;
            for (column, &entry) in row.iter().enumerate() {
                if entry % 2 != 0 && kgb_torus.bit(column) == Some(true) {
                    parity ^= 1;
                }
            }
            if parity != 0 {
                ones.push(row_index);
            }
        }
        ModTwoVector::from_ones(projection.m_real.len(), ones)
    }

    /// `InvolutionTable::real_unique` (involutions.cpp:334-342): the
    /// unique representative modulo the cocharacter lattice of a
    /// rational weight under `1-theta`: project to the `(1-theta)` image
    /// basis, reduce coordinates modulo `2*denominator`, lift back and
    /// halve. This is what `StandardReprMod::build/mod_reduce` apply to
    /// gamma-lambda.
    pub fn real_unique(
        &self,
        involution: InvolutionId,
        y: &mut RationalWeight,
    ) -> Result<(), StructureError> {
        let projection = self.projection(involution)?;
        let denominator = y.denominator();
        let doubled_denominator = 2_i64
            .checked_mul(denominator)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let mut v = Vec::new();
        v.try_reserve_exact(projection.m_real.len()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: projection.m_real.len(),
            }
        })?;
        for row in &projection.m_real {
            let mut total = 0_i64;
            for (column, &entry) in row.iter().enumerate() {
                total = total
                    .checked_add(
                        entry
                            .checked_mul(y.numerator().get(column).copied().unwrap_or(0))
                            .ok_or(StructureError::ArithmeticOverflow)?,
                    )
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            v.push(total.rem_euclid(doubled_denominator));
        }
        let lifted = projection.lift(&v)?;
        let normalized = RationalWeight::new(lifted, doubled_denominator)?;
        *y = normalized.normalized()?;
        Ok(())
    }

    /// `Rep_context::gamma_lambda` (repr.cpp:220-227): the lambda-shifted
    /// infinitesimal character (gamma - lambda) of a parameter with the
    /// given torsion part, used to match common-block elements.
    pub fn gamma_lambda(
        &self,
        x: KgbId,
        y_bits: &ModTwoVector,
        gamma: &RationalWeight,
    ) -> Result<RationalWeight, StructureError> {
        let theta = self.theta_at(x)?;
        let gamma_rho = gamma.sub(self.rho())?;
        let theta_gamma_rho = gamma_rho.apply_matrix(theta.weight_matrix())?;
        let involution = self.involution_of(x)?;
        let y_lift = self.y_lift(involution, y_bits)?;
        let difference = gamma_rho.sub(&theta_gamma_rho)?;
        let difference = difference.sub(&RationalWeight::from_weight(&y_lift)?)?;
        difference.halve()?.normalized()
    }

    /// `Rep_context::reducibility_points` (repr.cpp:825-925): the
    /// reducibility fractions of a standard parameter, as (numerator,
    /// denominator) pairs sorted ascending.
    pub fn reducibility_points(&self, z: &StandardRepr) -> Result<Vec<(i64, i64)>, StructureError> {
        let system = self.inner_class.root_system();
        let numer = z.gamma().numerator();
        let d = z.gamma().denominator();
        let lam_rho = self.lambda_rho(z)?;
        let x = z.x();
        let pos_real = self.positive_real_roots_at(x)?;
        let two_rho_real = self.two_rho_of(&pos_real)?;
        let involution = self.involution_of(x)?;
        let record = self
            .table
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: self.table.involution_count(),
            })?;
        let root_involution = record.twisted_involution().root_involution();
        // (abs(num) -> strict lower bound lwb), split by parity of k.
        let mut odds: std::collections::BTreeMap<i64, i64> = std::collections::BTreeMap::new();
        let mut evens: std::collections::BTreeMap<i64, i64> = std::collections::BTreeMap::new();
        for &alpha in &pos_real {
            let coroot = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.index(),
                    upper_bound: system.roots().len(),
                })?;
            let num = dot_numerator(coroot, numer);
            if num != 0 {
                let lam_alpha =
                    i64::from(pair(&lam_rho, coroot)?) + i64::from(self.colevel(alpha)?);
                let two_rho_dot = i64::from(pair(&two_rho_real, coroot)?);
                let do_odd = (lam_alpha + two_rho_dot / 2) % 2 == 0;
                (if do_odd { &mut odds } else { &mut evens })
                    .entry(num.unsigned_abs() as i64)
                    .or_insert(0);
            }
        }
        // Complex positive roots: beta = theta(alpha).
        let mut pos_complex = Vec::new();
        for index in 0..system.roots().len() {
            let id = RootId::from_usize(index);
            if system.is_positive(id) == Some(true)
                && root_involution.kind(id) == Some(RootKind::Complex)
            {
                pos_complex.push(id);
            }
        }
        for &alpha in &pos_complex {
            let beta = root_involution
                .image(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.index(),
                    upper_bound: system.roots().len(),
                })?;
            let coroot_alpha = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.index(),
                    upper_bound: system.roots().len(),
                })?;
            let coroot_beta = system.coroot(beta).ok_or(StructureError::IndexOutOfRange {
                index: beta.index(),
                upper_bound: system.roots().len(),
            })?;
            let vala = dot_numerator(coroot_alpha, numer);
            let valb = dot_numerator(coroot_beta, numer);
            let num = vala - valb;
            if num != 0 {
                let lwb = (vala + valb).unsigned_abs() / d as u64;
                let table = if lwb.is_multiple_of(2) {
                    &mut evens
                } else {
                    &mut odds
                };
                let entry = table.entry(num.unsigned_abs() as i64).or_insert(0);
                *entry = (*entry).min(lwb as i64);
            }
        }
        let mut fracs: std::collections::BTreeSet<(i64, i64)> = std::collections::BTreeSet::new();
        for (num, lwb) in &evens {
            let mut s = d * (lwb + 2);
            while s <= *num {
                fracs.insert((s, *num));
                s += 2 * d;
            }
        }
        for (num, lwb) in &odds {
            let mut s = if *lwb == 0 { d } else { d * (lwb + 2) };
            while s <= *num {
                fracs.insert((s, *num));
                s += 2 * d;
            }
        }
        Ok(fracs.into_iter().collect())
    }

    /// `Rep_context::finals_for` (repr.cpp:1205-1297): decompose a
    /// (not necessarily final) standard parameter into its final
    /// constituents with integral coefficients, by descending through
    /// the non-dominant generators (reflections, Cayley and inverse
    /// Cayley transforms) and recording the resulting final parameters.
    pub fn finals_for(&self, z: &StandardRepr) -> Result<Vec<(StandardRepr, i32)>, StructureError> {
        let rd = self.inner_class.datum();
        let mut result: Vec<(StandardRepr, i32)> = Vec::new();
        let mut to_do: Vec<(StandardRepr, i32)> = vec![(z.clone(), 1)];
        while let Some((current, mut coef)) = to_do.pop() {
            let mut x = current.x();
            let mut lr = self.lambda_rho(&current)?;
            let mut gamma_num = current.gamma().numerator().to_vec();
            let denominator = current.gamma().denominator();
            let mut drop = false;
            'restart: loop {
                for s in 0..rd.semisimple_rank() {
                    let coroot = &rd.simple_coroots()[s];
                    let mut eval: i64 = 0;
                    for (index, &coordinate) in coroot.as_slice().iter().enumerate() {
                        if coordinate != 0 {
                            let entry = gamma_num.get(index).ok_or_else(|| {
                                StructureError::RankMismatch {
                                    expected: gamma_num.len(),
                                    actual: coroot.as_slice().len(),
                                }
                            })?;
                            eval += i64::from(coordinate) * *entry;
                        }
                    }
                    if eval > 0 {
                        continue;
                    }
                    match self.kgb_status(x, s)? {
                        KgbStatus::ImaginaryCompact => {
                            if eval == 0 {
                                drop = true;
                                break 'restart;
                            }
                            self.simple_reflect(s, &mut lr, 1)?;
                            self.simple_reflect_numerator(s, &mut gamma_num)?;
                            coef = -coef;
                            continue 'restart;
                        }
                        KgbStatus::ImaginaryNoncompact => {
                            if eval == 0 {
                                continue;
                            }
                            let sx =
                                self.graph
                                    .cross(x, s)
                                    .ok_or(StructureError::IndexOutOfRange {
                                        index: x.index(),
                                        upper_bound: self.graph.size(),
                                    })?;
                            let cx = self.graph.cayley(x, s)?.ok_or(
                                StructureError::RepInvariantViolation {
                                    invariant: "noncompact imaginary Cayley image",
                                },
                            )?;
                            let gamma = RationalWeight::new(gamma_num.clone(), denominator)?;
                            let t1 = self.sr_gamma(cx, &lr, &gamma)?;
                            insert_into_by_height(&mut to_do, t1, coef)?;
                            if sx == x {
                                let mut t2_coordinates = Vec::new();
                                for (&entry, &root_entry) in
                                    lr.as_slice().iter().zip(rd.simple_roots()[s].as_slice())
                                {
                                    t2_coordinates.push(entry + root_entry);
                                }
                                let t2_lr = Weight::new(t2_coordinates);
                                let t2 = self.sr_gamma(cx, &t2_lr, &gamma)?;
                                insert_into_by_height(&mut to_do, t2, coef)?;
                            }
                            x = sx;
                            self.simple_reflect(s, &mut lr, 1)?;
                            self.simple_reflect_numerator(s, &mut gamma_num)?;
                            coef = -coef;
                            continue 'restart;
                        }
                        KgbStatus::Complex => {
                            if eval == 0 && !self.is_complex_descent(x, s)? {
                                continue;
                            }
                            x = self
                                .graph
                                .cross(x, s)
                                .ok_or(StructureError::IndexOutOfRange {
                                    index: x.index(),
                                    upper_bound: self.graph.size(),
                                })?;
                            self.simple_reflect(s, &mut lr, 1)?;
                            self.simple_reflect_numerator(s, &mut gamma_num)?;
                            continue 'restart;
                        }
                        KgbStatus::Real => {
                            if eval == 0 {
                                let mut eval_lr: i32 = pair(&lr, coroot)?;
                                if eval_lr % 2 == 0 {
                                    continue;
                                }
                                let shift = ((eval_lr + 1) / 2) as i64;
                                let mut lr_coordinates = lr.as_slice().to_vec();
                                for (entry, &root_entry) in lr_coordinates
                                    .iter_mut()
                                    .zip(rd.simple_roots()[s].as_slice())
                                {
                                    *entry -= (shift * i64::from(root_entry)) as i32;
                                }
                                lr = Weight::new(lr_coordinates);
                                eval_lr = pair(&lr, coroot)?;
                                let _ = eval_lr;
                                let gamma = RationalWeight::new(gamma_num, denominator)?;
                                let pair_image = self.graph.inverse_cayley(x, s)?;
                                let (first, second) = match pair_image {
                                    Some((first, Some(second))) => (Some(first), Some(second)),
                                    Some((first, None)) => (Some(first), None),
                                    None => (None, None),
                                };
                                if let Some(second) = second {
                                    insert_into_by_height(
                                        &mut to_do,
                                        self.sr_gamma(second, &lr, &gamma)?,
                                        coef,
                                    )?;
                                }
                                if let Some(first) = first {
                                    insert_into_by_height(
                                        &mut to_do,
                                        self.sr_gamma(first, &lr, &gamma)?,
                                        coef,
                                    )?;
                                }
                                drop = true;
                                break 'restart;
                            }
                            self.simple_reflect(s, &mut lr, 0)?;
                            self.simple_reflect_numerator(s, &mut gamma_num)?;
                            continue 'restart;
                        }
                    }
                }
                // No generator handled: the current parameter is final.
                let gamma = RationalWeight::new(gamma_num, denominator)?;
                let final_sr = self.sr_gamma(x, &lr, &gamma)?;
                result.push((final_sr, coef));
                break;
            }
            if drop {
                // Contribute nothing for this branch.
            }
        }
        // result was built LIFO; upstream prepends at the front, so reverse.
        result.reverse();
        Ok(result)
    }

    pub fn sr_gamma(
        &self,
        x: KgbId,
        lambda_rho: &Weight,
        gamma: &RationalWeight,
    ) -> Result<StandardRepr, StructureError> {
        if lambda_rho.rank() != self.rank() || gamma.rank() != self.rank() {
            return Err(StructureError::RankMismatch {
                expected: self.rank(),
                actual: lambda_rho.rank(),
            });
        }
        let involution = self.involution_of(x)?;
        let theta = self.theta_at(x)?;
        let theta_gamma = gamma.apply_matrix(theta.weight_matrix())?;
        let doubled = gamma.add(&theta_gamma)?;
        let projection = doubled.integral_coordinates()?;
        let mut th1_gamma = Vec::new();
        th1_gamma
            .try_reserve_exact(self.rank())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.rank(),
            })?;
        for entry in projection {
            th1_gamma.push(i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?);
        }
        let y_bits = self.y_pack(involution, lambda_rho)?;
        let height = self.height(&Weight::new(th1_gamma))?;
        Ok(StandardRepr {
            x,
            y_bits,
            gamma: gamma.clone(),
            height,
        })
    }
    /// `Rep_table::deformation_terms` (repr.cpp:1933-2025), simplified for
    /// the case the frozen `domain/deform` contract exercises: identity
    /// block modifier, empty singular system (a regular dominant gamma),
    /// and a constant `lambda_rho` across the block. Returns the
    /// `(StandardRepr, integer coefficient)` deformation terms of the
    /// final element `y` at `gamma`.
    ///
    /// The full algorithm walks the finals in reverse, maintaining the
    /// remainder/accumulator vectors, and evaluates every KL polynomial
    /// in the column at `q = -1` (alternating the sign when the length
    /// difference is odd), exactly like repr.cpp:1947-1988.
    pub fn deformation_terms(
        &self,
        block: &BlockGraph,
        y: usize,
        gamma: &RationalWeight,
        lambda_rho: &Weight,
        kl_table: &crate::KlTable<'_>,
    ) -> Result<Vec<(StandardRepr, i32)>, StructureError> {
        if block.length(y) == Some(0) {
            return Ok(Vec::new()); // easy case, null result (repr.cpp:1941)
        }

        // With an empty singular system every element is final; the
        // reverse-accumulated list is [y, y-1, ..., 0] (repr.cpp:1944-1947).
        let finals: Vec<usize> = (0..=y).rev().collect();
        let mut index = vec![0_usize; y + 1];
        for (position, &z) in finals.iter().enumerate() {
            index[z] = position;
        }

        let mut acc = vec![0_i32; finals.len()];
        let mut remainder = vec![0_i32; finals.len()];
        remainder[0] = 1; // we initialised remainder = 1*sr_y
        let y_parity = block.length(y).expect("valid element") % 2;

        for (position, &z) in finals.iter().enumerate() {
            let c_cur = remainder[position];
            if c_cur == 0 {
                continue;
            }
            let contribute = block.length(z).expect("valid element") % 2 != y_parity;
            // for x from z down to 0 inclusive (repr.cpp:1970-1988)
            for x in (0..=z).rev() {
                let index_kl = kl_table.kl_pol(x, z)?;
                let pol = kl_table
                    .pool()
                    .get(index_kl)
                    .cloned()
                    .unwrap_or_else(crate::KlPol::zero);
                let mut eval = pol.evaluate_at_minus_one();
                if eval == 0 {
                    continue; // polynomials with -1 as a root do not contribute
                }
                if !(block.length(z).expect("valid element")
                    - block.length(x).expect("valid element"))
                .is_multiple_of(2)
                {
                    eval = -eval; // alternating sum of the KL column at -1
                }
                let j = index[x];
                let c = c_cur * eval;
                remainder[j] -= c;
                if contribute {
                    acc[j] += c;
                }
            }
        }

        // Orientation-number differences: A2 (no compact Cartan, no real
        // roots) has orientation 0 everywhere, so the Split-value scaling
        // of repr.cpp:1996-2017 is an integer coefficient.
        let mut result = Vec::new();
        for (position, &z) in finals.iter().enumerate() {
            let c = acc[position];
            if c != 0 {
                let sr = self.sr_gamma(block.x(z).expect("valid element"), lambda_rho, gamma)?;
                result.push((sr, c));
            }
        }
        Ok(result)
    }

    /// `Rep_context::sr` (repr.h:242-244): the parameter of the
    /// `(x, lambda-rho, nu)` triplet.
    pub fn sr(
        &self,
        x: KgbId,
        lambda_rho: &Weight,
        nu: &RationalWeight,
    ) -> Result<StandardRepr, StructureError> {
        let gamma = self.gamma(x, lambda_rho, nu)?;
        self.sr_gamma(x, lambda_rho, &gamma)
    }

    /// `Rep_context::sr(const K_type&)` (repr.h:252-253): extend a
    /// K-type with `nu = 0`.
    pub fn sr_of_ktype(&self, t: &KType) -> Result<StandardRepr, StructureError> {
        let zero = RationalWeight::zero(self.rank())?;
        self.sr(t.x(), t.lambda_rho(), &zero)
    }

    /// `Rep_context::sr_K(const StandardRepr&)` (repr.h:232-233): restrict
    /// a parameter to K, carrying the elected `lambda-rho` and the height.
    pub fn sr_k_of_standard(&self, z: &StandardRepr) -> Result<KType, StructureError> {
        Ok(KType::new(z.x, self.lambda_rho(z)?, z.height))
    }

    /// `Rep_context::finals_for(StandardRepr)` (repr.cpp:1205-1297): the
    /// final-parameter expansion — make the infinitesimal character
    /// dominant through crosses and Cayley transforms, drop singular
    /// compact factors, and split along parity real roots. Returns the
    /// signed multiplicity list of final parameters (unordered; the
    /// language layer merges like terms in the polynomial's canonical
    /// order).
    pub fn finals_for_standard(
        &self,
        z: &StandardRepr,
    ) -> Result<Vec<(StandardRepr, i32)>, StructureError> {
        let datum = self.inner_class.datum();
        let mut result = Vec::new();
        let mut todo = vec![(z.clone(), 1_i32)];
        while let Some((repr, mut coef)) = todo.pop() {
            let mut x = repr.x();
            let mut lr = self.lambda_rho(&repr)?;
            let mut gamma_numerator = repr.gamma().numerator().to_vec();
            let gamma_denominator = repr.gamma().denominator();
            let mut dropped = false;
            'restart: loop {
                for s in 0..datum.semisimple_rank() {
                    // The sign of `<alpha_s^v, gamma>`; upstream evaluates
                    // the coroot against the raw numerator
                    // (repr.cpp:1224).
                    let eval = pair_i64(&gamma_numerator, &datum.simple_coroots()[s])?;
                    if eval > 0 {
                        continue;
                    }
                    match self.kgb_status(x, s)? {
                        KgbStatus::ImaginaryCompact => {
                            if eval == 0 {
                                dropped = true;
                                break 'restart;
                            }
                            self.simple_reflect(s, &mut lr, 1)?;
                            self.simple_reflect_numerator(s, &mut gamma_numerator)?;
                            coef = -coef;
                            continue 'restart;
                        }
                        KgbStatus::ImaginaryNoncompact => {
                            if eval == 0 {
                                continue;
                            }
                            let sx = self.cross_at(x, s)?;
                            let cx = self.graph().cayley(x, s)?.ok_or(
                                StructureError::RepInvariantViolation {
                                    invariant: "noncompact imaginary Cayley",
                                },
                            )?;
                            let gamma =
                                RationalWeight::new(gamma_numerator.clone(), gamma_denominator)?;
                            todo.push((self.sr_gamma(cx, &lr, &gamma)?, coef));
                            if sx == x {
                                let shifted = checked_add_weights(&lr, &datum.simple_roots()[s])?;
                                todo.push((self.sr_gamma(cx, &shifted, &gamma)?, coef));
                            }
                            x = sx;
                            self.simple_reflect(s, &mut lr, 1)?;
                            self.simple_reflect_numerator(s, &mut gamma_numerator)?;
                            coef = -coef;
                            continue 'restart;
                        }
                        KgbStatus::Complex => {
                            if eval == 0 && !self.is_complex_descent(x, s)? {
                                continue;
                            }
                            x = self.cross_at(x, s)?;
                            self.simple_reflect(s, &mut lr, 1)?;
                            self.simple_reflect_numerator(s, &mut gamma_numerator)?;
                            continue 'restart;
                        }
                        KgbStatus::Real => {
                            let eval_lr = pair(&lr, &datum.simple_coroots()[s])?;
                            if eval_lr % 2 == 0 {
                                continue;
                            }
                            let shift = (eval_lr + 1) / 2;
                            let mut projected = Vec::new();
                            projected.try_reserve_exact(lr.rank()).map_err(|_| {
                                StructureError::AllocationFailed {
                                    requested: lr.rank(),
                                }
                            })?;
                            for (&entry, &root_entry) in
                                lr.as_slice().iter().zip(datum.simple_roots()[s].as_slice())
                            {
                                projected.push(
                                    entry
                                        .checked_sub(
                                            root_entry
                                                .checked_mul(shift)
                                                .ok_or(StructureError::ArithmeticOverflow)?,
                                        )
                                        .ok_or(StructureError::ArithmeticOverflow)?,
                                );
                            }
                            lr = Weight::new(projected);
                            let gamma =
                                RationalWeight::new(gamma_numerator.clone(), gamma_denominator)?;
                            let Some((first, second)) = self.graph().inverse_cayley(x, s)? else {
                                return Err(StructureError::RepInvariantViolation {
                                    invariant: "parity real inverse Cayley",
                                });
                            };
                            if let Some(second) = second {
                                todo.push((self.sr_gamma(second, &lr, &gamma)?, coef));
                            }
                            todo.push((self.sr_gamma(first, &lr, &gamma)?, coef));
                            dropped = true;
                            break 'restart;
                        }
                    }
                }
                break;
            }
            if dropped {
                continue;
            }
            // The loop terminated with a dominant gamma: contribute the
            // fresh, now-final parameter (repr.cpp:1287-1290).
            let gamma = RationalWeight::new(gamma_numerator, gamma_denominator)?;
            result.push((self.sr_gamma(x, &lr, &gamma)?, coef));
        }
        Ok(result)
    }

    /// `Rep_context::expand_final(StandardRepr)` (repr.cpp:1299-1309):
    /// the ParamPol term list of a parameter's final expansion, one
    /// integer multiplicity per final parameter (Split coefficients are
    /// assembled at the language layer).
    pub fn expand_final(
        &self,
        z: &StandardRepr,
    ) -> Result<Vec<(StandardRepr, i32)>, StructureError> {
        self.finals_for_standard(z)
    }

    /// `Rep_context::scale(StandardRepr, f)` (repr.cpp:701-709): scale
    /// the infinitesimal character by the rational `f` along its
    /// `nu = (gamma - theta*gamma)/2` direction: the new gamma is
    /// `(gamma + theta*gamma + 2*nu*f)/2`, with the x and torsion parts
    /// unchanged (and the stored height carried, as upstream does).
    pub fn scale(
        &self,
        z: &StandardRepr,
        numerator: i64,
        denominator: i64,
    ) -> Result<StandardRepr, StructureError> {
        let theta = self.theta_at(z.x)?;
        let image = z.gamma.apply_matrix(theta.weight_matrix())?;
        let difference = z.gamma.sub(&image)?; // 2*nu(z)
        let mut scaled = z
            .gamma
            .add(&image)?
            .add(&difference.scale(numerator, denominator)?)?;
        scaled = scaled.halve()?.normalized()?;
        Ok(StandardRepr {
            x: z.x,
            y_bits: z.y_bits.clone(),
            gamma: scaled,
            height: z.height,
        })
    }

    /// `status(kgb, x, alpha)` for an arbitrary (positive) root
    /// (kgb.cpp:819-830): conjugate the root to a simple one by
    /// reflecting along its descents, crossing the KGB element in
    /// parallel, and return the simple status.
    pub(crate) fn root_status_at(
        &self,
        x: KgbId,
        root: RootId,
    ) -> Result<KgbStatus, StructureError> {
        let datum = self.inner_class().datum();
        let system = self.inner_class().root_system();
        let mut x = x;
        let mut alpha = system
            .root(root)
            .ok_or(StructureError::IndexOutOfRange {
                index: root.0,
                upper_bound: system.roots().len(),
            })?
            .clone();
        debug_assert_eq!(
            system.is_positive(root),
            Some(true),
            "positive root expected"
        );
        let simple_roots = datum.simple_roots();
        let simple_coroots = datum.simple_coroots();
        let simple_ids = system.simple_root_ids();
        loop {
            // find_descent: the first simple generator whose coroot
            // pairing is positive.
            let mut descent = None;
            for (generator, &simple) in simple_ids.iter().enumerate() {
                if pair(&alpha, &simple_coroots[generator])? > 0 {
                    descent = Some(generator);
                    let _ = simple;
                    break;
                }
            }
            let Some(generator) = descent else {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "positive root descent",
                });
            };
            if alpha == simple_roots[generator] {
                return self.kgb_status(x, generator);
            }
            let pairing = pair(&alpha, &simple_coroots[generator])?;
            let mut reflected = Vec::new();
            reflected.try_reserve_exact(alpha.rank()).map_err(|_| {
                StructureError::AllocationFailed {
                    requested: alpha.rank(),
                }
            })?;
            for (&entry, &root_entry) in alpha
                .as_slice()
                .iter()
                .zip(simple_roots[generator].as_slice())
            {
                reflected.push(
                    entry
                        .checked_sub(
                            pairing
                                .checked_mul(root_entry)
                                .ok_or(StructureError::ArithmeticOverflow)?,
                        )
                        .ok_or(StructureError::ArithmeticOverflow)?,
                );
            }
            alpha = Weight::new(reflected);
            x = self.cross_at(x, generator)?;
        }
    }

    /// `Rep_context::height_bound` (K_repr.cpp:511-545): `ceil(height)` of
    /// the orthogonal projection of a rational weight onto the dominant
    /// cone, computed by the incremental projector construction of the
    /// upstream (each projector is the simple root orthogonalized against
    /// the previous ones, scaled to unit coroot pairing).
    pub fn height_bound(&self, lambda: &RationalWeight) -> Result<u32, StructureError> {
        let datum = self.inner_class().datum();
        struct Projector {
            generator: usize,
            vector: RationalWeight,
        }
        let mut projectors: Vec<Projector> = Vec::new();
        let mut selected = vec![false; datum.semisimple_rank()];
        let mut lambda = lambda.clone();
        loop {
            let mut reflected = false;
            for (generator, &chosen) in selected.iter().enumerate() {
                if chosen {
                    continue;
                }
                let (dot_numerator, _) = lambda.dot_coroot(&datum.simple_coroots()[generator])?;
                if dot_numerator < 0 {
                    // alpha = simpleRoot(s)/1, orthogonalized against the
                    // previous projectors, then scaled to unit coroot
                    // pairing.
                    let mut alpha = RationalWeight::from_weight(&datum.simple_roots()[generator])?;
                    for projector in &projectors {
                        let (num, den) =
                            alpha.dot_coroot(&datum.simple_coroots()[projector.generator])?;
                        alpha = alpha.sub(&projector.vector.scale(num, den)?)?;
                    }
                    let (num, den) = alpha.dot_coroot(&datum.simple_coroots()[generator])?;
                    alpha = alpha.scale(den, num)?.normalized()?;
                    // lambda -= alpha * <lambda, alpha_s^v>
                    let (num, den) = lambda.dot_coroot(&datum.simple_coroots()[generator])?;
                    lambda = lambda.sub(&alpha.scale(num, den)?)?;
                    projectors.push(Projector {
                        generator,
                        vector: alpha,
                    });
                    selected[generator] = true;
                    reflected = true;
                    break;
                }
            }
            if !reflected {
                break;
            }
        }
        // ceil(<lambda, 2rho^v>): the projection is dominant, so the
        // pairing is nonnegative (K_repr.cpp:543-545).
        let denominator = lambda.denominator();
        let mut sum = 0_i64;
        for (&entry, &coordinate) in lambda.numerator().iter().zip(self.dual_two_rho.as_slice()) {
            sum = sum
                .checked_add(
                    entry
                        .checked_mul(i64::from(coordinate))
                        .ok_or(StructureError::ArithmeticOverflow)?,
                )
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        let ceiling = sum
            .checked_add(denominator - 1)
            .ok_or(StructureError::ArithmeticOverflow)?
            / denominator;
        u32::try_from(ceiling).map_err(|_| StructureError::ArithmeticOverflow)
    }

    /// `monomial_product` of one term by the root `shift`
    /// (K_repr.cpp:466-485): shift lambda-rho, re-elect the coset
    /// representative, and recompute the height.
    pub fn monomial_shift(&self, ktype: &KType, shift: &Weight) -> Result<KType, StructureError> {
        let mut new_exp = checked_add_weights(ktype.lambda_rho(), shift)?;
        let involution = self.involution_of(ktype.x())?;
        new_exp = self.lambda_unique(involution, &new_exp)?;
        let theta = self.theta_at(ktype.x())?;
        let th1 = checked_add_weights(
            &checked_add_weights(&new_exp, &theta.act_on_weight(&new_exp)?)?,
            &self.theta_plus_one_rho_at(ktype.x())?,
        )?;
        let height = self.height(&th1)?;
        Ok(KType::new(ktype.x(), new_exp, height))
    }

    /// `Rep_context::K_type_formula` (K_repr.cpp:549-591): the K-type
    /// formula of a semifinal K-type with the given height cutoff — the
    /// KGP set expanded by the nilpotent (1-X^alpha) factors of the
    /// parabolic, pruned by `height_bound` and re-expanded to final
    /// K-types. The returned list is unordered with integer
    /// multiplicities (the language layer merges like terms in the pol
    /// order).
    pub fn k_type_formula(
        &self,
        t: &KType,
        max_level: u32,
    ) -> Result<Vec<(KType, i32)>, StructureError> {
        let system = self.inner_class().root_system();
        let terms = t.kgp_set(self)?;
        let theta_stable = &terms[0];
        let max_length =
            self.graph()
                .length(theta_stable.x())
                .ok_or(StructureError::IndexOutOfRange {
                    index: theta_stable.x().index(),
                    upper_bound: self.graph().size(),
                })?;
        // The nilpotent positive roots of the parabolic at the
        // theta-stable element: all positive roots that are not real.
        let theta_involution = self.involution_of(theta_stable.x())?;
        let theta_root_data = self
            .table()
            .record(theta_involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: theta_involution.0,
                upper_bound: self.table().involution_count(),
            })?
            .twisted_involution()
            .root_involution();
        let mut radical_posroots = Vec::new();
        for (id, _, _) in system.entries() {
            if system
                .is_positive(id)
                .ok_or(StructureError::IndexOutOfRange {
                    index: id.0,
                    upper_bound: system.roots().len(),
                })?
                && theta_root_data.kind(id) != Some(RootKind::Real)
            {
                radical_posroots.push(id);
            }
        }

        let mut result: Vec<(KType, i32)> = Vec::new();
        for term in &terms {
            let x = term.x();
            let lr = term.lambda_rho();
            let theta = self.theta_at(x)?;
            let lambda_0 = RationalWeight::new(
                checked_add_weights(
                    &checked_add_weights(lr, &theta.act_on_weight(lr)?)?,
                    &self.theta_plus_one_rho_at(x)?,
                )?
                .as_slice()
                .iter()
                .map(|&entry| i64::from(entry))
                .collect(),
                2,
            )?
            .normalized()?;
            if self.height_bound(&lambda_0)? > max_level {
                continue;
            }
            let involution = self.involution_of(x)?;
            let root_data = self
                .table()
                .record(involution)
                .ok_or(StructureError::IndexOutOfRange {
                    index: involution.0,
                    upper_bound: self.table().involution_count(),
                })?
                .twisted_involution()
                .root_involution();
            let mut sum_set = Vec::new();
            for &root in &radical_posroots {
                match root_data.kind(root) {
                    Some(RootKind::Complex) => {
                        if let Some(image) = root_data.image(root) {
                            // First complex of a swapped pair
                            // (K_repr.cpp:557-558).
                            if image.0 > root.0 {
                                sum_set.push(root);
                            }
                        }
                    }
                    Some(RootKind::Imaginary)
                        if self.root_status_at(x, root)? == KgbStatus::ImaginaryNoncompact =>
                    {
                        sum_set.push(root);
                    }
                    _ => {}
                }
            }
            let term_length = self
                .graph()
                .length(x)
                .ok_or(StructureError::IndexOutOfRange {
                    index: x.index(),
                    upper_bound: self.graph().size(),
                })?;
            let sign = if (max_length as i64 - term_length as i64) % 2 == 0 {
                1
            } else {
                -1
            };
            let mut product: Vec<(KType, i32)> = vec![(term.clone(), sign)];
            for &root in &sum_set {
                let root_weight = system.root(root).ok_or(StructureError::IndexOutOfRange {
                    index: root.0,
                    upper_bound: system.roots().len(),
                })?;
                let mut shifted_terms: Vec<(KType, i32)> = Vec::new();
                for (ktype, coefficient) in &product {
                    let shifted = self.monomial_shift(ktype, root_weight)?;
                    let shifted_theta = self.theta_at(shifted.x())?;
                    let shifted_lambda_0 = RationalWeight::new(
                        checked_add_weights(
                            &checked_add_weights(
                                shifted.lambda_rho(),
                                &shifted_theta.act_on_weight(shifted.lambda_rho())?,
                            )?,
                            &self.theta_plus_one_rho_at(shifted.x())?,
                        )?
                        .as_slice()
                        .iter()
                        .map(|&entry| i64::from(entry))
                        .collect(),
                        2,
                    )?
                    .normalized()?;
                    if self.height_bound(&shifted_lambda_0)? <= max_level {
                        merge_ktype_terms(&mut shifted_terms, shifted, *coefficient);
                    }
                }
                for (ktype, coefficient) in shifted_terms {
                    // Multiply the product by (1 - X^alpha).
                    merge_ktype_terms(&mut product, ktype, -coefficient);
                }
            }
            for (ktype, coefficient) in product {
                let finals = ktype.finals_for(self)?;
                for (final_ktype, multiplicity) in finals {
                    if final_ktype.height() <= max_level {
                        let combined = coefficient
                            .checked_mul(multiplicity)
                            .ok_or(StructureError::ArithmeticOverflow)?;
                        merge_ktype_terms(&mut result, final_ktype, combined);
                    }
                }
            }
        }
        Ok(result)
    }

    /// `Rep_context::theta` (repr.cpp:179-180): the involution of `z`'s
    /// KGB element.
    pub fn theta(&self, z: &StandardRepr) -> Result<LatticeInvolution, StructureError> {
        Ok(self.theta_at(z.x)?.clone())
    }
}

// ---------------------------------------------------------------------------
// Shared KGB/root helpers and the `StandardRepr` predicate/operation set
// (gkmod/repr.cpp:359-699).
// ---------------------------------------------------------------------------

impl RepContext<'_> {
    pub fn kgb_status(&self, x: KgbId, generator: usize) -> Result<KgbStatus, StructureError> {
        self.graph.status(x, generator).ok_or({
            StructureError::IndexOutOfRange {
                index: x.index(),
                upper_bound: self.graph.size(),
            }
        })
    }

    pub(crate) fn is_complex_descent(
        &self,
        x: KgbId,
        generator: usize,
    ) -> Result<bool, StructureError> {
        if self.kgb_status(x, generator)? != KgbStatus::Complex {
            return Ok(false);
        }
        self.graph.is_descent(x, generator).ok_or({
            StructureError::IndexOutOfRange {
                index: x.index(),
                upper_bound: self.graph.size(),
            }
        })
    }

    pub(crate) fn cross_at(&self, x: KgbId, generator: usize) -> Result<KgbId, StructureError> {
        self.graph.cross(x, generator).ok_or({
            StructureError::IndexOutOfRange {
                index: x.index(),
                upper_bound: self.graph.size(),
            }
        })
    }

    pub(crate) fn is_complex_simple(
        &self,
        x: KgbId,
        generator: usize,
    ) -> Result<bool, StructureError> {
        let involution = self.involution_of(x)?;
        Ok(self.table.simple_root_kind(involution, generator) == Some(RootKind::Complex))
    }

    pub(crate) fn imaginary_simple_roots_at(
        &self,
        x: KgbId,
    ) -> Result<Vec<RootId>, StructureError> {
        let involution = self.involution_of(x)?;
        let record = self
            .table
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: self.table.involution_count(),
            })?;
        Ok(record
            .twisted_involution()
            .root_involution()
            .imaginary_simple_roots()
            .to_vec())
    }

    pub(crate) fn real_simple_roots_at(&self, x: KgbId) -> Result<Vec<RootId>, StructureError> {
        let involution = self.involution_of(x)?;
        let record = self
            .table
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: self.table.involution_count(),
            })?;
        Ok(record
            .twisted_involution()
            .root_involution()
            .real_simple_roots()
            .to_vec())
    }

    /// `i_tab.root_involution(n, alpha)` (involutions.h:159-160): the
    /// involution's root permutation applied to `alpha`.
    pub(crate) fn root_involution_image_at(
        &self,
        x: KgbId,
        alpha: RootId,
    ) -> Result<RootId, StructureError> {
        let involution = self.involution_of(x)?;
        let record = self
            .table
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: self.table.involution_count(),
            })?;
        record
            .twisted_involution()
            .root_involution()
            .image(alpha)
            .ok_or(StructureError::IndexOutOfRange {
                index: alpha.0,
                upper_bound: self.inner_class.root_system().roots().len(),
            })
    }

    /// The dominance defect `sum over positive coroots of max(0, -<v, beta^v>)`,
    /// an exact step bound for the greedy dominance loops: it strictly
    /// decreases at every reflection (rootdata.h:638-640).
    pub(crate) fn weight_defect(&self, weight: &Weight) -> Result<i64, StructureError> {
        let system = self.inner_class.root_system();
        let mut defect = 0_i64;
        for (id, _, coroot) in system.entries() {
            if system
                .is_positive(id)
                .ok_or(StructureError::IndexOutOfRange {
                    index: id.0,
                    upper_bound: system.roots().len(),
                })?
            {
                let pairing = pair(weight, coroot)?;
                if pairing < 0 {
                    defect = defect
                        .checked_add(i64::from(-pairing))
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
            }
        }
        Ok(defect)
    }

    /// The positive real roots of `x`'s involution: upstream
    /// `i_tab.real_roots(i_x) & rd.posroot_set()` (repr.cpp:407-409).
    pub fn positive_real_roots_at(&self, x: KgbId) -> Result<Vec<RootId>, StructureError> {
        let involution = self.involution_of(x)?;
        let record = self
            .table
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: self.table.involution_count(),
            })?;
        let system = self.inner_class.root_system();
        let mut roots = Vec::new();
        for id in record
            .twisted_involution()
            .root_involution()
            .roots_of_kind(RootKind::Real)
        {
            if system
                .is_positive(id)
                .ok_or(StructureError::IndexOutOfRange {
                    index: id.0,
                    upper_bound: system.roots().len(),
                })?
            {
                roots.push(id);
            }
        }
        Ok(roots)
    }

    /// `TitsCoset::simple_imaginary_grading` (tits.cpp:704-717): the
    /// compactness grading of the positive simply-imaginary root `alpha`
    /// at `x`; `true` means NONCOMPACT, matching `Grading::is_noncompact`.
    pub(crate) fn simple_imaginary_grading(
        &self,
        x: KgbId,
        alpha: RootId,
    ) -> Result<bool, StructureError> {
        let system = self.inner_class.root_system();
        let datum = self.inner_class.datum();
        let coordinates =
            system
                .simple_coordinates(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.0,
                    upper_bound: system.roots().len(),
                })?;
        let element = self
            .graph
            .element(x)
            .ok_or(StructureError::IndexOutOfRange {
                index: x.index(),
                upper_bound: self.graph.size(),
            })?;
        let base_grading = self.graph.base_grading();
        // The mod-2 reduction of alpha's simple-root expression
        // (tits.cpp:713): the complement-of-base-grading parity and the
        // dual-m_alpha torus evaluations both fold over its set bits.
        let mut complement_parity = false;
        let mut evaluation_parity = false;
        for (generator, &coefficient) in coordinates.iter().enumerate() {
            if coefficient.rem_euclid(2) == 0 {
                continue;
            }
            if !base_grading[generator] {
                complement_parity = !complement_parity;
            }
            let mut dot = false;
            let simple_root = &datum.simple_roots()[generator];
            for (index, &coordinate) in simple_root.as_slice().iter().enumerate() {
                if coordinate.rem_euclid(2) != 0 && element.torus_bits().bit(index) == Some(true) {
                    dot = !dot;
                }
            }
            if dot {
                evaluation_parity = !evaluation_parity;
            }
        }
        Ok(complement_parity ^ evaluation_parity ^ true)
    }

    /// `rd.simple_reflect(s, v, d)` (rootdata.h:617-618):
    /// `v -= (<v, alpha_s^v> + d) * alpha_s`.
    pub(crate) fn simple_reflect(
        &self,
        generator: usize,
        weight: &mut Weight,
        offset: i32,
    ) -> Result<(), StructureError> {
        let datum = self.inner_class.datum();
        let pairing = pair(weight, &datum.simple_coroots()[generator])?
            .checked_add(offset)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let mut coordinates = Vec::new();
        coordinates.try_reserve_exact(weight.rank()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: weight.rank(),
            }
        })?;
        for (&entry, &root_entry) in weight
            .as_slice()
            .iter()
            .zip(datum.simple_roots()[generator].as_slice())
        {
            let shift = pairing
                .checked_mul(root_entry)
                .ok_or(StructureError::ArithmeticOverflow)?;
            coordinates.push(
                entry
                    .checked_sub(shift)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            );
        }
        *weight = Weight::new(coordinates);
        Ok(())
    }

    /// The same shifted reflection on a rational weight's numerator
    /// (repr.cpp:572: `rd.simple_reflect(s,numer)`).
    pub(crate) fn simple_reflect_numerator(
        &self,
        generator: usize,
        numerator: &mut [i64],
    ) -> Result<(), StructureError> {
        let datum = self.inner_class.datum();
        let coroot = &datum.simple_coroots()[generator];
        if numerator.len() != coroot.rank() {
            return Err(StructureError::RankMismatch {
                expected: coroot.rank(),
                actual: numerator.len(),
            });
        }
        let mut pairing = 0_i64;
        for (&entry, &coroot_entry) in numerator.iter().zip(coroot.as_slice()) {
            let product = entry
                .checked_mul(i64::from(coroot_entry))
                .ok_or(StructureError::ArithmeticOverflow)?;
            pairing = pairing
                .checked_add(product)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        for (entry, &root_entry) in numerator
            .iter_mut()
            .zip(datum.simple_roots()[generator].as_slice())
        {
            let shift = pairing
                .checked_mul(i64::from(root_entry))
                .ok_or(StructureError::ArithmeticOverflow)?;
            *entry = entry
                .checked_sub(shift)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        Ok(())
    }

    /// The dominance defect `sum over positive coroots of
    /// max(0, -<v, beta^v>)`, an exact step bound for the greedy
    /// dominance loops: it strictly decreases at every reflection.
    pub(crate) fn numerator_defect(&self, numerator: &[i64]) -> Result<i64, StructureError> {
        let system = self.inner_class.root_system();
        let mut defect = 0_i64;
        for (id, _, coroot) in system.entries() {
            if !system
                .is_positive(id)
                .ok_or(StructureError::IndexOutOfRange {
                    index: id.0,
                    upper_bound: system.roots().len(),
                })?
            {
                continue;
            }
            let mut pairing = 0_i64;
            for (&entry, &coroot_entry) in numerator.iter().zip(coroot.as_slice()) {
                let product = entry
                    .checked_mul(i64::from(coroot_entry))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                pairing = pairing
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            if pairing < 0 {
                defect = defect
                    .checked_add(-pairing)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
        }
        Ok(defect)
    }

    pub(crate) fn simple_coroot_numerator_pairing(
        &self,
        generator: usize,
        numerator: &[i64],
    ) -> Result<i64, StructureError> {
        let coroot = &self.inner_class.datum().simple_coroots()[generator];
        let mut pairing = 0_i64;
        for (&entry, &coroot_entry) in numerator.iter().zip(coroot.as_slice()) {
            let product = entry
                .checked_mul(i64::from(coroot_entry))
                .ok_or(StructureError::ArithmeticOverflow)?;
            pairing = pairing
                .checked_add(product)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        Ok(pairing)
    }

    /// `Rep_context::singular_simples` (repr.cpp:526-535): the simple
    /// generators with `<gamma, alpha_s^v> == 0`.
    pub(crate) fn singular_simples(&self, z: &StandardRepr) -> Result<Vec<bool>, StructureError> {
        let datum = self.inner_class.datum();
        let mut singulars = try_capacity(datum.semisimple_rank())?;
        for generator in 0..datum.semisimple_rank() {
            singulars
                .push(self.simple_coroot_numerator_pairing(generator, z.gamma.numerator())? == 0);
        }
        Ok(singulars)
    }

    /// `Rep_context::complex_crosses` (repr.cpp:590-611): cross `z`
    /// through the word's singular complex simple generators, reflecting
    /// `lambda-rho` with the `rho`-based shift, then re-pack `y_bits`.
    pub(crate) fn complex_crosses(
        &self,
        z: &mut StandardRepr,
        word: &[usize],
    ) -> Result<(), StructureError> {
        let mut lr = self.lambda_rho(z)?;
        for &generator in word {
            if self.simple_coroot_numerator_pairing(generator, z.gamma.numerator())? != 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "singular complex cross",
                });
            }
            if !self.is_complex_simple(z.x, generator)? {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "complex simple cross",
                });
            }
            z.x = self.cross_at(z.x, generator)?;
            self.simple_reflect(generator, &mut lr, 1)?;
        }
        z.y_bits = self.y_pack(self.involution_of(z.x)?, &lr)?;
        Ok(())
    }

    /// `Rep_context::complex_descent_w` (repr.cpp:537-554): the word
    /// exhausting singular complex descents from `z.x()`.
    pub(crate) fn complex_descent_w(
        &self,
        z: &StandardRepr,
        singulars: &[bool],
    ) -> Result<Vec<usize>, StructureError> {
        let mut x = z.x;
        let mut word = Vec::new();
        // Every cross is a descent, so the involution length strictly
        // decreases; the graph size is a generous termination cap.
        for _ in 0..=self.graph.size() {
            let mut stepped = false;
            for (generator, &singular) in singulars.iter().enumerate() {
                if singular && self.is_complex_descent(x, generator)? {
                    x = self.cross_at(x, generator)?;
                    word.push(generator);
                    stepped = true;
                    break;
                }
            }
            if !stepped {
                return Ok(word);
            }
        }
        Err(StructureError::RepInvariantViolation {
            invariant: "complex descent termination",
        })
    }

    /// `Rep_context::to_singular_canonical` (repr.cpp:613-620): move `z`
    /// to the canonical involution of its Cartan class using only the
    /// singular generators, then verify the involution landed as
    /// `canonicalize` predicted.
    pub(crate) fn to_singular_canonical(
        &self,
        z: &mut StandardRepr,
        singulars: &[bool],
    ) -> Result<(), StructureError> {
        let involution = self.involution_of(z.x)?;
        let twisted = self
            .table
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: self.table.involution_count(),
            })?
            .twisted_involution()
            .clone();
        let (canonical, word) = self
            .inner_class
            .canonicalize_with_generators(twisted, singulars)?;
        self.complex_crosses(z, &word)?;
        let landed = self.involution_of(z.x)?;
        let landed_twisted = self
            .table
            .record(landed)
            .ok_or(StructureError::IndexOutOfRange {
                index: landed.0,
                upper_bound: self.table.involution_count(),
            })?
            .twisted_involution();
        if *landed_twisted != canonical {
            return Err(StructureError::RepInvariantViolation {
                invariant: "singular canonical landing",
            });
        }
        Ok(())
    }
}

impl StandardRepr {
    /// `Rep_context::is_standard` (repr.cpp:359-375): `gamma` is weakly
    /// dominant on the simply-imaginary coroots.
    pub fn is_standard(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let system = rc.inner_class.root_system();
        for alpha in rc.imaginary_simple_roots_at(self.x)? {
            let coroot = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.0,
                    upper_bound: system.roots().len(),
                })?;
            let mut pairing = 0_i64;
            for (&entry, &coroot_entry) in self.gamma.numerator().iter().zip(coroot.as_slice()) {
                let product = entry
                    .checked_mul(i64::from(coroot_entry))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                pairing = pairing
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            if pairing < 0 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Rep_context::is_dominant` (repr.h:336-337, via
    /// `is_dominant_ratweight` rootdata.cpp:1616): `gamma` is dominant on
    /// every simple coroot.
    pub fn is_dominant(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let datum = rc.inner_class.datum();
        for generator in 0..datum.semisimple_rank() {
            if rc.simple_coroot_numerator_pairing(generator, self.gamma.numerator())? < 0 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Rep_context::is_nonzero` (repr.cpp:377-392): no singular compact
    /// simply-imaginary root. Assumes `is_standard`, exactly as upstream
    /// does (the interpreter's adjective chain calls it in that order).
    pub fn is_nonzero(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let system = rc.inner_class.root_system();
        for alpha in rc.imaginary_simple_roots_at(self.x)? {
            let coroot = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.0,
                    upper_bound: system.roots().len(),
                })?;
            let mut pairing = 0_i64;
            for (&entry, &coroot_entry) in self.gamma.numerator().iter().zip(coroot.as_slice()) {
                let product = entry
                    .checked_mul(i64::from(coroot_entry))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                pairing = pairing
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            if pairing == 0 && !rc.simple_imaginary_grading(self.x, alpha)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Rep_context::is_semifinal` (repr.cpp:403-420): no positive real
    /// root is both singular on `gamma` and odd on the shifted test
    /// weight `y_lift(y) + 2rho - 2rho_R`.
    pub fn is_semifinal(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let involution = rc.involution_of(self.x)?;
        let positive_real = rc.positive_real_roots_at(self.x)?;
        let test_weight = checked_add_weights(
            &rc.y_lift(involution, &self.y_bits)?,
            &checked_sub_weights(&rc.two_rho, &rc.two_rho_of(&positive_real)?)?,
        )?;
        let system = rc.inner_class.root_system();
        for alpha in positive_real {
            let coroot = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.0,
                    upper_bound: system.roots().len(),
                })?;
            let mut gamma_pairing = 0_i64;
            for (&entry, &coroot_entry) in self.gamma.numerator().iter().zip(coroot.as_slice()) {
                let product = entry
                    .checked_mul(i64::from(coroot_entry))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                gamma_pairing = gamma_pairing
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            if gamma_pairing == 0 && pair(&test_weight, coroot)? % 4 != 0 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Rep_context::is_final` (repr.cpp:422-453): `gamma` dominant and
    /// no singular descent of any kind, then the full simply-imaginary
    /// `is_nonzero` check.
    pub fn is_final(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let datum = rc.inner_class.datum();
        let involution = rc.involution_of(self.x)?;
        let y_lift = rc.y_lift(involution, &self.y_bits)?;
        for generator in 0..datum.semisimple_rank() {
            let pairing = rc.simple_coroot_numerator_pairing(generator, self.gamma.numerator())?;
            if pairing < 0 {
                return Ok(false);
            }
            if pairing == 0 {
                match rc.kgb_status(self.x, generator)? {
                    KgbStatus::Complex => {
                        if rc.is_complex_descent(self.x, generator)? {
                            return Ok(false);
                        }
                    }
                    KgbStatus::ImaginaryCompact => return Ok(false),
                    KgbStatus::Real => {
                        if pair(&y_lift, &datum.simple_coroots()[generator])? % 4 != 0 {
                            return Ok(false);
                        }
                    }
                    KgbStatus::ImaginaryNoncompact => {}
                }
            }
        }
        self.is_nonzero(rc)
    }

    /// `Rep_context::is_normal` (repr.cpp:394-399): `z` equals its
    /// normal form. Upstream's adjective chain calls this only after the
    /// other four predicates pass.
    pub fn is_normal(&self, rc: &RepContext) -> Result<bool, StructureError> {
        Ok(self.normalised(rc)? == *self)
    }

    /// `Rep_context::make_dominant` (repr.cpp:556-587): cross through
    /// complex or real simple roots with negative `gamma` pairing,
    /// reflecting `gamma`'s numerator and `lambda-rho`, until `gamma` is
    /// dominant. Imaginary negative roots fail, as upstream's
    /// "Non standard parameter in make_dominant". The stored height is
    /// invariant under these crosses (the `(1+theta)gamma` weight moves
    /// by Weyl conjugates), so it is carried unchanged, as upstream does.
    pub fn made_dominant(&self, rc: &RepContext) -> Result<StandardRepr, StructureError> {
        let datum = rc.inner_class.datum();
        let mut z = self.clone();
        let mut lr = rc.lambda_rho(&z)?;
        let mut numerator = z.gamma.numerator().to_vec();
        let mut remaining_steps = rc.numerator_defect(&numerator)?;
        loop {
            let mut reflected = false;
            for generator in 0..datum.semisimple_rank() {
                if rc.simple_coroot_numerator_pairing(generator, &numerator)? < 0 {
                    let offset = match rc.kgb_status(z.x, generator)? {
                        KgbStatus::Complex => 1,
                        KgbStatus::Real => 0,
                        _ => {
                            return Err(StructureError::RepInvariantViolation {
                                invariant: "standard parameter in make_dominant",
                            })
                        }
                    };
                    rc.simple_reflect_numerator(generator, &mut numerator)?;
                    rc.simple_reflect(generator, &mut lr, offset)?;
                    z.x = rc.cross_at(z.x, generator)?;
                    reflected = true;
                    break;
                }
            }
            if !reflected {
                break;
            }
            remaining_steps -= 1;
            if remaining_steps < 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "dominance termination",
                });
            }
        }
        z.gamma = RationalWeight::new(numerator, z.gamma.denominator())?;
        z.y_bits = rc.y_pack(rc.involution_of(z.x)?, &lr)?;
        Ok(z)
    }

    /// `Rep_context::deform_readjust` (repr.cpp:622-654): make `gamma`
    /// dominant while exhausting singular complex descents. Unlike
    /// `made_dominant` this only acts on complex roots, and an
    /// `eval == 0` with a descent also reflects (the singular complex
    /// descent case). Returns the readjusted parameter.
    pub fn deform_readjust(&self, rc: &RepContext) -> Result<StandardRepr, StructureError> {
        let datum = rc.inner_class.datum();
        let mut z = self.clone();
        let mut lr = rc.lambda_rho(&z)?;
        let mut numer = z.gamma.numerator().to_vec();
        loop {
            let mut moved = false;
            for generator in 0..datum.semisimple_rank() {
                if rc.kgb_status(z.x, generator)? != KgbStatus::Complex {
                    continue;
                }
                let eval = rc.simple_coroot_numerator_pairing(generator, &numer)?;
                if eval < 0 {
                    rc.simple_reflect_numerator(generator, &mut numer)?;
                    rc.simple_reflect(generator, &mut lr, 1)?;
                    z.x = rc.cross_at(z.x, generator)?;
                    moved = true;
                    break;
                } else if eval == 0 && rc.is_complex_descent(z.x, generator)? {
                    rc.simple_reflect(generator, &mut lr, 1)?;
                    z.x = rc.cross_at(z.x, generator)?;
                    moved = true;
                    break;
                }
            }
            if !moved {
                break;
            }
        }
        z.gamma = RationalWeight::new(numer, z.gamma.denominator())?;
        z.y_bits = rc.y_pack(rc.involution_of(z.x)?, &lr)?;
        Ok(z)
    }

    /// `Rep_context::normalise` (repr.cpp:659-667): make dominant, move
    /// to the singular-canonical involution, then exhaust singular
    /// complex descents.
    pub fn normalised(&self, rc: &RepContext) -> Result<StandardRepr, StructureError> {
        let mut z = self.made_dominant(rc)?;
        let singulars = rc.singular_simples(&z)?;
        rc.to_singular_canonical(&mut z, &singulars)?;
        let descent_word = rc.complex_descent_w(&z, &singulars)?;
        rc.complex_crosses(&mut z, &descent_word)?;
        Ok(z)
    }

    /// `Rep_context::equivalent` (repr.cpp:678-699): equality after
    /// `make_dominant` and `to_singular_canonical`; strict equality when
    /// either parameter fails `is_standard`.
    pub fn equivalent(
        &self,
        rc: &RepContext,
        other: &StandardRepr,
    ) -> Result<bool, StructureError> {
        let self_cartan = rc
            .graph
            .cartan_of(self.x)
            .ok_or(StructureError::IndexOutOfRange {
                index: self.x.index(),
                upper_bound: rc.graph.size(),
            })?;
        let other_cartan = rc
            .graph
            .cartan_of(other.x)
            .ok_or(StructureError::IndexOutOfRange {
                index: other.x.index(),
                upper_bound: rc.graph.size(),
            })?;
        if self_cartan != other_cartan {
            return Ok(false);
        }
        if !(self.is_standard(rc)? && other.is_standard(rc)?) {
            return Ok(*self == *other);
        }
        let mut left = self.made_dominant(rc)?;
        let mut right = other.made_dominant(rc)?;
        if left.gamma != right.gamma {
            return Ok(false);
        }
        let singulars = rc.singular_simples(&left)?;
        rc.to_singular_canonical(&mut left, &singulars)?;
        rc.to_singular_canonical(&mut right, &singulars)?;
        Ok(left == right)
    }
}
