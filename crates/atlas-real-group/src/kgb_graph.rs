//! The KGB graph for one weak real form (KGB stage e).
//!
//! BFS from the stage-(d) seed over based cross actions and Cayley
//! transforms with reduce-and-dedup, statuses classified from the
//! involution table, the upstream involution ordering — (involution
//! length, Weyl length, parabolic piece list), the
//! `Cartan_orbits::comparer` key of involutions.cpp:420-428 whose third
//! leg compares `WeylElt::pieces` lexicographically — applied through the
//! stable counting-sort standardization, and inverse-Cayley links
//! installed by the ascending post-pass. The graph is HYBRID
//! self-contained: per-involution-position data and the cocharacter are
//! copied in, so every accessor except [`KgbGraph::torus_factor`] (which
//! needs theta from the table) is substrate-free. Element numbering
//! reproduces the upstream `KGB::KGB` numbering (kgb.cpp:489-683)
//! exactly: BFS discovery order within each tau packet, packets ordered
//! by the sorted involutions.

use std::collections::BTreeMap;

use malachite::{Integer, Rational};

use crate::grading::try_capacity;
use crate::tits_element::apply_matrix_mod_two;
use crate::{
    CartanClassification, CartanId, InnerClass, InvolutionId, InvolutionTable, LatticeInvolution,
    ModTwoVector, ParabolicPieces, RationalCoweight, RealFormSeed, RootKind,
    StrongRealClassification, StructureError, TitsCoset, TitsElement, WeakRealFormId, WeylElement,
    WeylInterface,
};

/// Stable identifier of one KGB element in one graph's numbering.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KgbId(pub(crate) usize);

impl KgbId {
    /// The element's position in the graph's sorted numbering.
    pub fn index(&self) -> usize {
        self.0
    }
}

/// The status of one simple generator at one KGB element.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KgbStatus {
    Complex,
    ImaginaryCompact,
    Real,
    ImaginaryNoncompact,
}

/// One weak real form's KGB graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KgbGraph {
    form: WeakRealFormId,
    rank: usize,
    cocharacter: RationalCoweight,
    /// The seed's grading offset — upstream `KGB_base::base_grading`
    /// (kgb.h:339), the `Base grading: [...]` header of `var_print_KGB`.
    base_grading: Vec<bool>,
    elements: Vec<TitsElement>,
    /// Sorted involution position per element.
    element_position: Vec<usize>,
    /// Flat `x * rank + s`.
    statuses: Vec<KgbStatus>,
    cross: Vec<KgbId>,
    cayley: Vec<Option<KgbId>>,
    inverse_cayley: Vec<Option<(KgbId, Option<KgbId>)>>,
    /// Per sorted involution position: (id, involution length, Cartan).
    positions: Vec<(InvolutionId, usize, CartanId)>,
    /// Cumulative counts; length `positions.len() + 1`.
    first_of_tau: Vec<usize>,
}

impl KgbGraph {
    /// Generate the KGB graph for the seed's weak real form.
    pub fn build(
        inner_class: &InnerClass,
        classification: &CartanClassification,
        strong: &StrongRealClassification,
        table: &mut InvolutionTable,
        seed: &RealFormSeed,
    ) -> Result<Self, StructureError> {
        if table.inner_class() != inner_class {
            return Err(StructureError::DatumMismatch);
        }
        let form = seed.form();
        let expected = strong
            .kgb_size(form)
            .ok_or(StructureError::IndexOutOfRange {
                index: form.0,
                upper_bound: strong.strong_real_data(CartanId(0)).map_or(0, |_| form.0),
            })?;
        let cartan_set =
            classification
                .cartan_set(form)
                .ok_or(StructureError::IndexOutOfRange {
                    index: form.0,
                    upper_bound: classification.weak_real_form_count(),
                })?;
        // Mutation phase: the form's ascending, idempotent Cartan additions.
        for &cartan in cartan_set {
            table.add_cartan(classification, cartan)?;
        }
        // Seed-table binding: the seed is definitionally at THIS table's
        // fundamental involution, with reduced bits.
        let identity = WeylElement::identity(table.root_system())?;
        if table.lookup(&identity) != Some(seed.element().involution()) {
            return Err(StructureError::KgbInvariantViolation {
                invariant: "seed element",
            });
        }
        let fundamental_record = table.record(seed.element().involution()).ok_or(
            StructureError::KgbInvariantViolation {
                invariant: "seed element",
            },
        )?;
        if fundamental_record
            .mod_space()
            .quotient_representative(seed.element().torus_bits().clone())?
            != *seed.element().torus_bits()
        {
            return Err(StructureError::KgbInvariantViolation {
                invariant: "seed element",
            });
        }
        // One gate here covers the whole BFS: the coset is built from the
        // SAME inner class the table was gated against.
        let coset = TitsCoset::new(inner_class, seed.grading_offset().to_vec())?;
        let rank = inner_class.datum().semisimple_rank();

        // BFS.
        let mut elements: Vec<TitsElement> = try_capacity(expected)?;
        let mut index: BTreeMap<TitsElement, usize> = BTreeMap::new();
        let mut statuses: Vec<Option<KgbStatus>> = try_capacity(expected * rank.max(1))?;
        let mut cross_raw: Vec<Option<usize>> = try_capacity(expected * rank.max(1))?;
        let mut cayley_raw: Vec<Option<usize>> = try_capacity(expected * rank.max(1))?;
        intern(
            seed.element().clone(),
            expected,
            rank,
            &mut elements,
            &mut index,
            &mut statuses,
            &mut cross_raw,
            &mut cayley_raw,
        )?;
        let mut cursor = 0_usize;
        while cursor < elements.len() {
            let current = elements[cursor].clone();
            let source_length = table
                .record(current.involution())
                .ok_or(StructureError::KgbInvariantViolation {
                    invariant: "status classification",
                })?
                .involution_length();
            for generator in 0..rank {
                let kind = table
                    .simple_root_kind(current.involution(), generator)
                    .ok_or(StructureError::KgbInvariantViolation {
                        invariant: "status classification",
                    })?;
                let crossed = coset.cross_pregated(table, generator, &current)?;
                let status = match kind {
                    RootKind::Complex => {
                        if crossed.involution() == current.involution() {
                            return Err(StructureError::KgbInvariantViolation {
                                invariant: "status classification",
                            });
                        }
                        KgbStatus::Complex
                    }
                    RootKind::Real => {
                        if crossed != current {
                            return Err(StructureError::KgbInvariantViolation {
                                invariant: "real cross fixed",
                            });
                        }
                        KgbStatus::Real
                    }
                    RootKind::Imaginary => {
                        if coset.simple_grading_pregated(&current, generator)? {
                            KgbStatus::ImaginaryNoncompact
                        } else {
                            KgbStatus::ImaginaryCompact
                        }
                    }
                };
                let slot = cursor * rank + generator;
                if statuses[slot].is_some() {
                    return Err(StructureError::KgbInvariantViolation {
                        invariant: "status write-once",
                    });
                }
                statuses[slot] = Some(status);
                let cross_target = intern(
                    crossed,
                    expected,
                    rank,
                    &mut elements,
                    &mut index,
                    &mut statuses,
                    &mut cross_raw,
                    &mut cayley_raw,
                )?;
                cross_raw[slot] = Some(cross_target);
                if status == KgbStatus::ImaginaryNoncompact {
                    let cayleyed = coset.cayley_pregated(table, generator, &current)?.ok_or(
                        StructureError::KgbInvariantViolation {
                            invariant: "cayley target missing",
                        },
                    )?;
                    let target_length = table
                        .record(cayleyed.involution())
                        .ok_or(StructureError::KgbInvariantViolation {
                            invariant: "Cayley length step",
                        })?
                        .involution_length();
                    if target_length != source_length + 1 {
                        return Err(StructureError::KgbInvariantViolation {
                            invariant: "Cayley length step",
                        });
                    }
                    let cayley_target = intern(
                        cayleyed,
                        expected,
                        rank,
                        &mut elements,
                        &mut index,
                        &mut statuses,
                        &mut cross_raw,
                        &mut cayley_raw,
                    )?;
                    cayley_raw[slot] = Some(cayley_target);
                }
            }
            cursor += 1;
        }
        drop(index);
        if elements.len() != expected {
            return Err(StructureError::KgbInvariantViolation {
                invariant: "kgb size",
            });
        }

        // The form's involutions, sorted by the upstream key: involution
        // length, then Weyl length, then the TwistedInvolution VALUE
        // compare — the parabolic piece list in the internal generator
        // order (`Cartan_orbits::comparer`, involutions.cpp:420-428; the
        // "internal number" comments in kgb.cpp and involutions.h are
        // stale). The key is a strict total order, so the sort's
        // stability is vacuous; the load-bearing stability is exclusively
        // the counting sort's below.
        let mut involutions: Vec<InvolutionId> = Vec::new();
        for &cartan in cartan_set {
            let (start, slice) =
                table
                    .orbit_slice(cartan)
                    .ok_or(StructureError::KgbInvariantViolation {
                        invariant: "involution bucket",
                    })?;
            for offset in 0..slice.len() {
                involutions.push(InvolutionId(start.0 + offset));
            }
        }
        let interface = WeylInterface::new(inner_class.datum().cartan_matrix())?;
        let pieces = ParabolicPieces::build(table.root_system(), &interface)?;
        let mut keyed = try_capacity(involutions.len())?;
        for id in involutions {
            let record = table.record(id).expect("sorted involutions exist");
            keyed.push((
                record.involution_length(),
                record.weyl_length(),
                pieces.key(table.root_system(), &interface, record.weyl_element())?,
                id.0,
            ));
        }
        keyed.sort_unstable();
        let involutions: Vec<InvolutionId> = keyed
            .into_iter()
            .map(|(_, _, _, id)| InvolutionId(id))
            .collect();
        let buckets = involutions.len();
        let mut involution_bucket: Vec<Option<usize>> = try_capacity(table.involution_count())?;
        involution_bucket.resize(table.involution_count(), None);
        for (position, id) in involutions.iter().enumerate() {
            involution_bucket[id.0] = Some(position);
        }
        let mut positions = try_capacity(buckets)?;
        for id in &involutions {
            let record = table
                .record(*id)
                .ok_or(StructureError::KgbInvariantViolation {
                    invariant: "involution bucket",
                })?;
            let cartan = table
                .cartan_of(*id)
                .ok_or(StructureError::KgbInvariantViolation {
                    invariant: "involution bucket",
                })?;
            positions.push((*id, record.involution_length(), cartan));
        }

        // Counting-sort standardization: snapshot first_of_tau BEFORE the
        // placement loop.
        let mut buckets_of: Vec<usize> = try_capacity(elements.len())?;
        for element in &elements {
            buckets_of.push(involution_bucket[element.involution().0].ok_or(
                StructureError::KgbInvariantViolation {
                    invariant: "involution bucket",
                },
            )?);
        }
        let mut counts: Vec<usize> = try_capacity(buckets + 1)?;
        counts.resize(buckets + 1, 0);
        for &bucket in &buckets_of {
            counts[bucket + 1] = counts[bucket + 1]
                .checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        for position in 0..buckets {
            counts[position + 1] = counts[position + 1]
                .checked_add(counts[position])
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        let first_of_tau = counts.clone();
        let mut cursor_counts = counts;
        let mut forward: Vec<usize> = try_capacity(elements.len())?;
        for &bucket in &buckets_of {
            forward.push(cursor_counts[bucket]);
            cursor_counts[bucket] += 1;
        }

        // Permute records by the inverse map; renumber link targets by the
        // forward map.
        let size = elements.len();
        let mut new_elements: Vec<Option<TitsElement>> = try_capacity(size)?;
        new_elements.resize(size, None);
        let mut element_position: Vec<usize> = try_capacity(size)?;
        element_position.resize(size, 0);
        let mut new_statuses: Vec<Option<KgbStatus>> = try_capacity(size * rank.max(1))?;
        new_statuses.resize(size * rank.max(1), None);
        let mut new_cross: Vec<Option<KgbId>> = try_capacity(size * rank.max(1))?;
        new_cross.resize(size * rank.max(1), None);
        let mut new_cayley: Vec<Option<KgbId>> = try_capacity(size * rank.max(1))?;
        new_cayley.resize(size * rank.max(1), None);
        for (old, element) in elements.into_iter().enumerate() {
            let new = forward[old];
            new_elements[new] = Some(element);
            element_position[new] = buckets_of[old];
            for generator in 0..rank {
                let old_slot = old * rank + generator;
                let new_slot = new * rank + generator;
                new_statuses[new_slot] = statuses[old_slot];
                new_cross[new_slot] = cross_raw[old_slot].map(|target| KgbId(forward[target]));
                new_cayley[new_slot] = cayley_raw[old_slot].map(|target| KgbId(forward[target]));
            }
        }
        let elements = new_elements.into_iter().collect::<Option<Vec<_>>>().ok_or(
            StructureError::KgbInvariantViolation {
                invariant: "kgb size",
            },
        )?;
        let statuses = new_statuses.into_iter().collect::<Option<Vec<_>>>().ok_or(
            StructureError::KgbInvariantViolation {
                invariant: "status write-once",
            },
        )?;
        let cross = new_cross.into_iter().collect::<Option<Vec<_>>>().ok_or(
            StructureError::KgbInvariantViolation {
                invariant: "kgb size",
            },
        )?;
        let cayley = new_cayley;

        // Inverse-Cayley installation, ascending over the sorted numbering.
        let mut inverse_cayley: Vec<Option<(KgbId, Option<KgbId>)>> =
            try_capacity(size * rank.max(1))?;
        inverse_cayley.resize(size * rank.max(1), None);
        for source in 0..size {
            for generator in 0..rank {
                if let Some(target) = cayley[source * rank + generator] {
                    let slot = &mut inverse_cayley[target.0 * rank + generator];
                    match slot {
                        None => *slot = Some((KgbId(source), None)),
                        Some((_, second @ None)) => *second = Some(KgbId(source)),
                        Some((_, Some(_))) => {
                            return Err(StructureError::KgbInvariantViolation {
                                invariant: "inverse Cayley pair",
                            })
                        }
                    }
                }
            }
        }

        Ok(Self {
            form,
            rank,
            cocharacter: seed.square_class_cocharacter().clone(),
            base_grading: seed.grading_offset().to_vec(),
            elements,
            element_position,
            statuses,
            cross,
            cayley,
            inverse_cayley,
            positions,
            first_of_tau,
        })
    }

    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// The element ids in ascending (sorted-numbering) order, for external
    /// consumers that cannot construct [`KgbId`] values directly.
    pub fn ids(&self) -> impl ExactSizeIterator<Item = KgbId> {
        (0..self.elements.len()).map(KgbId)
    }

    pub fn form(&self) -> WeakRealFormId {
        self.form
    }

    /// The seed's elected square-class cocharacter — upstream
    /// `RealReductiveGroup::g_rho_check` (realgroups.h), whose pairings
    /// with the simple roots give the form's base grading.
    pub fn cocharacter(&self) -> &RationalCoweight {
        &self.cocharacter
    }

    /// The form's base grading over the simple roots (upstream
    /// `KGB_base::base_grading`): set = the simple root is graded
    /// (noncompact at the base), the bit printed `1` in the header.
    pub fn base_grading(&self) -> &[bool] {
        &self.base_grading
    }

    pub fn semisimple_rank(&self) -> usize {
        self.rank
    }

    pub fn element(&self, id: KgbId) -> Option<&TitsElement> {
        self.elements.get(id.0)
    }

    pub fn involution_of(&self, id: KgbId) -> Option<InvolutionId> {
        let position = *self.element_position.get(id.0)?;
        Some(self.positions[position].0)
    }

    /// Involution length of the element's involution (upstream parity:
    /// never per-element state).
    pub fn length(&self, id: KgbId) -> Option<usize> {
        let position = *self.element_position.get(id.0)?;
        Some(self.positions[position].1)
    }

    pub fn cartan_of(&self, id: KgbId) -> Option<CartanId> {
        let position = *self.element_position.get(id.0)?;
        Some(self.positions[position].2)
    }

    pub fn status(&self, id: KgbId, generator: usize) -> Option<KgbStatus> {
        if generator >= self.rank {
            return None;
        }
        self.statuses.get(id.0 * self.rank + generator).copied()
    }

    /// Real generators are always descents, imaginary never, complex by the
    /// involution-length drop across the stored cross link.
    pub fn is_descent(&self, id: KgbId, generator: usize) -> Option<bool> {
        match self.status(id, generator)? {
            KgbStatus::Real => Some(true),
            KgbStatus::ImaginaryCompact | KgbStatus::ImaginaryNoncompact => Some(false),
            KgbStatus::Complex => {
                let target = self.cross(id, generator)?;
                Some(self.length(target)? < self.length(id)?)
            }
        }
    }

    pub fn cross(&self, id: KgbId, generator: usize) -> Option<KgbId> {
        if generator >= self.rank {
            return None;
        }
        self.cross.get(id.0 * self.rank + generator).copied()
    }

    /// `Ok(None)` = the generator is not noncompact imaginary at this
    /// element (no Cayley link).
    pub fn cayley(&self, id: KgbId, generator: usize) -> Result<Option<KgbId>, StructureError> {
        self.check_indices(id, generator)?;
        Ok(self.cayley[id.0 * self.rank + generator])
    }

    /// `Ok(None)` = the generator is not real at this element; otherwise
    /// `(first, None)` is type II and `(first, Some(second))` type I with
    /// `first < second`.
    pub fn inverse_cayley(
        &self,
        id: KgbId,
        generator: usize,
    ) -> Result<Option<(KgbId, Option<KgbId>)>, StructureError> {
        self.check_indices(id, generator)?;
        Ok(self.inverse_cayley[id.0 * self.rank + generator])
    }

    pub fn packet_count(&self) -> usize {
        self.positions.len()
    }

    /// The tau packet at one sorted involution position: its first element
    /// and size.
    pub fn tau_packet(&self, position: usize) -> Option<(KgbId, usize)> {
        if position >= self.positions.len() {
            return None;
        }
        let start = self.first_of_tau[position];
        let end = self.first_of_tau[position + 1];
        Some((KgbId(start), end - start))
    }

    pub fn packet_involution(&self, position: usize) -> Option<InvolutionId> {
        self.positions.get(position).map(|entry| entry.0)
    }

    /// `(g_rho_check - lift(bits) + theta^T applied) / 2`, exact rational —
    /// the one accessor that still needs the table (theta is per-involution
    /// table data).
    pub fn torus_factor(
        &self,
        id: KgbId,
        table: &InvolutionTable,
    ) -> Result<RationalCoweight, StructureError> {
        let element = self
            .elements
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.elements.len(),
            })?;
        let record =
            table
                .record(element.involution())
                .ok_or(StructureError::KgbInvariantViolation {
                    invariant: "involution bucket",
                })?;
        let coordinates = self.cocharacter.coordinates();
        let dimension = coordinates.len();
        let mut difference = try_capacity(dimension)?;
        for (index, value) in coordinates.iter().enumerate() {
            let bit = element.torus_bits().bit(index).unwrap_or(false);
            let lifted = if bit {
                Rational::from(1)
            } else {
                Rational::from(0)
            };
            difference.push(value - lifted);
        }
        let matrix = record.theta().coweight_matrix();
        let mut result = try_capacity(dimension)?;
        for (row_index, row) in matrix.iter().enumerate() {
            let mut transported = Rational::from(0);
            for (column, &entry) in row.iter().enumerate() {
                transported += Rational::from(entry) * &difference[column];
            }
            result.push((&difference[row_index] + transported) / Rational::from(2));
        }
        Ok(RationalCoweight::from_coordinates(result))
    }

    /// Port of upstream `KGB::lookup` (gkmod/kgb.cpp:716-726): reduce the
    /// candidate torus bits against the involution's mod space, then
    /// raw-compare left torus parts across the fiber over the involution.
    /// `Ok(None)` is upstream's `UndefKGB`.
    pub fn lookup(
        &self,
        table: &InvolutionTable,
        involution: InvolutionId,
        torus: ModTwoVector,
    ) -> Result<Option<KgbId>, StructureError> {
        let record = table
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: table.involution_count(),
            })?;
        let reduced = record.mod_space().quotient_representative(torus)?;
        let Some(position) = self
            .positions
            .iter()
            .position(|entry| entry.0 == involution)
        else {
            return Ok(None);
        };
        for index in self.first_of_tau[position]..self.first_of_tau[position + 1] {
            if self.elements[index].torus_bits() == &reduced {
                return Ok(Some(KgbId(index)));
            }
        }
        Ok(None)
    }

    /// Port of the torus-factor arithmetic in upstream
    /// `build_KGB_element_wrapper` (interpreter/atlas-types.w:4585-4595):
    /// make the rational factor theta-fixed (`num += theta.right_prod(num)`
    /// — the transposed action on the numerator), halve, subtract
    /// `g_rho_check`, and require integral coordinates. `Ok(None)` is
    /// upstream's denominator rejection ("Torus factor not in cocharacter
    /// coset of real form"); on success the integer vector's parity is the
    /// `TorusPart`. This runs on the RAW matrix: upstream performs the
    /// arithmetic before `twisted_from_involution` validates theta, and the
    /// diagnostic order depends on it.
    pub fn seed_torus_part(
        &self,
        theta: &[Vec<i32>],
        factor: &[Rational],
    ) -> Result<Option<ModTwoVector>, StructureError> {
        let coordinates = self.cocharacter.coordinates();
        let dimension = coordinates.len();
        if factor.len() != dimension {
            return Err(StructureError::RankMismatch {
                expected: dimension,
                actual: factor.len(),
            });
        }
        if theta.len() != dimension || theta.iter().any(|row| row.len() != dimension) {
            return Err(StructureError::InvalidIntegerMatrixShape);
        }
        let mut ones = try_capacity(dimension)?;
        for (column, value) in factor.iter().enumerate() {
            // right_prod: applied[column] = sum over rows of
            // factor[row] * theta[row][column].
            let mut transported = Rational::from(0);
            for (row, theta_row) in theta.iter().enumerate() {
                transported += &factor[row] * Rational::from(theta_row[column]);
            }
            let symmetrized = (value + transported) / Rational::from(2);
            let shifted = &symmetrized - &coordinates[column];
            let Ok(integer) = Integer::try_from(&shifted) else {
                return Ok(None);
            };
            let parity = i64::try_from(&integer).map_err(|_| StructureError::ArithmeticOverflow)?;
            if parity % 2 != 0 {
                ones.push(column);
            }
        }
        Ok(Some(ModTwoVector::from_ones(dimension, ones)?))
    }

    /// Upstream `KGB::twisted` (gkmod/kgb.cpp:729-745): act by an external
    /// twist on one element. `delta` must be a based root datum involution
    /// commuting with the inner class's distinguished involution, and
    /// `twist` its induced simple-root permutation — both the caller's
    /// contract (upstream `test_compatible`; the language adapter runs
    /// [`InnerClass::based_involution_twist`] first). The Weyl part is
    /// renamed letter-by-letter (upstream `WeylGroup::translation`), the
    /// torus part transports by delta's mod-2 coweight action plus the
    /// grading correction `g - g*delta`, and the result is looked up by
    /// RAW bits without reducing, exactly as upstream's `lookup`.
    /// `Ok(None)` is upstream's `UndefKGB`: the correction is
    /// non-integral, or the twisted element is not in this form's graph.
    pub fn twisted(
        &self,
        id: KgbId,
        table: &InvolutionTable,
        delta: &LatticeInvolution,
        twist: &[usize],
    ) -> Result<Option<KgbId>, StructureError> {
        let element = self
            .elements
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.elements.len(),
            })?;
        let record =
            table
                .record(element.involution())
                .ok_or(StructureError::KgbInvariantViolation {
                    invariant: "involution bucket",
                })?;
        let system = table.root_system();

        // Weyl_group().translation(a.w(), twist): rename the letters of a
        // reduced word by the diagram permutation.
        let word = record.weyl_element().reduced_word(system)?;
        let mut renamed = WeylElement::identity(system)?;
        for generator in word {
            let image = *twist
                .get(generator)
                .ok_or(StructureError::IndexOutOfRange {
                    index: generator,
                    upper_bound: twist.len(),
                })?;
            renamed = renamed.multiply(system, &WeylElement::simple_reflection(system, image)?)?;
        }
        let Some(target) = table.lookup(&renamed) else {
            return Ok(None);
        };

        // The torus part: t*delta2 + corr, where corr is the numerator of
        // g - g*delta and a non-integral entry aborts (UndefKGB). The
        // mod-2 apply already gated delta's matrix to the lattice rank.
        let mut bits = apply_matrix_mod_two(delta.coweight_matrix(), element.torus_bits())?;
        let coordinates = self.cocharacter.coordinates();
        let mut correction = try_capacity(coordinates.len())?;
        for (row_index, row) in delta.coweight_matrix().iter().enumerate() {
            let mut transported = Rational::from(0);
            for (column, &entry) in row.iter().enumerate() {
                transported += Rational::from(entry) * &coordinates[column];
            }
            let Ok(correction_entry) = Integer::try_from(&coordinates[row_index] - transported)
            else {
                return Ok(None);
            };
            let parity =
                i64::try_from(&correction_entry).map_err(|_| StructureError::ArithmeticOverflow)?;
            if parity % 2 != 0 {
                correction.push(row_index);
            }
        }
        bits.xor_assign(&ModTwoVector::from_ones(coordinates.len(), correction)?)?;

        // lookup: the fiber over the renamed involution, raw-bit equality.
        let Some(position) = self.positions.iter().position(|entry| entry.0 == target) else {
            return Ok(None);
        };
        for index in self.first_of_tau[position]..self.first_of_tau[position + 1] {
            let candidate = &self.elements[index];
            if candidate.torus_bits() == &bits {
                return Ok(Some(KgbId(index)));
            }
        }
        Ok(None)
    }

    fn check_indices(&self, id: KgbId, generator: usize) -> Result<(), StructureError> {
        if id.0 >= self.elements.len() {
            return Err(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.elements.len(),
            });
        }
        if generator >= self.rank {
            return Err(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: self.rank,
            });
        }
        Ok(())
    }
}

/// Insert-or-lookup against the a-priori size bound; new elements extend the
/// flat per-generator slots.
#[allow(clippy::too_many_arguments)]
fn intern(
    element: TitsElement,
    expected: usize,
    rank: usize,
    elements: &mut Vec<TitsElement>,
    index: &mut BTreeMap<TitsElement, usize>,
    statuses: &mut Vec<Option<KgbStatus>>,
    cross_raw: &mut Vec<Option<usize>>,
    cayley_raw: &mut Vec<Option<usize>>,
) -> Result<usize, StructureError> {
    if let Some(&existing) = index.get(&element) {
        return Ok(existing);
    }
    if elements.len() == expected {
        return Err(StructureError::KgbInvariantViolation {
            invariant: "kgb size",
        });
    }
    let id = elements.len();
    index.insert(element.clone(), id);
    elements.push(element);
    for _ in 0..rank {
        statuses.push(None);
        cross_raw.push(None);
        cayley_raw.push(None);
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassificationBudget, Coweight,
        IntegerLatticeBudget, InvolutionTableBudget, LatticeInvolution, Weight,
    };

    use super::*;

    fn class_budget(weyl: usize) -> CartanClassificationBudget {
        CartanClassificationBudget::new(
            IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            AdjointFiberBudget::new(
                IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                50_000,
                100_000,
            ),
            weyl,
            64,
            64,
        )
    }

    struct Pipeline {
        inner_class: InnerClass,
        classification: CartanClassification,
        strong: StrongRealClassification,
        table: InvolutionTable,
    }

    fn pipeline(
        datum: BasedRootDatum,
        distinguished: Option<Vec<Vec<i32>>>,
        roots: usize,
        weyl: usize,
    ) -> Pipeline {
        let distinguished = match distinguished {
            Some(matrix) => LatticeInvolution::new(&datum, matrix.clone(), matrix).unwrap(),
            None => LatticeInvolution::identity(&datum).unwrap(),
        };
        let inner_class = InnerClass::new(datum, distinguished, roots).unwrap();
        let classification =
            CartanClassification::build(&inner_class, &class_budget(weyl)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        Pipeline {
            inner_class,
            classification,
            strong,
            table,
        }
    }

    fn build_graph(pipeline: &mut Pipeline, form: usize) -> KgbGraph {
        let seed = RealFormSeed::build(
            &pipeline.inner_class,
            &pipeline.classification,
            &pipeline.strong,
            &pipeline.table,
            WeakRealFormId(form),
            &IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            4_096,
        )
        .unwrap();
        KgbGraph::build(
            &pipeline.inner_class,
            &pipeline.classification,
            &pipeline.strong,
            &mut pipeline.table,
            &seed,
        )
        .unwrap()
    }

    fn sl2_datum() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap()
    }

    #[test]
    fn sl2r_kgb_has_three_elements_with_the_type_one_pair() {
        let mut pipeline = pipeline(sl2_datum(), None, 2, 2);
        // The seed needs the fundamental Cartan in the table first.
        pipeline
            .table
            .add_cartan(&pipeline.classification, CartanId(0))
            .unwrap();
        let graph = build_graph(&mut pipeline, 0);
        assert_eq!(graph.size(), 3);
        assert!(graph.element(KgbId(0)).unwrap().torus_bits().is_zero());
        assert_eq!(graph.length(KgbId(0)), Some(0));
        assert_eq!(graph.length(KgbId(2)), Some(1));
        assert_eq!(
            graph.status(KgbId(0), 0),
            Some(KgbStatus::ImaginaryNoncompact)
        );
        assert_eq!(
            graph.status(KgbId(1), 0),
            Some(KgbStatus::ImaginaryNoncompact)
        );
        assert_eq!(graph.status(KgbId(2), 0), Some(KgbStatus::Real));
        // Type-I fiber swap; the split element is cross-fixed.
        assert_eq!(graph.cross(KgbId(0), 0), Some(KgbId(1)));
        assert_eq!(graph.cross(KgbId(1), 0), Some(KgbId(0)));
        assert_eq!(graph.cross(KgbId(2), 0), Some(KgbId(2)));
        assert_eq!(graph.cayley(KgbId(0), 0).unwrap(), Some(KgbId(2)));
        assert_eq!(graph.cayley(KgbId(1), 0).unwrap(), Some(KgbId(2)));
        assert_eq!(
            graph.inverse_cayley(KgbId(2), 0).unwrap(),
            Some((KgbId(0), Some(KgbId(1))))
        );
        assert_eq!(graph.is_descent(KgbId(2), 0), Some(true));
        assert_eq!(graph.is_descent(KgbId(0), 0), Some(false));
        // torus_factor is theta-fixed at every element.
        for id in 0..3 {
            let factor = graph.torus_factor(KgbId(id), &pipeline.table).unwrap();
            let record = pipeline
                .table
                .record(graph.involution_of(KgbId(id)).unwrap())
                .unwrap();
            let matrix = record.theta().coweight_matrix();
            let coordinates = factor.coordinates();
            for (row_index, row) in matrix.iter().enumerate() {
                let mut transported = malachite::Rational::from(0);
                for (column, &entry) in row.iter().enumerate() {
                    transported += malachite::Rational::from(entry) * &coordinates[column];
                }
                assert_eq!(transported, coordinates[row_index]);
            }
        }
    }

    #[test]
    fn pgl2r_kgb_has_two_elements_with_the_type_two_pair() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let mut pipeline = pipeline(datum, None, 2, 2);
        pipeline
            .table
            .add_cartan(&pipeline.classification, CartanId(0))
            .unwrap();
        let graph = build_graph(&mut pipeline, 0);
        assert_eq!(graph.size(), 2);
        assert_eq!(graph.cross(KgbId(0), 0), Some(KgbId(0)));
        assert_eq!(
            graph.inverse_cayley(KgbId(1), 0).unwrap(),
            Some((KgbId(0), None))
        );
    }

    #[test]
    fn sp4r_kgb_has_eleven_elements_with_oracle_length_counts() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let mut pipeline = pipeline(datum, None, 8, 8);
        pipeline
            .table
            .add_cartan(&pipeline.classification, CartanId(0))
            .unwrap();
        let graph = build_graph(&mut pipeline, 0);
        assert_eq!(graph.size(), 11);
        let mut per_length = vec![0_usize; 4];
        for id in 0..graph.size() {
            per_length[graph.length(KgbId(id)).unwrap()] += 1;
        }
        assert_eq!(per_length, vec![4, 3, 3, 1]);
        // Tau packets per Cartan sum to orbitSize x fiberSize.
        for (index, class) in pipeline.classification.cartan_classes().iter().enumerate() {
            let fiber = pipeline
                .strong
                .fiber_size(WeakRealFormId(0), CartanId(index))
                .unwrap();
            let mut total = 0_usize;
            for position in 0..graph.packet_count() {
                let involution = graph.packet_involution(position).unwrap();
                if pipeline.table.cartan_of(involution) == Some(CartanId(index)) {
                    total += graph.tau_packet(position).unwrap().1;
                }
            }
            assert_eq!(total, class.twisted_involution_count() * fiber);
        }
        // Determinism.
        let again = build_graph(&mut pipeline, 0);
        assert_eq!(graph, again);
    }

    /// The frozen B2 full-KGB probe's split-form table, verbatim from the
    /// oracle (tests/reference/domain/strong_real_b2_full_kgb_probe.events
    /// .json, job 3502700): packet order, cross/Cayley links, statuses,
    /// and the inverse-Cayley pairs of the eleven-element numbering.
    #[test]
    fn sp4r_kgb_matches_the_oracle_numbering() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let mut pipeline = pipeline(datum, None, 8, 8);
        pipeline
            .table
            .add_cartan(&pipeline.classification, CartanId(0))
            .unwrap();
        let graph = build_graph(&mut pipeline, 0);
        let system = pipeline.table.root_system();
        let word = |letters: &[usize]| {
            let mut element = WeylElement::identity(system).unwrap();
            for &generator in letters {
                let (next, _) = element.right_multiply_simple(system, generator).unwrap();
                element = next;
            }
            element
        };
        let involution_word = |id: usize| {
            let involution = graph.involution_of(KgbId(id)).unwrap();
            pipeline
                .table
                .record(involution)
                .unwrap()
                .weyl_element()
                .clone()
        };
        // Packet order: identity (0..4), s_0 (4,5), s_1 (6), s_0 s_1 s_0
        // (7), s_1 s_0 s_1 (8,9), longest (10).
        for id in 0..4 {
            assert_eq!(involution_word(id), word(&[]));
        }
        assert_eq!(involution_word(4), word(&[0]));
        assert_eq!(involution_word(5), word(&[0]));
        assert_eq!(involution_word(6), word(&[1]));
        assert_eq!(involution_word(7), word(&[0, 1, 0]));
        assert_eq!(involution_word(8), word(&[1, 0, 1]));
        assert_eq!(involution_word(9), word(&[1, 0, 1]));
        assert_eq!(involution_word(10), word(&[0, 1, 0, 1]));
        // Statuses and links, row by row.
        use KgbStatus::{
            Complex as C, ImaginaryCompact as Ic, ImaginaryNoncompact as In, Real as R,
        };
        type Row = ([KgbStatus; 2], [usize; 2], [Option<usize>; 2]);
        let expected: [Row; 11] = [
            ([In, In], [1, 2], [Some(4), Some(6)]),
            ([In, Ic], [0, 1], [Some(4), None]),
            ([In, In], [3, 0], [Some(5), Some(6)]),
            ([In, Ic], [2, 3], [Some(5), None]),
            ([R, C], [4, 8], [None, None]),
            ([R, C], [5, 9], [None, None]),
            ([C, R], [7, 6], [None, None]),
            ([C, In], [6, 7], [None, Some(10)]),
            ([In, C], [9, 4], [Some(10), None]),
            ([In, C], [8, 5], [Some(10), None]),
            ([R, R], [10, 10], [None, None]),
        ];
        for (id, (statuses, crosses, cayleys)) in expected.iter().enumerate() {
            for generator in 0..2 {
                assert_eq!(
                    graph.status(KgbId(id), generator),
                    Some(statuses[generator]),
                    "status of {id} at {generator}"
                );
                assert_eq!(
                    graph.cross(KgbId(id), generator),
                    Some(KgbId(crosses[generator])),
                    "cross of {id} at {generator}"
                );
                assert_eq!(
                    graph.cayley(KgbId(id), generator).unwrap(),
                    cayleys[generator].map(KgbId),
                    "Cayley of {id} at {generator}"
                );
            }
        }
        assert_eq!(
            graph.inverse_cayley(KgbId(4), 0).unwrap(),
            Some((KgbId(0), Some(KgbId(1))))
        );
        assert_eq!(
            graph.inverse_cayley(KgbId(6), 1).unwrap(),
            Some((KgbId(0), Some(KgbId(2))))
        );
        assert_eq!(
            graph.inverse_cayley(KgbId(10), 0).unwrap(),
            Some((KgbId(8), Some(KgbId(9))))
        );
        assert_eq!(
            graph.inverse_cayley(KgbId(10), 1).unwrap(),
            Some((KgbId(7), None))
        );
    }

    #[test]
    fn su21_kgb_has_six_elements_and_compact_forms_have_one() {
        // su(2,1) is the quasisplit form of the EQUAL-RANK A2 inner class
        // (delta = identity, root-lattice datum) — the flipped class holds
        // sl(3,R) instead, whose quasisplit KGB is 4.
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let mut pipeline = pipeline(datum, None, 6, 6);
        pipeline
            .table
            .add_cartan(&pipeline.classification, CartanId(0))
            .unwrap();
        let graph = build_graph(&mut pipeline, 0);
        assert_eq!(graph.size(), 6);

        // The compact form of SL(2): one element, all-compact, self-cross.
        let mut compact_pipeline = pipeline_sl2_for_compact();
        let compact = build_graph(&mut compact_pipeline, 1);
        assert_eq!(compact.size(), 1);
        assert_eq!(
            compact.status(KgbId(0), 0),
            Some(KgbStatus::ImaginaryCompact)
        );
        assert_eq!(compact.cross(KgbId(0), 0), Some(KgbId(0)));
        assert_eq!(compact.cayley(KgbId(0), 0).unwrap(), None);
    }

    fn pipeline_sl2_for_compact() -> Pipeline {
        let mut result = pipeline(sl2_datum(), None, 2, 2);
        result
            .table
            .add_cartan(&result.classification, CartanId(0))
            .unwrap();
        result
    }

    #[test]
    fn distinguished_twist_fixes_every_sl2r_element() {
        let mut pipeline = pipeline(sl2_datum(), None, 2, 2);
        pipeline
            .table
            .add_cartan(&pipeline.classification, CartanId(0))
            .unwrap();
        let graph = build_graph(&mut pipeline, 0);
        let delta = pipeline
            .inner_class
            .distinguished_involution()
            .involution()
            .clone();
        let twist = pipeline
            .inner_class
            .based_involution_twist(delta.clone())
            .unwrap();
        assert_eq!(twist, vec![0]);
        for id in 0..graph.size() {
            assert_eq!(
                graph
                    .twisted(KgbId(id), &pipeline.table, &delta, &twist)
                    .unwrap(),
                Some(KgbId(id))
            );
        }
    }

    #[test]
    fn outer_flip_twist_round_trips_on_su21() {
        // The A2 diagram flip is a based involution of the split (identity
        // delta) class commuting with the distinguished involution, so it
        // is a legal OUTER twist; it acts on su(2,1)'s six elements as an
        // involution (flip(corr) == -corr makes the torus correction
        // cancel on the second application).
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let mut pipeline = pipeline(datum.clone(), None, 6, 6);
        pipeline
            .table
            .add_cartan(&pipeline.classification, CartanId(0))
            .unwrap();
        let graph = build_graph(&mut pipeline, 0);
        assert_eq!(graph.size(), 6);
        let flip = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let twist = pipeline
            .inner_class
            .based_involution_twist(flip.clone())
            .unwrap();
        assert_eq!(twist, vec![1, 0]);
        let mut moved = false;
        for id in 0..graph.size() {
            let once = graph
                .twisted(KgbId(id), &pipeline.table, &flip, &twist)
                .unwrap()
                .expect("the flip preserves the quasisplit form");
            moved |= once != KgbId(id);
            let twice = graph
                .twisted(once, &pipeline.table, &flip, &twist)
                .unwrap()
                .expect("the flip preserves the quasisplit form");
            assert_eq!(twice, KgbId(id));
        }
        assert!(moved, "the outer twist swaps the s1 and s2 fibers");
    }

    #[test]
    fn seed_x0_lookup_and_torus_part_match_the_frozen_a1_anchors() {
        // The fixture's split SL(2,R) scenarios (g_rho_check = [0]).
        let mut pipeline = pipeline(sl2_datum(), None, 2, 2);
        pipeline
            .table
            .add_cartan(&pipeline.classification, CartanId(0))
            .unwrap();
        let graph = build_graph(&mut pipeline, 0);
        assert_eq!(graph.size(), 3);
        let zero = ModTwoVector::from_ones(1, vec![]).unwrap();
        let odd = ModTwoVector::from_ones(1, vec![0]).unwrap();
        let identity = WeylElement::identity(pipeline.table.root_system()).unwrap();
        let fundamental = pipeline.table.lookup(&identity).unwrap();
        // KGB_elt(rf, [[1]], [0]/1) = #0; the type-I partner #1 carries
        // the odd bit at the same involution.
        assert_eq!(
            graph
                .lookup(&pipeline.table, fundamental, zero.clone())
                .unwrap(),
            Some(KgbId(0))
        );
        assert_eq!(
            graph.lookup(&pipeline.table, fundamental, odd).unwrap(),
            Some(KgbId(1))
        );
        // KGB_elt(rf, [[-1]], [0]/1) = #2: -1 factors as w0 through the
        // compact class's twisted_from_involution.
        let datum = pipeline.inner_class.datum().clone();
        let longest = pipeline
            .inner_class
            .twisted_from_involution(
                LatticeInvolution::new(&datum, vec![vec![-1]], vec![vec![-1]]).unwrap(),
            )
            .unwrap();
        let split_involution = pipeline.table.lookup(&longest).unwrap();
        assert_eq!(
            graph
                .lookup(&pipeline.table, split_involution, zero.clone())
                .unwrap(),
            Some(KgbId(2))
        );
        // The torus-part arithmetic: [0]/1 symmetrizes to the zero coset;
        // [1]/2 is NOT in the split form's coset (denominator rejection);
        // the raw [[2]] still does arithmetic — the involution check runs
        // later, at the language layer.
        let half = malachite::Rational::from(1) / malachite::Rational::from(2);
        let zero_rational = malachite::Rational::from(0);
        assert_eq!(
            graph
                .seed_torus_part(&[vec![1]], std::slice::from_ref(&zero_rational))
                .unwrap(),
            Some(zero.clone())
        );
        assert_eq!(
            graph
                .seed_torus_part(&[vec![1]], std::slice::from_ref(&half))
                .unwrap(),
            None
        );
        assert_eq!(
            graph
                .seed_torus_part(&[vec![2]], std::slice::from_ref(&zero_rational))
                .unwrap(),
            Some(zero.clone())
        );

        // Compact SU(2) (g_rho_check = [1]/2): [1]/2 symmetrizes into the
        // zero coset, and the one-element graph answers #0.
        let compact = build_graph(&mut pipeline, 1);
        assert_eq!(compact.size(), 1);
        assert_eq!(
            compact.seed_torus_part(&[vec![1]], &[half]).unwrap(),
            Some(zero.clone())
        );
        assert_eq!(
            compact.lookup(&pipeline.table, fundamental, zero).unwrap(),
            Some(KgbId(0))
        );
    }

    #[test]
    fn seed_x0_b2_split_anchors_the_longest_element_packet() {
        // Split Sp(4,R) on the B2 datum: 11 elements, g_rho_check = [0,0].
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let mut pipeline = pipeline(datum, None, 8, 8);
        pipeline
            .table
            .add_cartan(&pipeline.classification, CartanId(0))
            .unwrap();
        let graph = build_graph(&mut pipeline, 0);
        assert_eq!(graph.size(), 11);
        let zero = ModTwoVector::from_ones(2, vec![]).unwrap();
        // The seed: identity theta and zero factor land on element #0.
        let identity = WeylElement::identity(pipeline.table.root_system()).unwrap();
        let fundamental = pipeline.table.lookup(&identity).unwrap();
        assert_eq!(
            graph
                .lookup(&pipeline.table, fundamental, zero.clone())
                .unwrap(),
            Some(KgbId(0))
        );
        assert_eq!(
            graph
                .seed_torus_part(
                    &[vec![1, 0], vec![0, 1]],
                    &[malachite::Rational::from(0), malachite::Rational::from(0)],
                )
                .unwrap(),
            Some(zero.clone())
        );
        // g = [0,0]: a half-integral factor is out of the coset.
        let half = malachite::Rational::from(1) / malachite::Rational::from(2);
        assert_eq!(
            graph
                .seed_torus_part(
                    &[vec![1, 0], vec![0, 1]],
                    &[half, malachite::Rational::from(0)],
                )
                .unwrap(),
            None
        );
        // -1 = w0 is central in B2: the split form has exactly one element
        // over the longest involution, and lookup round-trips its bits.
        let longest = pipeline
            .inner_class
            .twisted_from_involution(
                LatticeInvolution::new(
                    pipeline.inner_class.datum(),
                    vec![vec![-1, 0], vec![0, -1]],
                    vec![vec![-1, 0], vec![0, -1]],
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(longest.length(), 4);
        let split_involution = pipeline.table.lookup(&longest).unwrap();
        let packet: Vec<KgbId> = (0..graph.size())
            .map(KgbId)
            .filter(|&id| graph.involution_of(id) == Some(split_involution))
            .collect();
        assert_eq!(packet.len(), 1);
        let bits = graph.element(packet[0]).unwrap().torus_bits().clone();
        assert_eq!(
            graph
                .lookup(&pipeline.table, split_involution, bits)
                .unwrap(),
            Some(packet[0])
        );
        // A one-element packet declines the remaining bit patterns.
        assert_eq!(
            graph
                .lookup(&pipeline.table, split_involution, zero)
                .unwrap(),
            if graph.element(packet[0]).unwrap().torus_bits().is_zero() {
                Some(packet[0])
            } else {
                None
            }
        );
    }
}
