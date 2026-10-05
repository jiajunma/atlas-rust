# 知识来源包草案（待维护者逐条核对）

以下是完整的 Markdown 草案，可直接审阅后收录至 `kb/sources/`。所有内容仅依据题目给出的两个源文件字节；「阅读推断」均已显式标注，未做任何数学/性能/正确性验收。

---

```markdown
---
title: BasedRootDatum 与对偶内类（root_datum.rs 与 dual.rs）来源包
source: atlas-rust/root-datum-and-dual
ingestedAt: 2026-10-05T18:00:00Z
---

# 概览

本包覆盖 `crates/atlas-real-group/src/` 下两个文件：

- **root_datum.rs**：定义 `BasedRootDatum`——在一对 character/cocharacter 格中经过校验的
  简单根数据。区分 `lattice_rank`（整个约化环面的秩）与 `semisimple_rank`（简单根个数，
  即 Cartan 矩阵阶数），二者仅在无中心环面时相等；类型**故意不设全局秩上限**
  （结构体文档注释原话）。提供构造门控、radical/coradical 基计算、简单反射、
  可失败克隆。
- **dual.rs**：定义对偶内类相关自由函数——对偶根数据、最长 Weyl 元素的下坡行走、
  对偶内类构造、跨对偶的 Cartan 类对应、对偶实形式计数。不定义任何类型。

两文件的上游谱系（Atlas of Lie Groups C++ 代码）仅以文档注释形式出现
（rootdata.cpp:858/859、rootdata.cpp:1357-1365、innerclass.cpp:435-441、
complexredgp.cpp `InnerClass::numDualRealForms`），**本包未核对上游源码**。

---

# 文件一：root_datum.rs

## 1. 裸签名清单（完整）

依赖：`malachite::Rational`；`crate::lattice::try_copy_coordinates`；
`crate::{pair, Coweight, StructureError, Weight}`。

```rust
// 类型（字段全部私有）
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BasedRootDatum {
    lattice_rank: usize,
    cartan: Vec<Vec<i32>>,
    simple_roots: Vec<Weight>,
    simple_coroots: Vec<Coweight>,
}

impl BasedRootDatum {
    pub fn standard(cartan: Vec<Vec<i32>>) -> Result<Self, StructureError>;
    pub fn from_simple_data(
        lattice_rank: usize,
        cartan: Vec<Vec<i32>>,
        simple_roots: Vec<Weight>,
        simple_coroots: Vec<Coweight>,
    ) -> Result<Self, StructureError>;
    pub fn lattice_rank(&self) -> usize;
    pub fn semisimple_rank(&self) -> usize;
    pub fn cartan_matrix(&self) -> &[Vec<i32>];
    pub fn simple_roots(&self) -> &[Weight];
    pub fn simple_coroots(&self) -> &[Coweight];
    pub fn coradical_basis(&self) -> Result<Vec<Weight>, StructureError>;
    pub fn radical_basis(&self) -> Result<Vec<Coweight>, StructureError>;
    fn annihilator_matrix(&self, rows: &[Vec<i32>])
        -> Result<crate::integer_lattice::IntegerMatrix, StructureError>;      // 私有
    fn coroot_rows(&self) -> Vec<Vec<i32>>;                                    // 私有
    fn root_rows(&self) -> Vec<Vec<i32>>;                                      // 私有
    fn budget(&self) -> crate::IntegerLatticeBudget;                           // 私有
    pub fn reflect_weight(&self, generator: usize, weight: &Weight)
        -> Result<Weight, StructureError>;
    pub fn reflect_coweight(&self, generator: usize, coweight: &Coweight)
        -> Result<Coweight, StructureError>;
    pub(crate) fn try_clone(&self) -> Result<Self, StructureError>;
}

// 模块内私有自由函数
fn reflect_coordinates(values: &[i32], direction: &[i32], coefficient: i128)
    -> Result<Vec<i32>, StructureError>;
fn ensure_lattice_rank(expected: usize, actual: usize) -> Result<(), StructureError>;
fn validate_cartan(cartan: &[Vec<i32>]) -> Result<usize, StructureError>;
fn is_finite_type(cartan: &[Vec<i32>]) -> bool;
```

本文件无 `pub enum` / `pub trait`。测试模块：`mod tests`（7 个用例）与
`mod torus_radical_regression_tests`（2 个用例）。

## 2. 逐项说明

### 2.1 `BasedRootDatum`（struct）
- **已实现行为**：私有四字段；派生 `Clone, Debug, Eq, Hash, PartialEq`（`Eq`/`PartialEq`
  被 dual.rs 用于 datum 相等比较，见第 4 节）。文档注释说明两个秩的语义与「无全局秩上限」
  的设计决定。

### 2.2 `standard`
- **已实现行为**：先 `validate_cartan(&cartan)?` 得 `rank`；`simple_roots` 取标准坐标向量
  第 row 位为 1 的 `Weight`；`simple_coroots` 取 Cartan 矩阵的**列**构成的 `Coweight`；
  然后委托 `from_simple_data(rank, cartan, roots, coroots)`。故 `standard` 构造的 datum
  恒有 `lattice_rank == semisimple_rank`。
- **错误路径**：`validate_cartan` 与 `from_simple_data` 的全部错误原样传播。

### 2.3 `from_simple_data`（构造门控，检查顺序固定）
按代码顺序：
1. `validate_cartan(&cartan)?` 得 `semisimple_rank`；
2. `lattice_rank < semisimple_rank` →
   `StructureError::RankMismatch { expected: semisimple_rank, actual: lattice_rank }`；
3. `simple_roots.len() != semisimple_rank` → `RankMismatch`；
4. `simple_coroots.len() != semisimple_rank` → `RankMismatch`；
5. 每个 root 的 `rank()` 经 `ensure_lattice_rank(lattice_rank, _)` → `RankMismatch`；
6. 每个 coroot 同上；
7. 逐 `(row, column)` 校验配对：`actual = pair(root, coroot)?`（`pair` 自身错误传播），
   与 `cartan[row][column]` 不等 →
   `StructureError::RootPairingMismatch { row, column, expected, actual }`。
- **边界**：`lattice_rank > semisimple_rank` 合法（中心环面，测试以 lattice_rank=3、
  semisimple=1 验证）；空 Cartan + 空根/余根 + 正 lattice_rank 合法（纯环面，测试
  lattice_rank=3 验证）。

### 2.4 五个访问器（无文档注释，上次草案遗漏项，本次补齐）
- `lattice_rank()` → 字段直返。
- `semisimple_rank()` → `self.cartan.len()`。
- `cartan_matrix()` → `&[Vec<i32>]` 只读切片。
- `simple_roots()` / `simple_coroots()` → 只读切片。
- 均不可失败、无校验。

### 2.5 `coradical_basis` / `radical_basis`
- **已实现行为（coradical）**：对「以简单**余根**坐标为行」的矩阵
  （`self.coroot_rows()`，经 `annihilator_matrix`）调用
  `crate::integer_lattice::saturated_kernel(&matrix, &self.budget())?`；对核基的每一列，
  逐分量 `i32::try_from(entry)`，失败 → `StructureError::ArithmeticOverflow`；
  每列包成 `Weight` 返回。
- **已实现行为（radical）**：同上，但行取简单**根**（`self.root_rows()`），结果包成
  `Coweight`。
- 文档注释声称对应 rootdata.cpp:858/859 的 `lattice::perp`，并提到上游 wrapper
  `root_coradical` / `coroot_radical` 会把基向量作为列接在简单根/余根之后导出；
  这些 wrapper **不在本文件实现**。
- **待核对（文档/实现措辞）**：`radical_basis` 注释首行写 “`lattice::perp` of the
  coroots”，但紧随的定义与代码均为「以简单根为行的矩阵的核」（与所有简单根正交的
  余权）。请维护者确认首行措辞是否有误。
- **回归测试**：无根 datum（秩 0/1/2/4）的 coradical 与 radical 都返回 ambient 格的
  单位坐标基（`expected_columns`），即「无方程时核为整个 ambient 格」。

### 2.6 私有辅助：`annihilator_matrix` / `coroot_rows` / `root_rows` / `budget`
- `annihilator_matrix`：行为空时构造 `IntegerMatrix::zero(0, self.lattice_rank, budget)`
  ——注释说明：空行意味着全 ambient 格是核，若从空行推断列数会丢失秩；非空走
  `IntegerMatrix::from_i32_rows(rows, budget)`。两条路径的错误均传播。
- `coroot_rows` / `root_rows`：把 `as_slice().to_vec()` 收集成行向量列表。
- `budget()`：固定返回 `crate::IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`；
  四个参数的语义**不在本文件定义**（未覆盖面）。

### 2.7 `reflect_weight` / `reflect_coweight` / `reflect_coordinates`
- **检查顺序（固定）**：先校验秩（`RankMismatch { expected: lattice_rank, actual }`），
  再校验生成元下标——`reflect_weight` 用 `simple_roots.get(generator)`，
  `reflect_coweight` 用 `simple_coroots.get(generator)`，越界 →
  `StructureError::IndexOutOfRange { index: generator, upper_bound: self.semisimple_rank() }`。
  因此「秩错 + 下标错」时先报 `RankMismatch`。
- **公式**：`reflect_weight`：x ↦ x − ⟨x, α∨_gen⟩·α_gen，系数
  `i128::from(pair(weight, &self.simple_coroots[generator])?)`；
  `reflect_coweight` 为对偶形式 y ↦ y − ⟨α_gen, y⟩·α∨_gen
  （文档注释称其为 `reflect_weight` 的镜像）。
- **阅读推断**：越界检查用 `.get()` 作用于一个数组，随后用 `[generator]` 直接索引
  **另一个**等长数组；在构造器不变量（两根数组长度均为 semisimple_rank）下不会
  panic，但该安全性依赖构造门控。
- `reflect_coordinates`（私有）：先 `try_reserve_exact`，失败 →
  `StructureError::AllocationFailed { requested: values.len() }`；逐坐标
  `coefficient.checked_mul(i128::from(direction))`、`i128::checked_sub`、
  `i32::try_from`，全部失败 → `StructureError::ArithmeticOverflow`。
  `zip` 对不等长切片会截断（阅读推断：调用方保证两边都是 lattice_rank 长）。
- **测试断言的边界**：`reflect_coweight` 对 `[7,11]` 得 `[-7,11]`；输入含 `i32::MIN`
  时返回 `ArithmeticOverflow`；`generator=1`（semisimple_rank=1）返回
  `IndexOutOfRange { index: 1, upper_bound: 1 }`。

### 2.8 `try_clone`（`pub(crate)`）
- **已实现行为**：逐字段可失败复制——三个 `Vec` 各自 `try_reserve_exact`，失败 →
  `StructureError::AllocationFailed { requested: <len> }`；行/坐标经
  `try_copy_coordinates` 复制后重建 `Weight::new` / `Coweight::new`；
  直接构造 `Self`，**不重跑** `from_simple_data` 校验。注释说明动机：既不使用不可失败的
  `Clone`，也避免冗余重校验。
- **注意**：本函数在这两个文件内**无调用点**（dual.rs 用的是派生 `Clone`）；外部调用
  点不在本包范围。

### 2.9 `ensure_lattice_rank` / `validate_cartan` / `is_finite_type`（私有）
- `ensure_lattice_rank`：不等即 `RankMismatch { expected, actual }`。
- `validate_cartan` 依次：
  1. 任一行长度 ≠ 行数 → `StructureError::NonSquareCartan`；
  2. 每行：对角元必须为 2，且所有非对角元必须 ≤ 0，否则
     `StructureError::InvalidCartanMatrix`；
  3. 零模式对称：`(entry == 0) != (cartan[column][row] == 0)` → `InvalidCartanMatrix`；
  4. `!is_finite_type(cartan)` → `InvalidCartanMatrix`；
  返回 `Ok(rank)`。**阅读推断**：空矩阵（rank 0）通过全部检查（循环空转、
  `is_finite_type([])` 为 true），与纯环面测试一致。
- `is_finite_type`：两遍精确有理数（`malachite::Rational`）计算。
  - 第一遍：对每个连通分量播种 scale=1，沿非零 Cartan 边传播
    `expected = scale[row] * C[row][col] / C[col][row]`；已赋值的顶点与 expected 冲突 →
    `false`。
  - 第二遍：对 scale 加权矩阵做 LDLᵀ 式分解：`pivot = scales[col]*C[col][col] −
    Σ factor²·diag`；任一 `pivot <= 0` → `false`；否则回代填 `lower`，全过 → `true`。
  - **panic 路径**：两处 `.expect("pending Cartan vertex has a scale")` 与
    `.expect("every Cartan vertex receives a scale")`（实现内部不变量）。
  - **声明界限**：「该检查等价于有限型 Cartan 矩阵」是代码意图/命名含义，本包不做
    数学验收。
- **测试断言**：affine `[[2,-2],[-2,2]]` 与不定 `[[2,-3],[-3,2]]` 均
  `InvalidCartanMatrix`；B2 `[[2,-2],[-1,2]]` 通过；A33（rank 33）通过——印证
  「无全局秩上限」。

## 3. 测试覆盖摘要（root_datum.rs）
`keeps_total_and_semisimple_rank_separate`、`accepts_rank_above_atlas_compatibility_limit`
（A33）、`accepts_non_simply_laced_finite_type`（B2）、
`rejects_non_finite_cartan_matrices_and_insufficient_lattice_rank`、
`reflects_coweights_dually_and_rejects_narrowing_overflow`、
`fallible_clone_reproduces_the_datum`、`supports_a_central_torus_without_roots`；
回归模块：`root_free_coradical_retains_the_ambient_lattice`、
`root_free_radical_retains_the_ambient_lattice`。

---

# 文件二：dual.rs

## 1. 裸签名清单（完整）

本文件**不定义任何 struct/enum/trait/impl**，只有自由函数。依赖（节选）：
`adjoint_fiber::{AdjointCartanFiber, AdjointFiberBudget}`、`cartan_fiber::CartanFiber`、
`grading::CartanGradingData`、`integer_lattice::IntegerLatticeBudget`、
`twisted_involution::compose_matrices`、`weak_real_form::WeakRealFormPartition`，以及
`crate::{BasedRootDatum, CartanClassification, CartanId, Coweight, InnerClass,
LatticeInvolution, StructureError, TwistedInvolution, Weight, WeylAction, WeylElement,
WeylGroup, WeylInterface}`。

```rust
pub fn dual_datum(datum: &BasedRootDatum) -> Result<BasedRootDatum, StructureError>;
fn two_rho(inner_class: &InnerClass) -> Result<Weight, StructureError>;              // 私有
pub fn longest_action(inner_class: &InnerClass, weyl_budget: usize)
    -> Result<WeylAction, StructureError>;
fn dual_involution(inner_class: &InnerClass, weyl_budget: usize)
    -> Result<Vec<Vec<i32>>, StructureError>;                                        // 私有
pub fn dual_inner_class(
    inner_class: &InnerClass,
    weyl_budget: usize,
    root_budget: usize,
) -> Result<InnerClass, StructureError>;
pub fn dual_cartan_correspondence(
    inner_class: &InnerClass,
    classification: &CartanClassification,
    dual: &InnerClass,
    dual_classification: &CartanClassification,
    _weyl_budget: usize,                       // 继承的旧预算参数，未使用
) -> Result<Vec<(CartanId, usize)>, StructureError>;
pub(crate) fn dual_twisted_representative(
    primal: &InnerClass,
    twisted: &TwistedInvolution,
    dual: &InnerClass,
    longest: &WeylAction,
) -> Result<TwistedInvolution, StructureError>;
pub fn dual_real_form_count(
    inner_class: &InnerClass,
    weyl_budget: usize,
    integer_budget: &IntegerLatticeBudget,
    adjoint_budget: &AdjointFiberBudget,
    fiber_budget: usize,
    root_budget: usize,
) -> Result<usize, StructureError>;
```

## 2. 模块级文档声明（注释内容，未核对上游）
- 对偶 based involution 对应上游 `rootdata::dualBasedInvolution`（rootdata.cpp:1357-1365）：
  取 `w0 = rd.to_dominant(-rd.twoRho())`（以「把 2ρ 送到 −2ρ」刻画最长元），结果为
  `(q * rd.action_matrix(w0)).negative_transposed()`，`q` 为原内类的 distinguished
  involution。
- 对偶 based root datum 转置 Cartan 并交换简单根/余根（上游
  `RootDatum(rd, tags::DualTag)`）。
- 对偶实形式数 = 对偶内类在其 distinguished involution 处 fundamental
  weak-real-form partition 的大小（complexredgp.cpp `InnerClass::numDualRealForms`）。

## 3. 逐项说明

### 3.1 `dual_datum`
- **契约**：返回「Cartan 转置 + 简单根/余根互换」的 datum。`transposed[row]` 取原矩阵
  第 row 列；原 `simple_coroots` 坐标原样包成 `Weight` 作为对偶根，原 `simple_roots`
  包成 `Coweight` 作为对偶余根；最终以
  `BasedRootDatum::from_simple_data(datum.lattice_rank(), transposed, dual_roots,
  dual_coroots)` 收尾——即**完整重走构造门控**。
- **错误路径**：全部来自 `from_simple_data`（见 root_datum 第 2.3 节）。
- **阅读推断**：原 datum 已校验，转置后配对 ⟨对偶根_row, 对偶余根_col⟩
  = `cartan[col][row]` = `transposed[row][col]`，故重校验理应通过；但签名保持可失败，
  本包不声称其永真。

### 3.2 `two_rho`（私有）
- **已实现行为**：以 `i64` 累加器（长度 = `lattice_rank`）遍历
  `root_system.entries()`，仅计入 `root_system.is_positive(id) == Some(true)` 的条目
  （`!= Some(true)` 即跳过——`None` 同样跳过）；逐坐标 `checked_add`，失败 →
  `StructureError::ArithmeticOverflow`；再逐分量 `i32::try_from`，失败同错。
- 文档注释称其为 `RootDatum::twoRho` 的对应物（「正根和的两倍」以实现为「每个正根
  各加一次」呈现；该恒等式属文档意图，本包不验收）。
- **阅读推断**：`zip(root.as_slice())` 在根坐标短于 `lattice_rank` 时会静默截断；
  正常情况下根坐标长度由构造保证。

### 3.3 `longest_action`
- **契约**：返回把 `2ρ` 送到 `−2ρ` 的 `WeylAction`（最长 Weyl 元素的作用），对应上游
  `rd.to_dominant(-rd.twoRho())`。文档注释称：不走全 Weyl 枚举，而是下坡行走，每步
  取与当前权有正余根配对的生成元反射，步数恰等于最长元约化长度（上游
  `WeylGroup::longest` 为 transducer O(rank)，此为等价 O(length) 行走）——**该复杂度
  声明为注释内容，本包不验收**。
- **算法细节（已实现）**：
  - `negative` 由 `checked_neg` 逐坐标构造（`i32::MIN` → `ArithmeticOverflow`）。
  - `WeylGroup::new(datum.clone())`（**不可失败**构造，使用派生 `Clone` 而非
    `try_clone`）；`WeylAction::identity(datum)?` 起步。
  - 循环 `while current != negative_rho`：按 `s = 0..rank` 升序找第一个配对 > 0 的
    生成元（配对 = `Σ coroot[s][i] * current[i]`，**`i64` 未检查累加**——与
    `two_rho` 的 `checked_add` 不同，此处无溢出防护，属事实记录而非评价）；
    找到则 `action = reflection.compose(&action)?; current = reflection.act(&current)?;`
    并 `break`。
  - 每轮结束 `steps += 1`；若 `steps > weyl_budget || !advanced` →
    `StructureError::LayoutInvariantViolation { invariant: "longest Weyl element" }`。
    即预算恰好允许 `weyl_budget` 步；找不到正配对（未推进）也报同一不变量错误。
- **错误路径**：`two_rho` 的溢出、`checked_neg` 溢出、`WeylAction::identity`、
  `group.simple_reflection(s)`、`compose`、`act` 的传播错误，以及上述
  `LayoutInvariantViolation`。

### 3.4 `dual_involution`（私有）
- **已实现行为**：`longest = longest_action(inner_class, weyl_budget)?`；返回
  `compose_matrices(distinguished.weight_matrix(), longest.matrix())`，即乘积矩阵
  `M = q * W0`（**本函数不做取负/转置**）。代码注释说明：对偶 involution 是 `−Mᵀ`；
  余权作用 `−M` 在构造处推导（因 M 是 involution）。
- **测试锚点**：A2 单位 involution 下返回 `[[0,-1],[-1,0]]`（即 W0）。

### 3.5 `dual_inner_class`
- **契约**：构造对偶内类。`dual = dual_datum(...)`；`product = dual_involution(...)`；
  逐元素 `checked_neg`（失败 → `ArithmeticOverflow`）填入：
  `dual_weight[column][row] = −product[row][column]`（即权作用 = `−(q·W0)ᵀ`）、
  `dual_coweight[row][column] = −product[row][column]`（即余权作用 = `−(q·W0)`）——
  与模块文档的 `negative_transposed` 对应。
- 然后 `LatticeInvolution::new(&dual, dual_weight, dual_coweight)?`，
  `InnerClass::new(dual, involution, root_budget)`。
- **预算语义**：`weyl_budget` 只约束最长元定位；`root_budget` 传给 `InnerClass::new`
  约束对偶根系闭包（文档注释原话）。

### 3.6 `dual_cartan_correspondence`
- **契约**：对 `classification` 中每个 Cartan 类（按 crate Cartan 顺序），输出
  `(对偶 CartanId, 该对偶类的 weak-real-form 计数)`（上游
  `CartanClass::numDualRealForms`，即对偶 fiber 的 weak-real partition 大小）。
  第五个参数 `_weyl_budget` **未使用**（文档称为遗留预算参数）。
- **前置校验（错误路径）**：
  1. `classification.cartan_ids().next()` 为空 →
     `StructureError::CartanClassificationInvariantViolation { invariant: "dual Cartan
     correspondence" }`；
  2. 原 fundamental 类代表元的 datum ≠ `inner_class.datum()` →
     `StructureError::DatumMismatch`；
  3. 对偶侧同样两步（空 → `CartanClassificationInvariantViolation`；datum 不等 →
     `DatumMismatch`）。
- **panic 路径**：三处 `.expect("cartan_ids yields in-range ids")`
  （两处前置校验、一处循环内取对偶类）。
- **主流程（已实现）**：
  - `partition = Arc::clone(dual_classification.twisted_partition())`；
    建 `cartan_of_raw: Vec<Option<CartanId>>`（长度 = `partition.classes().len()`），
    对每个对偶 Cartan 类 `raw = partition.class_of(representative)?` 记录
    `cartan_of_raw[raw] = Some(id)`。**阅读推断**：`cartan_of_raw[raw]` 直接索引，
    假定 `class_of` 返回值 `< classes().len()`。
  - **预算替换（重要）**：注释明言公开的预算参数不是该查找的预算；实际以
    `longest_action(dual, dual.root_system().roots().len())?` 用已枚举根数作为
    最长元行走预算（在**对偶** datum 上）。
  - 对每个原 Cartan 类：`twisted = dual_twisted_representative(inner_class,
    class.representative(), dual, &longest)?`；`raw = partition.class_of(&twisted)?`；
    `cartan_of_raw[raw]` 为 `None` → `CartanClassificationInvariantViolation`
    （文档原话："A class miss is an invariant violation, never a hole."）；
    `form_count = dual_classification.cartan_class(dual_id).expect(..).partition()
    .class_count()`；压入 `(dual_id, form_count)`。
- **文档注释中的设计说明（未做独立验证）**：上游按反序从原 Cartan 表建对偶表，
  把代表元 `tw` 与 `tw * w0` 的对偶 Cartan 配对（innerclass.cpp:435-441）；由于对偶
  distinguished involution 为 `−(δ·w0)ᵀ` 且转置是逆步的，配对 involution 化简为
  `−(w·δ)|co`；上游随后规范化对偶 twisted involution，故存储代表元一般是 `tw*w0`
  的**共轭**，矩阵比较不可靠；实现改以 lattice map 在对偶根上诱导的根像置换为键，
  即 `crate::TwistedConjugacyPartition::class_of` 的键。注释还说明 partition 可
  「枚举成员或按需规范化」。

### 3.7 `dual_twisted_representative`（`pub(crate)`)
- **契约**：文档称其为 RealWeylContext 对偶 fiber 所用的同一「原词重放」：把原
  代表元的 Weyl 字在**对偶** datum 上重放得到 `w`，再合成 `w * w0`；与裸根置换查找
  不同，它保留两种格作用与 distinguished-involution 来源信息。
- **已实现行为**：`WeylInterface::new(primal.datum().cartan_matrix())?`；
  `word = WeylElement::from_action(system, twisted.weyl_action())?
  .canonical_word(system, &interface)?`；`WeylGroup::new(dual.datum().clone())`
  （不可失败）；`action = group.identity()?`；逐生成元
  `action.compose(&group.simple_reflection(g)?)?`；末步
  `action.compose(longest)?`；最终 `TwistedInvolution::new(dual.datum(),
  dual.root_system(), dual.distinguished_involution().involution(), action)`。
- **错误路径**：上述全部 `?` 传播。

### 3.8 `dual_real_form_count`
- **契约**：对偶内类 fundamental weak-real-form partition 的大小（上游
  `InnerClass::numDualRealForms`）。
- **管线（顺序固定，全部 `?` 传播）**：
  1. `dual = dual_inner_class(inner_class, weyl_budget, root_budget)?`；
  2. `fundamental = CartanFiber::build(dual.distinguished_involution().involution(),
     integer_budget)?`；
  3. `adjoint = AdjointCartanFiber::build(dual.root_system(),
     dual.distinguished_involution(), &fundamental, adjoint_budget)?`；
  4. `grading = CartanGradingData::build(dual.root_system(),
     dual.distinguished_involution(), &adjoint)?`；
  5. `partition = WeakRealFormPartition::build(&grading, fiber_budget)?`；
  6. 返回 `partition.class_count()`。
- **预算分工**：`weyl_budget` → 最长元；`root_budget` → 对偶内类根系；
  `integer_budget` → Cartan fiber；`adjoint_budget` → 伴随 fiber；`fiber_budget` →
  weak-real partition。

## 4. 测试锚点（dual.rs，断言值照录）
- sc A1 → 2 个对偶实形式；adjoint A1 → 2；等秩 A2 → 1；扭 A2 → 2；紧 B2 → 3
  （测试注释称这些是 oracle 输出，如 sc A2 「2 real forms and 1 dual real form」、
  B2 中 `w0 = −1` 使紧/分裂内类两侧重合）。
- `dual_involution`（A2、单位 q）= `[[0,-1],[-1,0]]`。
- sc A1 对应表 = `[(CartanId(1), 1), (CartanId(0), 2)]`（反序配对）。
- 紧 B2：对应表长 4；首条计数 1；全格 `−1` involution 的类计数 3；
  对偶 CartanId 集合为 `{0,1,2,3}`（双射）；所有计数 ≥ 1。
- 测试中 `InnerClass::new(..., root_budget)` 与预算构造
  （`CartanClassificationBudget::new(integer, adjoint, 2/8, 64, 64)`）的具体取值仅作
  测试上下文记录。

---

# 两文件之间的调用关系

1. `dual::dual_datum` → `BasedRootDatum::{semisimple_rank, cartan_matrix, simple_roots,
   simple_coroots, lattice_rank}`（只读访问），收尾于 `from_simple_data`——**对偶构造
   复用 root_datum 的全部构造门控**（含 Cartan 校验与逐格配对校验）。
2. `dual::longest_action` → `datum.semisimple_rank()`、`datum.simple_coroots()`、
   `datum.clone()`（派生 `Clone`，非 `try_clone`）；反射经 `WeylGroup`/`WeylAction`
   （外部类型），**未直接调用** `BasedRootDatum::reflect_weight`。
3. `dual::dual_twisted_representative` → `primal.datum().cartan_matrix()`（喂给
   `WeylInterface::new`）与 `dual.datum().clone()`。
4. `dual_cartan_correspondence` 的 datum 一致性校验依赖 `BasedRootDatum` 派生的
   `PartialEq`/`Eq`。
5. `BasedRootDatum::try_clone`、`coradical_basis`、`radical_basis`、`reflect_weight`、
   `reflect_coweight` 在这两个文件内**无调用点**（外部调用者不在本包范围）。
6. 预算常数呼应：root_datum 内部固定 `IntegerLatticeBudget::new(64, 100_000, 100_000,
   128)`；dual.rs 测试内构造同值预算（语义未在本包定义）。

---

# 限制与未覆盖面

1. **范围**：仅这两个文件的字节。`Weight`、`Coweight`、`pair`、`StructureError`
   完整变体集、`IntegerLatticeBudget` 四参数语义、`integer_lattice::{saturated_kernel,
   IntegerMatrix}`、`WeylGroup/WeylAction/WeylElement/WeylInterface`、`InnerClass`、
   `LatticeInvolution`、`CartanClassification`、`TwistedConjugacyPartition`、各 fiber
   类型等**均未在本包定义**，其契约未知。
2. **上游引用未核对**：所有 `rootdata.cpp` / `innerclass.cpp` / `complexredgp.cpp`
   行号与对应关系仅转录自文档注释。
3. **无验收声明**：`is_finite_type` 与「有限型」的等价、`two_rho` 与 2ρ 的恒等、
   `longest_action` 的 O(length) 与「恰好约化长度」、对偶实形式计数的数学正确性，
   均为代码意图/注释声明，本包不作数学、性能或正确性结论。
4. **待维护者核对的观察点**：
   - `radical_basis` 文档首行 “perp of the coroots” 与实现（以根为行的核）措辞疑似
     不一致；
   - `longest_action` 内层配对用未检查的 `i64` 累加，与 `two_rho` 的 `checked_add`
     风格不同，是否有意；
   - `reflect_weight/reflect_coweight` 与 `dual_cartan_correspondence` 中存在「先
     `.get()` 检查兄弟数组、再直接索引」与 `cartan_of_raw[raw]` 直接索引，安全性依赖
     构造不变量；
   - panic 路径清单：`is_finite_type` 两处 `expect`；
     `dual_cartan_correspondence` 三处 `expect("cartan_ids yields in-range ids")`；
   - `two_rho` 对 `is_positive(id) == None` 的条目按非正根跳过；
   - `try_clone` 在两文件内无调用点，其外部使用方未覆盖；
   - `dual_cartan_correspondence` 忽略公开 `_weyl_budget`，改用
     `dual.root_system().roots().len()` 作为最长元行走预算（注释已明言，仍请确认
     该约定对外文档化）。
```

---

**交付说明（给 Codex 协调方）**：以上为完整草案，可直接落入 `kb/sources/`。已按要求做到：两文件各自的裸签名清单完整（含无注释的 5 个访问器、全部私有函数、`pub(crate)` 项），逐项契约与错误路径（含精确错误变体与字段、检查顺序、预算常数、panic 点、测试断言边界值），跨文件调用关系，以及限制与待核对清单；「已实现行为」与「阅读推断」已逐条区分，未做任何数学/性能/正确性验收。