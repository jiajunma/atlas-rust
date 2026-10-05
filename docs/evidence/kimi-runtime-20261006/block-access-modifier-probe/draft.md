---
title: Atlas Rust 重实现：BlockTopology 只读块拓扑与 block_modifier Weyl 姿态修正（block_access.rs / block_modifier.rs）来源包草案
source: atlas-rust/block-access-modifier
ingestedAt: 2026-10-05T18:00:00Z
---

# 0. 范围与核对约定

- 本草案仅依据两份完整源文件字节：
  - `crates/atlas-real-group/src/block_access.rs`
  - `crates/atlas-real-group/src/block_modifier.rs`
- 标注约定：**【已实现】**＝可直接从所给字节（含文档注释）读出的行为或设计陈述；**【阅读推断】**＝由代码用法推出、但相关定义不在所给字节内的结论。
- 不做任何数学验收、性能或正确性声明；所有上游行号均转述自源文件注释，未独立核实。
- `BlockDescent`、`BlockGraph`、`PartialBlock`、`RepContext`、`BlockLocator`、`RationalWeight`、`StructureError` 等类型的定义本体不在所给字节内，本包只描述其在两文件中的使用面。

# 1. 文件一裸签名清单：`block_access.rs`

导入：`std::collections::BTreeSet`；`std::sync::Arc`；`crate::{BlockDescent, BlockGraph, PartialBlock}`。
本文件**不定义任何 struct/enum**；无 `#[cfg(test)]` 模块。

```rust
pub(crate) mod sealed {
    pub trait Sealed {}   // 无方法
}

pub trait BlockTopology: sealed::Sealed {
    fn size(&self) -> usize;
    fn rank(&self) -> usize;
    fn length(&self, element: usize) -> Option<usize>;
    fn descent(&self, element: usize, generator: usize) -> Option<BlockDescent>;
    fn cross(&self, element: usize, generator: usize) -> Option<usize>;
    fn cayley(&self, element: usize, generator: usize)
        -> Option<(Option<usize>, Option<usize>)>;
    fn inverse_cayley(&self, element: usize, generator: usize)
        -> Option<(Option<usize>, Option<usize>)>;
}

pub fn bruhat_hasse<B: BlockTopology + ?Sized>(block: &B) -> Vec<Vec<usize>>;
```

私有项（非 pub，列出仅供完整性核对）：

```rust
fn insert_ascents<B: BlockTopology + ?Sized>(
    block: &B,
    hr: &[usize],
    s: usize,
    hs: &mut BTreeSet<usize>,
);
```

可见 impl 块（均无文档注释，全为委托/标记实现）：

- `impl sealed::Sealed for BlockGraph`
- `impl BlockTopology for BlockGraph`
  - `size→BlockGraph::size`，`rank→BlockGraph::rank`，`length→BlockGraph::length`，`descent→BlockGraph::descent_value`（**注意名称不同**），`cross→BlockGraph::cross`，`cayley→BlockGraph::cayley`，`inverse_cayley→BlockGraph::inverse_cayley`；参数顺序均不变。
- `impl sealed::Sealed for PartialBlock`
- `impl BlockTopology for PartialBlock`（`cross`/`cayley`/`inverse_cayley` 含参数交换与门控，详见 §3.1.4）
- `impl<T: BlockTopology + ?Sized> sealed::Sealed for &T`
- `impl<T: BlockTopology + ?Sized> BlockTopology for &T`（7 个方法全部逐字转发 `T::*(self, …)`）
- `impl<T: BlockTopology + ?Sized> sealed::Sealed for Arc<T>`
- `impl<T: BlockTopology + ?Sized> BlockTopology for Arc<T>`（同上转发）

# 2. 文件二裸签名清单：`block_modifier.rs`

导入：`crate::rep_table::RepTable`；`crate::{BasedRootDatum, BlockLocator, IntegralSubsystem, KgbStatus, RationalWeight, RepContext, RootId, RootSystem, StandardRepr, StandardReprMod, StructureError, WeylElement}`。

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockModifier {
    locator: BlockLocator,   // 私有字段
    shift: RationalWeight,   // 私有字段
}

impl BlockModifier {
    pub fn trivial(system: &RootSystem, simp_int: Vec<RootId>)
        -> Result<Self, StructureError>;
    pub fn from_locator(locator: BlockLocator, shift: RationalWeight) -> Self;
    pub fn clear(&mut self, system: &RootSystem, block_rank: usize)
        -> Result<(), StructureError>;
    pub fn locator(&self) -> &BlockLocator;
    pub fn shift(&self) -> &RationalWeight;
    pub fn int_sys(&self) -> u32;
    pub fn w(&self) -> &WeylElement;
    pub fn simp_int(&self) -> &[RootId];
    pub fn simple_pi(&self) -> &[usize];
}

impl RepContext<'_> {   // 对 crate 它处所定义类型的扩展 impl 块
    pub fn transform_srm<const LEFT_TO_RIGHT: bool>(
        &self, w: &WeylElement, srm: &mut StandardReprMod,
    ) -> Result<(), StructureError>;
    pub fn shift_srm(
        &self, amount: &RationalWeight, srm: &mut StandardReprMod,
    ) -> Result<(), StructureError>;
    pub fn make_diff_integral_orthogonal(
        &self, gamlam: &RationalWeight, srm: &StandardReprMod,
    ) -> Result<RationalWeight, StructureError>;
    pub fn make_relative_to(
        &self,
        loc: &BlockLocator,
        srm0: &StandardReprMod,
        bm: &mut BlockModifier,
        mut srm1: StandardReprMod,
    ) -> Result<(), StructureError>;
    pub fn sr_with_modifier(
        &self,
        srm: &StandardReprMod,
        bm: &BlockModifier,
        gamma: &RationalWeight,
    ) -> Result<StandardRepr, StructureError>;
}
```

私有自由函数：

```rust
fn simple_reflect_numerator(
    datum: &BasedRootDatum,
    generator: usize,
    numerator: &mut [i64],
    offset: i64,
) -> Result<(), StructureError>;
```

`#[cfg(test)] mod tests`（内部项均私有）：辅助 `class_budget`、`ContextFixture{inner_class, table, graph}` 及其 `rc()`、`sl3r_fixture()`、`param_srm(...)`、`rational(...)`、`same_parameter(...)`；两个测试见 §6。

# 3. 逐项解释

## 3.1 `BlockTopology` 方法契约与 sealed 模式

### 3.1.1 契约（模块级与 trait 级文档）【已实现·文档陈述】

- 定位：上游 `klsupport::KLSupport` 只消费 `const Block_base&`，不依赖具体 classic/common-block 表示；`BlockTopology` 是对应的 Rust 只读边界。trait 文档称消费方为 `KlSupport` 与 `KlTable`。
- 两层无效性约定：
  1. 外层 `None`：无效的 element/generator；
  2. 内层 `None`：`cayley`/`inverse_cayley` 返回对中未定义的 Cayley 链。
- 结构不变量（密封理由，逐点转述）：`rank() <= 32`；元素按非递减长度排序；每个 element/generator 格子都存在；每个已定义的 cross 或 Cayley 目标 `< size()`。
- 文档称：KL 构造在递归开始前校验这些不变量（校验本体不在所给字节内）。

### 3.1.2 方法形状【已实现】

- `size`/`rank`：返回 `usize`，不可失败。
- `length`/`descent`/`cross`：`Option<…>`，外层 `None` 表无效格子。
- `cayley`/`inverse_cayley`：`Option<(Option<usize>, Option<usize>)>`，内外两层语义如 §3.1.1；两个内层分量允许各自独立缺失。

### 3.1.3 sealed 机制【已实现】

- `pub(crate) mod sealed { pub trait Sealed {} }`，`pub trait BlockTopology: sealed::Sealed`。
- 文档原话要点：该可见性「允许 crate 内的不变量测试（实现该 trait），同时不允许下游 crate 实现 `BlockTopology`」。
- 泛型 `bruhat_hasse<B: BlockTopology + ?Sized>` 与 `&T`/`Arc<T>` 泛型实现均显式带 `?Sized`，因此 `&dyn BlockTopology`、`Arc<dyn BlockTopology>` 满足 `BlockTopology` 约束【已实现：impl 与 `?Sized` 存在；dyn 用法为其直接推论】。

### 3.1.4 两个具体实现者的委托细节【已实现】

- `BlockGraph`：`descent` 委托到固有方法 `descent_value`，其余同名委托。
- `PartialBlock`：
  - `descent(element, generator)` → `PartialBlock::descent(self, element, generator)`（**顺序不变**）。
  - `cross(element, generator)` → `PartialBlock::cross(self, generator, element)`（**参数交换**）。
  - `cayley(element, generator)`：先 `PartialBlock::descent(self, element, generator)?`（外层 `None` 原样传播）；若 `.is_descent()` 为真 → 返回 `Some((None, None))`；否则 `PartialBlock::cayley(self, generator, element)`（**参数交换**）。
  - `inverse_cayley`：同一下降查询；若 `!is_descent()` → `Some((None, None))`；否则同样落到 `PartialBlock::cayley(self, generator, element)`。
  - 即：对有效格子，「因下降状态而无 Cayley 链」编码为 `Some((None, None))` 而非外层 `None`；`cayley` 与 `inverse_cayley` 共用同一个固有 `cayley`，仅门控相反。
  - `is_descent()` 是 `BlockDescent` 上的方法【阅读推断：定义不在字节内】。

## 3.2 `BlockDescent` 变体语义（仅基于本文件用法）【以阅读推断为主】

`BlockDescent` 的定义不在所给字节；以下仅为 `block_access.rs` 中的使用语义：

- 出现六个变体：`ComplexDescent`、`RealTypeI`、`RealTypeII`、`ComplexAscent`、`ImaginaryTypeI`、`ImaginaryTypeII`。
- 「strict good descent」集合＝`{ComplexDescent, RealTypeI}`；`bruhat_hasse` 对每个 `z` 按生成器序号 `0..rank` 取**首个** strict good。
- 无 strict good 时只有 `RealTypeII` 参与（经 `inverse_cayley` 的第一分量）。
- `insert_ascents` 中：`ComplexAscent→cross`；`ImaginaryTypeI→cayley` 第一分量；`ImaginaryTypeII→cayley` 两个分量。
- `is_descent()` 在 `PartialBlock` 实现中用作下降/上升二分门控。
- 变体的数学定义、与 KGB 状态的对应等不在字节内，本包不断言。

## 3.3 `bruhat_hasse` 与 `insert_ascents` 行为【已实现】

- 文档：块拓扑的 Bruhat Hasse 图（`blocks.cpp:1576-1656`）；第 `z` 行列出 `z` 的**直接下邻**；要求元素非递减长度序（与 `BlockTopology` 要求一致）。
- 流程（每个 `z ∈ 0..size`，`covered = BTreeSet::new()`）：
  - `strict_good = (0..rank).find(…)`，匹配 `Some(ComplexDescent | RealTypeI)`。
  - `ComplexDescent`：`sz = cross(z, s).expect("complex descent cross")`；插入 `sz`；`insert_ascents(block, &hasse[sz], s, &mut covered)`。
  - `RealTypeI`：`(first, second) = inverse_cayley(z, s).expect("type I inverse Cayley")`；`first = first.expect("type I first image")`；插入 `first` 与可选 `second`；`insert_ascents(block, &hasse[first], s, …)`（**只用 `first` 的行**）。
  - 其它分支 → `unreachable!("strict good descent match")`（由 `matches!` 保证逻辑上不可达，但仍以 panic 形式存在）。
  - 无 strict good：遍历所有 `s`，凡 `descent(z, s) == Some(RealTypeII)` 者，`inverse_cayley(z, s).expect("type II inverse Cayley").0` 若 `Some` 则插入（**第二分量不取**）。
- 顺序保证：外行按 `0..size`；行内升序去重（`BTreeSet` 迭代序）【已实现】。
- 关键前提：`hasse[sz]`、`hasse[first]` 直接索引已推送行；若实现返回的目标 `>= z`，将以索引越界 panic；本文件不做任何校验，依赖 §3.1.1 的不变量【已实现（代码形状）；越界后果为阅读推断】。
- `insert_ascents(block, hr, s, hs)`：对 `hr` 中每个 `z`：`ComplexAscent`→`cross(z, s)` 若 `Some` 插入；`ImaginaryTypeI`→`cayley(z, s).expect("type I Cayley").0` 若 `Some` 插入（`expect` 只作用于外层，`Some((None, _))` 时静默不插入）；`ImaginaryTypeII`→`cayley(z, s)` 若 `Some((a, b))`，`a`、`b` 各自若 `Some` 插入；其余变体忽略。
- panic 清单（全文件）：`expect("complex descent cross")`、`expect("type I inverse Cayley")`、`expect("type I first image")`、`expect("type II inverse Cayley")`、`expect("type I Cayley")`、`unreachable!("strict good descent match")`，外加 `hasse[sz]`/`hasse[first]` 两处潜在索引越界。本文件无 `Result`。

## 3.4 `block_modifier`：构造与访问器

### 3.4.1 模块定位【已实现·文档陈述】

- 「nonidentity generator attitude」切片**第 2 步**：上游 `repr::block_modifier`（`gkmod/repr.h:493-499`、`repr.cpp:1401-1419`）及 `Rep_context` 方法 `transform`（`repr.cpp:712-754`）、`shift`（`352-356`）、`make_diff_integral_orthogonal`（`317-329`）、`make_relative_to`（`338-350`）、带 modifier 的 `sr`（`815-823`）的纯移植，**尚未接线**。
- 明确声明：暂无任何 `RepTable` 或其它现存消费方调用；第 3 步才在 attitude gates 后接入块查找。
- 对应表（复用而非分叉）：`StandardReprMod`（`partial_block.rs`：KGB 元 + `real_unique` 归一化的 `gamma_lambda`）；`Rep_context::sr_gamma`→`RepContext::sr_gamma`；无 modifier 的 `sr`（`808-813`）→`StandardReprMod::to_standard`（内嵌 `gamma_lambda_rho(srm) = srm.gamma_lambda() + rho`，`repr.h:329-330`）；`InvolutionTable::real_unique`→`RepContext::real_unique`；`InnerClass::integrality_codec`（`innerclass.cpp:1184-1194`）→`RepTable::integral_codec`；`Rep_context::theta_1_preimage`（`repr.cpp:297-313`）→`rep_table.rs` 的 `IntegralCodec::theta_1_preimage`。
- 一处有意偏差（文档声明对目标域无语义差异）：上游 `transform` 走 `Weyl_group().word(w)`（Weyl 转换器随元素存储的词），crate 走 `WeylElement::reduced_word`（典范最左下降约化词）；文档论证：同为同一元素的约化词、逐字母操作是（偏）群作用、双向使用同一典范词使 `transform<false>` 成为 `transform<true>` 的逐字母逆——`make_relative_to` 与 `sr` 只依赖这一点。

### 3.4.2 `BlockModifier` 构造器【已实现】

- `trivial(system, simp_int)`：`block_rank = simp_int.len()`；`locator = BlockLocator::from_parts(u32::MAX, WeylElement::identity(system)?, simp_int, (0..block_rank).collect())`；`shift = RationalWeight::zero(system.lattice_rank())?`。`u32::MAX` 对应上游 `int_sys` 的 `-1` 哨兵；文档说明：crate 的 `PartialBlock` 不保留构造上下文，故 simply-integral 单根（`simp_int`）与根系由调用者传入；该平凡 modifier 仅供局部使用、其 datum id 永不读取（`repr.cpp:1403-1409`）。
- `from_locator(locator, shift)`：直接包装，**无校验、不可失败**；文档对应 `Reduced_param::reduce` 经 `locator&` 基子对象写入查询定位器（`repr.cpp:110-125`）与 `append_block_containing` 的拷贝（`1684-1685`）；`make_relative_to` 会覆盖 `shift`。
- `clear(system, block_rank)`：**保留** `int_sys()` 与 `simp_int().to_vec()`，重置 `w = WeylElement::identity(system)?`、`simple_pi = (0..block_rank).collect()`、`shift = RationalWeight::zero(system.lattice_rank())?`（`repr.cpp:1412-1419`；注释：`int_sys` 与 `simp_int`「不是相对的」故保留）。`block_rank` 由调用者给定，本文件未见其与 `simp_int` 长度的一致性检查【已实现：无检查代码】。

### 3.4.3 访问器【已实现】

`locator() -> &BlockLocator`；`shift() -> &RationalWeight`（`repr.h:494` 的 `RatWeight shift`）；`int_sys() -> u32`（`locator::int_sys_nr`）；`w() -> &WeylElement`（`locator::w`，文档：读存储行时由 `transform<false>` 施加）；`simp_int() -> &[RootId]`；`simple_pi() -> &[usize]`。派生 `Clone, Debug, Eq, PartialEq`（字段级语义）。

### 3.4.4 `RepContext` 扩展方法【已实现】

**`transform_srm<LEFT_TO_RIGHT>(w, srm)`**（`repr.cpp:712-754`）
- `word = w.reduced_word(self.root_system())?`；`LEFT_TO_RIGHT=true` 按原序施字母（首字母先施），`false` 逆序（末字母先施）——文档保证 `transform<false>` 是 `transform<true>` 的逐字母逆。
- 逐字母按 `self.kgb_status(x, s)?` 分支：
  - `KgbStatus::Complex`：`x = self.cross_at(x, s)?`；`simple_reflect_numerator(datum, s, &mut numerator, 0)?`。
  - `KgbStatus::Real`：`x` **不变**（文档：实根 cross 平凡）；`simple_reflect_numerator(datum, s, &mut numerator, denominator)?`（文档：以 `-\rho_R` 为中心的仿射反射，offset 取分母）。
  - `KgbStatus::ImaginaryCompact | ImaginaryNoncompact`：返回 `Err(StructureError::RepInvariantViolation { invariant: "Weyl group element SRM transform on an imaginary root" })`（上游抛 `Bad Weyl group element SRM transform`）。
- 收尾：`RationalWeight::new(numerator, denominator)?`；`involution = self.involution_of(x)?`（最终 `x` 处）；`self.real_unique(involution, &mut gamma_lambda)?`（`repr.cpp:753`）；`set_x(x)` + `set_gamma_lambda(gamma_lambda)`。

**`shift_srm(amount, srm)`**（`repr.cpp:352-356`）：`gamma_lambda = srm.gamma_lambda().add(amount)?`；在 `srm.x()` 不变的对合上 `real_unique?`；回写。

**`make_diff_integral_orthogonal(gamlam, srm)`**（`repr.cpp:317-329`）
- `result = gamlam.sub(srm.gamma_lambda())?`；**若为零直接返回**（短路，跳过子系统与 codec）。
- 否则：`IntegralSubsystem::integral(self.root_system(), srm.gamma_lambda())?` → `codec = RepTable::integral_codec(self, srm.x(), &subsystem)?` → `preimage = codec.theta_1_preimage(&result)?` → `result = result.sub(&RationalWeight::from_weight(&preimage)?)?`。
- 仅 debug 构建：`evaluations = codec.internalise(&result)?`；`debug_assert!(evaluations.iter().all(|&e| e == 0), "difference made orthogonal to the integral system")`（注释对应上游 `repr.cpp:326` 断言；`internalise` 经可逆行变换 `in` 后处理求值，故与原始求值同零）。

**`make_relative_to(loc, srm0, bm, srm1)`**（`repr.cpp:338-350`），调用顺序固定：
1. `bm.locator.make_relative_to(self.root_system(), loc)?`（`343-345`；文档：`bm.w` 右乘 `loc.w^{-1}`，`bm.simple_pi` 右复合 `loc.simple_pi` 的逆）；
2. `self.transform_srm::<true>(bm.w(), &mut srm1)?`（`347`；使用**更新后**的 `bm.w`）；
3. `shift = self.make_diff_integral_orthogonal(srm1.gamma_lambda(), srm0)?`（`348-349`）；`bm.shift = shift`。

**`sr_with_modifier(srm, bm, gamma)`**（`repr.cpp:815-823`）：克隆 `srm` → `set_gamma_lambda(gamma_lambda.add(bm.shift())?)`（`819`）→ `transform_srm::<false>(bm.w(), &mut srm)?`（`820`）→ `srm.to_standard(self, gamma)`（`821-822`；文档：`to_standard` 已即无 modifier 的 `sr`，`808-813`）。

### 3.4.5 `simple_reflect_numerator`（私有）【已实现】

- 公式（注释）：`v -= alpha_s * (<v, coroot_s> + offset)`，作用于有理权分子、分母不变；对应 `rootdata.h: