# 知识来源包草案（供维护者逐条核对）

以下为完整的 Markdown 草案，可直接审阅后收录至 `kb/sources/`。

````markdown
---
title: Atlas Rust 对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution 的表示、门控与组合
source: atlas-rust/involution-types
ingestedAt: 2026-10-05T18:00:00Z
---

# Atlas Rust 对合类型三件套（source packet 草案）

## 0. 阅读依据与标注约定

- 本包仅依据以下三个源文件的完整字节整理：
  - `crates/atlas-real-group/src/involution.rs`
  - `crates/atlas-real-group/src/root_involution.rs`
  - `crates/atlas-real-group/src/twisted_involution.rs`
- 标注约定：
  - 【实现】= 可直接从所给字节读出的行为（签名、分支、错误类型、顺序）。
  - 【推断】= 阅读推断或意图性解读，需维护者确认。
  - 【文档声明】= 代码注释/doc comment 中的声明，本包不核验其数学正确性。
- 本包不做任何数学验收、性能或正确性声明。
- `StructureError`、`BasedRootDatum`、`Weight`、`Coweight`、`RootSystem`、`RootId`、`WeylAction`、`WeylGroup`、`RestrictedRootSystem`、`pair` 等符号在这三个文件中被使用但不在其中定义；本包只记录其被观察到的用法，不引用其定义。

---

## 1. 裸签名清单

### 1.1 `involution.rs`

公开项（含无 doc 注释项）：

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LatticeInvolution {
    datum: BasedRootDatum,          // 私有字段
    weight_action: Vec<Vec<i32>>,   // 私有字段
    coweight_action: Vec<Vec<i32>>, // 私有字段
}

impl LatticeInvolution {
    pub fn new(
        datum: &BasedRootDatum,
        weight_action: Vec<Vec<i32>>,
        coweight_action: Vec<Vec<i32>>,
    ) -> Result<Self, StructureError>;
    pub fn identity(datum: &BasedRootDatum) -> Result<Self, StructureError>;
    pub fn lattice_rank(&self) -> usize;
    pub fn datum(&self) -> &BasedRootDatum;
    pub fn weight_matrix(&self) -> &[Vec<i32>];
    pub fn coweight_matrix(&self) -> &[Vec<i32>];
    pub fn anti_invariant_rank(&self) -> Result<usize, StructureError>;
    pub fn act_on_weight(&self, weight: &Weight) -> Result<Weight, StructureError>;
    pub fn act_on_coweight(&self, coweight: &Coweight) -> Result<Coweight, StructureError>;
}
```

私有辅助（非公开 API，仅供核对者定位）：

```rust
fn identity_matrix(rank: usize) -> Result<Vec<Vec<i32>>, StructureError>;
fn is_square_of_rank(matrix: &[Vec<i32>], rank: usize) -> bool;
fn is_identity_product(left: &[Vec<i32>], right: &[Vec<i32>]) -> Result<bool, StructureError>;
fn preserves_pairing(
    weight_action: &[Vec<i32>],
    coweight_action: &[Vec<i32>],
) -> Result<bool, StructureError>;
fn apply_matrix(matrix: &[Vec<i32>], coordinates: &[i32]) -> Result<Vec<i32>, StructureError>;
fn checked_sum(mut pairs: impl Iterator<Item = (i32, i32)>) -> Result<i128, StructureError>;
```

### 1.2 `root_involution.rs`

公开项（含无 doc 注释项）：

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootKind {
    Imaginary,
    Real,
    Complex,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootInvolutionData {
    involution: LatticeInvolution,              // 私有字段
    image_by_root: Vec<RootId>,                 // 私有字段
    kind_by_root: Vec<RootKind>,                // 私有字段
    imaginary_simple_roots: Vec<RootId>,        // 私有字段
    real_simple_roots: Vec<RootId>,             // 私有字段
}

impl RootInvolutionData {
    pub fn new(
        root_system: &RootSystem,
        involution: LatticeInvolution,
    ) -> Result<Self, StructureError>;
    pub fn involution(&self) -> &LatticeInvolution;
    pub fn image(&self, root: RootId) -> Option<RootId>;
    pub fn image_permutation(&self) -> &[RootId];
    pub fn kind(&self, root: RootId) -> Option<RootKind>;
    pub fn roots_of_kind(&self, kind: RootKind) -> impl Iterator<Item = RootId> + '_;
    pub fn imaginary_simple_roots(&self) -> &[RootId];
    pub fn real_simple_roots(&self) -> &[RootId];
}
```

私有辅助：

```rust
fn validate_simple_root_images(
    root_system: &RootSystem,
    involution: &LatticeInvolution,
) -> Result<(), StructureError>;
fn subsystem_simple_roots(
    root_system: &RootSystem,
    kinds: &[RootKind],
    kind: RootKind,
) -> Result<Vec<RootId>, StructureError>;
```

### 1.3 `twisted_involution.rs`

公开项（含无 doc 注释项）：

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TwistedInvolution {
    weyl_action: WeylAction,            // 私有字段
    root_involution: RootInvolutionData, // 私有字段
}

impl TwistedInvolution {
    pub fn new(
        datum: &BasedRootDatum,
        root_system: &RootSystem,
        distinguished: &LatticeInvolution,
        weyl_action: WeylAction,
    ) -> Result<Self, StructureError>;
    pub fn weyl_action(&self) -> &WeylAction;
    pub fn root_involution(&self) -> &RootInvolutionData;
    pub fn restricted_roots(
        &self,
        root_system: &RootSystem,
    ) -> Result<RestrictedRootSystem, StructureError>;
}

pub(crate) fn compose_matrices(
    left: &[Vec<i32>],
    right: &[Vec<i32>],
) -> Result<Vec<Vec<i32>>, StructureError>;
```

---

## 2. 逐项解释

### 2.1 `LatticeInvolution`（involution.rs）

**表示**【实现】
- 持有 `BasedRootDatum` 的克隆，以及两个 `Vec<Vec<i32>>` 方阵：`weight_action`（作用于 `Weight` 坐标）与 `coweight_action`（作用于 `Coweight` 坐标）。两者分开存储、一起验证。
- 作用方式：`apply_matrix` 以「行向量 · 坐标列」计算（`row.iter().zip(coordinates)` 点积），即矩阵左乘列坐标的约定【推断：从点积结构读出】。
- 字段均为私有，构造只能经 `new` 或 `identity`，因此构造门控所建立的不变量无法在 crate 外被绕过【实现：字段无私有性以外的访问器；推断：不变量维持依赖于"无其他构造路径"这一事实】。

**构造门控 `new`（分支与顺序）**【实现】
1. 方阵检查：`weight_action` 与 `coweight_action` 均须为 `rank × rank`，`rank = datum.lattice_rank()`；任一不满足 → `Err(StructureError::InvalidInvolution)`。
2. 对合性检查：先 `weight_action² == I`，后 `coweight_action² == I`（`is_identity_product`，逐元素 i128 累加，`checked_sum` 溢出 → `ArithmeticOverflow` 经 `?` 传播）；`||` 短路，weight 侧失败时 coweight 侧不求值。不成立 → `Err(StructureError::InvalidInvolution)`。
3. 配对保持检查：`preserves_pairing` 要求对所有 `(i, j)`：`∑_row weight_action[row][i] * coweight_action[row][j] == δ_ij`；不成立 → `Err(StructureError::InvalidRootAutomorphism)`；溢出同样传播 `ArithmeticOverflow`。
   - 【推断】该坐标条件对应 `W^T·C = I`，即配对保持 `⟨θλ, θμ∨⟩ = ⟨λ, μ∨⟩`；此意图由测试 `dual_actions_preserve_pairing` 佐证，但本包不作数学证明。
- 成功路径克隆 `datum` 并原样存入两个矩阵。

**`identity`**【实现】
- 两个矩阵均取 `identity_matrix(rank)`；该辅助对外层向量与每一行使用 `try_reserve_exact`，分配失败 → `StructureError::AllocationFailed { requested: rank }`；行内用 `resize(rank, 0)` 后置对角元为 1。除分配失败外无其他错误分支。

**访问器**【实现】
- `lattice_rank()` 返回 `weight_action.len()`（非 `datum.lattice_rank()`，但对经门控构造的值二者相等【推断】）。
- `datum()`、`weight_matrix()`、`coweight_matrix()` 返回不可变引用/切片。

**`anti_invariant_rank`**【实现 + 文档声明】
- 文档声明：计算 `X^*/ker(1−theta)` 的秩，来自该对合的 `-1` 特征空间，刻意避免浮点秩计算。
- 实现：trace 用 `i128` `checked_add` 累加（溢出 → `ArithmeticOverflow`）；`twice_rank = i128::try_from(lattice_rank())? − trace`（减法溢出 → `ArithmeticOverflow`）；若 `twice_rank < 0` 或 `twice_rank % 2 != 0` → `Err(StructureError::InvalidInvolution)`；返回 `usize::try_from(twice_rank / 2)`（转换失败 → `ArithmeticOverflow`）。
- 注意：本文件测试未覆盖此方法（见 §4）。

**`act_on_weight` / `act_on_coweight`**【实现】
- 委托 `apply_matrix` 后用 `Weight::new` / `Coweight::new` 包装。
- `apply_matrix` 错误分支：
  - `matrix.len() != coordinates.len()` → `StructureError::RankMismatch { expected: matrix.len(), actual: coordinates.len() }`；
  - 某行 `row.len() != coordinates.len()` → `StructureError::InvalidInvolution`（注意：此处不用 `RankMismatch`，与上一分支不对称，值得核对者留意）；
  - 行点积在 `i128` 中累加后 `i32::try_from` 失败 → `StructureError::ArithmeticOverflow`。

**明确不承诺的性质**【文档声明】
- doc comment 明确：本类型**不**声称该作用保持有限根系；更强的性质（根置换 + 余根运输）由 `crate::RootInvolutionData` 在枚举出的根系上建立。

### 2.2 `RootKind` 与 `RootInvolutionData`（root_involution.rs）

**`RootKind`**【实现】
- 三变元：`Imaginary` / `Real` / `Complex`；derive 为 `Clone, Copy, Debug, Eq, PartialEq`（无 `Ord`、`Hash`）。

**表示**【实现】
- 按值消费并持有一个 `LatticeInvolution`；`image_by_root: Vec<RootId>` 与 `kind_by_root: Vec<RootKind>` 以 `root_system.entries()` 的迭代顺序（即 `RootId` 下标）逐根填充；另存 `imaginary_simple_roots` 与 `real_simple_roots` 两个列表。

**构造门控 `new`（分支与顺序）**【实现】
1. `involution.datum() != root_system.datum()` → `StructureError::DatumMismatch`。
2. 秩不等 → `StructureError::RankMismatch { expected: root_system.lattice_rank(), actual: involution.lattice_rank() }`。
3. `validate_simple_root_images`：对 `datum.simple_roots()` 与 `datum.simple_coroots()` 的 zip 做 `enumerate`（下标记为 `simple_root`）：
   - `θ(α)` 经 `root_system.id_of` 找不到 → `StructureError::SimpleRootImageNotRoot { simple_root }`；
   - `root_system.coroot(image_id)` 为 `None` → `StructureError::IndexOutOfRange { index: image_id.0, upper_bound: root_system.roots().len() }`；
   - `θ(α∨) != *image_coroot` → `StructureError::SimpleCorootImageMismatch { simple_root, image_root }`（`image_root` 为 `Weight`）。
4. 主循环（对每个 `(_, root, coroot)` from `root_system.entries()`）：
   - `θ(root)` 不在根系 → `StructureError::InvalidRootAutomorphism`；
   - 像余根缺失 → `IndexOutOfRange { .. }`（同上载荷）；
   - `θ(coroot) != *image_coroot` → `StructureError::InvalidRootDatumAutomorphism`；
   - 计算 `-root` 时逐坐标 `checked_neg`，溢出 → `ArithmeticOverflow`；
   - 分类（优先级固定）：`image == *root` → `Imaginary`；否则 `image.as_slice() == negative.as_slice()` → `Real`；否则 `Complex`。
5. 分别对 `Imaginary`、`Real` 调 `subsystem_simple_roots`。
- 门控顺序后果【实现 + 测试佐证】：单根专属的 `SimpleRootImageNotRoot` / `SimpleCorootImageMismatch` 先于主循环的泛型错误（`InvalidRootAutomorphism` / `InvalidRootDatumAutomorphism`）被报告。
- doc comment 陈述的设计动机【文档声明】：仅配对保持不足以得到根数据自同构——存在固定所有根但移动余根中心环面坐标的作用；因此构造同时验证根置换与余根运输。

**`subsystem_simple_roots` 算法**【实现】
- 过滤：该 `kind` 且 `root_system.simple_coordinates(root)` 为 `Some` 且所有坐标 `>= 0` 的根（`is_some_and`），构成 `positive`（保持 `RootId` 升序）。
- 将 `positive` 的坐标向量收集进 `BTreeSet<Vec<i32>>`（`simple_coordinates` 为 `None` → `IndexOutOfRange { index, upper_bound }`）。
- 判定：对候选 `c`（按 `positive` 顺序），若集合中存在 `summand` 使逐坐标差 `c − summand`（`checked_sub`，溢出 → `ArithmeticOverflow`）也在集合中，则 `c` 可分解、被跳过；否则入选。
- 输出顺序 = `positive` 顺序，即 `RootId` 升序【实现：迭代结构决定；测试 `finds_the_inherited_simple_basis_of_the_imaginary_subsystem` 锚定了具体顺序】。
- 【推断】`BTreeSet` 仅用于成员判定，不影响输出顺序；候选自身作为 `summand` 时余量为零向量，零向量通常不在正根坐标集合中（代码未显式排除，本包不进一步断言）。

**访问器**【实现】
- `image(root)` / `kind(root)` 经 `.get(root.0)` 返回 `Option`：越界 `RootId` 不 panic，返回 `None`。
- `image_permutation()` 返回整个置换切片（按 `RootId` 下标索引）。
- `roots_of_kind(kind)` 返回惰性迭代器，按 `RootId` 升序产出。
- `imaginary_simple_roots()` / `real_simple_roots()` 返回切片；doc 声明为「继承正系中子系统的单根」。

### 2.3 `TwistedInvolution`（twisted_involution.rs）

**表示**【实现】
- 原样持有传入的 `weyl_action: WeylAction`，以及由合成对合构造出的 `RootInvolutionData`；不保留 `distinguished` 本身。

**构造门控 `new`（分支与顺序）**【实现】
1. datum 一致性：`root_system.datum() != datum || distinguished.datum() != datum || weyl_action.datum() != datum` → `StructureError::DatumMismatch`（任一即失败，先于一切秩检查）。
2. 三个秩检查，顺序为 `root_system.lattice_rank()`、`distinguished.lattice_rank()`、`weyl_action.rank()`，各自与 `datum.lattice_rank()` 比较，不等 → `StructureError::RankMismatch { expected: rank, actual: <该值> }`。
3. 矩阵合成：`compose_matrices(weyl_action.matrix(), distinguished.weight_matrix())?` 与 `compose_matrices(weyl_action.coweight_matrix(), distinguished.coweight_matrix())?`（w 在左、θ 在右，doc 记为 "w after theta"）。
4. 合成结果重新经 `LatticeInvolution::new(datum, ..)?` 完整门控——即 `(wθ)² = I` 与配对保持在此被重新验证（doc 声明：本类型只建立 `w theta` 平方为一的根论条件）。
5. `RootInvolutionData::new(root_system, involution)?` 再验证根置换与余根运输（错误类型见 §2.2）。
- doc 中的边界声明【文档声明】：Cayley/cross 分解在 `crate::CayleyCrossDecomposition`，Atlas 规范化在 `crate::InnerClass::canonicalize`（由 `crate::CartanClassification` 用于编号 Cartan 类）；这些符号不在本三文件内，本包仅记录引用。

**`compose_matrices`（pub(crate)）**【实现】
- `rank = left.len()`；若 `right.len() != rank` 或任一矩阵任一行长度 `!= rank` → `StructureError::RankMismatch { expected: rank, actual: right.len() }`。
  - 【推断】错误载荷的 `actual` 恒为 `right.len()`，即使实际问题是某行长度不符；载荷不区分失败来源。
- 乘法为标准三重循环（行 × 中间指标 × 列），i128 `checked_mul`/`checked_add` 累加（溢出 → `ArithmeticOverflow`），`i32::try_from` 收窄失败 → `ArithmeticOverflow`。

**`restricted_roots`**【实现】
- 直接委托 `RestrictedRootSystem::build(root_system, &self.root_involution)`；`RestrictedRootSystem` 跨文件定义，本包不展开。

### 2.4 三者之间的转换 / 组合关系

- `LatticeInvolution` → `RootInvolutionData`：`RootInvolutionData::new(root_system, involution)` 按值消费前者；`involution()` 可取回 `&LatticeInvolution`。【实现】
- `(distinguished: &LatticeInvolution, weyl_action: WeylAction)` → `TwistedInvolution`：经 `compose_matrices` 合成 `w∘θ` 的两格矩阵 → 新 `LatticeInvolution`（重门控）→ `RootInvolutionData`；`distinguished` 不被存储（无访问器可取回），`weyl_action` 原样存储并经 `weyl_action()` 暴露。【实现】
- `TwistedInvolution` → `RestrictedRootSystem`：`restricted_roots(root_system)`。【实现】
- 层级意图【文档声明】：`LatticeInvolution` 只保证配对保持的对合；`RootInvolutionData` 在其上叠加根系置换 + 余根运输；`TwistedInvolution` 是 distinguished 对合经 Weyl 元素平移后仍为对合的载体，其 Cartan 类编号相关的规范化不在本层。

---

## 3. 错误类型与边界条件汇总（本三文件内被构造/返回的 `StructureError` 变体）

| 变体 | 触发位置（文件 / 函数） |
|---|---|
| `InvalidInvolution` | involution.rs `new`（非方阵、非对合）、`anti_invariant_rank`（twice_rank 为负或奇）、`apply_matrix`（行长度不符） |
| `InvalidRootAutomorphism` | involution.rs `new`（配对不保持）；root_involution.rs 主循环（像非根） |
| `InvalidRootDatumAutomorphism` | root_involution.rs 主循环（余根运输不符） |
| `SimpleRootImageNotRoot { simple_root }` | root_involution.rs `validate_simple_root_images` |
| `SimpleCorootImageMismatch { simple_root, image_root }` | root_involution.rs `validate_simple_root_images` |
| `DatumMismatch` | root_involution.rs `new`；twisted_involution.rs `new` |
| `RankMismatch { expected, actual }` | 三文件均有（见各节；注意各处的 `expected/actual` 取值差异） |
| `IndexOutOfRange { index, upper_bound }` | root_involution.rs（`coroot(image_id)`、`simple_coordinates` 缺失处） |
| `ArithmeticOverflow` | 所有 i128 检验算术与 `i32::try_from`/`usize::try_from`/`checked_neg`/`checked_sub` 处 |
| `AllocationFailed { requested }` | involution.rs `identity_matrix`（`try_reserve_exact` 失败） |

- panic/断言【实现】：非测试代码无 `panic!`/`assert!`；私有辅助中存在下标索引（如 `right[middle][column]`、`row[index]`），其安全性依赖调用点的前置方阵检查【推断：当前调用点均有前置检查；字段私有保证实例必经门控构造】。
- 预算/上限【实现 + 推断】：三个构造函数均无显式预算参数；测试中 `RootSystem::enumerate(&datum, 2|4|6)` 的第二参数疑似根数上限（跨文件符号，仅按用法记录）。
- 顺序保证【实现】：`image_by_root`/`kind_by_root` 按 `entries()` 顺序（`RootId` 下标）；`roots_of_kind` 按 `RootId` 升序；`imaginary/real_simple_roots` 按 `RootId` 升序；`TwistedInvolution::new` 的错误优先级为 `DatumMismatch` → 秩（root_system → distinguished → weyl_action）→ 合成/重门控错误。

---

## 4. 测试锚点

### involution.rs（3 个）
- `dual_actions_preserve_pairing`：rank-2 datum（`from_simple_data(2, [[2]], [Weight(1,0)], [Coweight(2,0)])`），θ = diag(−1,1)（两格同矩阵）；`act_on_weight((3,5))`、`act_on_coweight((7,−11))` 后断言 `pair(&weight, &coweight) == Ok(-34)`。
- `rejects_actions_that_do_not_preserve_the_pairing`：A1 `standard([[2]])`；`W=[[-1]], C=[[1]]` → `InvalidRootAutomorphism`。
- `rejects_non_involutive_actions`：`W=[[2]], C=[[0]]` → `InvalidInvolution`。

### root_involution.rs（7 个）
- `classifies_real_and_complex_a2_roots`：A2；θ = `[[0,-1],[-1,0]]`（两格同）；断言 Real=2、Complex=4、Imaginary=0；`real_simple_roots() == [id_of(Weight(1,1))]`。
- `rejects_pairing_preserving_actions_that_do_not_permute_roots`：`W=[[-1,0],[1,1]]`、`C=[[-1,1],[0,1]]` 能通过 `LatticeInvolution::new`，但 `RootInvolutionData::new` → `SimpleRootImageNotRoot { simple_root: 0 }`。
- `reports_a_positive_simple_root_with_the_wrong_image_coroot`：→ `SimpleCorootImageMismatch { simple_root: 0, image_root: Weight(1,0) }`。
- `reports_a_negative_simple_root_with_the_wrong_image_coroot`：→ `SimpleCorootImageMismatch { simple_root: 0, image_root: Weight(-1,0) }`。
- `rejects_a_root_system_from_a_different_same_rank_datum`：A2 的 identity 作用于 A1×A1（`standard([[2,0],[0,2]])`）的根系 → `DatumMismatch`。
- `classifies_identity_as_imaginary`：rank-2 中心环面 datum + identity → Imaginary 计数 2。
- `finds_the_inherited_simple_basis_of_the_imaginary_subsystem`：A2 + identity → `imaginary_simple_roots() == [id_of(Weight(0,1)), id_of(Weight(1,0))]`（锚定了按枚举 `RootId` 顺序的输出）。

### twisted_involution.rs（4 个）
- `validates_a_weyl_translate_of_the_distinguished_involution`：A1；`w = simple_reflection(0)`，θ = identity；断言合成后 Real 根计数 2。
- `rejects_weyl_translates_that_are_not_involutive`：A2；`w = s0.compose(&s1)`（三阶元）→ `InvalidInvolution`。
- `rejects_a_weyl_action_owned_by_a_different_same_rank_datum`：datum=A2、根系来自 A2，但 `WeylGroup::new(b2).identity()`（B2：`[[2,-2],[-1,2]]`）→ `DatumMismatch`。
- `composes_a_nontrivial_central_torus_involution_with_a_weyl_action`：rank-2 中心环面 datum；distinguished = diag(1,−1)（两格同）；`w = s0` → Real 计数 2。

---

## 5. 限制与未覆盖面

- 【文档声明】`LatticeInvolution` 不验证根系保持；该性质完全由 `RootInvolutionData::new` 门控。
- 【实现】本三文件的测试未覆盖：`anti_invariant_rank`（含其 `InvalidInvolution`/`ArithmeticOverflow` 分支）；`act_on_weight`/`act_on_coweight` 的错误路径（`RankMismatch`、行长度不符、`ArithmeticOverflow`）；`image()`/`kind()` 对越界 `RootId` 返回 `None` 的行为；`image_permutation()`；`identity()` 的 `AllocationFailed` 路径；`compose_matrices` 的 `RankMismatch` 直接触发（在 `TwistedInvolution::new` 内有前置秩检查【推断：经此路径不可达，但该函数为 `pub(crate)`，其他 crate 内调用方可能触发】）。
- 【实现】`apply_matrix` 对「矩阵行数 ≠ 坐标长度」报 `RankMismatch`、对「某行长度 ≠ 坐标长度」报 `InvalidInvolution`，错误类型选择不对称。
- 【实现】`compose_matrices` 的 `RankMismatch` 载荷 `actual` 恒为 `right.len()`，不区分真实的失败来源。
- 【实现】`RootInvolutionData::new` 不含对「根置换本身为二阶」的独立检查（依赖 `LatticeInvolution` 的代数对合门控；本包不进一步断言其充分性）。
- 【实现】`TwistedInvolution` 不保留 distinguished 对合，也无分解/反解访问器。
- 【实现】`RootKind` 仅 derive `Clone, Copy, Debug, Eq, PartialEq`。
- 【推断】私有辅助中的裸下标索引依赖前置方阵/秩检查；若未来新增绕过 `new`/`identity` 的构造路径，需重新核对。
- 【实现】doc 引用的 `crate::CayleyCrossDecomposition`、`crate::InnerClass::canonicalize`、`crate::CartanClassification`、`crate::RootInvolutionData`（自 involution.rs 注释）等符号不在本三文件内，本包未核验其定义与行为。
- 【实现】测试中反复使用的 rank-2 datum（`from_simple_data(2, [[2]], [(1,0)], [(2,0)]`）带有一个中心环面方向坐标，用于构造「根方向 + 环面方向」的场景；本包只记录其作为测试夹具的事实，不解释其理论角色。
````

---

## 交接说明（给 Codex 协调者）

- 以上即完整的 source packet 草案正文（frontmatter 已按要求置于开头，`title`/`source`/`ingestedAt` 严格按给定值）。
- 待维护者重点核对的高风险点（均为【推断】标注处）：
  1. `preserves_pairing` 的坐标条件与「配对保持」语义的对应关系；
  2. `compose_matrices` 在 `TwistedInvolution::new` 内 `RankMismatch` 分支的可达性；
  3. `subsystem_simple_roots` 中零向量余量不可能落入正根坐标集合的假设；
  4. 私有辅助裸下标索引安全性对前置检查的依赖；
  5. `RootSystem::enumerate` 第二参数作为「根数上限」的解读。
- 未做任何数学验收、性能或正确性声明；所有「已实现行为」均可对照所给字节逐行复核。