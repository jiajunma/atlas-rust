---
title: GlobalKgb：内类范围 KGB 图与 print_X 布局（global_kgb.rs）知识来源包草案
source: atlas-rust/global-kgb
ingestedAt: 2026-10-06T00:00:00Z
---

> 核对说明：本草案仅依据 `crates/atlas-real-group/src/global_kgb.rs` 的字节内容撰写。
> 凡标注【注释声明】者，均为源码注释中对上游 atlas 代码（kgb.cpp、y_values.cpp 等）的对应关系或性质主张，本包未核对上游源码，不为其真实性背书。
> 凡标注【已实现】者，可直接从字节读出；【推断】为阅读控制流/类型得出、但未被字节显式固定的结论。
> 本包不做任何数学验收、性能或正确性声明。

## 0. 文件级元信息

- 路径：`crates/atlas-real-group/src/global_kgb.rs`。
- 【注释声明】模块文档称本文件是上游 `kgb::global_KGB`（kgb.h:213-266，kgb.cpp:190-233 与 331-479）的移植，并复现 `kgb_io::print_X` 的输出版式（io/kgb_io.cpp:57-159）。
- 【注释声明】一个 `GlobalKgb` 枚举同一内类（inner class）全部强实形的 KGB 元素，按扭对合（twisted involution）分为 tau 包；元素按上游 `InvolutionTable::x_pack` 指纹（involutions.cpp:279-295）去重；元素身份由指纹而非环面值定义；存储的环面代表元是“先到者”，其未再约化的算术历史（如负分子 `[0,-1]/2`）可在打印输出中观察到。
- 【注释声明】上游第二个构造器（从任意 `GlobalTitsElement` 播种）与 Bruhat/Hasse 层未移植。
- 本文件未定义任何 trait 或 enum；无 `pub(crate)` 项。

## 1. 裸签名清单

### 1.1 公开项（pub）

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalKgbPrintRow {
    pub length: usize,
    pub statuses: Vec<KgbStatus>,
    pub cross: Vec<usize>,
    pub cayley: Vec<Option<usize>>,
    pub torus_label: RationalWeight,
    pub cartan: usize,
    pub involution_word: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalKgbPrint {
    pub header: String,
    pub lattice_rank: usize,
    pub rows: Vec<GlobalKgbPrintRow>,
}

impl GlobalKgbPrint {
    pub fn render(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct GlobalKgb { /* 全部字段私有，见 §3.1 */ }

impl GlobalKgb {
    pub fn build(
        inner_class: &InnerClass,
        classification: &CartanClassification,
        table: &mut InvolutionTable,
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, StructureError>;
    pub fn size(&self) -> usize;
    pub fn semisimple_rank(&self) -> usize;
    pub fn lattice_rank(&self) -> usize;
    pub fn packet_count(&self) -> usize;
    pub fn tau_packet(&self, index: usize) -> Option<Range<usize>>;
    pub fn status(&self, element: usize, generator: usize) -> Option<KgbStatus>;
    pub fn cross(&self, generator: usize, element: usize) -> Option<usize>;
    pub fn cayley(&self, generator: usize, element: usize) -> Option<usize>;
    pub fn inverse_cayley(&self, generator: usize, element: usize) -> Option<(usize, Option<usize>)>;
    pub fn length(&self, element: usize) -> Option<usize>;
    pub fn cartan_of(&self, element: usize) -> Option<usize>;
    pub fn torus_label(&self, element: usize) -> Option<RationalWeight>;
    pub fn involution_word(&self, packet: usize) -> Option<&str>;
    pub fn print_layout(&self) -> Result<GlobalKgbPrint, StructureError>;
}
```

### 1.2 文件内私有项（非 pub；列出供核对，含无注释项）

```rust
fn digits(mut value: usize) -> usize;            // 有注释：digits(0) == 1
fn gcd_u64(mut left: u64, mut right: u64) -> u64; // 无注释

#[derive(Clone, Debug, Eq, PartialEq)]
struct GlobalTorusElement {
    numerator: Vec<i64>,
    denominator: i64,
}

impl GlobalTorusElement {
    fn reduce_raw(mut numerator: Vec<i64>, denominator: i64) -> Self;
    fn exp_pi(numerator: Vec<i64>, denominator: i64) -> Self;
    fn exp_2pi(mut numerator: Vec<i64>, mut denominator: i64) -> Result<Self, StructureError>;
    fn add(&self, right: &Self) -> Result<Self, StructureError>;
    fn add_torus_part(&mut self, part: &ModTwoVector) -> Result<(), StructureError>;
    fn evaluate_at(&self, weight: &[i32]) -> Result<(i64, i64), StructureError>;
    fn negative_at(&self, weight: &[i32]) -> Result<bool, StructureError>;
    fn simple_reflect(&mut self, root: &[i32], coroot: &[i32]) -> Result<(), StructureError>;
    fn imaginary_cross_act(&mut self, root: &[i32], coroot: &[i32]) -> Result<(), StructureError>;
    fn log_2pi(&self) -> Result<RationalWeight, StructureError>;
}

fn fingerprint(
    theta: &LatticeInvolution,
    torus: &GlobalTorusElement,
    budget: &IntegerLatticeBudget,
) -> Result<RationalWeight, StructureError>;

fn fundamental_fiber(
    delta: &LatticeInvolution,
    budget: &IntegerLatticeBudget,
) -> Result<ModTwoSubquotient, StructureError>;

fn square_class_generators(
    inner_class: &InnerClass,
    budget: &IntegerLatticeBudget,
) -> Result<Vec<usize>, StructureError>;

fn fundamental_coweights(datum: &BasedRootDatum) -> Result<Vec<RationalWeight>, StructureError>;

fn format_rational_weight(weight: &RationalWeight) -> String;
fn format_involution_word(expression: &[i32]) -> String;

struct ElementStore {
    semisimple_rank: usize,
    elements: Vec<GlobalTorusElement>,
    element_packet: Vec<usize>,
    statuses: Vec<Option<KgbStatus>>,
    cross: Vec<usize>,
    cayley: Vec<Option<usize>>,
    inverse_cayley: Vec<Option<(usize, Option<usize>)>>,
}

impl ElementStore {
    fn new(semisimple_rank: usize) -> Self;
    fn len(&self) -> usize;
    fn push(&mut self, torus: GlobalTorusElement, packet: usize) -> Result<(), StructureError>;
}

#[cfg(test)]
mod tests { /* 见 §9 */ }
```

### 1.3 导入（调用点可见的外部符号）

- `std::collections::HashMap`、`std::ops::Range`。
- `malachite::base::num::basic::traits::{One, Zero}`；`malachite::{Integer, Rational}`。
- `crate::grading::try_capacity`。
- `crate::integer_lattice::{adapted_basis, negative_coweight_eigenspace, reduce_basis_mod_two, IntegerLatticeBudget}`。
- `crate::mod_two::{ModTwoSubquotient, ModTwoSubspace, ModTwoVector}`。
- `crate::{BasedRootDatum, CartanClassification, CartanId, InnerClass, InvolutionId, InvolutionTable, KgbStatus, LatticeInvolution, RationalWeight, StructureError, WeylElement}`。
- 仅测试：`crate::{AdjointFiberBudget, CartanClassificationBudget, Coweight, InvolutionTableBudget, Weight}`。

## 2. 逐项解释

### 2.1 私有工具函数

- `digits`【已实现】：十进制位数；`count` 初值 1，`while value >= 10 { count += 1; value /= 10; }`，故 `digits(0) == 1`（与注释一致）。
- `gcd_u64`【已实现】：欧几里得迭代；`right == 0` 时返回 `left`（含 `gcd_u64(0,0) == 0` 的字面行为）。

### 2.2 `GlobalTorusElement`（私有）——表示纪律与不变量

【已实现】存储：`numerator: Vec<i64>` 与 `denominator: i64`。【注释声明】有理向量 `repr = numerator/denominator` 表示 `exp(i*pi*repr)`，坐标按模 `2Z^rank` 理解。

- `reduce_raw`【已实现】：模数 `modulus = 2 * denominator`（普通乘法，未做溢出检查）；逐坐标 `*entry = entry.rem_euclid(modulus)`，即分子被约化进 `[0, 2*denominator)`。
- `exp_pi`【已实现】：直接委派 `reduce_raw`。
- `exp_2pi`【已实现】：若 `denominator % 2 == 0` 则分母减半；否则逐分子 `checked_mul(2)`，溢出 → `StructureError::ArithmeticOverflow`；随后 `reduce_raw`。【注释声明】对应上游 `repr *= 2` 只消去偶分母因子 2 的行为。
- `add`【已实现】：
  1. `gcd = gcd_u64(self.denominator, right.denominator)`；`factor_self = right.denominator/gcd`、`factor_right = self.denominator/gcd`，两次 `i64::try_from` 失败 → `ArithmeticOverflow`。
  2. 新分母 `self.denominator * factor_self`（`checked_mul`，溢出 → `ArithmeticOverflow`）；分子逐坐标 `checked_mul`/`checked_add`（溢出 → `ArithmeticOverflow`）；分子向量经 `try_capacity(rank)?` 预分配。
  3. 归一化：`divisor` 取分母与全部分子的 gcd（`unsigned_abs`；`divisor == 1` 提前 break）；`divisor > 1` 时整除分母与分子（`i64::try_from(divisor)` 失败 → `ArithmeticOverflow`）。
  4. 之后每坐标**至多一次**条件减法：`if *entry >= 2*denominator { *entry -= 2*denominator; }`（`2 * denominator` 为普通乘法）。【注释声明】上游只纠正落在 `[2,4)` 的和，故反射后的非规范加数可原样通过。
- `add_torus_part`【已实现】：对 `part.bit(index) == Some(true)` 的坐标，若 `numerator[index] < denominator` 则加 `denominator`，否则减 `denominator`。**当前字节无任何错误分支，恒返回 `Ok(())`**（返回 `Result` 仅为签名形态）。
- `evaluate_at`【已实现】：逐坐标 `i64::from(coefficient) * numerator[index]` 累加（`checked_*`，溢出 → `ArithmeticOverflow`）；返回 `(dot.rem_euclid(2 * self.denominator), self.denominator)`，**不归一化**（`2 * self.denominator` 为普通乘法）。
- `negative_at`【已实现】：同法累加 `dot`；若 `dot % self.denominator != 0` → `StructureError::KgbInvariantViolation { invariant: "integral root evaluation" }`；否则返回 `(dot / self.denominator) % 2 != 0`（奇为 compact）。【注释声明】要求配对为整值。
- `simple_reflect`【已实现】：`pairing = <root, numerator>`（`checked_*`）；随后 `numerator[i] -= pairing * coroot[i]`（`checked_mul`/`checked_sub`，溢出 → `ArithmeticOverflow`）。**之后不做任何约化**——【注释声明】这正是负分子（如 `[0,-1]/2`）能到达打印输出的原因。
- `imaginary_cross_act`【已实现】：先 `evaluate_at(root)` 得 `(remainder, denominator)`；`r_numerator = remainder - denominator`；若非零，则构造 `shift = coroot * (-r_numerator)`（`checked_mul`）与加数 `Self::exp_2pi(shift, 2 * denominator)?`，再 `*self = self.add(&addend)?`；为零则不动作。
- `log_2pi`【已实现】：`RationalWeight::new(self.numerator.clone(), self.denominator.checked_mul(2).ok_or(ArithmeticOverflow)?)`，原样返回其 `Result`。【注释声明】仅做 gcd 归一化、不按模 1 约化。

### 2.3 `fingerprint`（私有）——去重键

【已实现】流程：
1. `log = torus.log_2pi()?`；`rank = theta.lattice_rank()`。
2. 由 `theta.weight_matrix()` 逐行克隆并对对角元 `checked_add(1)`（溢出 → `ArithmeticOverflow`），构造 `theta + I`。
3. `adapted = adapted_basis(&theta_plus_one, budget)?`；取 `components = adapted.diagonal.len()`。
4. 对每列 `column in 0..components`：以 `i128` 累加 `Σ_row entry * log.numerator()[row]`，其中 `entry = i64::try_from(adapted.basis.entry(row, column)).map_err(|_| ArithmeticOverflow)?`；再 `accumulator.rem_euclid(i128::from(log.denominator()))` 并经 `i64::try_from`（失败 → `ArithmeticOverflow`）压入投影向量。
5. 返回 `RationalWeight::new(projected, log.denominator())`。

【注释声明】注释称：上游用完整 `row_saturate(theta^T + I)` 矩阵保留 `rank` 个分量；非零行恰为同一饱和像的适应基，两基相差一个幺模（从而 mod-d 可逆）左因子，故仅保留 `diagonal.len()` 个适应基分量是无损键。本包不对该无损性作验收。
【注释声明】注释称整数格预算把条目限制在 64 位内，故 `i128` 累加不可能溢出。

### 2.4 `fundamental_fiber` / `square_class_generators` / `fundamental_coweights`（私有）

- `fundamental_fiber(delta, budget)`【已实现】：
  - `negative_coweight_eigenspace(delta.coweight_matrix(), budget)?` → `reduce_basis_mod_two(...)?` 作为分母子空间。
  - 方程子空间 `ModTwoSubspace::new(rank)?`：对 `delta.coweight_matrix()` 每行，先校验 `row.len() == rank`，否则 → `StructureError::InvalidInvolution`；收集奇条目列下标，再**额外压入对角下标 `row_index`**（【注释声明】因 `ModTwoVector::from_ones` 对重复位做 xor 翻转，这实现 `+ I` 项）；`equations.insert(...)?`。
  - `numerator = equations.right_kernel()?`；返回 `ModTwoSubquotient::new(numerator, denominator)`（返回 `Result`，原样作为函数返回值）。
  - 【注释声明】对应 `dualPi0(-delta^t)`（tori.cpp:163-175 经 cartanclass.cpp:209-212），与 `CartanFiber::build_owned`（cartan_fiber.rs:69-87）同公式以使基序一致。
- `square_class_generators(inner_class, budget)`【已实现】：
  - `twist = inner_class.generator_twist()?`；`fixed` = 满足 `image == generator` 的生成元下标（升序）。
  - 对 `distinguished_involution().involution().coweight_matrix()` 逐元 `checked_neg()`（溢出 → `ArithmeticOverflow`）得 `-delta_Y`；对其取 `negative_coweight_eigenspace`（即 `+1` 特征空间）并 `reduce_basis_mod_two` 得 `v_plus`。
  - 对 `v_plus.basis_vectors()` 的每个向量：在 `fixed` 各位置求对应单根（`datum.simple_roots()[generator].as_slice()`）与该向量的配对（`evaluation += i64::from(coefficient)`，**普通 `+=`，未检查溢出**），奇则在 `mapped` 子空间插入对应位。
  - `mapped.pivot_rows()` 标记主元位置；**非主元位置按升序**选出，返回其原生成元下标。
  - 【注释声明】对应 tits.cpp:318-356 `compute_square_classes`。
- `fundamental_coweights(datum)`【已实现】：
  - `cartan = datum.cartan_matrix()`；`rank == 0` 时立即返回空 `Vec`。
  - 用 `malachite::Rational` 对 `[C | I]` 做精确有理高斯消元：每列在 `column..rank` 内找非零主元，找不到 → `StructureError::InvalidCartanMatrix`；行交换、主元行归一、全列消元。
  - 每个生成元 `i`：坐标 `j = Σ_k inverse[k][i] * coroot_k[j]`（跳过零系数）；公分母用 lcm 式累积 `denominator *= next / gcd(denominator, next)`（`i64::try_from(value.denominator_ref())` 与 `checked_mul` 失败 → `ArithmeticOverflow`）。
  - `scaled = value * Rational::from(denominator)`；`Integer::try_from(&scaled)` 失败 → `KgbInvariantViolation { invariant: "fundamental coweight fraction" }`；`i64::try_from(&integer)` 失败 → `ArithmeticOverflow`。
  - 逐项 `RationalWeight::new(numerator, denominator)?` 收集。
  - 【注释声明】对应 `RootDatum::fundamental_coweight`（rootdata.cpp:1015-1018）。

### 2.5 `GlobalKgb` 的字段与布局不变量

【已实现】字段（均私有）：

| 字段 | 类型 | 含义（由构建/读取代码读出） |
|---|---|---|
| `semisimple_rank` | `usize` | 每元素的生成元槽数 |
| `lattice_rank` | `usize` | 打印标签宽度等的依据 |
| `elements` | `Vec<GlobalTorusElement>` | 元素存储（先到者代表元） |
| `element_packet` | `Vec<usize>` | 元素 → tau 包下标 |
| `involutions` | `Vec<InvolutionId>` | 生成顺序的扭对合列表 |
| `first_of_tau` | `Vec<usize>` | 包边界：`first_of_tau[i]..first_of_tau[i+1]` 为第 i 包 |
| `statuses` | `Vec<Option<KgbStatus>>` | 扁平 `(element, generator)` 状态 |
| `cross` | `Vec<usize>` | 扁平 cross 目标（构造期哨兵 `usize::MAX`） |
| `cayley` | `Vec<Option<usize>>` | 扁平 Cayley 目标 |
| `inverse_cayley` | `Vec<Option<(usize, Option<usize>)>>` | 扁平逆 Cayley（第一、第二原像） |
| `involution_lengths` | `Vec<usize>` | 按包（对合）索引 |
| `involution_cartans` | `Vec<usize>` | 按包索引的 Cartan 类号 |
| `involution_words` | `Vec<String>` | 按包索引的打印字 |
| `header_offset` | `RationalWeight` | 打印头偏移 |

【已实现】扁平索引一律为 `x * semisimple_rank + generator`（注释称对应上游 `data[s][x]` 布局）。
【已实现】读取接口全部经 `.get(...)`，越界返回 `None`（见 §3）；构造期内部写入多用直接索引（见 §7）。

### 2.6 `GlobalKgb::build` 构建流程（分阶段）

签名前置条件【已实现】：`table.inner_class() != inner_class` → `StructureError::DatumMismatch`。【注释声明】另要求 `classification` 是 `inner_class` 的 Cartan 分类（字节中未见对该条的显式检查）。上游按 `inner_class::global_KGB_size` 精确预留；本移植改为动态增长（注释称该预测只是分配提示）。

阶段 A【已实现】：对 `0..classification.cartan_classes().len()` 逐个 `table.add_cartan(classification, CartanId(cartan))?`。

阶段 B（对合生成，`generate_involutions`）【已实现】：
- `twist = inner_class.generator_twist()?`；`identity = WeylElement::identity(system)?`；`identity_id = table.lookup(&identity).ok_or(KgbInvariantViolation{"fundamental involution"})?`。
- `involutions` 以 `identity_id` 播种下标 0；`involution_location = vec![usize::MAX; total_involutions]`（`total_involutions = table.involution_count()`）记录生成序号。
- 长度区间 BFS：`while end_length < involutions.len()`，区间 `[start_length, end_length)`；外层生成器 `0..semisimple_rank`，内层区间下标。
- 对每个 `(generator, parent)`：`element.right_multiply_simple(system, twist[generator])?` 得 `(transported, change)`；`commutes = (change > 0) == transported.has_left_descent(system, generator)?`（【注释声明】同 inner_class.rs:522-531 的 `hasTwistedCommutation` 移植）。`commutes` 时 `child = table.cayley(generator, parent)?.ok_or(KgbInvariantViolation{"involution generation cross link"})?`，否则 `child = table.cross(generator, parent)?`。
- 未见过的 `child` 追加到 `involutions` 并记录序号。
- 收尾：`involutions.len() != total_involutions` → `KgbInvariantViolation{"involution generation count"}`。

阶段 C（每包派生数据）【已实现】：对每个对合 id 记录 `record.involution_length()`、`table.cartan_of(id).ok_or(KgbInvariantViolation{"involution Cartan class"})?.0`、`format_involution_word(&inner_class.canonical_involution_expr(record.weyl_element())?)`。`record` 缺失 → `StructureError::IndexOutOfRange { index: id.0, upper_bound: total_involutions }`。

阶段 D（基本纤维播种）【已实现】：
- `delta = inner_class.distinguished_involution().involution().clone()`；`fiber = fundamental_fiber(&delta, budget)?`；`fiber_basis = fiber.basis_representatives()`。
- `fiber_size = 2^fiber_rank`（`u32::try_from(fiber_rank)` 与 `checked_shl`，失败 → `ArithmeticOverflow`）。
- `generators = square_class_generators(inner_class, budget)?`；`coweights = fundamental_coweights(datum)?`。
- `subset_count = 2^generators.len()`（同样检查）；对每个 `subset`：`rcw = Σ_{bit 置位} coweights[generator]`（`RationalWeight::zero(lattice_rank)?` 起步，`rcw.add(...)?` 累加）；对每个 `lift in 0..fiber_size`：`torus = GlobalTorusElement::exp_pi(rcw.numerator().to_vec(), rcw.denominator())`；按 lift 位将 `fiber_basis` 代表元 `xor_assign` 进 `ModTwoVector::zero(lattice_rank)?` 并 `torus.add_torus_part(&part)?`；`store.push(torus, 0)?`（全部落入包 0）。
- 播种结束后 `first_of_tau.push(store.len())`。
- 去重表 `HashMap<(usize, RationalWeight), usize>`：以 `(identity_id.0, fingerprint(theta, torus, budget)?)` 为键插入全部种子；**重复键 → `KgbInvariantViolation{"fundamental fiber distinctness"}`**。

阶段 E（cross/Cayley 闭包，`generate`）【已实现】：
- 同样是长度区间 BFS，但作用于包区间：`while end_length < first_of_tau.len() - 1`，内层 `index in start_length..end_length`。
- 对每个 `(generator, index)`：
  - `cross_target = table.cross(generator, tw_id)?`；`new_number = involution_location[cross_target.0]`；`is_new = new_number >= first_of_tau.len() - 1`；若 `is_new && new_number != first_of_tau.len() - 1` → `KgbInvariantViolation{"involution generation order"}`。
  - `imaginary = new_number == index && !has_descent`（`has_descent` 来自 `record.weyl_element().has_left_descent(system, generator)?`）。
  - 对包内每个元素 `x`：
    - `length_change = target_weyl_length - source_weyl_length`；奇数 → `KgbInvariantViolation{"cross length parity"}`；`d = length_change / 2`。
    - `d != 0`：`child.simple_reflect(simple_root, simple_coroot)?`；否则若 `imaginary`：`child.imaginary_cross_act(...)?`；否则 `child` 为原样克隆。
    - 键 `(cross_target.0, fingerprint(theta, &child, budget)?)`：命中则复用既有下标；未命中时——若 `!is_new` → `KgbInvariantViolation{"cross image inside closed packet"}`；若 `involution_cartans[new_number] != involution_cartans[index]` → `KgbInvariantViolation{"cross Cartan class"}`；否则新建元素并入表。
    - 写 `store.cross[x * semisimple_rank + generator] = k;`。
    - 状态：`d != 0` → `KgbStatus::Complex`；`imaginary` → 依 `negative_at(simple_root)?` 取 `ImaginaryCompact`/`ImaginaryNoncompact`；否则（实根）要求 `k == x`，否则 → `KgbInvariantViolation{"real cross image"}`，置 `KgbStatus::Real`。
  - 若 `imaginary`（Cayley 链接）：`cayley_target = table.cayley(generator, tw_id)?.ok_or(KgbInvariantViolation{"Cayley involution"})?`；同样计算 `new_number/is_new` 并校验生成顺序；仅对状态为 `Some(KgbStatus::ImaginaryNoncompact)` 的 `x`：child 为**不修改的克隆**，按键去重或新建；`store.cayley[...] = Some(k)`；`inverse_cayley` 槽：`None → Some((x, None))`，`Some((first, _)) → Some((*first, Some(x)))`（首写者居 `.0`，次写者居 `.1`）。
  - 若 `is_new`：`first_of_tau.push(store.len())`。
- 收尾扫描：任一 `status` 仍为 `None` → `KgbInvariantViolation{"element status"}`。

阶段 F（打印头偏移）【已实现】：`dual_two_rho`（长 `lattice_rank` 的零向量）累加所有满足 `system.positivity()[id.0]` 的条目的 coroot 坐标（`checked_add`，溢出 → `ArithmeticOverflow`；`zip` 截断语义，见 §7）；`header_offset = GlobalTorusElement::exp_2pi(dual_two_rho, 4)?.log_2pi()?`。【注释声明】对应 `Tg.torus_element_offset().log_2pi()`，即 `exp_2pi(RatWeight(dual_twoRho, 4))`。

### 2.7 打印层

- `format_rational_weight`【已实现】：`[n0,n1,...]/d`（括号内逗号分隔、无空格）。【注释声明】对应 `ratvec::operator<<`。
- `format_involution_word`【已实现】：条目 `n >= 0` → 字符 `(b'1' + n as u8) as char` 加 `^`；条目 `< 0` → `(b'1' + (!n) as u8)` 加 `x`；结尾补 `e`。【注释声明】上游以字符 `'1' + n` 渲染生成元，此处连怪癖一并复现。
- `GlobalKgbPrint::render`【已实现】：
  - `width = digits(size.saturating_sub(1))`；`cwidth`/`lwidth` 分别取**末行** `cartan`/`length` 的位数，无行时为 1；`label_width = 3 * lattice_rank + 3`。
  - 先输出 `header` 与 `\n`；每行：`{index:>width$}:  `、`{length:>lwidth$}`、两个空格、`[` + 状态字符（`C`/`c`/`n`/`r` 对应 `Complex`/`ImaginaryCompact`/`ImaginaryNoncompact`/`Real`，逗号分隔）+ `]`、一个空格、各 cross 目标按 `width + 2` 右对齐、两个空格、各 cayley 目标按 `width + 2` 右对齐（`None` 输出 `*`）、两个空格、标签按 `label_width` 左对齐填充、一个空格、`{cartan:>cwidth$} `（含尾随空格）、`involution_word`、`\n`。
- `GlobalKgb::print_layout`【已实现】：构造 `header = "\\exp(i\\pi\\check\\rho) = \\exp(2i\\pi({offset}))"`；逐元素组行：状态缺失 → `KgbInvariantViolation{"element status"}`；cross 缺失 → `IndexOutOfRange { index: element, upper_bound: size }`；`torus_label` 直接调用 `self.elements[element].log_2pi()?`（传播错误，**不**吞错——与 `torus_label()` 读取器不同）；`length`/`cartan`/`involution_word` 取自按包索引数组（直接索引）。

## 3. 查询接口逐项

| 方法 | 行为（已实现） | 备注 |
|---|---|---|
| `size` | `elements.len()` | |
| `semisimple_rank` / `lattice_rank` | 返回同名字段 | |
| `packet_count` | `involutions.len()` | 注释：每扭对合一包 |
| `tau_packet(index)` | `index + 1 < first_of_tau.len()` 时 `Some(first_of_tau[index]..first_of_tau[index+1])`，否则 `None` | 对应 `KGB_base::tauPacket`（注释） |
| `status(element, generator)` | `*statuses.get(element * semisimple_rank + generator)?` | **越界与槽内 `None` 合并为 `None`** |
| `cross(generator, element)` | `cross.get(...).copied()` | 注意参数顺序与 `status` 相反（`generator` 在前） |
| `cayley(generator, element)` | `.copied().flatten()` | `None` 即上游 `UndefKGB`（打印 `*`） |
| `inverse_cayley(generator, element)` | `.copied().flatten()` | 第一及（二值 Cayley 时）第二原像 |
| `length(element)` | 经 `element_packet` → `involution_lengths`，两级 `.get` | 对应 `KGB_base::length`（注释） |
| `cartan_of(element)` | 经 `element_packet` → `involution_cartans` | 对应 `KGB_base::Cartan_class`（注释） |
| `torus_label(element)` | `self.elements.get(element)?.log_2pi().ok()` | **错误被 `.ok()` 吞为 `None`** |
| `involution_word(packet)` | `involution_words.get(packet).map(String::as_str)` | |
| `print_layout()` | 见 §2.7 | 返回 `Result` |

【推断】`build` 成功返回后 `cross` 槽不再残留构造期哨兵 `usize::MAX`：状态收尾扫描保证每个 `(x, generator)` 都被访问，而 cross 写入先于同一迭代内的状态写入。此结论由控制流推出，字节中无显式断言。

## 4. 错误分支汇总

本文件可见的 `StructureError` 变体：`ArithmeticOverflow`、`DatumMismatch`、`InvalidInvolution`、`InvalidCartanMatrix`、`IndexOutOfRange { index, upper_bound }`、`AllocationFailed { requested }`、`KgbInvariantViolation { invariant }`。

`KgbInvariantViolation` 的 `invariant` 字面量清单（逐字节摘录）：

| 字面量 | 触发位置 |
|---|---|
| `"integral root evaluation"` | `negative_at`：`dot % denominator != 0` |
| `"fundamental coweight fraction"` | `fundamental_coweights`：缩放后非整数 |
| `"fundamental involution"` | `build`：恒等对合未在表中 |
| `"involution generation cross link"` | `build` 阶段 B：扭转 commuting 时 `table.cayley` 为 `None` |
| `"involution generation count"` | `build` 阶段 B 收尾：生成数 ≠ 表计数 |
| `"involution Cartan class"` | `build` 阶段 C：`table.cartan_of` 为 `None` |
| `"fundamental fiber distinctness"` | `build` 阶段 D：种子指纹冲突 |
| `"involution generation order"` | `build` 阶段 E（cross 与 Cayley 两处）：新包序号 ≠ 下一待开包 |
| `"cross length parity"` | `build` 阶段 E：cross 两端 Weyl 长度差为奇 |
| `"cross image inside closed packet"` | `build` 阶段 E：已封闭包内出现新指纹 |
| `"cross Cartan class"` | `build` 阶段 E：cross 两端 Cartan 类号不一致 |
| `"real cross image"` | `build` 阶段 E：实根情形 `k != x` |
| `"Cayley involution"` | `build` 阶段 E：`table.cayley` 为 `None` |
| `"element status"` | `build` 收尾扫描；`print_layout` 组行 |

其余错误分支：`DatumMismatch`（`build` 入口）；`InvalidInvolution`（`fundamental_fiber` 行长不符）；`InvalidCartanMatrix`（`fundamental_coweights` 列无主元）；`IndexOutOfRange`（`build` 多处 `record` 查找与 `print_layout` 的 cross 缺失）；`AllocationFailed { requested: semisimple_rank }`（`ElementStore::push` 内四处 `try_reserve`）；`ArithmeticOverflow`（散布于全部 `checked_*`/`try_from`/`checked_shl` 站点，见 §2 各条）。

## 5. 预算与容量纪律

- `IntegerLatticeBudget` 经参数传入并透传到 `adapted_basis`、`negative_coweight_eigenspace`（后者经 `fundamental_fiber`、`square_class_generators`）以及 `fingerprint`。【注释声明】`fingerprint` 内注释称该预算把格条目限制在 64 位，从而其 `i128` 累加不会溢出。
- 大 Vec 的分配一律经 `crate::grading::try_capacity(n)?` 预分配（`add` 的分子、`fingerprint` 投影、`build` 各表、`print_layout` 行等）；其具体错误形态在调用点不可见（经 `?` 以 `StructureError` 传播，【推断】属同一错误类型族）。
- 规模上界检查：`fiber_size = 2^fiber_rank` 与 `subset_count = 2^generators.len()` 都经 `u32::try_from` + `checked_shl`，失败 → `ArithmeticOverflow`。
- 测试预算字面值（§9）：`IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`；`AdjointFiberBudget::new(同合格预算, 50_000, 100_000)`；`CartanClassificationBudget::new(..., weyl, 64, 64)`；`InvolutionTableBudget::new(64, lattice_budget())`。

## 6. 顺序与去重保证（已实现）

- 对合按长度区间 BFS 生成；同区间内按生成器 `0..semisimple_rank` 外序、区间下标内序访问；`involution_location` 以 `usize::MAX` 为“未生成”哨兵。
- tau 包按对合生成顺序依次开启；`first_of_tau` 单调递增，构造完成后长度为 `packet_count + 1`（由 `tau_packet` 的边界判断与构建流程可见）。
- 元素去重键为 `(involution_id.0, fingerprint)`（`HashMap<(usize, RationalWeight), usize>`）；命中即用既有下标，不覆盖——先到代表元得以保留（与模块文档一致）。
- `inverse_cayley` 槽的写入顺序决定 `.0`（首）与 `.1`（次）。
- `render` 的列宽依赖元素总数与**末行**的 `cartan`/`length` 位数。

## 7. panic / 断言 / 隐式前置条件（字节级观察）

- 非测试代码中**无**显式 `panic!`/`assert!`/`unreachable!`。
- 以下直接索引在违反前置条件时会 panic，字节中无守卫：`self.numerator[index]`（`add`/`add_torus_part`/`evaluate_at`/`negative_at`/`simple_reflect`，要求权重/根长度 ≤ 分子长度）、`twist[generator]`、`datum.simple_roots()[generator]`、`involution_location[child.0]`（含 cross/cayley 目标）、`store.statuses[...]`/`store.cross[...]` 等扁平写入、`first_of_tau[index + 1]`、`print_layout` 内的 `self.element_packet[element]` 等。
- 未检查算术（普通运算符，溢出时 debug panic / release 回绕）：`reduce_raw`、`evaluate_at`、`add` 末尾、`imaginary_cross_act` 中的 `2 * denominator`；`square_class_generators` 的 `evaluation += ...`。
- 除法/取余：`negative_at` 的 `dot % self.denominator`、`dot / self.denominator` 在 `denominator == 0` 时 panic；构造路径未见对 `denominator > 0` 的显式校验。
- `zip` 截断语义：`simple_reflect` 的 `numerator.zip(coroot)`（长度不一致时较短者截断更新）、`build` 阶段 F 的 `dual_two_rho.zip(coroot.as_slice())`。
- `format_involution_word` 中 `entry as u8` 为截断转换（大数值会回绕）；`b'1' + ...` 加法亦未检查。
- `fundamental_fiber` 中 `from_ones` 对重复下标做 xor 翻转（注释点明此行为被有意利用）。
- 测试代码使用 `.unwrap()` 与 `assert_eq!`（§9）。

## 8. 与其它模块的接口（仅调用点可见事实）

- `InnerClass`：`datum()`、`root_system()`、`generator_twist()?`（可按下标取 `usize` 像、有 `len()`）、`distinguished_involution().involution()`（得 `&LatticeInvolution`）、`canonical_involution_expr(weyl_element)?`（结果作 `&[i32]` 传入 `format_involution_word`）、`inner_class()`（供 `InvolutionTable` 比较）。
- `InvolutionTable`：`inner_class()`；`add_cartan(&CartanClassification, CartanId)?`；`lookup(&WeylElement) -> Option<InvolutionId>`；`involution_count() -> usize`；`record(InvolutionId) -> Option<_>`，记录上有 `weyl_element()`、`weyl_length()`、`involution_length()`、`theta()`（得 `&LatticeInvolution`）；`cayley(gen, id)? -> Option<InvolutionId>`；`cross(gen, id)? -> InvolutionId`；`cartan_of(id) -> Option<_>`（取 `.0` 得 `usize`）。
- `BasedRootDatum`：`semisimple_rank()`、`lattice_rank()`、`cartan_matrix()`（行可迭代取整条目）、`simple_roots()`/`simple_coroots()`（元素有 `.as_slice() -> &[i32]`）。
- 根系/Weyl：`system.entries()` 产出 `(id, _, coroot)`，`id.0` 索引 `system.positivity()`；`WeylElement::identity(system)?`；`right_multiply_simple(system, s)? -> (WeylElement, change)`，`change` 可与 0 比较；`has_left_descent(system, s)? -> bool`。
- `LatticeInvolution`：`lattice_rank()`、`weight_matrix()`、`coweight_matrix()`（均为行迭代矩阵，条目为整数）。
- `integer_lattice`：`adapted_basis(&matrix, budget)?`，结果有 `.diagonal`（取 `len()`）与 `.basis.entry(row, column)`（`i64::try_from` 可转）；`negative_coweight_eigenspace(&matrix, budget)?`；`reduce_basis_mod_two(&space)?`（产出 `ModTwoSubspace`）。
- `mod_two`：`ModTwoSubspace::new(rank)?`、`insert(vec)?`、`right_kernel()?`、`basis_vectors()`、`pivot_rows()`（产出 `(pivot, _)`）；`ModTwoVector::from_ones(rank, ones)?`、`zero(rank)?`、`bit(index) -> Option<bool>`、`xor_assign(&other)?`；`ModTwoSubquotient::new(numerator, denominator)` 返回 `Result`；`basis_representatives()` 产出可迭代代表元。
- `RationalWeight`：`new(numerator, denominator)` 返回 `Result`；`zero(rank)?`；`add(&other)?`；`numerator()`/`denominator()`。作为 `HashMap` 键使用（调用点要求其具备相等/散列能力）。
- `KgbStatus` 变体：`Complex`、`ImaginaryCompact`、`ImaginaryNoncompact`、`Real`；支持 `==` 比较与拷贝。
- `malachite`：`Rational::{ONE, ZERO}`（经 `One`/`Zero` trait）、`Rational::from(i32/i64)`、`/=`、`*`、`+=`、`-=`、`denominator_ref()`；`Integer::try_from(&Rational)`。
- `try_capacity(n)?`（`crate::grading`）：泛型 Vec 预分配，错误经 `?` 以 `StructureError` 传播（具体变体调用点不可见）。
- 测试专用：`BasedRootDatum::from_simple_data(rank, cartan, weights, coweights)`、`LatticeInvolution::identity(&datum)`、`InnerClass::new(datum, distinguished, roots_budget)`、`CartanClassification::build(&inner_class, &budget)`、`InvolutionTable::new(&inner_class, budget)`、`Weight::new(vec)`、`Coweight::new(vec)`。

## 9. 测试锚点（`#[cfg(test)] mod tests`）

辅助函数：`class_budget(weyl)`、`lattice_budget()`、`build_global_kgb(datum, roots, weyl)`（恒等对合为 distinguished，`InnerClass::new` → `CartanClassification::build` → `InvolutionTable::new` → `GlobalKgb::build`，全链 `.unwrap()`）。

数据构造器（字面值精确）：
- `simply_connected_a1`：`from_simple_data(1, [[2]], [Weight([2])], [Coweight([1])])`。
- `adjoint_a1`：`from_simple_data(1, [[2]], [Weight([1])], [Coweight([2])])`。
- `simply_connected_b2`：`from_simple_data(2, [[2,-2],[-1,2]], [Weight([2,-2]), Weight([-1,2])], [Coweight([1,0]), Coweight([0,1])])`；注释称 B2 有 8 根、Weyl 群阶 8。

测试用例：
1. `simply_connected_a1_print_matches_hpc_reference`：`build_global_kgb(datum, 2, 2)`；断言 `print_layout().unwrap().render()` 等于 5 行期望字节，头部为 `\exp(i\pi\check\rho) = \exp(2i\pi([1]/4))`，末行 `4:  1  [r]   4    *   [0]/1 1 1^e`。
2. `adjoint_a1_print_matches_hpc_reference`：3 行，头部 `([1]/2)`。
3. `simply_connected_b2_print_matches_hpc_reference`：`build_global_kgb(datum, 8, 8)`；17 行，头部 `([0,3]/4)`；元素 15 的标签为 `[0,-1]/2`（注释称该负分子钉死了反射后不再约化的纪律）；期望串中行 0–9 以 `\x20` 前导空格（`width = digits(16) = 2` 的对齐结果）；包含字 `1x2^e`、`2x1^e`、`1^2x1^e`。
4. `b2_packet_structure_and_link_invariants`：`size() == 17`；`packet_count() == 6`；各包大小 `[8, 2, 2, 2, 2, 1]`；包字 `["e", "1^e", "2^e", "1x2^e", "2x1^e", "1^2x1^e"]`；全图 cross 的对合性（`cross(g, cross(g, x)) == Some(x)`）；Cayley 与 inverse-Cayley 的配对一致性。
- 【注释声明】测试注释引用 fixture `tests/fixtures/domain/print_x.atlas` 与参考事件 `tests/reference/domain/print_x.events.json`（首个/第二/第三个 print_X 块）。
- 【注释声明】文件末尾 NOTE：半单秩 0 边界（平凡群与一维环面）有意未测，因共享内类机制在空生成元集合上 panic（weyl_transducer.rs:485，`InnerClass::new` 内越界），注释称修复超出本模块范围。

## 10. 限制与未覆盖面

- 【注释声明】未移植：上游第二构造器（任意 `GlobalTitsElement` 播种）与 Bruhat/Hasse 层。
- 【已实现】容量策略与上游不同：动态增长，未使用 `global_KGB_size` 预测（注释称其为纯分配提示）。
- 【已实现】`torus_label()` 读取器把 `log_2pi` 的错误吞为 `None`；`print_layout` 则传播同一错误——两条路径错误语义不一致（字节事实，不作评价）。
- 【已实现】`add_torus_part` 返回 `Result` 但当前无任何错误分支。
- 【已实现】无任何错误分支的单元测试（`DatumMismatch`、各 `KgbInvariantViolation`、溢出、`InvalidCartanMatrix` 等均无覆盖）；测试仅覆盖三个 datum 的打印字节与 B2 结构不变量；半单秩 0 未测（见 §9 NOTE）。
- 【已实现】多处隐式前置条件（维度匹配、`denominator != 0`、表返回 id 在界内）依赖调用方/构造不变量，违反即 panic（§7），无显式断言信息。
- 【推断】`GlobalKgb` 未实现 `Eq`/`PartialEq`/`Hash`（derive 仅 `Clone, Debug`）；快照对比只能借助 `GlobalKgbPrint`（derive 含 `Eq, PartialEq`）。
- 【注释声明】所有“对应上游某文件某行”的说法、以及 `fingerprint` 键的无损性论证，均为源码注释内容；本包未核对上游，亦不对其数学性质验收。
