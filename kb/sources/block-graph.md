---
title: 完整块图：实形式与对偶实形式的纤维积
source: atlas-rust/block-graph
ingestedAt: 2026-10-03T10:32:42Z
---

# 完整块图：实形式与对偶实形式的纤维积

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `block.rs` 的块图构造与访问语义；块枚举的正确性属于它自己的 HPC
证据链（rank6 inventory、PSp4R/GL2/G2 gate 等），本包不重述也不扩展。所读
字节见 [`snapshots/2026-10-03-block-graph.json`](snapshots/2026-10-03-block-graph.json)
（`block.rs` SHA-256
`3d3a88fa0bfc3ab7cf45b055883f3a7e9c37e5c7928e39bbc981b723883c9d8e`，dirty
工作区）。

## 块是两个单边参数集的纤维积

一个块（upstream `blocks::Block`）是一个实形式的 KGB 图与其对偶实形式的
KGB 图在 twisted involution $w$ 与 $\mathrm{dual}_w$ 上配对的纤维积
（`Block::Block(kgb, dual_kgb)`，gkmod/blocks.cpp:526-606）。配对映射是
twisted Weyl 群对偶 `dual_involution`（blocks.cpp:1701-1711）：在 involution
矩阵上是负转置；在 Weyl 元素上由 $f(e) = w_0$ 与 $f(s.w) = f(w)\,d(s)$ 刻画。

解释器从两个形式的**完整 KGB 集**构建块（`Block::build`，blocks.cpp:622-626，
由 `Block_value` 在 atlas-types.w:4753-4758 调用）：抽取的 KGB 坐标编号必须
保持各形式自身的 KGB 编号——`common_Cartans` 受限重载（blocks.cpp:610-619）
会改变编号（atlas-types.w:4739-4745 的注释）。对 common Cartans 的限制是隐式
的：某 primal involution 的对偶若不在对偶形式的 KGB 中，就贡献空 packet，
恰如上游 `KGB_base::tauPacket` 在该处返回 `(0,0)`（gkmod/kgb.cpp:131-140）。

## BlockDescent：八值 descent 状态

按上游 `descents::DescentStatus::Value` 顺序（gkmod/descents.h:40）：
`ComplexAscent`、`RealNonparity`、`ImaginaryTypeI`、`ImaginaryTypeII`、
`ImaginaryCompact`、`ComplexDescent`、`RealTypeII`、`RealTypeI`。前四值是
ascent，后四值（索引位 `0x4` 置位）是 weak descent：

- `is_descent`：`index & 0x4 != 0`（descents.h:71）。
- `dual`：对偶块中对应生成元的状态——`ComplexAscent↔ComplexDescent`、
  `RealNonparity↔ImaginaryCompact`、`ImaginaryTypeI↔RealTypeII`、
  `ImaginaryTypeII↔RealTypeI`（descents.h:74-79）。
- `language_code`：Atlas 语言状态码，经 `tab = {4,5,6,7,1,0,3,2}` 重编号
  （`block_status_wrapper`，atlas-types.w:4911-4913），即 0=C-, 1=ic, 2=r1,
  3=r2, 4=C+, 5=rn, 6=i1, 7=i2。

## BlockGraph 的布局与构建

字段：`rank`；`xs`/`ys` 是每块元素的 primal/dual KGB 坐标（full-KGB 编号）；
`descent` 按 `z * rank + s` 平铺；`cross` 按 `s * size + z` 平铺（上游
`data[s][z]`）；`cayley_first`/`cayley_second` 对应上游
`Cayley_image.first/.second`，由直接 Cayley（ascent）与逆 Cayley（weak
descent）两个访问器共享；`lengths` 是每元素的 primal KGB 长度
（blocks.cpp:557 `kgb.length(x)`），供 KL 表的长度排序使用；
`first_z_of_x[x]` 是满足 $x(z) \ge x$ 的首个块元素（blocks.cpp:630-643），
长度 `xrange + 1` 并带 size 哨兵。

`BlockGraph::build(graph, table, dual_graph, dual_table, dual_inner_class,
weyl_budget)` 移植 `Block::Block(kgb, dual_kgb)`：`dual_inner_class` 提供对偶
twisted Weyl 群（其 distinguished twist 与最长元），`weyl_budget` 限定定位
最长元的枚举规模；两图的半单秩不一致或对偶表与对偶 inner class 的根系统不同
时报 `DatumMismatch`。

## 访问器语义

- `size`/`rank`；`x(z)`/`y(z)`/`length(z)` 返回 `Option`。
- `descent_value(z, s)`（`Block_base::descentValue`）；`status_code(z, s)` 即
  `language_code`。
- `cross(z, s)`：对每个生成元有定义。
- `cayley(z, s)`（blocks.h:143-148）：直接 Cayley 像对，在 weak descent 处
  强制无定义；内层 `None` 即上游 `UndefBlock`。
- `inverse_cayley(z, s)`（blocks.h:150-155）：恰在 weak descent 处有定义；
  生成元越界返回外层 `None`，非 descent 返回 `Some((None, None))`。
- `element(x, y)`（blocks.cpp:242-248）：先取 `x` 的 `first_z_of_x` 区间，
  再按连续 `y` 偏移定位；找到的坐标会被校验（相当于上游 assert 的角色），
  失败返回 `Err(StructureError)`。

## dual() 与 Bruhat Hasse 图

`BlockGraph::dual` 对应上游 `Bare_block::dual`（blocks.cpp:474-509）：同一对
实形式角色互换的块，纯数据变换：元素序反转（$z' = \mathrm{size}-1-z$），
`x`/`y` 坐标互换，长度反射为 $\mathrm{max\_len} - \ell(z)$（
$\mathrm{max\_len} = \ell(\mathrm{size}-1)$），每个 descent 状态经
`BlockDescent::dual` 映射，每条 cross/Cayley 链 $c$ 映为 $\mathrm{size}-1-c$
（Cayley 第二像仅在第一像有定义时映射）。上游被标注 "probably not right" 的
`orbits`/`dd` 字段没有 Rust 对应物。与部分块的 `dual()`（见
[部分公共块](partial-common-block.md)）不同，full block 的 cross 链总有定义。

`bruhat_hasse` 是块的 Bruhat Hasse 图（blocks.cpp:1603-1656
`complete_Hasse_diagram`）：每个元素的直接下邻；递归沿首个严格 good descent
（complex 或 type-I real）进行，与 KGB 情形相同；在 split principal series
处，前驱恰为各 type-II 实 descent 的逆 Cayley 变换。`n_bruhat_comparable` 按 poset.cpp:197-229 的 `n_comparable_from_Hasse`
统计 Bruhat 可比对数（含自身）。

## 来源与限制

- 源码：[block.rs](../../../crates/atlas-real-group/src/block.rs)；阅读快照
  [`2026-10-03-block-graph.json`](snapshots/2026-10-03-block-graph.json)。
- 上游行号均转述自源码注释，未独立重读上游，随版本演进可能漂移。
- 关联：[KGB 图结构](kgb-graph-structure.md)、
  [KLV 多项式](kl-polynomial-table.md)、
  [部分公共块](partial-common-block.md)、
  [Weyl 身份与共享](weyl-context-identity-and-sharing.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  83.1s）。草案由维护者对照源码逐条核对改写；其「待源码核对」项中涉及
  `BlockDescent` 八值序与 dual 映射、`build` 签名、`inverse_cayley` 返回
  形态、`element` 校验语义的内容均已按源码落实，其余骨架内容未采用。
  调用记录见快照的 `kimi_assist`。
