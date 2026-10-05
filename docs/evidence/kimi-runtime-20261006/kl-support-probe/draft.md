---
title: KlSupport：逐块 KL 支撑数据（kl_support.rs）来源包草案
source: atlas-rust/kl-support
ingestedAt: 2026-10-05T18:00:00Z
---

# KlSupport：逐块 KL 支撑数据（来源包草案）

> 草案状态说明：本包仅依据 `crates/atlas-real-group/src/kl_support.rs` 的完整字节撰写。文中以「已实现」标记可直接从字节读出的行为，以「阅读推断」标记由代码结构推出、但未在本文件中被断言或测试固定的结论。本包不含任何数学验收、性能或正确性声明；上游（gkmod）行号引用均来自文件内注释，未独立核实。

## 1. 文件概览与上游对应（来自模块文档注释）

- 模块职责：为每个块元素预计算
  - 下降集（"tau-invariant"）与 good-ascent 集（注释指向 klsupport.cpp:63-89）；
  - length-stop 表（注释指向 klsupport.cpp:46-62）；
  - 本原索引表：把块元素映射到其在给定下降集的本原元素列表中的位置（注释指向 klsupport.cpp:109-144）。
- 文档中的核心定义：`x` 对 `y` 的下降集**本原**（primitive），当且仅当 `x` 的 good ascent 中没有一个是 `y` 的下降（klsupport.h `is_primitive`）。
- 文档中的存储约定：KLV 多项式 `P_{x,y}` 存放在第 `y` 列的 `prim_index(x, desc(y))` 处（注释指向 kl.cpp:124-148）。
- 文件内定义的项：`RankFlags`、`KlSupport<B>`、私有 `PrimIndexRecord`、私有函数 `validate_topology` / `validate_target`，以及 `#[cfg(test)]` 测试模块。本文件**没有**定义任何 trait 或 enum，也**没有** `pub(crate)` 项。

## 2. 裸签名清单

### 2.1 导入（外部依赖，定义均在本文件之外）

```rust
use std::collections::BTreeMap;
use crate::block::BlockDescent;
use crate::{BlockTopology, StructureError};
```

### 2.2 pub 项（含无注释项，按文件出现顺序）

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankFlags {
    bits: u32, // 字段私有
}

impl RankFlags {
    pub fn empty() -> Self;
    pub fn set(&mut self, generator: usize);
    pub fn contains(&self, other: &Self) -> bool;
    pub fn intersect(&self, other: &Self) -> Self;   // self & other
    pub fn difference(&self, other: &Self) -> Self;  // self - other
    pub fn none(&self) -> bool;
    pub fn first_bit(&self) -> Option<usize>;
    pub fn is_set(&self, generator: usize) -> bool;
}

pub struct KlSupport<B: BlockTopology> {
    // 全部字段私有：
    // block: B
    // descents: Vec<RankFlags>
    // good_ascents: Vec<RankFlags>
    // length_stop: Vec<usize>
    // prim_index: BTreeMap<u32, PrimIndexRecord>
}

impl<B: BlockTopology> KlSupport<B> {
    pub fn new(block: B) -> Result<Self, StructureError>;
    pub fn block(&self) -> &B;
    pub fn size(&self) -> usize;
    pub fn rank(&self) -> usize;
    pub fn length(&self, z: usize) -> usize;
    pub fn length_less(&self, l: usize) -> usize;
    pub fn length_floor(&self, y: usize) -> usize;
    pub fn descent_set(&self, z: usize) -> &RankFlags;
    pub fn good_ascent_set(&self, z: usize) -> &RankFlags;
    pub fn ascent_descent(&self, x: usize, y: usize) -> Option<usize>;
    pub fn is_extremal(&self, x: usize, desc_y: &RankFlags) -> bool;
    pub fn is_primitive(&self, x: usize, desc_y: &RankFlags) -> bool;
    pub fn prim_back_up(&self, x: &mut usize, desc_y: &RankFlags) -> bool;
    pub fn col_size(&self, y: usize) -> usize;
    pub fn unique_ascent(&self, s: usize, z: usize) -> Option<usize>;
    pub fn prim_index(&self, x: usize, desc_y: &RankFlags) -> usize;
    pub fn nr_of_primitives(&self, desc_y: &RankFlags) -> usize;
    pub fn self_index(&self, y: usize) -> usize;
    pub fn prepare_prim_index(&mut self, desc_y: &RankFlags);
}
```

### 2.3 本文件内可见的私有项（非 pub，列出以便核对）

```rust
#[derive(Clone, Debug)]
struct PrimIndexRecord {
    index: Vec<usize>,
    range: usize,
}

fn validate_topology(block: &impl BlockTopology) -> Result<(), StructureError>;
fn validate_target(target: Option<usize>, size: usize) -> Result<(), StructureError>;
```

### 2.4 测试模块（`#[cfg(test)] mod tests`）

```rust
struct FakeTopology { rank: usize, lengths: Vec<usize>, cross_target: usize }
impl crate::block_access::sealed::Sealed for FakeTopology;
impl crate::BlockTopology for FakeTopology; // size/rank/length/descent/cross/cayley/inverse_cayley

#[test] fn rank_flags_set_and_query();
#[test] fn rejects_topology_rank_above_rank_flags_capacity();
#[test] fn rejects_topology_not_sorted_by_length();
#[test] fn rejects_topology_link_target_outside_the_block();
```

## 3. `RankFlags` 的位集语义

- 底层表示：私有字段 `bits: u32`，第 `i` 位对应第 `i` 个简单生成元。文档注释明确：与上游 KL 算法使用的 `RankFlags` 语义一致，**rank ≤ 32**。
- 各方法行为（均已实现）：
  - `empty()`：`bits = 0`。
  - `set(generator)`：`bits |= 1 << generator`。**无边界检查**；本文件未对 `generator >= 32` 做防护（阅读推断：在 `KlSupport` 使用路径上，构造门控保证 `rank ≤ 32` 且循环上界为 `rank`，故 `generator ≤ 31`；单独使用 `RankFlags` 时越界移位行为由 Rust 移位语义决定，本文件未加防护）。
  - `contains(other)`：`self.bits & other.bits == other.bits`，即 `self ⊇ other`（超集判定；测试锚点确认「空集被任何集合包含」）。
  - `intersect(other)`：按位与，返回新值。
  - `difference(other)`：`self.bits & !other.bits`，返回新值。
  - `none()`：`bits == 0`。
  - `first_bit()`：在固定范围 `0..32` 内升序扫描，返回最低置位生成元的 `Some(index)`，无置位返回 `None`（注释指向上游 `RankFlags::firstBit`）。注意扫描范围固定为 32，与实际 rank 无关。
  - `is_set(generator)`：位测试，同样无边界检查。
- 派生：`Clone, Debug, Eq, PartialEq`（**非 `Copy`**）。`bits` 无 pub 构造器/访问器；同模块内 `KlSupport` 直接读写 `desc_y.bits` 作为 `BTreeMap` 键。

## 4. `KlSupport<B>`：泛型参数与构造门控

### 4.1 泛型与字段

- 泛型参数 `B: BlockTopology`，按值持有拓扑（`block: B`），`block()` 返回 `&B`。`KlSupport` 本身无任何 derive。
- 字段（均私有）：`descents`、`good_ascents`（逐元素 `RankFlags`）、`length_stop: Vec<usize>`、`prim_index: BTreeMap<u32, PrimIndexRecord>`（以下降集的原始 `u32` 位掩码为键，懒填充）。

### 4.2 `new` 的构建流程（已实现）

1. 先调用 `validate_topology(&block)?`——所有拓扑不变量在可失败的构造边界集中检查（函数注释明示：避免在后续列填充深处发生索引或 `expect` panic）。
2. 逐元素 `z in 0..size`、逐生成元 `s in 0..rank` 重新取 `block.descent(z, s)`；若为 `None`，返回
   `StructureError::IndexOutOfRange { index: z * rank + s, upper_bound: size * rank }`。
   （阅读推断：经 `validate_topology` 后该分支不可达，属纵深防御。）
3. 分类（已实现）：
   - `value.is_descent()` → 记入 `descents[z]`；
   - 否则若 `value != BlockDescent::ImaginaryTypeII` → 记入 `good_ascents[z]`；
   - 即 **`ImaginaryTypeII` 既不是下降也不是 good ascent**。
4. length-stop 表：逐元素取 `block.length(z)`，为 `None` 时返回
   `StructureError::IndexOutOfRange { index: z, upper_bound: size }`（阅读推断：同样不可达）。代码注释语义：`length_stop[l] =` 首个长度 `>= l` 的元素，不存在时为 `size`。构造方式为：长度非降（已由门控保证）地扫描，`while length_stop.len() <= length { push(z) }`，循环结束后再 `push(size)`。
5. `max_length` 以 `unwrap_or(0)` 聚合计算后被 `let _ = max_length;` 显式丢弃——**已计算但未使用**。
6. `prim_index` 初始为空 `BTreeMap`；本原索引表由 `prepare_prim_index` 按需懒构建。

### 4.3 `validate_topology` 门控（私有；错误清单，均为已实现）

- `RANK_FLAGS_CAPACITY = u32::BITS as usize`（即 32）。`block.rank() > 32` →
  `Err(StructureError::BlockInvariantViolation { invariant: "KL topology rank exceeds RankFlags capacity" })`。
- 逐元素：
  - `length(element)` 为 `None` → `"...element has no length"`；
  - 长度非非降（`length < previous`）→ `"...lengths are not nondecreasing"`（**顺序保证在构造期强制**）；
  - 逐生成元：
    - `descent(element, generator)` 为 `None` → `"...element has no descent status"`；
    - `cayley(element, generator)` 为 `None` → `"...element has no Cayley cell"`；
    - `inverse_cayley(element, generator)` 为 `None` → `"...element has no inverse-Cayley cell"`；
    - `cross`、`cayley.0`、`cayley.1`、`inverse_cayley.0`、`inverse_cayley.1` 经 `validate_target` 检查：`Some(t)` 且 `t >= size` → `"...link target is outside the block"`；`None` 合法。
- 全部通过返回 `Ok(())`。

## 5. 下降 / 上升 / 本原 / 极值的判定语义

- `descent_set(z)` / `good_ascent_set(z)`（已实现）：直接 `self.descents[z]` / `self.good_ascents[z]` 索引；`z >= size` 时按 Rust 索引语义 panic，无显式边界检查。
- `ascent_descent(x, y)`（已实现）：计算 `descent_set(y).difference(descent_set(x)).first_bit()`，即「是 `y` 的下降、但不是 `x` 的下降」的最低生成元，无则 `None`。文档注释称其为「`x` 的、同时是 `y` 的下降的第一个上升」（klsupport.h `ascent_descent`）。（阅读推断：按集合差的纯定义，返回的生成元对 `x` 而言是「非下降」，这既包括 good ascent 也包括 `ImaginaryTypeII`，以及 `BlockDescent` 中本文件未出现的其他非下降变体；`BlockDescent` 的完整变体集与 `is_descent()` 的定义在本文件之外。）
- `is_extremal(x, desc_y)`（已实现）：`descent_set(x).contains(desc_y)`——`desc_y` 中每个生成元都是 `x` 的下降（klsupport.h `is_extremal`）。
- `is_primitive(x, desc_y)`（已实现）：`good_ascent_set(x).intersect(desc_y).none()`——`x` 的 good ascent 与 `desc_y` 无交（klsupport.h `is_primitive`）。
- `unique_ascent(s, z)`（已实现；注释指向 blocks.h `unique_ascent`）：
  - `block.descent(z, s)` 为 `None` → 经 `?` 返回 `None`；
  - `BlockDescent::ComplexAscent` → 返回 `block.cross(z, s)`（可为 `None`）；
  - `BlockDescent::ImaginaryTypeI` → 返回 `block.cayley(z, s)?.0`（第一个 Cayley 像；外层 `None` 短路，内层 `.0` 为 `None` 时返回 `None`）；
  - 其他一切取值（含 `ImaginaryTypeII`、`RealNonparity` 及各类下降）→ `None`。

## 6. 长度表与列尺寸

- `length(z)`（已实现）：`block.length(z).unwrap_or(0)`——缺失长度静默按 0 处理（阅读推断：构造门控后正常路径不会缺失）。
- `length_less(l)`（已实现；klsupport.h `length_less`）：`length_stop.get(l).copied().unwrap_or(self.size())`——`l` 超出表长时回退为 `size()`。文档语义：长度 `< l` 的元素个数。
- `length_floor(y)`（已实现；klsupport.h `length_floor`）：`length_less(self.length(y))`。
- `col_size(y)`（已实现；klsupport.h `col_size`）：
  1. `x = self.length_floor(y)`，`desc_y = self.descent_set(y)`；
  2. `prim_back_up(&mut x, desc_y)` 成功 → 返回 `self.prim_index(x, desc_y) + 1`；失败 → 返回 `0`。
  （阅读推断：该调用链经由 `prim_index` 索引 `self.prim_index[&desc_y.bits]`，因此同样要求事先对 `descent_set(y)` 调用过 `prepare_prim_index`，否则 panic；见第 7、10 节。）
- `prim_back_up(x: &mut usize, desc_y)`（已实现；klsupport.h `prim_back_up`）：`while *x > 0 { *x -= 1; if is_primitive(*x, desc_y) { return true } } false`。
  逐字节观察（不作评判）：
  - 循环**先自减再判定**，初始的 `*x` 本身不会被检查；文档注释写「Walk `x` (inclusive) down」；
  - 返回 `false` 时 `*x` 已被改为 `0`（原地修改是契约的一部分）。

## 7. 本原索引表

- `PrimIndexRecord`（私有；注释对应 klsupport.h `prim_index_tp`）：`index[z]` 为 `z` 的「本原化」在该下降集本原元素中的位置，`z` 无本原元素时为 `range`；`range` 为本原元素总数。
- `prepare_prim_index(desc_y)`（已实现；klsupport.cpp:109-144），`&mut self` 懒填充：
  1. 幂等：键 `desc_y.bits` 已存在则直接返回。
  2. 逆序（`x = size-1 … 0`）扫描，`DEAD_END = usize::MAX` 作哨兵：
     - `good = good_ascent_set(x) ∩ desc_y`；`good.none()` → `x` 本原：`index[x] = count; count += 1`（`count` 即「已见的、序号更大的本原元素」个数）；
     - 否则 `s = good.first_bit().expect("nonempty good ascent")`（前面已判非空），`value = block.descent(x, s).expect("valid generator")`（阅读推断：构造门控已保证 `Some`，正常路径不触发）；
     - `value == BlockDescent::RealNonparity` → `index[x] = DEAD_END`；
     - 否则按 `unique_ascent(s, x)`：`Some(ascended)` → `index[x] = index[ascended]`；`None` → `DEAD_END`。
     （阅读推断：由于是降序遍历，只有当 `ascended > x` 时 `index[ascended]` 才已被写入；若 `ascended <= x` 将读到初始值 `0`。代码隐含「上升像的元素序号大于 `x`」的假定，本文件内无显式检查。）
  3. `range = count`；重写各槽：`DEAD_END` → `range`；否则 `slot = range - 1 - slot`（注释：reverse indices）。（阅读推断：`range == 0` 时所有槽均为 `DEAD_END`，故 `range - 1 - slot` 不会下溢。）
  4. 以 `desc_y.bits` 为键插入 `PrimIndexRecord { index, range }`。
- `prim_index(x, desc_y)`（已实现；klsupport.h `prim_index`）：`self.prim_index[&desc_y.bits]` 取记录后 `record.index[x]`。文档明示前置条件：**必须先对同一下降集调用 `prepare_prim_index`**；`BTreeMap` 的 `Index` 在缺键时 panic，`x >= size` 时 Vec 索引 panic。
- `nr_of_primitives(desc_y)`（已实现；klsupport.h `nr_of_primitives`）：返回 `.range`；同样缺键即 panic。
- `self_index(y)`（已实现；klsupport.h `self_index`）：`prim_index(y, self.descent_set(y))`。

## 8. 与块拓扑（`BlockTopology`）的接口

- 消费的方法（签名形状取自本文件的调用点与测试中 `FakeTopology` 的实现）：
  - `size() -> usize`、`rank() -> usize`；
  - `length(element) -> Option<usize>`；
  - `descent(element, generator) -> Option<BlockDescent>`；
  - `cross(element, generator) -> Option<usize>`；
  - `cayley(element, generator) -> Option<(Option<usize>, Option<usize>)>`；
  - `inverse_cayley(element, generator) -> Option<(Option<usize>, Option<usize>)>`。
- 引用的 `BlockDescent` 变体：`ComplexAscent`、`ImaginaryTypeI`、`ImaginaryTypeII`、`RealNonparity`；并调用其方法 `is_descent()`（定义在 `crate::block`，完整变体集不在本文件内）。
- 构造的 `StructureError` 变体：`IndexOutOfRange { index, upper_bound }` 与 `BlockInvariantViolation { invariant }`（不变量为 `&'static str` 字面量，逐条见第 4.3 节）。
- Sealed 模式（阅读推断）：测试中为 `FakeTopology` 实现了 `crate::block_access::sealed::Sealed` 才能实现 `crate::BlockTopology`，提示 `BlockTopology` 是 sealed trait，外部 crate 不可实现。
- 上游映射（均来自注释，未核实）：klsupport.h（`is_primitive` / `is_extremal` / `ascent_descent` / `prim_back_up` / `col_size` / `prim_index` / `nr_of_primitives` / `self_index` / `length_less` / `length_floor` / `prim_index_tp`）、klsupport.cpp:46-62、63-89、109-144、blocks.h `unique_ascent`、kl.cpp:124-148。

## 9. 测试锚点

测试替身 `FakeTopology { rank, lengths, cross_target }`：`size = lengths.len()`；`length` 查 Vec；`descent` 在界内恒为 `Some(BlockDescent::ComplexAscent)`；`cross` 在界内恒为 `Some(cross_target)`；`cayley` / `inverse_cayley` 在界内恒为 `Some((None, None))`。

1. `rank_flags_set_and_query`：`empty()` 时 `none()` 为真；`set(2)` 后 `is_set(2)` 为真、`first_bit() == Some(2)`；对空集 `other`，`flags.contains(&other)` 为真（注释：the empty set is contained）。
2. `rejects_topology_rank_above_rank_flags_capacity`：`rank: 33, lengths: vec![0]` → `Err(BlockInvariantViolation { invariant: "KL topology rank exceeds RankFlags capacity" })`。
3. `rejects_topology_not_sorted_by_length`：`rank: 1, lengths: vec![1, 0]` → `"KL topology lengths are not nondecreasing"`。
4. `rejects_topology_link_target_outside_the_block`：`rank: 1, lengths: vec![0], cross_target: 1`（size 为 1，目标越界）→ `"KL topology link target is outside the block"`。

## 10. panic / 错误 / 边界条件汇总

- `Result` 错误（已实现）：
  - `StructureError::IndexOutOfRange { index: z*rank + s, upper_bound: size*rank }`（`new` 中 `descent` 为 `None`）；
  - `StructureError::IndexOutOfRange { index: z, upper_bound: size }`（`new` 中 `length` 为 `None`）；
  - `StructureError::BlockInvariantViolation { invariant }`，五条不变量文案逐条见第 4.3 节。
- panic 路径（按 Rust 语义或 `expect` 字面）：
  - `descent_set` / `good_ascent_set`：`z >= size` 时 Vec 索引 panic；
  - `prim_index`：下降集未 `prepare` 时 map 索引 panic；`x >= size` 时 Vec 索引 panic；
  - `nr_of_primitives`：缺键 panic；
  - `col_size` / `self_index`：经由 `prim_index` 继承相同前置条件（阅读推断）；
  - `prepare_prim_index`：`expect("nonempty good ascent")`、`expect("valid generator")`（前者由 `none()` 判定保护；后者阅读推断为不可达）；
  - `RankFlags::set` / `is_set`：`generator >= 32` 无防护。
- 边界与顺序保证：元素枚举区间 `0..size`；长度非降由构造门控强制，`length_stop` 与逆序本原扫描依赖该顺序（后者为阅读推断）；`length_less(l)` 对超表 `l` 回退 `size()`；`prim_back_up` 失败时把 `*x` 置 `0`。

## 11. 限制与未覆盖面

- rank 硬上限 32（`u32` 位集；`rank > 32` 在构造期拒绝）。
- `ImaginaryTypeII` 被排除在下降集与 good-ascent 集之外；`ascent_descent` 的集合差定义可能返回对 `x` 非下降的任意生成元（阅读推断，见第 5 节）。
- `max_length` 计算后未使用（`let _ = max_length;`）。
- `prim_index` 表按键懒填充、无淘汰；`prim_index` / `nr_of_primitives` / `col_size` / `self_index` 的正确使用依赖调用方先行 `prepare_prim_index`（仅 `prim_index` 的文档注释显式写出此前置条件）。
- `prepare_prim_index` 隐含假定上升像序号大于当前元素（阅读推断）；`RealNonparity` 与「无唯一上升像」统一走 `DEAD_END` 哨兵（`usize::MAX`）。
- 本文件测试仅覆盖：`RankFlags` 基本位操作与三条构造拒绝路径（rank 超容量、长度非非降、链接目标越界）。**未覆盖**：`new` 的成功路径；`length_less` / `length_floor` / `col_size`；`ascent_descent` / `is_extremal` / `is_primitive` / `prim_back_up`；`unique_ascent`；`prepare_prim_index` / `prim_index` / `nr_of_primitives` / `self_index`；以及「缺 length / 缺 descent / 缺 Cayley / 缺 inverse-Cayley」四条门控分支。
- 本文件不含任何数学正确性或性能验证；上游行号、`P_{x,y}` 存储约定等仅为注释引用，收录前需维护者对照 gkmod 源逐条核对。