---
title: StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）
source: atlas-rust/error-global-tits
ingestedAt: 2026-10-06T00:00:00Z
---

# 来源包草案：`crates/atlas-real-group/src/error.rs` 与 `crates/atlas-real-group/src/global_tits.rs`

## 0. 范围与标注约定

- 本草案**只依据任务给出的两个文件的完整字节**：`error.rs`（全文）与 `global_tits.rs`（全文，含其 `#[cfg(test)]` 模块）。凡涉及其它模块（如 `InnerClass`、`TwistedInvolution`、`WeylAction`、`RationalCoweight`、`crate::grading::try_capacity`、`crate::twisted_involution::compose_matrices`、`crate::TitsCoset` 等）的陈述，均仅为「调用点可见」的事实，其定义未读本。
- 标注约定：
  - 【实现】= 直接可从所给字节读到的事实（签名、分支、文案、测试断言）。
  - 【推断】= 阅读推断或编辑性归纳，需维护者核对后方可收录。
- 本草案**不做任何数学验收、性能或正确性声明**；对公式、规范化区间等仅转述代码与文档注释原文。

---

## 1. `error.rs`

### 1.1 裸签名清单（完整）

```rust
use std::fmt;
use crate::lattice::Weight;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructureError {
    EmptyRootDatum,
    NonSquareCartan,
    InvalidCartanMatrix,
    RankMismatch { expected: usize, actual: usize },
    DatumMismatch,
    IndexOutOfRange { index: usize, upper_bound: usize },
    InvalidInvolution,
    InvalidBasedAutomorphism,
    InvalidRootAutomorphism,
    InvalidRootDatumAutomorphism,
    SimpleRootImageNotRoot { simple_root: usize },
    SimpleCorootImageMismatch { simple_root: usize, image_root: Weight },
    RootSystemTooLarge,
    WeylGroupTooLarge,
    ResourceLimitExceeded { limit: usize },
    AllocationFailed { requested: usize },
    InvalidIntegerMatrixShape,
    IntegerLatticeInvariantViolation,
    IntegerLatticeResourceLimit { resource: &'static str, limit: u64 },
    RootSystemResourceLimit { resource: &'static str, limit: usize },
    RootSystemInvariantViolation { invariant: &'static str },
    GradingShiftsNotFaithful,
    ImpossibleGrading,
    InvalidStrongTorusFactor,
    WeakRealFormResourceLimit { resource: &'static str, limit: usize },
    CayleyCrossResourceLimit { resource: &'static str, limit: usize },
    CayleyCrossInvariantViolation { invariant: &'static str },
    RealFormLabelInvariantViolation { invariant: &'static str },
    CartanClassificationInvariantViolation { invariant: &'static str },
    StrongRealResourceLimit { resource: &'static str, limit: usize },
    StrongRealInvariantViolation { invariant: &'static str },
    WeylElementInvariantViolation { invariant: &'static str },
    InvolutionTableResourceLimit { resource: &'static str, limit: usize },
    InvolutionTableInvariantViolation { invariant: &'static str },
    TitsCosetInvariantViolation { invariant: &'static str },
    SeedResourceLimit { resource: &'static str, limit: usize },
    SeedInvariantViolation { invariant: &'static str },
    KgbInvariantViolation { invariant: &'static str },
    BlockInvariantViolation { invariant: &'static str },
    RepInvariantViolation { invariant: &'static str },
    RealFormOrderInvariantViolation { invariant: &'static str },
    LayoutInvariantViolation { invariant: &'static str },
    DistinguishedInvolutionMismatch,
    ModTwoSubquotientInvariantViolation,
    NotInModTwoSubspace,
    CartanFiberMismatch,
    CartanFiberInvolutionMismatch,
    CartanFiberMapDoesNotDescend { relation: &'static str },
    AdjointFiberResourceLimit { resource: &'static str, limit: usize },
    ArithmeticOverflow,
    /// A code path whose behavior the upstream oracle defines but this
    /// crate has not ported yet; raised loudly instead of computing on a
    /// wrong approximation.
    NotYetImplemented { feature: &'static str },
    RootPairingMismatch { row: usize, column: usize, expected: i32, actual: i32 },
    RealFormNotDefinedOnCartan,
}

impl fmt::Display for StructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl std::error::Error for StructureError {}
```

【实现】要点：
- 共 **53 个变体**：22 个无字段（unit）变体 + 31 个带字段变体。
- 派生：`Clone, Debug, Eq, PartialEq`。
- `impl std::error::Error for StructureError {}` 为空实现体（未覆盖 `source()` 等默认方法）。
- 文件内**无** `pub fn`、无构造函数、无 `From` 转换实现、无测试模块。
- 唯一带文档注释的变体是 `NotYetImplemented`（注释原文："A code path whose behavior the upstream oracle defines but this crate has not ported yet; raised loudly instead of computing on a wrong approximation."）。
- 本文件唯一的外部类型依赖：`crate::lattice::Weight`（用于 `SimpleCorootImageMismatch.image_root`）。

### 1.2 字段形态家族【实现】

| 字段形态 | 变体 |
|---|---|
| 无字段 | `EmptyRootDatum`, `NonSquareCartan`, `InvalidCartanMatrix`, `DatumMismatch`, `InvalidInvolution`, `InvalidBasedAutomorphism`, `InvalidRootAutomorphism`, `InvalidRootDatumAutomorphism`, `RootSystemTooLarge`, `WeylGroupTooLarge`, `InvalidIntegerMatrixShape`, `IntegerLatticeInvariantViolation`, `GradingShiftsNotFaithful`, `ImpossibleGrading`, `InvalidStrongTorusFactor`, `DistinguishedInvolutionMismatch`, `ModTwoSubquotientInvariantViolation`, `NotInModTwoSubspace`, `CartanFiberMismatch`, `CartanFiberInvolutionMismatch`, `ArithmeticOverflow`, `RealFormNotDefinedOnCartan` |
| `{ expected: usize, actual: usize }` | `RankMismatch` |
| `{ index: usize, upper_bound: usize }` | `IndexOutOfRange` |
| `{ limit: usize }` | `ResourceLimitExceeded` |
| `{ requested: usize }` | `AllocationFailed` |
| `{ resource: &'static str, limit: u64 }` | `IntegerLatticeResourceLimit`（**唯一** `limit: u64`） |
| `{ resource: &'static str, limit: usize }` | `RootSystemResourceLimit`, `WeakRealFormResourceLimit`, `CayleyCrossResourceLimit`, `StrongRealResourceLimit`, `InvolutionTableResourceLimit`, `SeedResourceLimit`, `AdjointFiberResourceLimit`（共 7 个） |
| `{ invariant: &'static str }` | `RootSystemInvariantViolation`, `CayleyCrossInvariantViolation`, `RealFormLabelInvariantViolation`, `CartanClassificationInvariantViolation`, `StrongRealInvariantViolation`, `WeylElementInvariantViolation`, `InvolutionTableInvariantViolation`, `TitsCosetInvariantViolation`, `SeedInvariantViolation`, `KgbInvariantViolation`, `BlockInvariantViolation`, `RepInvariantViolation`, `RealFormOrderInvariantViolation`, `LayoutInvariantViolation`（共 14 个） |
| `{ simple_root: usize }` | `SimpleRootImageNotRoot` |
| `{ simple_root: usize, image_root: Weight }` | `SimpleCorootImageMismatch`（唯一含 `Weight`；Display 用 `{image_root:?}` 即 Debug 格式化） |
| `{ relation: &'static str }` | `CartanFiberMapDoesNotDescend` |
| `{ feature: &'static str }` | `NotYetImplemented` |
| `{ row: usize, column: usize, expected: i32, actual: i32 }` | `RootPairingMismatch`（唯一含 `i32` 字段） |

### 1.3 变体语义分组（编辑性分组 = 【推断】；Display 文案 = 【实现】原文）

文件本身没有分组注释；以下分组是按命名前缀与 Display 文案归纳的编辑性分类，供维护者核对。

**G1 根数据 / 格输入验证**

| 变体 | Display 原文 |
|---|---|
| `EmptyRootDatum` | `root datum must have positive rank` |
| `NonSquareCartan` | `Cartan matrix must be square` |
| `InvalidCartanMatrix` | `invalid generalized Cartan matrix` |
| `RankMismatch` | `lattice rank mismatch: expected {expected}, got {actual}` |
| `DatumMismatch` | `operations require the same based root datum` |
| `IndexOutOfRange` | `index {index} is outside the range 0..{upper_bound}` |
| `RootPairingMismatch` | `root pairing ({row},{column}) is {actual}, expected {expected}` |

**G2 对合与自同构合法性**

| 变体 | Display 原文 |
|---|---|
| `InvalidInvolution` | `Cartan involution is not an involution` |
| `InvalidBasedAutomorphism` | `distinguished involution does not preserve the simple root/coroot system` |
| `InvalidRootAutomorphism` | `root automorphism does not preserve the Cartan pairing` |
| `InvalidRootDatumAutomorphism` | `root permutation does not transport coroots to coroots` |
| `SimpleRootImageNotRoot` | `simple root {simple_root} does not map to a root` |
| `SimpleCorootImageMismatch` | `simple coroot {simple_root} does not map to the coroot of image root {image_root:?}` |
| `DistinguishedInvolutionMismatch` | `twisted involution does not factor through this inner class's distinguished involution` |

**G3 分级、纤维与强对合**

| 变体 | Display 原文 |
|---|---|
| `GradingShiftsNotFaithful` | `grading shifts do not determine a unique fiber element` |
| `ImpossibleGrading` | `no fiber element has the requested grading` |
| `InvalidStrongTorusFactor` | `Torus factor does not define a valid strong involution` |
| `ModTwoSubquotientInvariantViolation` | `mod-two subquotient invariant was violated` |
| `NotInModTwoSubspace` | `mod-two vector does not belong to the required subspace` |
| `CartanFiberMismatch` | `Cartan-fiber elements belong to different fibers` |
| `CartanFiberInvolutionMismatch` | `Cartan-fiber map requires the same Cartan involution` |
| `CartanFiberMapDoesNotDescend` | `Cartan-fiber map does not preserve the {relation} relation` |
| `AdjointFiberResourceLimit` | `adjoint-fiber {resource} exceeded its limit of {limit}` |
| `RealFormNotDefinedOnCartan` | `Cartan class not defined for real form` |

**G4 规模与资源预算**

| 变体 | Display 原文 |
|---|---|
| `RootSystemTooLarge` | `root-system closure exceeded the finite-system limit` |
| `WeylGroupTooLarge` | `Weyl-group closure exceeded the finite-system limit` |
| `ResourceLimitExceeded` | `enumeration exceeded its resource limit of {limit}` |
| `AllocationFailed` | `could not reserve storage for {requested} elements` |
| `IntegerLatticeResourceLimit` | `integer-lattice {resource} exceeded its limit of {limit}` |
| `RootSystemResourceLimit` | `root-system {resource} exceeded its limit of {limit}` |
| `WeakRealFormResourceLimit` | `weak-real-form {resource} exceeded its limit of {limit}` |
| `CayleyCrossResourceLimit` | `Cayley-cross {resource} exceeded its limit of {limit}` |
| `StrongRealResourceLimit` | `strong-real {resource} exceeded its limit of {limit}` |
| `InvolutionTableResourceLimit` | `involution-table {resource} exceeded its limit of {limit}` |
| `SeedResourceLimit` | `seed {resource} exceeded its limit of {limit}` |

**G5 子系统不变量违例（命名前缀即子系统名）**

| 变体 | Display 原文 |
|---|---|
| `IntegerLatticeInvariantViolation` | `integer-lattice reduction invariant was violated`（无字段，形态与其它 invariant 变体不同） |
| `RootSystemInvariantViolation` | `root-system {invariant} invariant was violated` |
| `CayleyCrossInvariantViolation` | `Cayley-cross {invariant} invariant was violated` |
| `RealFormLabelInvariantViolation` | `real-form-label {invariant} invariant was violated` |
| `CartanClassificationInvariantViolation` | `Cartan-classification {invariant} invariant was violated` |
| `StrongRealInvariantViolation` | `strong-real {invariant} invariant was violated` |
| `WeylElementInvariantViolation` | `Weyl-element {invariant} invariant was violated` |
| `InvolutionTableInvariantViolation` | `involution-table {invariant} invariant was violated` |
| `TitsCosetInvariantViolation` | `Tits-coset {invariant} invariant was violated` |
| `SeedInvariantViolation` | `seed {invariant} invariant was violated` |
| `KgbInvariantViolation` | `KGB {invariant} invariant was violated` |
| `BlockInvariantViolation` | `block {invariant} invariant was violated` |
| `RepInvariantViolation` | `rep-context {invariant} invariant was violated` |
| `RealFormOrderInvariantViolation` | `real-form order {invariant} invariant was violated` |
| `LayoutInvariantViolation` | `inner-class layout {invariant} invariant was violated` |

**G6 算术与整数矩阵**

| 变体 | Display 原文 |
|---|---|
| `InvalidIntegerMatrixShape` | `integer matrix has an invalid shape` |
| `ArithmeticOverflow` | `root-system arithmetic overflow` |

**G7 移植状态**

| 变体 | Display 原文 |
|---|---|
| `NotYetImplemented` | `{feature} is not yet implemented` |

---

## 2. `global_tits.rs`

### 2.1 模块定位（模块文档原文要点）【实现】

- 模块文档："Exact rational torus transport for Atlas's global Tits cross action."
- 与 `crate::TitsCoset` 不同，本载体保留**完整有理余特征（full rational cocharacter），含中心坐标（central coordinates）**。
- 用途（文档原文语义）：在「合成强对合（synthetic strong involution）被移动到典范 Cartan 代表元」期间所需的传输；向纤维的 mod-two 商的规约**发生在之后**（"reduction to a fiber's mod-two quotient happens later"）。
- 结构体文档：环面坐标是 `[0, 2)` 中的典范代表元；字段保持私有，使每个值都经过 rank、datum、distinguished-involution 三重来源门槛（provenance gates）。

### 2.2 裸签名清单（完整，含私有项与测试模块）

```rust
// 依赖导入【实现】
use malachite::base::num::arithmetic::traits::Floor;
use malachite::base::num::basic::traits::Zero;
use malachite::Rational;
use crate::grading::try_capacity;
use crate::twisted_involution::compose_matrices;
use crate::{
    InnerClass, RationalCoweight, RootKind, StructureError, TwistedInvolution, Weight, WeylAction,
};

/// (文档：环面坐标为 [0,2) 典范代表元；字段私有)
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GlobalTitsElement {
    torus_factor: RationalCoweight,        // 私有字段
    twisted_involution: TwistedInvolution, // 私有字段
}

impl GlobalTitsElement {
    pub(crate) fn new(
        inner_class: &InnerClass,
        torus_factor: RationalCoweight,
        twisted_involution: TwistedInvolution,
    ) -> Result<Self, StructureError>;

    pub(crate) fn torus_factor(&self) -> &RationalCoweight;

    pub(crate) fn twisted_involution(&self) -> &TwistedInvolution;

    /// Apply one simple cross action `s * (t,w) * delta(s)`.
    pub(crate) fn crossed_generator(
        &self,
        inner_class: &InnerClass,
        generator: usize,
    ) -> Result<Self, StructureError>;

    /// Apply generators from first to last, matching upstream
    /// `cross_act(GlobalTitsElement&, const WeylWord&)`.
    pub(crate) fn crossed_word(
        &self,
        inner_class: &InnerClass,
        word: &[usize],
    ) -> Result<Self, StructureError>;
}

// 以下为模块私有自由函数（无 pub 标记）【实现】
fn validate_context(
    inner_class: &InnerClass,
    twisted: &TwistedInvolution,
) -> Result<(), StructureError>;

fn distinguished_generator_image(
    inner_class: &InnerClass,
    generator: usize,
) -> Result<usize, StructureError>;

fn rational_pair(weight: &Weight, coweight: &[Rational]) -> Result<Rational, StructureError>;

fn add_scaled_coroot(
    coordinates: &mut [Rational],
    coroot: &crate::Coweight,   // 注意：签名中使用全路径 crate::Coweight
    scale: Rational,
) -> Result<(), StructureError>;

fn normalize_torus_factor(coordinates: &[Rational]) -> Result<RationalCoweight, StructureError>;

fn copy_rationals(coordinates: &[Rational]) -> Result<Vec<Rational>, StructureError>;

fn normalize_coordinates(coordinates: &mut [Rational]);   // 无返回值、无错误分支

#[cfg(test)]
mod tests {
    // 辅助：fn rational(i32, i32) -> Rational
    // 辅助：fn compact_inner(BasedRootDatum, usize) -> InnerClass
    // 辅助：fn twisted(&InnerClass, WeylAction) -> TwistedInvolution
    // 辅助：fn element(&InnerClass, Vec<Rational>, WeylAction) -> GlobalTitsElement
    // 辅助：fn adjoint_a1() -> BasedRootDatum
    // 10 个 #[test] 函数，见 §4
}
```

【实现】本文件**没有任何 `pub` 项**；最高可见性为 `pub(crate)`。派生 `Clone, Debug, Eq, PartialEq`（`Eq`/`PartialEq` 被测试直接用于整元素比较）。

### 2.3 构造门槛：`new` 的失败分支与顺序【实现】

按代码顺序：

1. `rank = inner_class.datum().lattice_rank()`；若 `torus_factor.dimension() != rank` → 返回 `Err(StructureError::RankMismatch { expected: rank, actual: torus_factor.dimension() })`。
2. `validate_context(inner_class, &twisted_involution)?`（其内部分支见 §2.6：可能返回 `DatumMismatch`、`DistinguishedInvolutionMismatch`，并传播 `compose_matrices` 的错误）。
3. 存储 `normalize_torus_factor(torus_factor.coordinates())?`（复制 → 逐坐标 mod 2 规范化 → `RationalCoweight::from_coordinates`），`twisted_involution` 原样存储（不再变换）。

两个访问器 `torus_factor()` / `twisted_involution()` 仅返回不可变引用，无失败分支。

### 2.4 `crossed_generator` 流程与错误分支【实现】

文档注释原文："Apply one simple cross action `s * (t,w) * delta(s)`." 按代码顺序：

1. `validate_context(inner_class, &self.twisted_involution)?` —— 每次调用**重新**验证。
2. `semisimple_rank = inner_class.datum().semisimple_rank()`；若 `generator >= semisimple_rank` → `Err(IndexOutOfRange { index: generator, upper_bound: semisimple_rank })`。
3. `roots.simple_root_ids().get(generator)` 缺失 → `Err(IndexOutOfRange { index: generator, upper_bound: roots.simple_root_ids().len() })`（防御性分支；与第 2 步的关系不可从这两文件判定【推断】）。
4. `self.twisted_involution.root_involution().kind(simple_root)` 为 `None` → `Err(InvalidRootAutomorphism)`。
5. 复制环面坐标（`copy_rationals`，经 `try_capacity` 预算门槛），按 `RootKind` 三分支就地更新：
   - `RootKind::Complex`：`pairing = rational_pair(&datum.simple_roots()[generator], &coords)?`；`add_scaled_coroot(&mut coords, &datum.simple_coroots()[generator], -pairing)?`。即 `t ← t − pairing·α∨`（转述代码，不作数学断言）。
   - `RootKind::Imaginary`：先算 `pairing`；**若 `pairing != pairing.clone().floor()` → `Err(InvalidStrongTorusFactor)`**（整性门槛）；否则以 `Rational::from(1) - pairing` 为缩放调用 `add_scaled_coroot`。
   - `RootKind::Real`：坐标不变（空分支）。
6. `normalize_coordinates(&mut coords)`：逐坐标 `coordinate -= floor(coordinate/2) * 2`（代码形态：`let quotient = (coordinate.clone() / Rational::from(2)).floor(); *coordinate -= Rational::from(quotient) * Rational::from(2);`）。
7. Weyl 侧更新：`twisted_generator = distinguished_generator_image(inner_class, generator)?`；`left = WeylAction::simple_reflection(datum, generator)?`；`right = WeylAction::simple_reflection(datum, twisted_generator)?`；`action = left.compose(self.twisted_involution.weyl_action())?.compose(&right)?`；`TwistedInvolution::new(datum, roots, inner_class.distinguished_involution().involution(), action)?`。
8. 返回新 `Self`（`&self` 方法，返回新值；原值不变）。

注意【实现】：`datum.simple_roots()[generator]` 与 `datum.simple_coroots()[generator]` 使用**直接下标索引**（非 `.get`），其越界保护依赖第 2 步的 `generator < semisimple_rank`；`simple_roots()` 长度与 `semisimple_rank` 的关系不在本两文件内【推断：存在标准 slice 越界 panic 的潜在面，是否有上层不变量保证不可判定】。

### 2.5 `crossed_word` 顺序保证【实现】

- 文档原文："Apply generators from first to last, matching upstream `cross_act(GlobalTitsElement&, const WeylWord&)`."
- 实现：先 `validate_context` 一次；`let mut current = self.clone();`，随后**按切片顺序**（`for &generator in word`）逐个 `current.crossed_generator(inner_class, generator)?` 折叠。
- 顺序语义被测试 `a2_word_execution_is_forward_and_noncommuting` 锚定（见 §4）：`[0,1]` 等价于先 0 后 1，且与 `[1,0]` 结果不同。
- 错误语义【实现+推断】：word 中非法 generator 不在入口处预检，而是在折叠到该元素时由 `crossed_generator` 报 `IndexOutOfRange`（该路径无直接测试覆盖，属推断）。

### 2.6 私有辅助函数契约【实现】

- `validate_context`：
  - 若 `twisted.weyl_action().datum() != inner_class.datum()` **或** `twisted.root_involution().involution().datum() != inner_class.datum()` → `Err(DatumMismatch)`。
  - 否则比较矩阵复合：`compose_matrices(twisted.weyl_action().matrix(), distinguished.weight_matrix())? != stored.weight_matrix()` **或** `compose_matrices(twisted.weyl_action().coweight_matrix(), distinguished.coweight_matrix())? != stored.coweight_matrix()` → `Err(DistinguishedInvolutionMismatch)`（其中 `distinguished = inner_class.distinguished_involution().involution()`，`stored = twisted.root_involution().involution()`；`compose_matrices` 自身错误经 `?` 传播，变体不可判定）。
- `distinguished_generator_image`：`simple_root_ids().get(generator)` 缺失 → `IndexOutOfRange { index: generator, upper_bound: simple_roots.len() }`；`distinguished_involution().image(simple_root)` 为 `None` → `InvalidBasedAutomorphism`；像不在 `simple_root_ids` 中（`position` 查找失败）→ `InvalidBasedAutomorphism`。
- `rational_pair(weight, coweight)`：`weight.rank() != coweight.len()` → `RankMismatch { expected: weight.rank(), actual: coweight.len() }`；否则自 `Rational::ZERO` 起折叠累加 `Rational::from(coefficient) * coordinate`（精确有理数点积）。
- `add_scaled_coroot(coordinates, coroot, scale)`：`coordinates.len() != coroot.rank()` → `RankMismatch { expected: coroot.rank(), actual: coordinates.len() }`；否则逐坐标就地 `*coordinate += Rational::from(direction) * &scale`。
- `normalize_torus_factor` = `copy_rationals` → `normalize_coordinates` → `RationalCoweight::from_coordinates`。
- `copy_rationals`：`try_capacity(coordinates.len())?`（来自 `crate::grading`，**预算/分配门槛**；其错误变体不在本两文件内，不可判定【推断：与 `AllocationFailed` 形态相符，但未证实】）后 `extend` 克隆。
- `normalize_coordinates`：无错误分支；逐坐标减去 `floor(coordinate/2)*2`。

---

## 3. 两文件的接口关系

- 【实现】依赖方向**单向**：`global_tits.rs` 通过 `use crate::{..., StructureError, ...}` 依赖 `error.rs` 定义的 `StructureError`；`error.rs` 不引用 `global_tits.rs` 的任何符号（其唯一 crate 内依赖是 `crate::lattice::Weight`）。
- 【推断】`crate::StructureError` 的写法暗示 crate 根有重导出，但重导出位置不在所给字节内。
- 【实现】`global_tits.rs` **直接构造**的 `StructureError` 变体共 7 个：

| 变体 | 构造点 | 触发条件 |
|---|---|---|
| `RankMismatch` | `new`；`rational_pair`；`add_scaled_coroot` | 环面因子维数 ≠ `lattice_rank()`；`weight.rank() != coweight.len()`；`coordinates.len() != coroot.rank()` |
| `IndexOutOfRange` | `crossed_generator`（两处）；`distinguished_generator_image` | `generator >= semisimple_rank`；`simple_root_ids().get(generator)` 缺失 |
| `InvalidRootAutomorphism` | `crossed_generator` | `root_involution().kind(simple_root)` 返回 `None` |
| `InvalidStrongTorusFactor` | `crossed_generator` 的 `RootKind::Imaginary` 分支 | `pairing != pairing.clone().floor()`（根配对非整） |
| `DatumMismatch` | `validate_context` | weyl_action 或 root_involution 的 datum 与 `inner_class.datum()` 不同 |
| `DistinguishedInvolutionMismatch` | `validate_context` | `w·δ` 的 weight/coweight 矩阵与存储对合矩阵不符（任一不等即错） |
| `InvalidBasedAutomorphism` | `distinguished_generator_image` | distinguished involution 的像缺失或像不在单根集合中 |

- 【实现】其余 46 个变体（53 − 7）在这两文件范围内**没有**被 `global_tits.rs` 构造。
- 【实现】以下外部调用以 `?` 传播错误进入 `global_tits.rs` 的 `Result<_, StructureError>`，但**具体产生哪些变体不可从这两文件判定**：`try_capacity`、`compose_matrices`、`WeylAction::simple_reflection`、`WeylAction::compose`、`TwistedInvolution::new`。
- 【推断】命名对齐观察：`error.rs` 存在 `TitsCosetInvariantViolation`，`global_tits.rs` 模块文档显式对照 `crate::TitsCoset`；`NotYetImplemented` 的注释提到 "upstream oracle"，`crossed_word` 文档提到 upstream `cross_act(GlobalTitsElement&, const WeylWord&)`——两者共同指向「与上游 oracle 对齐」的工程立场，但本包不含上游来源。
- 【实现】`global_tits.rs` 对 `InvalidStrongTorusFactor` 的复用：该变体 Display 文案为 "Torus factor does not define a valid strong involution"，在本文件中用于 Imaginary 分支的非整配对拒绝；两文件中没有 GlobalTits 专属变体。

---

## 4. 测试锚点

【实现】`error.rs` 无测试；`global_tits.rs` 的 `#[cfg(test)] mod tests` 是两文件范围内唯一的测试锚点。测试导入：`use super::*;` 与 `crate::{BasedRootDatum, Coweight, LatticeInvolution, RootSystem, WeylGroup}`。

辅助函数（均在测试中 `.unwrap()`，失败即测试 panic）：
- `rational(numerator: i32, denominator: i32) -> Rational`
- `compact_inner(datum: BasedRootDatum, root_budget: usize) -> InnerClass`：`LatticeInvolution::identity(&datum)` + `InnerClass::new(datum, distinguished, root_budget)`
- `twisted(inner_class, action) -> TwistedInvolution`
- `element(inner_class, coordinates: Vec<Rational>, action) -> GlobalTitsElement`
- `adjoint_a1() -> BasedRootDatum`：`BasedRootDatum::from_simple_data(1, vec![vec![2]], vec![Weight::new(vec![2])], vec![Coweight::new(vec![1])])`

10 个 `#[test]`：

| 测试 | 锚定内容 |
|---|---|
| `normalizes_every_coordinate_modulo_two` | rank-2 datum（Cartan `[[2]]`，根 `[2,0]`，余根 `[1,0]`，budget 2）；坐标 `(-1/2, 9/2)` 经构造规范化为 `(3/2, 1/2)` |
| `rank_zero_and_an_empty_word_are_identity_transport` | rank 0 datum（budget 0）；空坐标、空 word 的 `crossed_word(&[])` 等于原元素 |
| `a1_imaginary_cross_distinguishes_compact_and_noncompact_factors` | adjoint A1（budget 2）、identity Weyl 作用：`compact`（坐标 `1/2`）交叉后仍为 `1/2`；`noncompact`（坐标 `0`）交叉后为 `1` |
| `a1_imaginary_cross_requires_an_integral_root_pairing` | adjoint A1、坐标 `1/4`：`crossed_generator(0)` 恰为 `Err(StructureError::InvalidStrongTorusFactor)`（`assert_eq!` 全等比较） |
| `a1_real_cross_leaves_both_components_unchanged` | adjoint A1、twisted 作用取 `WeylGroup::simple_reflection(0)`、坐标 `3/4`：`crossed_generator(0)` 返回**与原元素完全相等**的元素（两分量不变） |
| `a2_complex_cross_reflects_the_rational_coweight` | A2 标准 datum（`[[2,-1],[-1,2]]`，budget 6）；初值 `(1/3, 1/2)`、作用 `s0`；`crossed_generator(1)` 后坐标为 `(5/6, 3/2)`，且 twisted 作用等于 `s1.compose(&s0).compose(&s1)`（测试变量 `second/first`） |
| `a2_word_execution_is_forward_and_noncommuting` | 顺序保证锚点：`crossed_word(&[0,1])` == 先 `crossed_generator(0)` 再 `crossed_generator(1)`；forward 坐标 `(0, 1)`，reverse（`[1,0]`）坐标 `(1, 0)`；`assert_ne!(forward, reverse)` |
| `b2_complex_cross_uses_the_coroot_not_the_root_direction` | B2 标准 datum（`[[2,-2],[-1,2]]`，budget 8）；初值 `(0, 1/2)`、作用 `s0`；`crossed_generator(1)` 后坐标为 `(1, 3/2)` |
| `a1_with_central_torus_preserves_the_central_coordinate` | rank-2 datum（Cartan `[[2]]`，budget 2）；初值 `(0, 7/3)`；`crossed_generator(0)` 后为 `(1, 1/3)`（中心坐标经 mod-2 规范化保留） |
| `rejects_rank_generator_datum_and_distinguished_mismatches` | 四个精确错误断言：① 空坐标构造 → `Err(RankMismatch { expected: 1, actual: 0 })`；② `crossed_generator(1)` → `Err(IndexOutOfRange { index: 1, upper_bound: 1 })`；③ 外来 datum（`BasedRootDatum::standard(vec![vec![2]])`）的 twisted → `Err(DatumMismatch)`；④ A2 上 distinguished 取 `[[0,1],[1,0]]` 交换对合（经 `RootSystem::enumerate(&a2, 6)`、`LatticeInvolution::new`）而 inner class 为 compact（identity distinguished）→ `Err(DistinguishedInvolutionMismatch)` |

未被测试覆盖的代码分支【实现，对照 §2.4/§2.6】：`InvalidRootAutomorphism`（`kind()` 为 `None`）、`InvalidBasedAutomorphism`（`distinguished_generator_image` 的两个 `ok_or`）、`rational_pair`/`add_scaled_coroot` 内部的 `RankMismatch`（调用点已保证同秩）、`try_capacity` 失败路径、`crossed_word` 中非空 word 的错误传播。

---

## 5. 限制与未覆盖面

1. **依据范围**：仅两文件字节。`InnerClass`、`TwistedInvolution`、`WeylAction`、`RationalCoweight`、`Weight`、`Coweight`、`RootKind`、`RootSystem`、`BasedRootDatum`、`WeylGroup`、`LatticeInvolution`、`try_capacity`、`compose_matrices`、`crate::TitsCoset` 的定义与契约均未读；凡涉及它们的语义陈述均为调用点观察【推断】。
2. **传播错误不可判定**：`try_capacity` / `compose_matrices` / `WeylAction::*` / `TwistedInvolution::new` 会返回哪些 `StructureError` 变体，无法从这两文件确定；因此 `new`/`crossed_generator`/`crossed_word` 的完整错误集合大于 §3 表中直接构造的 7 个变体。
3. **panic 面**：`crossed_generator` 中 `datum.simple_roots()[generator]`、`datum.simple_coroots()[generator]` 为直接下标索引，存在标准越界 panic 的潜在面；是否有「`semisimple_rank <= simple_roots().len()`」之类不变量保证不可从本包判定【推断】。测试辅助函数大量使用 `.unwrap()`（测试上下文）。
4. **相等语义**：`GlobalTitsElement` 的 `Eq/PartialEq` 派生自 `RationalCoweight` 与 `TwistedInvolution`，二者的相等定义不在本包内；测试中的整元素 `assert_eq!`/`assert_ne!` 依赖该语义。
5. **数值细节未验收**：`normalize_coordinates` 使用 malachite 的 `Floor`；`Rational::from(quotient)` 的用法暗示 `floor()` 返回非 `Rational` 类型（如 `Integer`），但具体类型未在本包确认；对「坐标落在 `[0,2)`」仅作文档注释层面的转述，不作数学验证。
6. **error.rs 自身缺口**：无构造函数、无 `From` 转换、无错误码/序列化；`impl std::error::Error` 为空体（`source()` 等用默认实现）；Display 文案全为英文固定字符串，含字段插值（`{expected}`、`{image_root:?}` 等）。
7. **未使用的共享词汇**：`RootPairingMismatch` 的 `(row, column, expected, actual)` 语义、`RealFormNotDefinedOnCartan`、`TitsCosetInvariantViolation` 等 46 个变体在这两文件内没有构造点，其使用处不在本包范围。
8. **预算参数**：测试中 `InnerClass::new(datum, distinguished, root_budget)` 的 `root_budget`（取值 0/2/6/8）语义未知；`RootSystemTooLarge`、`WeylGroupTooLarge`、`ResourceLimitExceeded`、`AllocationFailed` 与本文件的 `try_capacity` 门槛之间的对应关系未证实。
9. **未做声明**：本草案不含任何性能、复杂度、数学正确性或与上游 Atlas 行为等价性的验收结论；`crossed_word` 文档所述「matching upstream `cross_act(GlobalTitsElement&, const WeylWord&)`」仅为注释转述。