//! Shared partial/full common-block storage for one real form.
//!
//! This is the `Rep_table` slice from upstream `gkmod/repr.cpp` with the
//! canonical keying of the locator slice wired in (step 3): reduced keys are
//! upstream `Reduced_param` values `(x, int_sys_nr, residue)` computed by
//! `Reduced_param::reduce` (repr.cpp:110-125) — `InnerClass::int_item`
//! canonicalizes the integral datum across the Weyl group, the srm is
//! transported by the locator attitude, and the residue comes from the
//! canonical datum's Smith codec.  A query whose integral subsystem matches
//! a stored block under a Weyl attitude therefore reuses the stored block;
//! the query-to-stored `block_modifier` (repr.cpp:338-350
//! `make_relative_to`) records the attitude difference, and consumers that
//! still assume the identity attitude are gated loudly on it.  Reduced keys
//! and their Smith codec remain private; consumers receive stable block
//! handles and query-relative representatives.

use std::cell::Cell;
#[cfg(test)]
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
#[cfg(test)]
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
#[cfg(test)]
use std::time::Duration;

use crate::matreduc::IntMatrix;
use crate::real_projection::RealProjection;
use crate::rep_context::RepContextDerived;
use crate::{
    bruhat_below, BlockLocator, BlockModifier, CommonContext, IntegralDatumItem,
    IntegralDatumTable, IntegralSubsystem, InvolutionTable, KgbGraph, KgbId, PartialBlock,
    RationalWeight, RepContext, StandardRepr, StandardReprMod, StructureError, Weight,
};

/// Hash-stable identity of a reduced parameter: upstream `Reduced_param`
/// (repr.h:476-482).  `x` is the KGB element AFTER transport by the
/// locator attitude (`transform<true>(loc.w, srm)`), `int_sys` the
/// canonical integral datum id (`locator::int_sys_nr`), and `residue` the
/// mixed-radix packing of the canonical codec's evaluations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ReducedParamKey {
    x: KgbId,
    int_sys: u32,
    residue: u32,
}

impl ReducedParamKey {
    const fn new(x: KgbId, int_sys: u32, residue: u32) -> Self {
        Self {
            x,
            int_sys,
            residue,
        }
    }
}

/// The output of [`RepTable::reduce`]: the canonical key, the query's
/// locator (`Reduced_param::reduce` writes it through the `locator&` base
/// subobject of the `block_modifier`, repr.cpp:110-125), and the interned
/// canonical datum's simple-coroot matrix reused for row registration.
struct ReducedQuery {
    key: ReducedParamKey,
    locator: BlockLocator,
    coroots: IntMatrix,
}

/// Smith-style codec for integral-coroot evaluations modulo
/// `(1-theta)X^*`.
///
/// Crate-public so the block-modifier arithmetic can reuse it for
/// `Rep_context::make_diff_integral_orthogonal` (repr.cpp:317-329); the
/// crate's codec constructor is upstream `codec::codec` (repr.cpp:73-95).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IntegralCodec {
    coroots: IntMatrix,
    diagonal: Vec<i32>,
    input: IntMatrix,
    output: IntMatrix,
}

impl IntegralCodec {
    pub(crate) fn new(
        projection: &RealProjection,
        coroots: &IntMatrix,
    ) -> Result<Self, StructureError> {
        let ambient_rank = projection.lift_mat.len();
        let image_rank = projection.image_rank();
        if coroots.n_columns() != ambient_rank
            || projection
                .lift_mat
                .iter()
                .any(|row| row.len() != image_rank)
            || projection
                .m_real
                .iter()
                .any(|row| row.len() != ambient_rank)
        {
            return Err(StructureError::InvalidIntegerMatrixShape);
        }

        // Upstream codec::codec: A is the integral-coroot evaluation map
        // restricted to the transported basis of (1-theta)X^*.
        let mut image_evaluations = IntMatrix::new(coroots.n_rows(), image_rank);
        for row in 0..coroots.n_rows() {
            for column in 0..image_rank {
                let mut value = 0_i128;
                for index in 0..ambient_rank {
                    let term = i128::from(coroots.get(row, index))
                        .checked_mul(i128::from(projection.lift_mat[index][column]))
                        .ok_or(StructureError::ArithmeticOverflow)?;
                    value = value
                        .checked_add(term)
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
                image_evaluations.set(
                    row,
                    column,
                    i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?,
                );
            }
        }

        let (mut input, columns, mut diagonal) = crate::matreduc::diagonalise(&image_evaluations);
        if diagonal.first().is_some_and(|entry| *entry < 0) {
            diagonal[0] = diagonal[0]
                .checked_neg()
                .ok_or(StructureError::ArithmeticOverflow)?;
            for column in 0..input.n_columns() {
                input.set(
                    0,
                    column,
                    input
                        .get(0, column)
                        .checked_neg()
                        .ok_or(StructureError::ArithmeticOverflow)?,
                );
            }
        }

        // Keep only the columns corresponding to nonzero invariant factors.
        // As upstream notes, every use of `col` is immediately preceded by
        // multiplication with the transported image basis.
        let mut output = IntMatrix::new(ambient_rank, diagonal.len());
        for row in 0..ambient_rank {
            for column in 0..diagonal.len() {
                let mut value = 0_i128;
                for index in 0..image_rank {
                    let term = i128::from(projection.lift_mat[row][index])
                        .checked_mul(i128::from(columns.get(index, column)))
                        .ok_or(StructureError::ArithmeticOverflow)?;
                    value = value
                        .checked_add(term)
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
                output.set(
                    row,
                    column,
                    i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?,
                );
            }
        }

        Ok(Self {
            coroots: coroots.clone(),
            diagonal,
            input,
            output,
        })
    }

    pub(crate) fn internalise(
        &self,
        gamma_lambda: &RationalWeight,
    ) -> Result<Vec<i32>, StructureError> {
        if gamma_lambda.rank() != self.coroots.n_columns() {
            return Err(StructureError::RankMismatch {
                expected: self.coroots.n_columns(),
                actual: gamma_lambda.rank(),
            });
        }

        let denominator = i128::from(gamma_lambda.denominator());
        let mut evaluations = Vec::new();
        evaluations
            .try_reserve_exact(self.coroots.n_rows())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.coroots.n_rows(),
            })?;
        for row in 0..self.coroots.n_rows() {
            let mut numerator = 0_i128;
            for (column, &coordinate) in gamma_lambda.numerator().iter().enumerate() {
                let term = i128::from(self.coroots.get(row, column))
                    .checked_mul(i128::from(coordinate))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                numerator = numerator
                    .checked_add(term)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            if numerator.rem_euclid(denominator) != 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "integral coroot evaluation",
                });
            }
            evaluations.push(
                i32::try_from(numerator / denominator)
                    .map_err(|_| StructureError::ArithmeticOverflow)?,
            );
        }
        self.input.apply_to(&mut evaluations);
        Ok(evaluations)
    }

    fn residue(&self, gamma_lambda: &RationalWeight) -> Result<u32, StructureError> {
        let evaluations = self.internalise(gamma_lambda)?;
        let mut packed = 0_u32;
        for (index, &modulus) in self.diagonal.iter().enumerate() {
            debug_assert!(modulus > 0);
            let radix =
                u32::try_from(modulus).map_err(|_| StructureError::RepInvariantViolation {
                    invariant: "positive codec diagonal",
                })?;
            let digit = u32::try_from(evaluations[index].rem_euclid(modulus))
                .map_err(|_| StructureError::ArithmeticOverflow)?;
            packed = packed.wrapping_mul(radix).wrapping_add(digit);
        }
        Ok(packed)
    }

    fn reduced_key(
        &self,
        x: KgbId,
        int_sys: u32,
        gamma_lambda: &RationalWeight,
    ) -> Result<ReducedParamKey, StructureError> {
        Ok(ReducedParamKey::new(
            x,
            int_sys,
            self.residue(gamma_lambda)?,
        ))
    }

    /// `Rep_context::theta_1_preimage` (repr.cpp:297-313): the fixed
    /// preimage in `(1-theta)X^*` with the same integral-coroot evaluations
    /// as `difference`.
    pub(crate) fn theta_1_preimage(
        &self,
        difference: &RationalWeight,
    ) -> Result<Weight, StructureError> {
        let evaluations = self.internalise(difference)?;
        let mut coordinates = Vec::new();
        coordinates
            .try_reserve_exact(self.diagonal.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.diagonal.len(),
            })?;
        for (index, &modulus) in self.diagonal.iter().enumerate() {
            if evaluations[index].rem_euclid(modulus) != 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "theta-1 preimage divisibility",
                });
            }
            coordinates.push(evaluations[index] / modulus);
        }
        if evaluations[self.diagonal.len()..]
            .iter()
            .any(|&entry| entry != 0)
        {
            return Err(StructureError::RepInvariantViolation {
                invariant: "theta-1 preimage trailing evaluations",
            });
        }

        let mut weight = Vec::new();
        weight
            .try_reserve_exact(self.output.n_rows())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.output.n_rows(),
            })?;
        for row in 0..self.output.n_rows() {
            let mut value = 0_i64;
            for (column, &coordinate) in coordinates.iter().enumerate() {
                let term = i64::from(self.output.get(row, column))
                    .checked_mul(i64::from(coordinate))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                value = value
                    .checked_add(term)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            weight.push(i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?);
        }
        Ok(Weight::new(weight))
    }
}

/// Stable identity of a materialized common block.
///
/// IDs are append-only. Promotion leaves the old ID as a tombstone, so an
/// already returned [`LocatedBlock`] keeps its `Arc` alive while fresh table
/// resolution no longer returns the superseded block.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct BlockId(usize);

impl BlockId {
    /// The append-only sequence number, primarily useful for diagnostics.
    const fn index(self) -> usize {
        self.0
    }
}

struct BlockRecord {
    id: BlockId,
    block: Arc<PartialBlock>,
    full: bool,
    /// The generating query's locator (upstream `located_block::second`, the
    /// `pair<common_block, locator>`): the attitude at which the block's
    /// rows are stored and under which its row keys are registered
    /// (`Reduced_param::co_reduce`, repr.cpp:127-142).
    locator: BlockLocator,
    kl_table: Mutex<Option<crate::SharedKlTable>>,
}

thread_local! {
    static ACTIVE_KL_CALLBACK: Cell<bool> = const { Cell::new(false) };
}

struct ActiveKlCallback;

impl ActiveKlCallback {
    fn enter() -> Result<Self, StructureError> {
        let already_active = ACTIVE_KL_CALLBACK.with(|active| active.replace(true));
        if already_active {
            return Err(StructureError::RepInvariantViolation {
                invariant: "representation block KL table nested callback",
            });
        }
        Ok(Self)
    }
}

impl Drop for ActiveKlCallback {
    fn drop(&mut self) {
        ACTIVE_KL_CALLBACK.with(|active| {
            // The flag clear must execute in release builds too: inside
            // debug_assert! the whole `replace` is compiled out and the
            // thread-local stays true, failing every later with_kl_table
            // with "nested callback" (HPC differential 3547776).
            let was_active = active.replace(false);
            debug_assert!(was_active);
        });
    }
}

impl std::fmt::Debug for BlockRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BlockRecord")
            .field("id", &self.id)
            .field("block", &self.block)
            .field("full", &self.full)
            .field("locator", &self.locator)
            .field("kl_table", &"<lazy>")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Place {
    block: BlockId,
    row: usize,
}

#[derive(Debug)]
enum BlockSlot {
    Active(Arc<BlockRecord>),
    Superseded,
}

#[derive(Debug, Default)]
struct State {
    slots: Vec<BlockSlot>,
    places: HashMap<ReducedParamKey, Place>,
    /// The canonical integral-datum interner: upstream's per-inner-class
    /// `int_hash`/`int_table` pair (innerclass.h:238), kept per table; ids
    /// are table-local and never cross tables.
    integral_data: IntegralDatumTable,
}

impl State {
    fn active(&self, id: BlockId) -> Option<Arc<BlockRecord>> {
        match self.slots.get(id.0) {
            Some(BlockSlot::Active(record)) => Some(Arc::clone(record)),
            Some(BlockSlot::Superseded) | None => None,
        }
    }

    fn active_place(&self, key: &ReducedParamKey) -> Option<(Arc<BlockRecord>, usize)> {
        let place = *self.places.get(key)?;
        Some((self.active(place.block)?, place.row))
    }

    fn insert_record(
        &mut self,
        block: Arc<PartialBlock>,
        full: bool,
        locator: BlockLocator,
    ) -> Arc<BlockRecord> {
        let record = Arc::new(BlockRecord {
            id: BlockId(self.slots.len()),
            block,
            full,
            locator,
            kl_table: Mutex::new(None),
        });
        self.slots.push(BlockSlot::Active(Arc::clone(&record)));
        record
    }

    /// Remove every reverse entry for superseded blocks, not merely the keys
    /// which happened to collide with their replacement.
    fn retire_all(&mut self, ids: &HashSet<BlockId>) {
        for &id in ids {
            if let Some(slot) = self.slots.get_mut(id.0) {
                *slot = BlockSlot::Superseded;
            }
        }
        self.places.retain(|_, place| !ids.contains(&place.block));
    }

    fn reverse_register(&mut self, record: &BlockRecord, row_keys: &[(ReducedParamKey, usize)]) {
        for &(key, row) in row_keys.iter().rev() {
            self.places.insert(
                key,
                Place {
                    block: record.id,
                    row,
                },
            );
        }
    }

    fn commit_partial(
        &mut self,
        block: Arc<PartialBlock>,
        key: ReducedParamKey,
        exact_seed_row: usize,
        row_keys: &[(ReducedParamKey, usize)],
        locator: BlockLocator,
    ) -> Result<(Arc<BlockRecord>, usize), StructureError> {
        if let Some(existing) = self.active_place(&key) {
            return Ok(existing);
        }
        if !row_keys
            .iter()
            .any(|&(candidate, row)| candidate == key && row == exact_seed_row)
        {
            return Err(StructureError::RepInvariantViolation {
                invariant: "partial representation block exact seed row",
            });
        }

        let overlaps: HashSet<BlockId> = row_keys
            .iter()
            .filter_map(|(candidate, _)| self.places.get(candidate))
            .filter_map(|place| self.active(place.block).map(|_| place.block))
            .collect();
        if !overlaps.is_empty() {
            return Err(StructureError::NotYetImplemented {
                feature: "merging overlapping partial representation blocks",
            });
        }

        let record = self.insert_record(block, false, locator);
        self.reverse_register(&record, row_keys);
        Ok((record, exact_seed_row))
    }
}

/// One query located in a shared partial or full common block.
///
/// `raw_row` is the stored block's numbering. `modifier` is the
/// query-to-stored `block_modifier` of upstream `Rep_table::lookup`
/// (repr.cpp:338-350 `make_relative_to`): its `shift` transports the stored
/// representative to the query, and its locator part (`w`, `simple_pi`)
/// records the attitude difference.  `adapted_representative` is the stored
/// representative read back through the modifier (shift, then
/// `transform<false>`), which the lookup invariant guarantees equals the
/// query's own srm.
#[derive(Clone, Debug)]
pub struct LocatedBlock {
    record: Arc<BlockRecord>,
    raw_row: usize,
    prepared_query: StandardRepr,
    modifier: BlockModifier,
    adapted_representative: StandardReprMod,
}

impl LocatedBlock {
    fn block_id(&self) -> BlockId {
        self.record.id
    }

    /// The materialized common block containing the query.
    pub fn block(&self) -> Arc<PartialBlock> {
        Arc::clone(&self.record.block)
    }

    /// The query's row in the stored block numbering.
    pub fn raw_row(&self) -> usize {
        self.raw_row
    }

    /// The query after the lookup operation's required preparation.
    ///
    /// Partial lookup stores the normalised query; full lookup stores the
    /// dominant query. This is the exact parameter from which the reduced key
    /// and block-relative representative were computed.
    pub fn prepared_query(&self) -> &StandardRepr {
        &self.prepared_query
    }

    /// Whether this handle refers to a full common block.
    pub fn is_full(&self) -> bool {
        self.record.full
    }

    /// Whether common-block generator numbers are already in the query's
    /// integral-subsystem order: the query-to-stored block modifier has
    /// identity `w` and identity `simple_pi` (upstream's `bm` after
    /// `make_relative_to`).  Only then may a consumer read stored rows with
    /// a plain central shift; a nontrivial modifier requires the Weyl
    /// transport of `Rep_context::sr(srm, bm, gamma)` (repr.cpp:815-823).
    pub fn has_identity_generator_attitude(&self) -> bool {
        self.modifier.w().is_identity()
            && self
                .modifier
                .simple_pi()
                .iter()
                .enumerate()
                .all(|(index, &image)| index == image)
    }

    /// The query-to-stored `block_modifier` (repr.h:493-499) filled by the
    /// lookup: the query's locator made relative to the stored block's
    /// generating locator, plus the integral-orthogonal shift.
    pub fn block_modifier(&self) -> &BlockModifier {
        &self.modifier
    }

    /// The central shift from the stored representative to this query (the
    /// `block_modifier::shift` field, repr.h:494).
    pub fn relative_shift(&self) -> &RationalWeight {
        self.modifier.shift()
    }

    /// The stored representative adapted by [`Self::relative_shift`].
    pub fn adapted_representative(&self) -> &StandardReprMod {
        &self.adapted_representative
    }

    /// Run `operation` with this record's lazily constructed shared KL table.
    ///
    /// The record-local mutex remains locked for the entire callback. Calls
    /// for the same block are serialized. A KL callback must not invoke
    /// `with_kl_table` again for any block; same-thread nesting returns a
    /// stable invariant error before acquiring another record lock.
    pub fn with_kl_table<R>(
        &self,
        operation: impl FnOnce(&mut crate::SharedKlTable) -> Result<R, StructureError>,
    ) -> Result<R, StructureError> {
        let _active = ActiveKlCallback::enter()?;
        let mut cached =
            self.record
                .kl_table
                .lock()
                .map_err(|_| StructureError::RepInvariantViolation {
                    invariant: "representation block KL table mutex",
                })?;
        if cached.is_none() {
            *cached = Some(crate::KlTable::from_handle(Arc::clone(&self.record.block))?);
        }
        let table = cached
            .as_mut()
            .ok_or(StructureError::RepInvariantViolation {
                invariant: "representation block KL table initialized",
            })?;
        operation(table)
    }
}

/// Shared representation-block kernel for one [`RepContext`].
///
/// The mutex protects only structural probe/commit operations. Bruhat
/// generation and full block materialization happen without holding it; the
/// commit path re-probes to collapse concurrent duplicate work.
#[cfg(test)]
#[derive(Clone, Debug)]
struct TestGate {
    reached_tx: Sender<()>,
    reached_rx: Arc<Mutex<Receiver<()>>>,
    release_tx: Sender<()>,
    release_rx: Arc<Mutex<Receiver<()>>>,
}

#[cfg(test)]
impl TestGate {
    fn new() -> Self {
        let (reached_tx, reached_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        Self {
            reached_tx,
            reached_rx: Arc::new(Mutex::new(reached_rx)),
            release_tx,
            release_rx: Arc::new(Mutex::new(release_rx)),
        }
    }

    fn pause(self) {
        self.reached_tx.send(()).unwrap();
        self.release_rx
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .expect("test gate release timed out");
    }

    fn wait_until_reached(&self) {
        self.reached_rx
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .expect("worker did not reach test gate");
    }

    fn release(&self) {
        self.release_tx.send(()).unwrap();
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
struct TestHooks {
    partial: Mutex<VecDeque<TestGate>>,
    full: Mutex<VecDeque<TestGate>>,
}

#[cfg(test)]
impl TestHooks {
    fn pause_before_commit(&self, full: bool) {
        let gate = if full {
            self.full.lock().unwrap().pop_front()
        } else {
            self.partial.lock().unwrap().pop_front()
        };
        if let Some(gate) = gate {
            gate.pause();
        }
    }
}

#[derive(Debug)]
pub(crate) struct RepTable {
    state: Mutex<State>,
    #[cfg(test)]
    hooks: TestHooks,
}

impl RepTable {
    fn new() -> Self {
        Self {
            state: Mutex::new(State::default()),
            #[cfg(test)]
            hooks: TestHooks::default(),
        }
    }

    #[cfg(test)]
    fn with_test_gates(partial: Vec<TestGate>, full: Vec<TestGate>) -> Self {
        Self {
            state: Mutex::new(State::default()),
            hooks: TestHooks {
                partial: Mutex::new(partial.into()),
                full: Mutex::new(full.into()),
            },
        }
    }

    /// Resolve or materialize the smallest partial block below `query`.
    ///
    /// Upstream `Rep_table::lookup` (repr.cpp:1796-1824): normalise,
    /// `mod_reduce`, `Reduced_param::reduce` for the canonical key and the
    /// query locator, probe, and on a miss build the Bruhat interval below
    /// the seed in the query's own attitude.
    fn lookup(
        &self,
        rc: &RepContext<'_>,
        query: &StandardRepr,
    ) -> Result<LocatedBlock, StructureError> {
        let query = query.normalised(rc)?;
        let seed = StandardReprMod::mod_reduce(rc, &query)?;
        let reduced = self.reduce(rc, &seed, query.gamma())?;
        if let Some((record, row)) = self.probe(&reduced.key)? {
            return Self::located(rc, record, row, &seed, query, reduced.locator);
        }

        let context = Self::query_context(rc, query.gamma())?;
        let interval = bruhat_below(&context, &seed)?;
        let block = PartialBlock::build(&context, &interval)?;
        let exact_seed_row = block
            .lookup(&seed)
            .ok_or(StructureError::RepInvariantViolation {
                invariant: "partial representation block seed row",
            })?;
        let block = Arc::new(block);
        let row_keys = Self::row_keys_for(rc, &block, &reduced.locator, &reduced.coroots)?;
        #[cfg(test)]
        self.hooks.pause_before_commit(false);

        let (record, row) = {
            let mut state = self.lock_state()?;
            state.commit_partial(
                block,
                reduced.key,
                exact_seed_row,
                &row_keys,
                reduced.locator.clone(),
            )?
        };
        Self::located(rc, record, row, &seed, query, reduced.locator)
    }

    /// Resolve or materialize the full common block containing `query`.
    ///
    /// Upstream `Rep_table::lookup_full_block` (repr.cpp:1773-1794):
    /// `make_dominant`, `mod_reduce`, `reduce`, and on a miss (or a
    /// partial-only hit) `add_block`, which may retire older partials.
    fn lookup_full_block(
        &self,
        rc: &RepContext<'_>,
        query: &StandardRepr,
    ) -> Result<LocatedBlock, StructureError> {
        let query = query.made_dominant(rc)?;
        let seed = StandardReprMod::mod_reduce(rc, &query)?;
        let reduced = self.reduce(rc, &seed, query.gamma())?;
        if let Some((record, row)) = self.probe(&reduced.key)? {
            if record.full {
                return Self::located(rc, record, row, &seed, query, reduced.locator);
            }
        }

        let context = Self::query_context(rc, query.gamma())?;
        let (block, exact_seed_row) = PartialBlock::build_full(&context, &seed)?;
        let block = Arc::new(block);
        let row_keys = Self::row_keys_for(rc, &block, &reduced.locator, &reduced.coroots)?;
        if !row_keys
            .iter()
            .any(|&(candidate, row)| candidate == reduced.key && row == exact_seed_row)
        {
            return Err(StructureError::RepInvariantViolation {
                invariant: "full representation block exact seed row",
            });
        }
        #[cfg(test)]
        self.hooks.pause_before_commit(true);

        let (record, row) = {
            let mut state = self.lock_state()?;
            if let Some((record, row)) = state.active_place(&reduced.key) {
                if record.full {
                    drop(state);
                    return Self::located(rc, record, row, &seed, query, reduced.locator);
                }
            }

            let mut partials = HashSet::new();
            for (candidate, _) in &row_keys {
                let Some(place) = state.places.get(candidate).copied() else {
                    continue;
                };
                let Some(existing) = state.active(place.block) else {
                    continue;
                };
                if existing.full {
                    return Err(StructureError::RepInvariantViolation {
                        invariant: "overlapping active full representation blocks",
                    });
                }
                partials.insert(existing.id);
            }
            state.retire_all(&partials);

            let record = state.insert_record(block, true, reduced.locator.clone());
            state.reverse_register(&record, &row_keys);
            let row = state
                .places
                .get(&reduced.key)
                .ok_or(StructureError::RepInvariantViolation {
                    invariant: "full representation block seed registered",
                })?
                .row;
            (record, row)
        };
        Self::located(rc, record, row, &seed, query, reduced.locator)
    }

    /// Resolve an active stable ID. Superseded IDs deliberately return
    /// `None`; previously returned `LocatedBlock` handles remain usable.
    fn block(&self, id: BlockId) -> Result<Option<Arc<PartialBlock>>, StructureError> {
        Ok(self
            .lock_state()?
            .active(id)
            .map(|record| Arc::clone(&record.block)))
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, State>, StructureError> {
        self.state
            .lock()
            .map_err(|_| StructureError::RepInvariantViolation {
                invariant: "representation table mutex",
            })
    }

    fn probe(
        &self,
        key: &ReducedParamKey,
    ) -> Result<Option<(Arc<BlockRecord>, usize)>, StructureError> {
        Ok(self.lock_state()?.active_place(key))
    }

    /// The query-attitude generation context of upstream
    /// `common_context(rc, gamma)` (repr.cpp:2666-2670), used when a fresh
    /// block is built.  `common_context(rc, bm)` (repr.cpp:2672-2677) builds
    /// the same subsystem: `bm.simp_int` is `image_simples(bm.w)`, the
    /// images of the canonical datum's simples, which are exactly the simple
    /// basis of gamma's integral roots in upstream positive-root order.
    fn query_context<'r, 'context>(
        rc: &'r RepContext<'context>,
        gamma: &RationalWeight,
    ) -> Result<CommonContext<'r, 'context>, StructureError> {
        if let Some(context) = CommonContext::full_if_integral(rc, gamma)? {
            return Ok(context);
        }
        CommonContext::integral(rc, gamma)
    }

    /// `Reduced_param::reduce` (repr.cpp:110-125): the canonical reduced key
    /// of a query, plus the query locator that `int_item` fills
    /// (innerclass.cpp:1116-1182).  The interned canonical datum's simple
    /// coroots are returned alongside; row registration (`co_reduce`) uses
    /// the same matrix.
    fn reduce(
        &self,
        rc: &RepContext<'_>,
        srm: &StandardReprMod,
        gamma: &RationalWeight,
    ) -> Result<ReducedQuery, StructureError> {
        let (locator, coroots) = {
            let mut state = self.lock_state()?;
            let (int_sys, locator) = state.integral_data.int_item(rc.root_system(), gamma)?;
            let coroots = Self::canonical_coroots(rc, state.integral_data.item(int_sys)?)?;
            (locator, coroots)
        };
        let key = Self::canonical_key(rc, srm, &locator, &coroots)?;
        Ok(ReducedQuery {
            key,
            locator,
            coroots,
        })
    }

    /// The coroot matrix of the interned canonical datum's simple roots
    /// (`integral_datum_item::simple_coroots`, subsystem.cpp:182-188), one
    /// row per canonical simple root.
    fn canonical_coroots(
        rc: &RepContext<'_>,
        item: &IntegralDatumItem,
    ) -> Result<IntMatrix, StructureError> {
        let simple_coroots = item.simple_coroots();
        let mut coroots = IntMatrix::new(simple_coroots.len(), rc.rank());
        for (row, coroot) in simple_coroots.iter().enumerate() {
            for (column, &entry) in coroot.iter().enumerate() {
                coroots.set(row, column, entry);
            }
        }
        Ok(coroots)
    }

    /// `Reduced_param::co_reduce` (repr.cpp:127-142), which is also the key
    /// half of `reduce`: transport the srm by the locator attitude
    /// (`transform<true>(loc.w, srm)`), then pack the canonical datum
    /// codec's residue of the transported `gamma_lambda`
    /// (`integral_datum_item::data(inv_nr)`, subsystem.cpp:220-221).
    fn canonical_key(
        rc: &RepContext<'_>,
        srm: &StandardReprMod,
        locator: &BlockLocator,
        coroots: &IntMatrix,
    ) -> Result<ReducedParamKey, StructureError> {
        let mut transported = srm.clone();
        rc.transform_srm::<true>(locator.w(), &mut transported)?;
        let codec = IntegralCodec::new(rc.projection_at(transported.x())?, coroots)?;
        codec.reduced_key(
            transported.x(),
            locator.int_sys(),
            transported.gamma_lambda(),
        )
    }

    /// `InnerClass::integrality_codec` (innerclass.cpp:1184-1194) with the
    /// subsystem supplied instead of recomputed from a weight: the coroot
    /// matrix of the subsystem simples paired with the real projection at
    /// `x`.
    pub(crate) fn integral_codec(
        rc: &RepContext<'_>,
        x: KgbId,
        subsystem: &IntegralSubsystem,
    ) -> Result<IntegralCodec, StructureError> {
        let mut coroots = IntMatrix::new(subsystem.rank(), rc.rank());
        for row in 0..subsystem.rank() {
            let root = subsystem
                .parent_root(row)
                .ok_or(StructureError::RepInvariantViolation {
                    invariant: "integral subsystem codec parent root",
                })?;
            let coroot =
                rc.root_system()
                    .coroot(root)
                    .ok_or(StructureError::RepInvariantViolation {
                        invariant: "integral subsystem codec coroot",
                    })?;
            for (column, &entry) in coroot.as_slice().iter().enumerate() {
                coroots.set(row, column, entry);
            }
        }
        IntegralCodec::new(rc.projection_at(x)?, &coroots)
    }

    #[cfg(test)]
    fn full_integral_codec(rc: &RepContext<'_>, x: KgbId) -> Result<IntegralCodec, StructureError> {
        let simple_coroots = rc.datum().simple_coroots();
        let mut coroots = IntMatrix::new(simple_coroots.len(), rc.rank());
        for (row, coroot) in simple_coroots.iter().enumerate() {
            for (column, &entry) in coroot.as_slice().iter().enumerate() {
                coroots.set(row, column, entry);
            }
        }
        IntegralCodec::new(rc.projection_at(x)?, &coroots)
    }

    /// Per-row registration keys of a fresh block: `co_reduce` of every row
    /// under the block's own (generating) locator, as
    /// `swallow_blocks_and_append` registers them (repr.cpp:1725-1740).
    fn row_keys_for(
        rc: &RepContext<'_>,
        block: &PartialBlock,
        locator: &BlockLocator,
        coroots: &IntMatrix,
    ) -> Result<Vec<(ReducedParamKey, usize)>, StructureError> {
        let mut keys = Vec::new();
        keys.try_reserve_exact(block.size())
            .map_err(|_| StructureError::AllocationFailed {
                requested: block.size(),
            })?;
        for row in 0..block.size() {
            let representative =
                block
                    .element(row)
                    .ok_or(StructureError::RepInvariantViolation {
                        invariant: "representation block row representative",
                    })?;
            keys.push((
                Self::canonical_key(rc, representative, locator, coroots)?,
                row,
            ));
        }
        Ok(keys)
    }

    /// The `make_relative_to` step of both lookups (repr.cpp:1789-1792,
    /// 1810-1814): adapt the query's locator to the stored block's
    /// generating locator, then verify the modifier round-trip — shift the
    /// stored row by `bm.shift` and transport by `transform<false>(bm.w)`
    /// (the srm-level half of `sr(srm, bm, gamma)`, repr.cpp:815-823) —
    /// restores the query exactly.  On the fresh-build path the record's
    /// locator IS the query's, so the modifier comes out trivial, matching
    /// upstream's `bm.clear(block.rank(), root_datum().rank())`
    /// (repr.cpp:1821).
    fn located(
        rc: &RepContext<'_>,
        record: Arc<BlockRecord>,
        row: usize,
        query: &StandardReprMod,
        prepared_query: StandardRepr,
        query_locator: BlockLocator,
    ) -> Result<LocatedBlock, StructureError> {
        let stored = record
            .block
            .element(row)
            .ok_or(StructureError::IndexOutOfRange {
                index: row,
                upper_bound: record.block.size(),
            })?;
        let mut modifier =
            BlockModifier::from_locator(query_locator, RationalWeight::zero(rc.rank())?);
        rc.make_relative_to(&record.locator, stored, &mut modifier, query.clone())?;
        let mut adapted_representative = stored.clone();
        let shifted = adapted_representative
            .gamma_lambda()
            .add(modifier.shift())?;
        adapted_representative.set_gamma_lambda(shifted);
        rc.transform_srm::<false>(modifier.w(), &mut adapted_representative)?;
        if adapted_representative != *query {
            return Err(StructureError::RepInvariantViolation {
                invariant: "block modifier restores reduced query",
            });
        }
        Ok(LocatedBlock {
            record,
            raw_row: row,
            prepared_query,
            modifier,
            adapted_representative,
        })
    }

    #[cfg(test)]
    fn state_counts(&self) -> (usize, usize, usize) {
        let state = self.state.lock().unwrap();
        let active = state
            .slots
            .iter()
            .filter(|slot| matches!(slot, BlockSlot::Active(_)))
            .count();
        (state.slots.len(), active, state.places.len())
    }

    #[cfg(test)]
    fn state_snapshot(&self) -> StateSnapshot {
        let state = self.state.lock().unwrap();
        StateSnapshot {
            slots: state
                .slots
                .iter()
                .map(|slot| match slot {
                    BlockSlot::Active(record) => Some(record.full),
                    BlockSlot::Superseded => None,
                })
                .collect(),
            places: state.places.clone(),
        }
    }

    #[cfg(test)]
    fn state_is_consistent(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.places.values().all(|place| {
            state
                .active(place.block)
                .is_some_and(|record| place.row < record.block.size())
        })
    }
}

/// Owned representation context substrates and their shared block cache.
///
/// A fresh [`RepContext`] is borrowed from the owned table and graph for each
/// operation. Cloning an `Arc<RepTableOwner>` therefore shares the same cache
/// without making the cache self-referential.
#[derive(Debug)]
pub struct RepTableOwner {
    table: Arc<InvolutionTable>,
    graph: Arc<KgbGraph>,
    derived: Arc<RepContextDerived>,
    blocks: RepTable,
}

impl RepTableOwner {
    /// Validate and bind an owned involution table/KGB graph pair.
    pub fn new(table: InvolutionTable, graph: KgbGraph) -> Result<Self, StructureError> {
        Self::from_shared(Arc::new(table), Arc::new(graph))
    }

    /// Validate and bind already-shared involution table/KGB graph substrates.
    pub fn from_shared(
        table: Arc<InvolutionTable>,
        graph: Arc<KgbGraph>,
    ) -> Result<Self, StructureError> {
        let rc = RepContext::new(table.inner_class(), table.as_ref(), graph.as_ref())?;
        Self::validate_substrates(&rc)?;
        let derived = Arc::clone(rc.derived());
        Ok(Self {
            table,
            graph,
            derived,
            blocks: RepTable::new(),
        })
    }

    #[cfg(test)]
    fn with_test_gates(
        table: InvolutionTable,
        graph: KgbGraph,
        partial: Vec<TestGate>,
        full: Vec<TestGate>,
    ) -> Result<Self, StructureError> {
        let table = Arc::new(table);
        let graph = Arc::new(graph);
        let rc = RepContext::new(table.inner_class(), table.as_ref(), graph.as_ref())?;
        Self::validate_substrates(&rc)?;
        let derived = Arc::clone(rc.derived());
        Ok(Self {
            table,
            graph,
            derived,
            blocks: RepTable::with_test_gates(partial, full),
        })
    }

    /// Borrow a temporary representation context from this owner.
    pub fn context(&self) -> RepContext<'_> {
        RepContext::from_derived(
            self.table.as_ref(),
            self.graph.as_ref(),
            Arc::clone(&self.derived),
        )
    }

    /// Resolve or materialize the smallest partial block below `query`.
    pub fn lookup(&self, query: &StandardRepr) -> Result<LocatedBlock, StructureError> {
        self.blocks.lookup(&self.context(), query)
    }

    /// Resolve or materialize the full common block containing `query`.
    pub fn lookup_full_block(&self, query: &StandardRepr) -> Result<LocatedBlock, StructureError> {
        self.blocks.lookup_full_block(&self.context(), query)
    }

    /// Transitional access to the owned involution table.
    pub fn table(&self) -> &InvolutionTable {
        self.table.as_ref()
    }

    /// Transitional access to the owned KGB graph.
    pub fn graph(&self) -> &KgbGraph {
        self.graph.as_ref()
    }

    fn validate_substrates(rc: &RepContext<'_>) -> Result<(), StructureError> {
        let table = rc.table();
        let graph = rc.graph();
        if graph.semisimple_rank() != rc.datum().semisimple_rank()
            || graph.cocharacter().coordinates().len() != rc.rank()
            || graph.base_grading().len() != rc.datum().semisimple_rank()
        {
            return Err(StructureError::DatumMismatch);
        }
        for x in graph.ids() {
            let involution =
                graph
                    .involution_of(x)
                    .ok_or(StructureError::KgbInvariantViolation {
                        invariant: "involution bucket",
                    })?;
            let record = table
                .record(involution)
                .ok_or(StructureError::KgbInvariantViolation {
                    invariant: "involution bucket",
                })?;
            if graph.length(x) != Some(record.involution_length())
                || graph.cartan_of(x) != table.cartan_of(involution)
            {
                return Err(StructureError::KgbInvariantViolation {
                    invariant: "involution bucket",
                });
            }
            for generator in 0..graph.semisimple_rank() {
                let compatible = matches!(
                    (
                        table.simple_root_kind(involution, generator),
                        graph.status(x, generator),
                    ),
                    (
                        Some(crate::RootKind::Complex),
                        Some(crate::KgbStatus::Complex)
                    ) | (Some(crate::RootKind::Real), Some(crate::KgbStatus::Real))
                        | (
                            Some(crate::RootKind::Imaginary),
                            Some(
                                crate::KgbStatus::ImaginaryCompact
                                    | crate::KgbStatus::ImaginaryNoncompact,
                            ),
                        )
                );
                if !compatible {
                    return Err(StructureError::KgbInvariantViolation {
                        invariant: "status classification",
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct StateSnapshot {
    slots: Vec<Option<bool>>,
    places: HashMap<ReducedParamKey, Place>,
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    use super::*;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget,
        CartanId, InnerClass, IntegerLatticeBudget, InvolutionTable, InvolutionTableBudget,
        KgbGraph, LatticeInvolution, RealFormSeed, StandardReprMod, StrongRealClassification,
        WeakRealFormId,
    };

    struct ReleaseOnDrop(Option<Sender<()>>);

    impl ReleaseOnDrop {
        fn release(&mut self) {
            if let Some(sender) = self.0.take() {
                sender.send(()).unwrap();
            }
        }
    }

    impl Drop for ReleaseOnDrop {
        fn drop(&mut self) {
            if let Some(sender) = self.0.take() {
                let _ = sender.send(());
            }
        }
    }

    #[test]
    fn active_kl_callback_flag_clears_for_sequential_calls() {
        // Release-only regression (HPC differential 3547776): when the flag
        // clear lived inside `debug_assert!`, release builds compiled it out
        // and every second with_kl_table on the same thread failed with
        // "representation block KL table nested callback". In a debug build
        // this test passed either way, so it must also be run under
        // `--release` to be meaningful.
        for _ in 0..2 {
            let guard = ActiveKlCallback::enter().expect("sequential enter must succeed");
            drop(guard);
        }
        assert!(!ACTIVE_KL_CALLBACK.with(|active| active.get()));
    }

    #[test]
    fn active_kl_callback_still_rejects_nesting() {
        let guard = ActiveKlCallback::enter().expect("first enter must succeed");
        let nested = ActiveKlCallback::enter();
        assert!(matches!(
            nested,
            Err(StructureError::RepInvariantViolation { .. })
        ));
        drop(guard);
        assert!(ActiveKlCallback::enter().is_ok());
    }

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

    fn graph_with_size(
        inner_class: &InnerClass,
        classification: &CartanClassification,
        strong: &StrongRealClassification,
        table: &mut InvolutionTable,
        size: usize,
    ) -> KgbGraph {
        for form in 0..classification.weak_real_form_count() {
            if strong.kgb_size(WeakRealFormId(form)) != Some(size) {
                continue;
            }
            table.add_cartan(classification, CartanId(0)).unwrap();
            let seed = RealFormSeed::build(
                inner_class,
                classification,
                strong,
                table,
                WeakRealFormId(form),
                &IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                4_096,
            )
            .unwrap();
            return KgbGraph::build(inner_class, classification, strong, table, &seed).unwrap();
        }
        panic!("no real form with KGB size {size}");
    }

    struct ContextFixture {
        inner_class: InnerClass,
        table: InvolutionTable,
        graph: KgbGraph,
    }

    impl ContextFixture {
        fn rc(&self) -> crate::RepContext<'_> {
            crate::RepContext::new(&self.inner_class, &self.table, &self.graph).unwrap()
        }
    }

    fn fixture(
        datum: BasedRootDatum,
        involution: LatticeInvolution,
        weyl: usize,
        kgb_size: usize,
    ) -> ContextFixture {
        let inner_class = InnerClass::new(datum, involution, weyl).unwrap();
        let classification =
            CartanClassification::build(&inner_class, &class_budget(weyl)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let graph = graph_with_size(&inner_class, &classification, &strong, &mut table, kgb_size);
        ContextFixture {
            inner_class,
            table,
            graph,
        }
    }

    fn a1_fixture() -> ContextFixture {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![crate::Coweight::new(vec![1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(datum, involution, 2, 3)
    }

    fn a1_t1_fixture() -> ContextFixture {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![2, 0])],
            vec![crate::Coweight::new(vec![1, 0])],
        )
        .unwrap();
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![1, 0], vec![0, -1]],
            vec![vec![1, 0], vec![0, -1]],
        )
        .unwrap();
        fixture(datum, involution, 2, 3)
    }

    fn opposite_based_a1_fixture() -> ContextFixture {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![-2])],
            vec![crate::Coweight::new(vec![-1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(datum, involution, 2, 3)
    }

    fn b2_fixture() -> ContextFixture {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![
                crate::Coweight::new(vec![1, 0]),
                crate::Coweight::new(vec![0, 1]),
            ],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(datum, involution, 8, 11)
    }

    fn a1_query(rc: &RepContext<'_>, x: usize) -> StandardRepr {
        StandardReprMod::build(rc, KgbId(x), &RationalWeight::zero(1).unwrap())
            .unwrap()
            .to_standard(rc, &RationalWeight::new(vec![1], 1).unwrap())
            .unwrap()
    }

    fn owner(fixture: &ContextFixture) -> RepTableOwner {
        RepTableOwner::new(fixture.table.clone(), fixture.graph.clone()).unwrap()
    }

    /// Keying of the full integral system: interning gamma=0 yields the
    /// full datum at identity attitude (locator.rs `int_item` tests), so
    /// this reproduces the retired `full_integral_*` helpers.
    fn full_system_keying(rc: &RepContext<'_>) -> (BlockLocator, IntMatrix) {
        let mut table = IntegralDatumTable::new();
        let zero = RationalWeight::zero(rc.rank()).unwrap();
        let (int_sys, locator) = table.int_item(rc.root_system(), &zero).unwrap();
        assert_eq!(int_sys, 0);
        let coroots = RepTable::canonical_coroots(rc, table.item(int_sys).unwrap()).unwrap();
        (locator, coroots)
    }

    fn full_system_key(rc: &RepContext<'_>, srm: &StandardReprMod) -> ReducedParamKey {
        let (locator, coroots) = full_system_keying(rc);
        RepTable::canonical_key(rc, srm, &locator, &coroots).unwrap()
    }

    /// The SL(3,R) anchor inner class of
    /// `tests/fixtures/domain/common_block_locator.atlas`, documented in
    /// block_modifier.rs `sl3r_fixture`.
    fn sl3r_fixture() -> ContextFixture {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-1, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-1, 2])],
            vec![
                crate::Coweight::new(vec![1, 0]),
                crate::Coweight::new(vec![0, 1]),
            ],
        )
        .unwrap();
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        fixture(datum, involution, 6, 4)
    }

    /// `param(KGB(rf,x), [0,0], nu)`, as block_modifier.rs `param_srm`.
    fn sl3r_param(
        rc: &RepContext<'_>,
        x: usize,
        nu: &RationalWeight,
    ) -> (StandardReprMod, StandardRepr) {
        let lambda_rho = Weight::new(vec![0, 0]);
        let gamma = rc.gamma(KgbId(x), &lambda_rho, nu).unwrap();
        let sr = rc.sr_gamma(KgbId(x), &lambda_rho, &gamma).unwrap();
        let srm = StandardReprMod::mod_reduce(rc, &sr).unwrap();
        (srm, sr)
    }

    fn fill_marker(kl: &mut crate::SharedKlTable) -> Result<(usize, Vec<bool>), StructureError> {
        assert!((0..kl.support().size()).all(|y| kl.prim_map(y).is_empty()));
        kl.fill(0)?;
        (0..kl.support().size())
            .find_map(|y| {
                let map = kl.prim_map(y);
                (!map.is_empty()).then_some((y, map))
            })
            .ok_or(StructureError::RepInvariantViolation {
                invariant: "filled KL test marker",
            })
    }

    fn projection(lift_entries: &[i64]) -> RealProjection {
        let rank = lift_entries.len();
        let mut lift_mat = vec![vec![0_i64; rank]; rank];
        let mut m_real = vec![vec![0_i64; rank]; rank];
        for (index, &entry) in lift_entries.iter().enumerate() {
            lift_mat[index][index] = entry;
            m_real[index][index] = 1;
        }
        RealProjection { lift_mat, m_real }
    }

    fn diagonal_matrix(entries: &[i32]) -> IntMatrix {
        let rank = entries.len();
        let mut matrix = IntMatrix::new(rank, rank);
        for (index, &entry) in entries.iter().enumerate() {
            matrix.set(index, index, entry);
        }
        matrix
    }

    #[test]
    fn residue_uses_euclidean_remainder_for_negative_evaluation() {
        let codec = IntegralCodec::new(&projection(&[2]), &diagonal_matrix(&[1])).unwrap();
        let gamma_lambda = RationalWeight::new(vec![-1], 1).unwrap();

        assert_eq!(codec.residue(&gamma_lambda), Ok(1));
    }

    #[test]
    fn residue_packs_multiple_digits_in_upstream_order() {
        let codec = IntegralCodec::new(&projection(&[2, 3]), &diagonal_matrix(&[1, 1])).unwrap();
        let gamma_lambda = RationalWeight::new(vec![-1, 5], 1).unwrap();

        assert_eq!(codec.residue(&gamma_lambda), Ok(5));
    }

    #[test]
    fn residue_deliberately_wraps_u32_mixed_radix_overflow() {
        let codec = IntegralCodec::new(
            &projection(&[65_536, 65_536, 2]),
            &diagonal_matrix(&[1, 1, 1]),
        )
        .unwrap();
        let gamma_lambda = RationalWeight::new(vec![65_535, 65_535, 0], 1).unwrap();

        assert_eq!(codec.residue(&gamma_lambda), Ok(u32::MAX - 1));
    }

    #[test]
    fn internalise_rejects_nonintegral_coroot_evaluations() {
        let codec = IntegralCodec::new(&projection(&[2]), &diagonal_matrix(&[1])).unwrap();
        let gamma_lambda = RationalWeight::new(vec![1], 2).unwrap();

        assert_eq!(
            codec.internalise(&gamma_lambda),
            Err(StructureError::RepInvariantViolation {
                invariant: "integral coroot evaluation",
            })
        );
    }

    #[test]
    fn construction_rejects_incompatible_matrix_shapes() {
        let coroots = IntMatrix::from_entries(1, 2, vec![1, 0]);

        assert_eq!(
            IntegralCodec::new(&projection(&[2]), &coroots),
            Err(StructureError::InvalidIntegerMatrixShape)
        );
    }

    #[test]
    fn owner_clones_substrates_and_exposes_a_temporary_context() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();

        assert_eq!(owner.table().inner_class(), fixture.table.inner_class());
        assert_eq!(owner.graph(), &fixture.graph);
        assert!(std::ptr::eq(rc.table(), owner.table()));
        assert!(std::ptr::eq(rc.graph(), owner.graph()));
    }

    #[test]
    fn owner_can_share_existing_arc_substrates_without_cloning_data() {
        let fixture = a1_fixture();
        let table = Arc::new(fixture.table);
        let graph = Arc::new(fixture.graph);
        let owner = RepTableOwner::from_shared(Arc::clone(&table), Arc::clone(&graph)).unwrap();

        assert!(std::ptr::eq(owner.table(), table.as_ref()));
        assert!(std::ptr::eq(owner.graph(), graph.as_ref()));
        assert_eq!(Arc::strong_count(&table), 2);
        assert_eq!(Arc::strong_count(&graph), 2);
    }

    #[test]
    fn owner_context_reuses_precomputed_derived_constants() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let first = owner.context();
        let second = owner.context();

        assert!(Arc::ptr_eq(first.derived(), second.derived()));
        assert!(Arc::ptr_eq(first.derived(), &owner.derived));
    }

    #[test]
    fn owner_substrates_and_derived_share_one_inner_class_arc() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);

        assert!(Arc::ptr_eq(
            owner.table.inner_class_shared(),
            owner.graph.inner_class_shared()
        ));
        assert!(Arc::ptr_eq(
            owner.table.inner_class_shared(),
            &owner.derived.inner_class
        ));
    }

    #[test]
    fn cloned_kgb_graph_shares_inner_class_provenance() {
        let fixture = a1_fixture();
        let clone = fixture.graph.clone();

        assert!(Arc::ptr_eq(
            fixture.graph.inner_class_shared(),
            clone.inner_class_shared()
        ));
    }

    #[test]
    fn owner_rejects_an_incompatible_table_and_graph() {
        let a1 = a1_fixture();
        let b2 = b2_fixture();

        assert!(matches!(
            RepTableOwner::new(a1.table, b2.graph),
            Err(StructureError::DatumMismatch)
        ));
    }

    #[test]
    fn owner_rejects_same_shape_graph_from_a_different_inner_class() {
        let primal = a1_fixture();
        let opposite = opposite_based_a1_fixture();

        assert_ne!(primal.table.inner_class(), opposite.table.inner_class());
        assert_eq!(
            primal.graph.semisimple_rank(),
            opposite.graph.semisimple_rank()
        );
        assert_eq!(primal.graph.size(), opposite.graph.size());
        for x in primal.graph.ids() {
            assert_eq!(primal.graph.cartan_of(x), opposite.graph.cartan_of(x));
            for generator in 0..primal.graph.semisimple_rank() {
                assert_eq!(
                    primal.graph.status(x, generator),
                    opposite.graph.status(x, generator)
                );
            }
        }

        assert!(matches!(
            RepContext::new(primal.table.inner_class(), &primal.table, &opposite.graph),
            Err(StructureError::DatumMismatch)
        ));
        assert!(matches!(
            RepTableOwner::new(primal.table, opposite.graph),
            Err(StructureError::DatumMismatch)
        ));
    }

    #[test]
    fn theta_1_preimage_recovers_an_image_weight() {
        let codec = IntegralCodec::new(&projection(&[2, 3]), &diagonal_matrix(&[1, 1])).unwrap();
        let difference = RationalWeight::new(vec![8, -6], 1).unwrap();

        assert_eq!(
            codec.theta_1_preimage(&difference),
            Ok(Weight::new(vec![8, -6]))
        );
    }

    #[test]
    fn reduced_keys_have_stable_equality_and_hash_identity() {
        let full = ReducedParamKey::new(KgbId(7), 0, 11);
        let same = ReducedParamKey::new(KgbId(7), 0, 11);
        let other_datum = ReducedParamKey::new(KgbId(7), 1, 11);
        let other_residue = ReducedParamKey::new(KgbId(7), 0, 12);

        assert_eq!(full, same);
        assert_ne!(full, other_datum);
        assert_ne!(full, other_residue);

        let mut set = HashSet::new();
        assert!(set.insert(full));
        assert!(!set.insert(same));
        assert!(set.insert(other_datum));

        let mut map = HashMap::new();
        map.insert(full, "full");
        map.insert(other_residue, "other residue");
        assert_eq!(map[&same], "full");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn reduced_key_combines_codec_residue_with_canonical_datum() {
        let codec = IntegralCodec::new(&projection(&[2]), &diagonal_matrix(&[1])).unwrap();
        let gamma_lambda = RationalWeight::new(vec![-1], 1).unwrap();

        assert_eq!(
            codec.reduced_key(KgbId(3), 0, &gamma_lambda),
            Ok(ReducedParamKey::new(KgbId(3), 0, 1))
        );
    }

    #[test]
    fn canonical_keying_interns_the_canonical_datum_once() {
        let fixture = b2_fixture();
        let rc = fixture.rc();
        let gamma = RationalWeight::new(vec![3, 1], 2).unwrap();
        let seed = StandardReprMod::build(&rc, KgbId(5), &gamma).unwrap();
        let table = RepTable::new();

        let first = table.reduce(&rc, &seed, &gamma).unwrap();
        let repeated = table.reduce(&rc, &seed, &gamma).unwrap();

        assert_eq!(first.key, repeated.key);
        assert_eq!(first.locator, repeated.locator);
        assert_eq!(first.key.int_sys, 0);
        assert_eq!(table.state.lock().unwrap().integral_data.len(), 1);
    }

    #[test]
    fn proper_integral_codec_uses_the_subsystem_coroots() {
        let fixture = b2_fixture();
        let rc = fixture.rc();
        let subsystem = IntegralSubsystem::integral(
            rc.root_system(),
            &RationalWeight::new(vec![3, 1], 2).unwrap(),
        )
        .unwrap();
        let root = subsystem.parent_root(0).unwrap();
        let expected_coroot = rc.root_system().coroot(root).unwrap();

        let codec = RepTable::integral_codec(&rc, KgbId(5), &subsystem).unwrap();

        assert_eq!(codec.coroots.n_rows(), 1);
        for column in 0..rc.rank() {
            assert_eq!(
                codec.coroots.get(0, column),
                expected_coroot.as_slice()[column]
            );
        }
    }

    #[test]
    fn proper_integral_full_lookup_materializes_the_subsystem_block() {
        let fixture = b2_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let gamma = RationalWeight::new(vec![3, 1], 2).unwrap();
        let seed = StandardReprMod::build(&rc, KgbId(5), &gamma).unwrap();
        let query = seed.to_standard(&rc, &gamma).unwrap();

        let located = owner.lookup_full_block(&query).unwrap();

        assert_eq!(located.block().size(), 3);
        assert_eq!(located.raw_row(), 1);
        assert_eq!(located.adapted_representative(), &seed);
    }

    #[test]
    fn a1_partial_then_full_promotes_raw_row_zero_to_one() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let gamma = RationalWeight::new(vec![1], 1).unwrap();
        let query = StandardReprMod::build(&rc, KgbId(1), &RationalWeight::zero(1).unwrap())
            .unwrap()
            .to_standard(&rc, &gamma)
            .unwrap();

        let partial = owner.lookup(&query).unwrap();
        assert_eq!(partial.raw_row(), 0);
        assert!(!partial.is_full());

        let full = owner.lookup_full_block(&query).unwrap();
        assert_eq!(full.raw_row(), 1);
        assert!(full.is_full());
        assert_ne!(partial.block_id(), full.block_id());
        assert_eq!(partial.block_id().index(), 0);
        assert_eq!(full.block_id().index(), 1);
    }

    #[test]
    fn lookup_exposes_the_normalised_prepared_query() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let query = StandardReprMod::build(&rc, KgbId(2), &RationalWeight::zero(1).unwrap())
            .unwrap()
            .to_standard(&rc, &RationalWeight::new(vec![-1], 1).unwrap())
            .unwrap();
        let expected = query.normalised(&rc).unwrap();
        assert_ne!(query, expected, "fixture must exercise preparation");

        let located = owner.lookup(&query).unwrap();

        assert_eq!(located.prepared_query(), &expected);
    }

    #[test]
    fn lookup_full_block_exposes_the_dominant_prepared_query() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let query = StandardReprMod::build(&rc, KgbId(2), &RationalWeight::zero(1).unwrap())
            .unwrap()
            .to_standard(&rc, &RationalWeight::new(vec![-1], 1).unwrap())
            .unwrap();
        let expected = query.made_dominant(&rc).unwrap();
        assert_ne!(query, expected, "fixture must exercise preparation");

        let located = owner.lookup_full_block(&query).unwrap();

        assert_eq!(located.prepared_query(), &expected);
    }

    #[test]
    fn located_blocks_for_one_record_share_filled_kl_table() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let first = owner.lookup_full_block(&a1_query(&rc, 1)).unwrap();
        let second = owner.lookup_full_block(&a1_query(&rc, 2)).unwrap();
        assert_eq!(first.block_id(), second.block_id());

        let (first_address, marker_y, marker_map) = first
            .with_kl_table(|kl| {
                let (y, map) = fill_marker(kl)?;
                Ok((kl as *mut _ as usize, y, map))
            })
            .unwrap();
        let (second_address, second_map) = second
            .with_kl_table(|kl| Ok((kl as *mut _ as usize, kl.prim_map(marker_y))))
            .unwrap();

        assert_eq!(first_address, second_address);
        assert_eq!(second_map, marker_map);
    }

    #[test]
    fn concurrent_kl_callbacks_are_serialized_on_one_instance() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let located = owner.lookup_full_block(&a1_query(&rc, 1)).unwrap();
        let first = located.clone();
        let second = located;
        let (first_entered_tx, first_entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (second_attempted_tx, second_attempted_rx) = mpsc::channel();
        let (second_entered_tx, second_entered_rx) = mpsc::channel();

        let (first_address, second_address) = std::thread::scope(|scope| {
            let first_worker = scope.spawn(move || {
                first
                    .with_kl_table(|kl| {
                        let (y, map) = fill_marker(kl)?;
                        first_entered_tx
                            .send((kl as *mut _ as usize, y, map))
                            .unwrap();
                        release_rx
                            .recv_timeout(Duration::from_secs(5))
                            .expect("KL callback release timed out");
                        Ok(kl as *mut _ as usize)
                    })
                    .unwrap()
            });
            let mut release = ReleaseOnDrop(Some(release_tx));
            let (entered_address, marker_y, marker_map) = first_entered_rx
                .recv_timeout(Duration::from_secs(5))
                .expect("first KL callback did not enter");
            let second_worker = scope.spawn(move || {
                second_attempted_tx.send(()).unwrap();
                second
                    .with_kl_table(|kl| {
                        assert_eq!(kl.prim_map(marker_y), marker_map);
                        second_entered_tx.send(kl as *mut _ as usize).unwrap();
                        Ok(kl as *mut _ as usize)
                    })
                    .unwrap()
            });
            second_attempted_rx
                .recv_timeout(Duration::from_secs(5))
                .expect("second KL callback was not attempted");
            assert_eq!(
                second_entered_rx.recv_timeout(Duration::from_secs(1)),
                Err(mpsc::RecvTimeoutError::Timeout)
            );
            release.release();
            let entered_second_address = second_entered_rx
                .recv_timeout(Duration::from_secs(5))
                .expect("second KL callback did not enter after release");
            let first_address = first_worker.join().unwrap();
            assert_eq!(first_address, entered_address);
            let second_address = second_worker.join().unwrap();
            assert_eq!(second_address, entered_second_address);
            (first_address, second_address)
        });

        assert_eq!(first_address, second_address);
    }

    #[test]
    fn promoted_partial_keeps_old_kl_and_full_gets_a_fresh_table() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let partial = owner.lookup(&a1_query(&rc, 0)).unwrap();
        let partial_address = partial
            .with_kl_table(|kl| {
                kl.fill(0)?;
                Ok(kl as *mut _ as usize)
            })
            .unwrap();

        let full = owner.lookup_full_block(&a1_query(&rc, 2)).unwrap();
        assert_ne!(partial.block_id(), full.block_id());
        let old_address = partial
            .with_kl_table(|kl| {
                assert_eq!(kl.kl_pol(0, 0)?, 1);
                Ok(kl as *mut _ as usize)
            })
            .unwrap();
        let full_address = full.with_kl_table(|kl| Ok(kl as *mut _ as usize)).unwrap();

        assert_eq!(old_address, partial_address);
        assert_ne!(old_address, full_address);
    }

    #[test]
    fn poisoned_kl_cache_returns_a_stable_error() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let located = owner.lookup_full_block(&a1_query(&rc, 1)).unwrap();
        let poison = located.clone();

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = poison.with_kl_table::<()>(|_| panic!("poison KL cache"));
        }));
        assert!(panic.is_err());
        assert!(matches!(
            located.with_kl_table(|_| Ok(())),
            Err(StructureError::RepInvariantViolation {
                invariant: "representation block KL table mutex"
            })
        ));
    }

    #[test]
    fn nested_kl_callback_on_same_handle_returns_nested_callback_error() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let located = owner.lookup_full_block(&a1_query(&rc, 1)).unwrap();
        let nested = located.clone();
        let (result_tx, result_rx) = mpsc::channel();

        std::thread::spawn(move || {
            let result = located.with_kl_table(|_| nested.with_kl_table(|_| Ok(())));
            result_tx.send(result).unwrap();
        });

        assert!(matches!(
            result_rx.recv_timeout(Duration::from_secs(1)),
            Ok(Err(StructureError::RepInvariantViolation {
                invariant: "representation block KL table nested callback"
            }))
        ));
    }

    #[test]
    fn nested_kl_callback_on_second_record_handle_returns_nested_callback_error() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let first = owner.lookup_full_block(&a1_query(&rc, 1)).unwrap();
        let second = owner.lookup_full_block(&a1_query(&rc, 2)).unwrap();
        assert_eq!(first.block_id(), second.block_id());
        let (result_tx, result_rx) = mpsc::channel();

        std::thread::spawn(move || {
            let result = first.with_kl_table(|_| second.with_kl_table(|_| Ok(())));
            result_tx.send(result).unwrap();
        });

        assert!(matches!(
            result_rx.recv_timeout(Duration::from_secs(1)),
            Ok(Err(StructureError::RepInvariantViolation {
                invariant: "representation block KL table nested callback"
            }))
        ));
    }

    #[test]
    fn nested_kl_callback_on_different_records_returns_nested_callback_error() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let partial = owner.lookup(&a1_query(&rc, 0)).unwrap();
        let full = owner.lookup_full_block(&a1_query(&rc, 2)).unwrap();
        assert_ne!(partial.block_id(), full.block_id());
        let (result_tx, result_rx) = mpsc::channel();

        std::thread::spawn(move || {
            let result = partial.with_kl_table(|_| full.with_kl_table(|_| Ok(())));
            result_tx.send(result).unwrap();
        });

        assert!(matches!(
            result_rx.recv_timeout(Duration::from_secs(1)),
            Ok(Err(StructureError::RepInvariantViolation {
                invariant: "representation block KL table nested callback"
            }))
        ));
    }

    #[test]
    fn kl_callback_error_clears_the_reentry_guard() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let located = owner.lookup_full_block(&a1_query(&rc, 1)).unwrap();

        assert_eq!(
            located.with_kl_table::<()>(|_| Err(StructureError::ArithmeticOverflow)),
            Err(StructureError::ArithmeticOverflow)
        );
        assert_eq!(located.with_kl_table(|_| Ok(7)), Ok(7));
    }

    #[test]
    fn b2_full_registers_every_row_and_keeps_the_two_top_keys_distinct() {
        let fixture = b2_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let gamma = RationalWeight::new(vec![2, 2], 1).unwrap();
        let seed = StandardReprMod::build(&rc, KgbId(0), &RationalWeight::zero(2).unwrap())
            .unwrap()
            .to_standard(&rc, &gamma)
            .unwrap();
        let installed = owner.lookup_full_block(&seed).unwrap();
        let block = installed.block();
        let (locator, coroots) = full_system_keying(&rc);
        let row_keys = RepTable::row_keys_for(&rc, &block, &locator, &coroots).unwrap();

        assert_eq!(block.size(), 12);
        // The two x=10 rows look like a tempting collision, but the pinned
        // transported projection is diag(2,2), hence Smith diagonal [2,2]:
        // gamma-lambda [0,0] and [1,0] have residues 0 and 2. Preserve this
        // evidence so a future pool does not silently merge distinct keys.
        assert_eq!(row_keys[10].0.residue, 0);
        assert_eq!(row_keys[11].0.residue, 2);
        assert_ne!(row_keys[10].0, row_keys[11].0);
        for row in 0..block.size() {
            let query = block
                .element(row)
                .unwrap()
                .to_standard(&rc, &gamma)
                .unwrap();
            let located = owner.lookup_full_block(&query).unwrap();
            assert_eq!(located.block_id(), installed.block_id(), "row {row}");
            assert_eq!(located.raw_row(), row, "row {row}");
            assert_eq!(
                located.adapted_representative(),
                block.element(row).unwrap(),
                "related query row {row}"
            );
        }
    }

    #[test]
    fn b2_reverse_registration_uses_the_smallest_row_for_a_duplicate_key() {
        let fixture = b2_fixture();
        let rc = fixture.rc();
        let gamma = RationalWeight::new(vec![2, 2], 1).unwrap();
        let seed = StandardReprMod::build(&rc, KgbId(0), &RationalWeight::zero(2).unwrap())
            .unwrap()
            .to_standard(&rc, &gamma)
            .unwrap();
        let context = CommonContext::integral(&rc, &gamma).unwrap();
        let seed = StandardReprMod::mod_reduce(&rc, &seed).unwrap();
        let (block, _) = PartialBlock::build_full(&context, &seed).unwrap();
        let block = Arc::new(block);
        let (locator, _) = full_system_keying(&rc);
        let duplicate = full_system_key(&rc, block.element(10).unwrap());
        let mut state = State::default();

        let (_, fresh_row) = state
            .commit_partial(
                Arc::clone(&block),
                duplicate,
                11,
                &[(duplicate, 10), (duplicate, 11)],
                locator.clone(),
            )
            .unwrap();

        assert_eq!(fresh_row, 11, "fresh materialization returns exact seed");
        assert_eq!(state.places[&duplicate].row, 10);

        let (_, existing_row) = state
            .commit_partial(
                block,
                duplicate,
                11,
                &[(duplicate, 10), (duplicate, 11)],
                locator,
            )
            .unwrap();
        assert_eq!(existing_row, 10, "later probes use the reverse place");
    }

    #[test]
    fn related_modifier_restores_an_integral_orthogonal_query() {
        let fixture = a1_t1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let gamma = RationalWeight::new(vec![2, 0], 1).unwrap();
        let stored =
            StandardReprMod::build(&rc, KgbId(1), &RationalWeight::zero(2).unwrap()).unwrap();
        let related =
            StandardReprMod::build(&rc, KgbId(1), &RationalWeight::new(vec![0, 1], 1).unwrap())
                .unwrap();
        let stored_key = full_system_key(&rc, &stored);
        let related_key = full_system_key(&rc, &related);
        assert_eq!(stored_key, related_key, "central difference is invisible");
        let codec = RepTable::full_integral_codec(&rc, stored.x()).unwrap();
        assert_eq!(
            codec.theta_1_preimage(&related.gamma_lambda().sub(stored.gamma_lambda()).unwrap()),
            Ok(Weight::new(vec![0, 0]))
        );
        let installed = owner
            .lookup_full_block(&stored.to_standard(&rc, &gamma).unwrap())
            .unwrap();

        let located = owner
            .lookup_full_block(&related.to_standard(&rc, &gamma).unwrap())
            .unwrap();

        assert_eq!(located.block_id(), installed.block_id());
        assert_eq!(located.raw_row(), 1);
        assert_eq!(located.adapted_representative(), &related);
        assert_eq!(
            located.relative_shift(),
            &RationalWeight::new(vec![0, 1], 1).unwrap()
        );
    }

    /// The anchor pair of `tests/fixtures/domain/common_block_locator.atlas`:
    /// p = `param(KGB(rf,3),[0,0],[2,1]/2)` installs the rank-one common
    /// block at its own (non-canonical) attitude; q =
    /// `param(KGB(rf,0),[0,0],[-2,-1]/2)` has a Weyl-conjugate integral
    /// system interning the SAME canonical datum, so its key collides with
    /// the stored block and the lookup must come back with the relative
    /// modifier the oracle prints as `<1>`.  The locator attitudes are
    /// hand-derived in block_modifier.rs `a2_sl3r_make_relative_to_round_trip`.
    #[test]
    fn weyl_conjugate_query_reuses_the_stored_block_at_nonidentity_attitude() {
        let fixture = sl3r_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let system = rc.root_system().clone();

        let (srm_p, p) = sl3r_param(&rc, 3, &RationalWeight::new(vec![2, 1], 2).unwrap());
        let (srm_q, q) = sl3r_param(&rc, 0, &RationalWeight::new(vec![-2, -1], 2).unwrap());

        let installed = owner.lookup_full_block(&p).unwrap();
        assert_eq!(installed.block().size(), 3);
        assert_eq!(installed.raw_row(), 1, "oracle: element 1");
        assert!(installed.has_identity_generator_attitude());
        // The fresh block normalizes the seed row representative to
        // (x=3, [0,1]/2), so the identity-attitude modifier carries the
        // compensating shift [0,1] (the pre-locator code computed the same
        // shift; upstream's zero shift relies on add_block keeping z
        // verbatim as the seed row).
        assert_eq!(
            installed.relative_shift(),
            &RationalWeight::new(vec![0, 1], 1).unwrap()
        );
        assert_eq!(installed.adapted_representative(), &srm_p);

        let located = owner.lookup_full_block(&q).unwrap();
        assert_eq!(located.block_id(), installed.block_id());
        assert_eq!(located.raw_row(), 0, "oracle: element 0");
        assert!(!located.has_identity_generator_attitude());
        let modifier = located.block_modifier();
        assert_eq!(
            modifier.w().reduced_word(&system).unwrap(),
            vec![1],
            "oracle: as transformed by <1>"
        );
        assert_eq!(modifier.simple_pi(), &[0]);
        assert_eq!(
            located.relative_shift(),
            &RationalWeight::new(vec![0, 1], 4).unwrap()
        );
        assert_eq!(
            located.adapted_representative(),
            &srm_q,
            "round trip restores the query srm"
        );

        // Re-querying p keeps the stored block's identity attitude.
        let again = owner.lookup_full_block(&p).unwrap();
        assert_eq!(again.block_id(), installed.block_id());
        assert_eq!(again.raw_row(), 1);
        assert!(again.has_identity_generator_attitude());
        assert_eq!(again.relative_shift(), installed.relative_shift());

        // A partial lookup of q resolves to the same full block, still at
        // the relative attitude.
        let partial = owner.lookup(&q).unwrap();
        assert_eq!(partial.block_id(), installed.block_id());
        assert!(!partial.has_identity_generator_attitude());
    }

    /// The rank-zero anchor of
    /// `tests/fixtures/domain/common_block_rank0_locator.atlas`: p0 =
    /// `param(KGB(rf,3),[0,0],[-3,-3]/4)` installs a singleton block; q0 =
    /// `param(KGB(rf,3),[0,0],[-3,1]/4)` has no integral roots either, so
    /// both intern the empty canonical datum and collide, with the
    /// attitude difference the oracle prints as `<0.1.0>`.
    #[test]
    fn rank_zero_query_reuses_the_singleton_block_at_nonidentity_attitude() {
        let fixture = sl3r_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let system = rc.root_system().clone();

        let p0 = sl3r_param(&rc, 3, &RationalWeight::new(vec![-3, -3], 4).unwrap()).1;
        let q0 = sl3r_param(&rc, 3, &RationalWeight::new(vec![-3, 1], 4).unwrap()).1;

        let installed = owner.lookup_full_block(&p0).unwrap();
        assert_eq!(installed.block().size(), 1);
        assert_eq!(installed.raw_row(), 0);
        assert!(installed.has_identity_generator_attitude());

        let located = owner.lookup_full_block(&q0).unwrap();
        assert_eq!(located.block_id(), installed.block_id());
        assert_eq!(located.raw_row(), 0);
        assert!(!located.has_identity_generator_attitude());
        let modifier = located.block_modifier();
        assert_eq!(
            modifier.w().reduced_word(&system).unwrap(),
            vec![0, 1, 0],
            "oracle: as transformed by <0.1.0>"
        );
        assert_eq!(modifier.simple_pi(), &[] as &[usize]);
        // Stored row [7,7]/4 shifted by [-5,-4]/4 reads back as the
        // oracle's printed [2,3]/4 before the closing transform.
        assert_eq!(
            located.relative_shift(),
            &RationalWeight::new(vec![-5, -4], 4).unwrap()
        );
        let expected = StandardReprMod::mod_reduce(&rc, &q0.made_dominant(&rc).unwrap()).unwrap();
        assert_eq!(located.adapted_representative(), &expected);
    }

    #[test]
    fn full_promotion_retires_every_partial_and_clears_all_places() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let gamma = RationalWeight::new(vec![1], 1).unwrap();
        let zero = RationalWeight::zero(1).unwrap();
        let query = |x| {
            StandardReprMod::build(&rc, KgbId(x), &zero)
                .unwrap()
                .to_standard(&rc, &gamma)
                .unwrap()
        };
        let left = owner.lookup(&query(0)).unwrap();
        let right = owner.lookup(&query(1)).unwrap();
        assert_ne!(left.block_id(), right.block_id());
        assert_eq!(owner.blocks.state_counts(), (2, 2, 2));

        let full = owner.lookup_full_block(&query(2)).unwrap();

        assert_eq!(full.block_id().index(), 2);
        assert_eq!(owner.blocks.state_counts(), (3, 1, 3));
        assert!(owner.blocks.block(left.block_id()).unwrap().is_none());
        assert!(owner.blocks.block(right.block_id()).unwrap().is_none());
        assert_eq!(left.block().size(), 1, "existing Arc remains valid");
        assert_eq!(right.block().size(), 1, "existing Arc remains valid");
        for x in 0..3 {
            assert_eq!(owner.lookup(&query(x)).unwrap().block_id(), full.block_id());
        }
    }

    #[test]
    fn unsupported_partial_overlap_is_failure_atomic() {
        let fixture = a1_fixture();
        let owner = owner(&fixture);
        let rc = owner.context();
        let gamma = RationalWeight::new(vec![1], 1).unwrap();
        let zero = RationalWeight::zero(1).unwrap();
        let query = |x| {
            StandardReprMod::build(&rc, KgbId(x), &zero)
                .unwrap()
                .to_standard(&rc, &gamma)
                .unwrap()
        };
        let first = owner.lookup(&query(0)).unwrap();
        let before = owner.blocks.state_counts();

        assert!(matches!(
            owner.lookup(&query(2)),
            Err(StructureError::NotYetImplemented {
                feature: "merging overlapping partial representation blocks"
            })
        ));
        assert_eq!(owner.blocks.state_counts(), before);
        assert!(owner.blocks.block(first.block_id()).unwrap().is_some());
        assert_eq!(
            owner.lookup(&query(0)).unwrap().block_id(),
            first.block_id()
        );
    }

    #[test]
    fn owners_keep_representation_blocks_isolated() {
        let first = a1_fixture();
        let second = a1_fixture();
        let first_owner = owner(&first);
        let second_owner = owner(&second);
        let first_rc = first_owner.context();
        let second_rc = second_owner.context();

        let first_block = first_owner.lookup(&a1_query(&first_rc, 0)).unwrap();
        let second_block = second_owner.lookup(&a1_query(&second_rc, 1)).unwrap();

        assert_eq!(first_block.block_id().index(), 0);
        assert_eq!(second_block.block_id().index(), 0);
        assert_eq!(first_owner.blocks.state_counts(), (1, 1, 1));
        assert_eq!(second_owner.blocks.state_counts(), (1, 1, 1));
    }

    #[test]
    fn arc_clones_share_one_owner_representation_table() {
        let fixture = a1_fixture();
        let owner = Arc::new(owner(&fixture));
        let first = Arc::clone(&owner);
        let second = Arc::clone(&owner);

        let ids = std::thread::scope(|scope| {
            let first_worker = scope.spawn(move || {
                let rc = first.context();
                first
                    .lookup_full_block(&a1_query(&rc, 1))
                    .unwrap()
                    .block_id()
            });
            let second_worker = scope.spawn(move || {
                let rc = second.context();
                second
                    .lookup_full_block(&a1_query(&rc, 2))
                    .unwrap()
                    .block_id()
            });
            [first_worker.join().unwrap(), second_worker.join().unwrap()]
        });

        assert_eq!(ids, [BlockId(0), BlockId(0)]);
        assert_eq!(owner.blocks.state_counts(), (1, 1, 3));
    }

    #[test]
    fn concurrent_full_materialization_commits_one_active_record() {
        let fixture = a1_fixture();
        let first_gate = TestGate::new();
        let second_gate = TestGate::new();
        let owner = Arc::new(
            RepTableOwner::with_test_gates(
                fixture.table,
                fixture.graph,
                Vec::new(),
                vec![first_gate.clone(), second_gate.clone()],
            )
            .unwrap(),
        );
        let ids = std::thread::scope(|scope| {
            let mut workers = Vec::new();
            for (index, gate) in [first_gate.clone(), second_gate.clone()]
                .into_iter()
                .enumerate()
            {
                let owner = Arc::clone(&owner);
                workers.push(scope.spawn(move || {
                    let rc = owner.context();
                    owner
                        .lookup_full_block(&a1_query(&rc, index + 1))
                        .unwrap()
                        .block_id()
                }));
                gate.wait_until_reached();
            }
            first_gate.release();
            second_gate.release();
            workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .collect::<Vec<_>>()
        });

        assert_eq!(ids, vec![BlockId(0), BlockId(0)]);
        assert_eq!(owner.blocks.state_counts(), (1, 1, 3));
        assert!(owner.blocks.state_is_consistent());
    }

    #[test]
    fn full_waiting_to_commit_retires_a_partial_committed_during_materialization() {
        let fixture = a1_fixture();
        let full_gate = TestGate::new();
        let owner = Arc::new(
            RepTableOwner::with_test_gates(
                fixture.table,
                fixture.graph,
                Vec::new(),
                vec![full_gate.clone()],
            )
            .unwrap(),
        );
        let (partial, full) = std::thread::scope(|scope| {
            let worker_owner = Arc::clone(&owner);
            let full_worker = scope.spawn(move || {
                let rc = worker_owner.context();
                worker_owner.lookup_full_block(&a1_query(&rc, 2)).unwrap()
            });
            full_gate.wait_until_reached();

            let rc = owner.context();
            let partial = owner.lookup(&a1_query(&rc, 0)).unwrap();
            assert_eq!(owner.blocks.state_counts(), (1, 1, 1));
            full_gate.release();
            (partial, full_worker.join().unwrap())
        });

        assert_eq!(owner.blocks.state_counts(), (2, 1, 3));
        assert_eq!(owner.blocks.state_snapshot().slots, vec![None, Some(true)]);
        assert!(owner.blocks.block(partial.block_id()).unwrap().is_none());
        assert_eq!(partial.block().size(), 1, "retired handle remains valid");
        assert!(full.is_full());
        assert!(owner.blocks.state_is_consistent());
    }

    #[test]
    fn partial_waiting_to_commit_reuses_a_full_committed_during_materialization() {
        let fixture = a1_fixture();
        let partial_gate = TestGate::new();
        let owner = Arc::new(
            RepTableOwner::with_test_gates(
                fixture.table,
                fixture.graph,
                vec![partial_gate.clone()],
                Vec::new(),
            )
            .unwrap(),
        );
        let (full, partial_result) = std::thread::scope(|scope| {
            let worker_owner = Arc::clone(&owner);
            let partial_worker = scope.spawn(move || {
                let rc = worker_owner.context();
                worker_owner.lookup(&a1_query(&rc, 2)).unwrap()
            });
            partial_gate.wait_until_reached();

            let rc = owner.context();
            let full = owner.lookup_full_block(&a1_query(&rc, 1)).unwrap();
            partial_gate.release();
            (full, partial_worker.join().unwrap())
        });

        assert_eq!(partial_result.block_id(), full.block_id());
        assert!(partial_result.is_full());
        assert_eq!(owner.blocks.state_counts(), (1, 1, 3));
        assert!(owner.blocks.state_is_consistent());
    }

    #[test]
    fn concurrent_overlapping_partials_leave_the_first_commit_unchanged() {
        let fixture = a1_fixture();
        let first_gate = TestGate::new();
        let overlap_gate = TestGate::new();
        let owner = Arc::new(
            RepTableOwner::with_test_gates(
                fixture.table,
                fixture.graph,
                vec![first_gate.clone(), overlap_gate.clone()],
                Vec::new(),
            )
            .unwrap(),
        );

        let (first, before) = std::thread::scope(|scope| {
            let first_owner = Arc::clone(&owner);
            let first_worker = scope.spawn(move || {
                let rc = first_owner.context();
                first_owner.lookup(&a1_query(&rc, 0))
            });
            first_gate.wait_until_reached();

            let overlap_owner = Arc::clone(&owner);
            let overlap_worker = scope.spawn(move || {
                let rc = overlap_owner.context();
                overlap_owner.lookup(&a1_query(&rc, 2))
            });
            overlap_gate.wait_until_reached();

            first_gate.release();
            let first = first_worker.join().unwrap().unwrap();
            let before = owner.blocks.state_snapshot();
            overlap_gate.release();
            assert!(matches!(
                overlap_worker.join().unwrap(),
                Err(StructureError::NotYetImplemented {
                    feature: "merging overlapping partial representation blocks"
                })
            ));
            (first, before)
        });

        assert_eq!(owner.blocks.state_snapshot(), before);
        assert!(owner.blocks.block(first.block_id()).unwrap().is_some());
        assert!(owner.blocks.state_is_consistent());
    }
}
