---
title: block.rs —— 实形式与对偶实形式的块（Block）图：纤维积构建、Cayley/交叉链接与对偶变换
source: atlas-rust/block
ingestedAt: 2026-10-06T00:00:00Z
---

# 知识来源包草案：`crates/atlas-real-group/src/block.rs`

> 核对约定：【已实现】= 源字节中可直接读到的事实；【注释转述】= 代码注释中对上游 C++（Atlas of Lie Groups）出处的陈述，未经本草案核对上游原文；【阅读推断】= 阅读代码得出但未被注释或测试显式固定的推断，需维护者确认。

## 1. 裸签名清单（含无注释项）

### 1.1 模块级

- 模块文档注释：块是两个单边参数集（实形式 KGB 图与对偶实形式 KGB 图）在扭曲对合 `w` / `dual_w` 上的纤维积；配对映射为 [`dual_involution`]。【注释转述：上游 `blocks::Block`、`Block::Block(kgb, dual_kgb)` gkmod/blocks.cpp:526-606、`blocks::dual_involution` blocks.cpp:1701-1711、`Block_value` interpreter/atlas-types.w:4753-4758、`KGB_base::tauPacket` gkmod/kgb.cpp:131-140】
- `use std::collections::HashMap;`
- `use crate::grading::try_capacity;`
- `use crate::{InnerClass, InvolutionTable, KgbGraph, KgbId, KgbStatus, RootSystem, StructureError, WeylElement};`

### 1.2 `pub enum BlockDescent`

`#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]`，8 个变体，按声明顺序：

1. `ComplexAscent`
2. `RealNonparity`
3. `ImaginaryTypeI`
4. `ImaginaryTypeII`
5. `ImaginaryCompact`
6. `ComplexDescent`
7. `RealTypeII`
8. `RealTypeI`

【注释转述：顺序即上游 `descents::DescentStatus::Value` 顺序（gkmod/descents.h:40），前四个为上升（ascent），后四个（bit `0x4` 置位）为弱下降（weak descent）。】

`impl BlockDescent`：

- `const ALL: [BlockDescent; 8]`（私有；按声明顺序列出全部变体）
- `fn index(self) -> usize`（私有；`ALL.iter().position(...)`，附带 `.expect("ALL covers every variant")`）
- `pub fn is_descent(self) -> bool` —— `self.index() & 0x4 != 0`
- `pub fn dual(self) -> BlockDescent` —— 查表 `DUAL: [BlockDescent; 8]`
- `pub fn language_code(self) -> u32` —— 查表 `TAB: [u32; 8] = [4, 5, 6, 7, 1, 0, 3, 2]`

### 1.3 自由函数 `dual_involution`

```rust
pub fn dual_involution(
    word: &[usize],
    dual_system: &RootSystem,
    dual_twist: &[usize],
    dual_longest: &WeylElement,
) -> Result<WeylElement, StructureError>
```

【注释转述：上游 `blocks::dual_involution`（blocks.cpp:1701-1711）；`word` 携带**外部**生成元编号，`dual_twist` 是对偶内类的 distinguished 生成元置换（`TwistedWeylGroup::twisted`）。】

### 1.4 `pub struct BlockGraph`

`#[derive(Clone, Debug, Eq, PartialEq)]`，全部字段私有：

| 字段 | 类型 | 注释中的布局说明 |
|---|---|---|
| `rank` | `usize` | —— |
| `xs` | `Vec<KgbId>` | 每个块元素的原 KGB 坐标（full-KGB 编号） |
| `ys` | `Vec<KgbId>` | 每个块元素的对偶 KGB 坐标 |
| `descent` | `Vec<BlockDescent>` | 扁平 `z * rank + s` |
| `cross` | `Vec<usize>` | 扁平 `s * size + z`【注释转述：上游 `data[s][z]`】 |
| `cayley_first` | `Vec<Option<usize>>` | 【注释转述：上游 `Cayley_image.first`】直接 Cayley 与逆 Cayley 访问器共享 |
| `cayley_second` | `Vec<Option<usize>>` | 【注释转述：上游 `Cayley_image.second`】同上 |
| `lengths` | `Vec<usize>` | 原 KGB 长度【注释转述：blocks.cpp:557 `kgb.length(x)`，供 KL 表长度排序】 |
| `first_z_of_x` | `Vec<usize>` | `first_z_of_x[x]` = 首个满足 `x(z) >= x` 的块元素【注释转述：blocks.cpp:630-643】；长度 `xrange + 1`，含 size 哨兵 |

`impl BlockGraph`（全部 `pub`）：

- `pub fn build(graph: &KgbGraph, table: &InvolutionTable, dual_graph: &KgbGraph, dual_table: &InvolutionTable, dual_inner_class: &InnerClass, weyl_budget: usize) -> Result<Self, StructureError>`
- `pub fn dual(&self) -> BlockGraph`
- `pub fn size(&self) -> usize`
- `pub fn rank(&self) -> usize`
- `pub fn x(&self, z: usize) -> Option<KgbId>`
- `pub fn y(&self, z: usize) -> Option<KgbId>`
- `pub fn length(&self, z: usize) -> Option<usize>`
- `pub fn descent_value(&self, z: usize, generator: usize) -> Option<BlockDescent>`
- `pub fn cross(&self, z: usize, generator: usize) -> Option<usize>`
- `pub fn cayley(&self, z: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)>`
- `pub fn inverse_cayley(&self, z: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)>`
- `pub fn element(&self, x: KgbId, y: KgbId) -> Result<usize, StructureError>`
- `pub fn status_code(&self, z: usize, generator: usize) -> Option<u32>`
- `pub fn bruhat_hasse(&self) -> Vec<Vec<usize>>`
- `pub fn n_bruhat_comparable(hasse: &[Vec<usize>]) -> usize`（关联函数，无 `self`）

### 1.5 私有自由函数

- `fn descents(x: KgbId, y: KgbId, generator: usize, graph: &KgbGraph, dual_graph: &KgbGraph) -> Result<BlockDescent, StructureError>`
- `fn first_zs(xs: &[KgbId], xrange: usize) -> Vec<usize>`
- `fn element_at(xs: &[KgbId], ys: &[KgbId], first_z_of_x: &[usize], x: KgbId, y: KgbId) -> Result<usize, StructureError>`
- `fn first_free_slot(first: &mut Option<usize>, second: &mut Option<usize>, z: usize) -> Result<(), StructureError>`

### 1.6 测试模块

`#[cfg(test)] mod tests`：辅助项 `class_budget`、`struct Pipeline`、`pipeline`、`pipeline_with_class`、`graph_with_size`、`sc_a1_datum`、`a1_blocks`；测试 7 个（见 §7）。

---

## 2. 数据结构与不变量

### 2.1 元素编号（顺序保证）

【已实现，与注释一致】`build` 中块元素的生成顺序为：按原 KGB 包的枚举顺序（`position in 0..graph.packet_count()`，注释称此为原 KGB 的排序对合序）逐包处理；包内 `x_offset in 0..x_count` 为外层循环、`y_offset in 0..y_count` 为内层循环，即 **x 外层、y 内层**的笛卡尔积，x、y 各自升序。【注释转述：blocks.cpp:548-558。】

派生不变量：

- 【已实现】`xs` 单调弱增——`first_zs` 的算法依赖此性质（注释明示 “x values weakly increase”）。
- 【已实现】同一 `x` 的连续元素 `y` 连续升序——`element_at` 用 `first + (y - base_y)` 的直接偏移定位（见 §3.2）。
- 【已实现】`xs.len() == ys.len() == descent.len()/rank == lengths.len() == cross.len()/rank`（`size()` 返回 `xs.len()`；`build` 末尾显式检查 `xs.len() == size`，否则 `StructureError::BlockInvariantViolation { invariant: "block size" }`）。

### 2.2 对合包与空包语义

- 【已实现】对偶侧包以 `HashMap<WeylElement, usize>`（`dual_position`）按键 `record.weyl_element().clone()` 索引；若某个原侧对合的对偶 `dual` 不在此 map 中，则 `y_count = 0`，该包贡献零对（`continue` 跳过积的生成），但仍向 `dual_w` 推入该对偶元素。
- 【注释转述】这对应上游 `KGB_base::tauPacket` 返回 `(0,0)` 的行为；对共同 Cartan 的限制因此是隐式的。模块文档还声明：解释器从两个形式的**完整** KGB 集建块，提取的 KGB 坐标必须保持形式自身的 KGB 编号（上游 `common_Cartans` 受限重载会改变编号）。

### 2.3 Cayley 槽位共享

【已实现，doc 注释明示】`cayley_first` / `cayley_second` 同时承载直接 Cayley（上升处）与逆 Cayley（弱下降处）的像：构建时，对 `ImaginaryTypeI`/`ImaginaryTypeII` 元素 `z` 写入其直接像，同时用 `first_free_slot` 把 `z` 自身填入像元素 `z0`（及 `z1`）的第一个空槽，从而使下降元素的同一对槽位给出逆 Cayley 原像（可双值）。

### 2.4 `first_z_of_x`

【已实现】长度 `xrange + 1`；`build` 中 `xrange = graph.size()`（原 KGB 全集大小）；`dual` 中 `xrange = xs.iter().map(|x| x.index() + 1).max().unwrap_or(0)`【注释转述：上游按 `max_y + 1` 定对偶的 x 范围】。末段用 size 哨兵填满。

---

## 3. 构建与查找流程

### 3.1 `BlockGraph::build` 流程（按代码顺序）

1. 【已实现】`rank = graph.semisimple_rank()`；若 `dual_graph.semisimple_rank() != rank` 或 `dual_table.root_system() != dual_inner_class.root_system()`，返回 `Err(StructureError::DatumMismatch)`。
2. 【已实现】取对偶扭曲 Weyl 数据：`dual_twist = dual_inner_class.generator_twist()?`；`longest = crate::dual::longest_action(dual_inner_class, weyl_budget)?`；`dual_longest = WeylElement::from_action(dual_system, &longest)?`。【注释转述：`weyl_budget` 界定定位最长元素的枚举。】
3. 【已实现】建立 `dual_position`（见 §2.2）；缺失条目分别报 `"dual packet involution"` / `"dual packet record"`。
4. 【已实现】对每个原侧包：取 `packet_involution` → `table.record(id)` → `record.weyl_element().reduced_word(table.root_system())?`（注意用的是**原侧** table 的根系）→ `dual_involution(&word, dual_system, &dual_twist, &dual_longest)?` 得 `dual`；取 `x_count`；查 `dual_position` 得 `y_count`（缺失为 0）；以 `checked_mul` / `checked_add` 累加 `size`，溢出报 `StructureError::ArithmeticOverflow`；`dual_w.push(dual)`。
5. 【已实现】按 §2.1 的顺序填 `xs`/`ys`/`lengths`/`descent`（`lengths` 取自 `graph.length(x)`，缺失报 `"block element length"`；每个生成元的下降状态由 `descents(x, y, generator, graph, dual_graph)?` 计算）。容量经 `try_capacity(size)` / `try_capacity(size * rank.max(1))` 预留。
6. 【已实现】`first_z_of_x = first_zs(&xs, graph.size())`。
7. 【已实现】交叉/Cayley 表：尺寸 `size * rank.max(1)`，`cross` 初始化为 0 后在 `generator in 0..rank`、`z in 0..size` 全循环中逐槽覆写（故所有槽均被写入）；`cayley_first/second` 初始化为 `None`，仅在 i1/i2 分支写入。槽位布局 `generator * size + z`。
   - 每槽先算 `cross_x = graph.cross(x, generator)`、`cross_y = dual_graph.cross(y, generator)`，再 `cross[slot] = element_at(...)` 回查块内编号。
   - `ImaginaryTypeII`：`graph.cayley(x, generator)?` 取 `cayley_x`；`dual_graph.inverse_cayley(y, generator)?` 取 `(dual_first, dual_second)`，其中 `dual_second` 必须为 `Some`，否则报 `"type II inverse Cayley pair"`。双值直接 Cayley【注释转述：blocks.cpp:575-582】：`z1 = element_at(cayley_x, dual_second)`，`cayley_second[slot(z)] = Some(z1)`，`cayley_first[slot(z1)] = Some(z)`；随后**落入** type I 分支【注释转述：blocks.cpp 583-590 的 fall-through】：`z0 = element_at(cayley_x, dual_first)`，`cayley_first[slot(z)] = Some(z0)`，并对 `z0` 槽位 `first_free_slot(..., z)`。
   - `ImaginaryTypeI`：单值；`z0 = element_at(cayley_x, dual_first)`，`cayley_first[slot(z)] = Some(z0)`，`first_free_slot` 回填 `z0`。
   - 其它下降状态：不写 Cayley 槽。

### 3.2 查找流程（`element` / `element_at` / `first_zs`）

- 【已实现】`first_zs`：线性扫描 `xs`；对每个 `z`，`while xx < x.index() { xx += 1; first_z_of_x[xx] = z; }`；扫描结束后 `while xx < xrange { xx += 1; first_z_of_x[xx] = size; }`。【阅读推断：若 `xs` 非弱增，算法不会 panic，但会产生不满足“首个 `x(z) >= x`”语义的表，后续由 `element_at` 的验证步骤兜住。】
- 【已实现】`element_at`：
  1. `first = first_z_of_x.get(x.index())`，缺界报 `StructureError::IndexOutOfRange { index: x.index(), upper_bound: first_z_of_x.len().saturating_sub(1) }`；
  2. `base_y = ys.get(first)`，缺失报 `BlockInvariantViolation { invariant: "element lookup" }`；
  3. `z = first + (y.index() - base_y.index())`：`checked_sub` 失败（`y < base_y`）报 `BlockInvariantViolation { invariant: "element fiber" }`；`checked_add` 溢出报 `ArithmeticOverflow`；
  4. 验证 `xs[z] == x && ys[z] == y`，否则报 `BlockInvariantViolation { invariant: "element fiber" }`。【注释转述：该验证承担上游 assert 的角色（`Block::element`，blocks.cpp:242-248）。】
- 【已实现】`BlockGraph::element(x, y)` 只是对 `element_at` 的转发。

### 3.3 `dual_involution` 流程

【已实现】`result = dual_longest.clone()`；对 `word.iter().rev()` 中每个 `generator`：`dual_twist.get(generator)` 缺界报 `IndexOutOfRange { index: generator, upper_bound: dual_twist.len() }`；`(next, _) = result.right_multiply_simple(dual_system, twisted)?`（二元组第二分量被丢弃——【阅读推断】其语义在本文件不可见）；`result = next`。【注释转述：即“从对偶最长元素出发，按 `w` 的约化字自右向左、以对偶扭曲字母右乘”，在对合矩阵上是负转置，在 Weyl 元素上由 `f(e) = w0`、`f(s.w) = f(w).dwist(s)` 刻画。】

---

## 4. Cayley / 交叉链接的访问语义

- 【已实现】`cross(z, generator)`：`generator >= rank` 返回 `None`；否则返回表值——**对每个生成元都有定义**（doc 注释明示）。
- 【已实现】`cayley(z, generator)`：`generator >= rank` → `None`；若 `descent_value(z, generator)` 是弱下降 → `Some((None, None))`（doc 注释：弱下降处强制未定义，内层 `None` 即上游 `UndefBlock`）；否则返回 `(cayley_first[slot], cayley_second[slot])` 原值。
- 【已实现】`inverse_cayley(z, generator)`：与 `cayley` 互补——非弱下降 → `Some((None, None))`；弱下降 → 原槽位值。
- 【已实现】`descent_value`、`status_code`：`generator >= rank` 或 `z` 越界 → `None`（经 `.get`）。`status_code = descent_value(...).language_code()`。
- 【已实现】`language_code` 表 `TAB = [4,5,6,7,1,0,3,2]`，doc 注释给出 Atlas 语言码含义：0=C-, 1=ic, 2=r1, 3=r2, 4=C+, 5=rn, 6=i1, 7=i2。【注释转述：解释器 `block_status_wrapper` 的重编号（atlas-types.w:4911-4913）。】
- 【已实现】`BlockDescent::dual` 静态表：`ComplexAscent↔ComplexDescent`、`RealNonparity↔ImaginaryCompact`、`ImaginaryTypeI↔RealTypeII`、`ImaginaryTypeII↔RealTypeI`；测试固定其为对合（`dual(dual(v)) == v`）。
- 【已实现】`is_descent`：索引位 `0x4`——即 `{ImaginaryCompact, ComplexDescent, RealTypeII, RealTypeI}` 为弱下降。

### 4.1 `descents()` 判定逻辑（每生成元）

【已实现】按原侧 `graph.status(x, generator)` 分情形：

- `KgbStatus::Complex`：`graph.is_descent(x, generator)` 为真 → `ComplexDescent`，否则 `ComplexAscent`。
- `KgbStatus::ImaginaryNoncompact`：`graph.cross(x, generator) != x` → `ImaginaryTypeI`，否则 `ImaginaryTypeII`。
- `KgbStatus::Real` 或 `KgbStatus::ImaginaryCompact`：查对偶侧 `dual_graph.status(y, generator)`——
  - 对偶为 `ImaginaryNoncompact`：`dual_graph.cross(y, generator) != y` → `RealTypeII`，否则 `RealTypeI`；
  - 否则原侧为 `Real` → `RealNonparity`；原侧为 `ImaginaryCompact` → `ImaginaryCompact`。
- 各步缺数据分别报 `"descent status"` / `"descent cross link"` / `"dual descent status"` / `"dual descent cross link"`。

### 4.2 `dual()` 变换

【已实现】纯数据变换，不重新建块：

- 元素序反转：`z' = size - 1 - z`；`xs`/`ys` 互换；`lengths` 反射为 `max_len - length(z)`，其中 `max_len = lengths.last().copied().unwrap_or(0)`（空块取 0）；
- 每个下降状态经 `BlockDescent::dual` 映射；
- 链接 `c → size-1-c`：`cross[target] = size - 1 - cross[source]`；Cayley 槽**仅当 first 有定义时**整体映射，second 仅随 first 映射（`first` 为 `None` 时两个槽都留 `None`）；
- `first_z_of_x` 用 `first_zs(&xs, xrange)` 重算（`xrange` 见 §2.4）。
- 【注释转述】上游 `Bare_block::dual`（blocks.cpp:474-509）的 `orbits`/`dd` 字段（上游自注 “probably not right”）无 Rust 对应物、未复现；上游双值 Cayley 对在对偶中也不重排（测试注释确认）。

### 4.3 Bruhat 相关

- 【已实现】`bruhat_hasse(&self)` 仅转发 `crate::block_access::bruhat_hasse(self)`，算法本体不在本文件。【注释转述：blocks.cpp:1603-1656 `complete_Hasse_diagram`；递归沿首个严格好下降（复或 I 型实），分裂主系列处前驱恰为 II 型实下降的逆 Cayley 像。】
- 【已实现】`n_bruhat_comparable(hasse)`：关联函数；逐行维护 `BTreeSet` 闭包，`cl.insert(closure.len())`（含自身），并 `cl.extend(closure[j])` 合并各下行邻点的闭包，累计 `cl.len()`。【注释转述：poset.cpp:197-229 `n_comparable_from_Hasse`，含元素与自身可比对。】【阅读推断：直接索引 `closure[j]`，若 Hasse 行引用下标 ≥ 当前行号的元素会 panic（索引越界）；即隐含要求 Hasse 图按拓扑序给出。】

---

## 5. 错误分支与预算

### 5.1 错误类型与触发点（全部 `StructureError` 变体）

| 变体 | 触发点（不变量字符串原样保留） |
|---|---|
| `DatumMismatch` | `build`：两半单秩不等，或 `dual_table.root_system() != dual_inner_class.root_system()` |
| `IndexOutOfRange { index, upper_bound }` | `dual_involution`：`generator >= dual_twist.len()`；`element_at`：`x.index()` 超出 `first_z_of_x` 范围（`upper_bound = len.saturating_sub(1)`） |
| `ArithmeticOverflow` | `build`：`size` 累乘/累加；`element_at`：`first + offset` 加法 |
| `BlockInvariantViolation { invariant }` | `"dual packet involution"`, `"dual packet record"`, `"packet involution"`, `"packet record"`, `"tau packet"`, `"dual tau packet"`, `"block element length"`, `"block size"`, `"cross link"`, `"dual cross link"`, `"Cayley link"`, `"dual inverse Cayley link"`, `"type II inverse Cayley pair"`, `"descent status"`, `"dual descent status"`, `"descent cross link"`, `"dual descent cross link"`, `"element lookup"`, `"element fiber"`, `"Cayley pair slots"` |
| 透传 `?`（本文件不定名具体变体） | `generator_twist()`、`crate::dual::longest_action(...)`、`WeylElement::from_action(...)`、`reduced_word(...)`、`right_multiply_simple(...)`、`graph.cayley(...)`（返回 `Result`）、`dual_graph.inverse_cayley(...)`（返回 `Result`）、`try_capacity(...)` |

### 5.2 预算

- 【已实现】`build` 的显式预算参数仅 `weyl_budget: usize`，传给 `crate::dual::longest_action`；注释称其界定定位对偶最长元素的枚举。
- 【已实现】容量预算经 `crate::grading::try_capacity` 作用于 `dual_w`（`packet_count`）、`xs`/`ys`/`lengths`（`size`）、`descent`/`cross`/`cayley_*`（`size * rank.max(1)`）。
- 【阅读推断】`size * rank.max(1)` 的乘法本身未做 `checked_mul`——溢出会在 `try_capacity` 之前于 debug 构建 panic；`dual()` 与访问器中的 `size * rank`、`generator * self.xs.len() + z` 同为普通乘法/加法。
- 【阅读推断】`rank == 0` 时 `build` 用 `rank.max(1)` 分配而访问器按 `z * rank` 索引且 `generator >= rank` 直接返回 `None`，不会越界读；`dual()` 中表尺寸为 `size * rank = 0`（空表），同样因访问器先查秩而不可达。

### 5.3 panic / 断言点

- 【已实现】`BlockDescent::index()` 中 `.expect("ALL covers every variant")`（仅当 `ALL` 与变体集不同步时可触发）。
- 【已实现】测试辅助 `graph_with_size` 找不到指定 KGB 大小的实形式时 `panic!("no real form with KGB size {size}")`；测试中大量使用 `unwrap()`。
- 【已实现】`n_bruhat_comparable` 与 `dual()`/`bruhat_hasse` 内无显式断言；`element_at` 用 `Err` 取代上游 assert（见 §3.2）。

---

## 6. 与其它模块的接口（仅本文件调用点可见的事实）

- `crate::grading::try_capacity(usize) -> Result<_, StructureError>`（按 `?` 用法）。
- `crate::dual`：`longest_action(&InnerClass, usize) -> Result<_, StructureError>`；测试中还用 `dual_inner_class(&InnerClass, usize, usize) -> Result<InnerClass, _>`。
- `crate::block_access::bruhat_hasse(&BlockGraph) -> Vec<Vec<usize>>`（自由函数，`bruhat_hasse` 方法与之等价，测试断言两者结果相等）。
- `KgbGraph`（方法均按本文件调用形式）：`semisimple_rank() -> usize`、`packet_count() -> usize`、`packet_involution(usize) -> Option<_>`、`tau_packet(usize) -> Option<(KgbId 起始, usize 计数)>`（以 `(x_start, x_count)` / `(y_start, y_count)` 解构，起始值调 `.index()`）、`size() -> usize`、`length(KgbId) -> Option<usize>`、`cross(KgbId, usize) -> Option<KgbId>`、`cayley(KgbId, usize) -> Result<Option<KgbId>, StructureError>`、`inverse_cayley(KgbId, usize) -> Result<Option<(KgbId, Option<KgbId>)>, StructureError>`、`status(KgbId, usize) -> Option<KgbStatus>`、`is_descent(KgbId, usize) -> Option<bool>`。
- `InvolutionTable`：`root_system() -> &RootSystem`、`record(id) -> Option<记录>`，记录上有 `weyl_element() -> &WeylElement`；`packet_involution` 的返回值类型即 `record` 键 `id`。
- `WeylElement`：`clone()`、`reduced_word(&RootSystem) -> Result<Vec<usize>, StructureError>`、`right_multiply_simple(&RootSystem, usize) -> Result<(WeylElement, _), StructureError>`、`from_action(&RootSystem, &_) -> Result<WeylElement, StructureError>`；测试中另有 `identity`、`simple_reflection`。
- `InnerClass`：`root_system() -> &RootSystem`、`generator_twist() -> Result<Vec<usize>, StructureError>`（按 `&[usize]` 使用）。
- `KgbId`：元组构造 `KgbId(usize)` 与 `.index() -> usize`；`KgbStatus` 变体 `Complex / ImaginaryNoncompact / Real / ImaginaryCompact`。
- 测试另引入：`AdjointFiberBudget`、`BasedRootDatum`（`from_simple_data`、`standard`）、`CartanClassification(::build)`、`CartanClassificationBudget(::new)`、`CartanId`、`Coweight`、`IntegerLatticeBudget(::new)`、`InvolutionTable(::new)`、`InvolutionTableBudget(::new)`、`LatticeInvolution(::identity)`、`RealFormSeed(::build)`、`StrongRealClassification(::build, kgb_size)`、`WeakRealFormId`、`Weight`。

---

## 7. 测试锚点

1. `a1_dual_involution_swaps_identity_and_reflection`：A1 两扭曲皆平凡、`w0 = s`；断言 `dual_involution(&[]) == reflection`、`dual_involution(&[0]) == identity`（注释：即负转置双射 `f(e)=s`、`f(s)=e`）。
2. `sl2r_pgl2r_block_matches_the_frozen_language_anchors`：【注释转述：domain/block_basic 冻结 fixture，capture 3501519】`block(SL(2,R), PGL(2,R))` 大小 3；坐标 `(x,y)` 依次为 `(0,1),(1,1),(2,0)`；`element` 反查 `0,1,2`；`descent_value` 为 `i1, i1, r1`；`status_code` 为 `6, 2`；`cross` 为 `1, 0, 2`；`cayley(0,0) == cayley(1,0) == (Some(2), None)`；`inverse_cayley(2,0) == (Some(0), Some(1))`；未定义情形 `cayley(2,0) == (None,None)`、`inverse_cayley(0,0) == (None,None)`（注释：wrapper 将其映回输入下标）。
3. `pgl2r_sl2r_dual_block_exercises_the_type_two_links`：伴随 datum（`BasedRootDatum::standard`）；块大小 3；坐标 `(0,2),(1,0),(1,1)`；状态 `i2, r2, r2`，语言码 `7, 3`；II 型 Cayley 双值 `cayley(0,0) == (Some(1), Some(2))`，逆 Cayley 单值 `(Some(0), None)` ×2；`cross(0,0) == 0`。
4. `a1_bruhat_hasse_uses_the_shared_topology_algorithm`：`bruhat_hasse() == [[], [], [0, 1]]`，且方法与 `crate::block_access::bruhat_hasse` 自由函数结果一致。
5. `block_descent_dual_matches_the_upstream_static_table`：逐项固定 `dual` 静态表及其对合性【注释转述：descents.h:74-79 的 `d[]` 表】。
6. `block_dual_reverses_swaps_and_maps_the_links`：对 `block(SL(2,R), PGL(2,R)).dual()` 逐元素/逐生成元核对反转、x/y 互换、长度反射、状态 `dual` 映射、`cross` 的 `size-1-c` 映射，以及 Cayley 槽 “first 有定义才映射” 的规则【注释转述：blocks.cpp:497-501】。
7. `block_dual_agrees_with_the_natively_built_dual_block`：`block(SL(2,R),PGL(2,R)).dual()` 与原生构建的 `block(PGL(2,R),SL(2,R))` 经 `element` 查找换编号后比对：长度、下降状态、交叉链接交换性、Cayley/逆 Cayley 作为**无序集合**相等（注释：换编号不保序，双值对不重排）；并断言两块均覆盖各自完整 KGB 范围时 `dual().dual()` 为恒等（含 `first_z_of_x` 表）。

测试流水线固定预算：`IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`、`AdjointFiberBudget(..., 50_000, 100_000)`、`StrongRealClassification::build(_, 4_096)`、`RealFormSeed::build(..., 4_096)`、`class_budget(weyl)` 尾部 `64, 64`；`graph_with_size` 通过 `strong.kgb_size(WeakRealFormId(form))` 选指定大小的形式，并在建图前 `table.add_cartan(&classification, CartanId(0))`、返回 `(graph, table.clone())`。

---

## 8. 限制与未覆盖面

- 【已实现/注释明示】上游 `Bare_block::dual` 的 `orbits`/`dd` 字段不复现（上游自注 “probably not right”）。
- 【阅读推断】`bruhat_hasse` 的算法实现不在本文件（委托 `crate::block_access`），本文件仅有秩 1（A1）拓扑 `[[], [], [0,1]]` 一个锚点；`n_bruhat_comparable` 无任何测试覆盖。
- 【阅读推断】测试全部限于秩 1（A1，单生成元）：未覆盖多生成元下 `z * rank + s` / `s * size + z` 两种扁平布局的区分、`ComplexAscent`/`ComplexDescent`/`RealNonparity`/`ImaginaryCompact` 状态在块级测试中的出现、`DatumMismatch` 及各 `BlockInvariantViolation`/溢出错误路径、空对偶包（`y_count = 0`、空块 `size = 0`）行为、`element` 的失败分支（`IndexOutOfRange` / `"element fiber"`）。
- 【阅读推断】`dual()` 中 `max_len = lengths.last()` 依赖“末元素长度最大”这一排序性质；空块时取 0 的语义无测试固定。`dual().dual()` 的恒等性仅在两块均覆盖完整 KGB 范围时被断言，注释亦限定于此情形。
- 【阅读推断】`cross` 表先填 0 再全量覆写，0 初值不可观测；但若未来循环边界改变，残留的 0 会是静默默认值而非错误。
- 【阅读推断】`dual_position` 以 `WeylElement` 为键，若对偶侧出现重复键将静默覆盖（`HashMap::insert`），本文件无防护亦无注释说明。
- 【注释转述，待核】所有上游行号引用（blocks.cpp、descents.h、kgb.cpp、atlas-types.w、poset.cpp）与 “capture 3501519” 冻结夹具均为注释陈述，本草案未核对上游来源。