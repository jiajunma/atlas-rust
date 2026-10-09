---
title: 完整块图：实形式与对偶实形式的纤维积
source: atlas-rust/block-graph
ingestedAt: 2026-10-03T10:32:42Z
---

# 完整块图：实形式与对偶实形式的纤维积

编辑状态：**结构性阅读（两次：2026-10-03 初读、2026-10-06 重读，字节未变，
SHA-256 相同）；两份草案均经 Kimi probe 起草、维护者逐条对照源码核对后改写合并**。
本包解释 `block.rs`（1095 行）的块图构造与访问语义；块枚举的正确性属于它
自己的 HPC 证据链（rank6 inventory、PSp4R/GL2/G2 gate 等），本包不重述也
不扩展。上游行号均转述自源码注释，未独立重读上游，随版本演进可能漂移。

## 块是两个单边参数集的纤维积

一个块（upstream `blocks::Block`）是一个实形式的 KGB 图与其对偶实形式的
KGB 图在 twisted involution $w$ 与 $\mathrm{dual}_w$ 上配对的纤维积
（`Block::Block(kgb, dual_kgb)`，gkmod/blocks.cpp:526-606）。配对映射是
twisted Weyl 群对偶 `dual_involution`（blocks.cpp:1701-1711）：在
involution 矩阵上是负转置；在 Weyl 元素上由 $f(e) = w_0$ 与
$f(s.w) = f(w)\,d(s)$ 刻画。实现上 `dual_involution(word, dual_system,
dual_twist, dual_longest)` 从对偶最长元出发，按 $w$ 的约化字**自右向左**
以对偶扭曲字母右乘；`word` 携带两侧共享的外部生成元编号。

解释器从两个形式的**完整 KGB 集**构建块（`Block::build`，blocks.cpp:622-626，
由 `Block_value` 在 atlas-types.w:4753-4758 调用）：抽取的 KGB 坐标编号必须
保持各形式自身的 KGB 编号——`common_Cartans` 受限重载（blocks.cpp:610-619）
会改变编号（atlas-types.w:4739-4745 的注释）。对 common Cartans 的限制是
隐式的：对偶包以 `HashMap<WeylElement, usize>` 索引（重复键会**静默覆盖**，
无防护——2026-10-06 阅读观察），某 primal involution 的对偶不在其中时贡献
零对，恰如上游 `KGB_base::tauPacket` 在该处返回 `(0,0)`
（gkmod/kgb.cpp:131-140）。

## BlockDescent：八值 descent 状态

按上游 `descents::DescentStatus::Value` 顺序（gkmod/descents.h:40）：
`ComplexAscent`、`RealNoncompact`→`RealNonparity`、`ImaginaryTypeI`、
`ImaginaryTypeII`、`ImaginaryCompact`、`ComplexDescent`、`RealTypeII`、
`RealTypeI`。前四值是 ascent，后四值（索引位 `0x4` 置位）是 weak descent：

- `is_descent`：`index & 0x4 != 0`（descents.h:71）。
- `dual`：`ComplexAscent↔ComplexDescent`、`RealNonparity↔ImaginaryCompact`、
  `ImaginaryTypeI↔RealTypeII`、`ImaginaryTypeII↔RealTypeI`（descents.h:74-79）。
- `language_code`：Atlas 语言状态码，经 `TAB = [4,5,6,7,1,0,3,2]` 重编号
  （`block_status_wrapper`，atlas-types.w:4911-4913），即 0=C-, 1=ic, 2=r1,
  3=r2, 4=C+, 5=rn, 6=i1, 7=i2。

每个生成元的块级状态由 `descents()` 判定（blocks.cpp:1541-1568）：复根看
`is_descent`（真 → C−，否则 C+）；虚非紧看 cross 是否动（动 → i1，不动 →
i2）；实/虚紧看对偶侧——对偶虚非紧且 cross 动 → r2、不动 → r1，否则
Real → rn、ImaginaryCompact → ic。

## BlockGraph 的布局与构建

字段：`rank`；`xs`/`ys` 是每块元素的 primal/dual KGB 坐标（full-KGB 编号）；
`descent` 按 `z * rank + s` 平铺；`cross` 按 `s * size + z` 平铺（上游
`data[s][z]`）；`cayley_first`/`cayley_second` 对应上游
`Cayley_image.first/.second`，由直接 Cayley（ascent）与逆 Cayley（weak
descent）两个访问器共享；`lengths` 是每元素的 primal KGB 长度
（blocks.cpp:557 `kgb.length(x)`），供 KL 表的长度排序使用；
`first_z_of_x[x]` 是满足 $x(z) \ge x$ 的首个块元素（blocks.cpp:630-643），
长度 `xrange + 1` 并带 size 哨兵（`xs` 弱增是该表与 `element_at` 定位
赖以成立的不变量）。

`BlockGraph::build(graph, table, dual_graph, dual_table, dual_inner_class,
weyl_budget)` 移植 `Block::Block(kgb, dual_kgb)`：`dual_inner_class` 提供对偶
twisted Weyl 群（其 distinguished twist 与最长元），`weyl_budget` 限定定位
最长元的枚举规模；两图的半单秩不一致或对偶表与对偶 inner class 的根系统不同
时报 `DatumMismatch`。元素编号：包按原 KGB 对合序，包内 **x 外层、y 内层**
的笛卡尔积（blocks.cpp:548-558），`xs.len() != size` 报 `"block size"`。
cross/Cayley 表构建：i1 是单值直接 Cayley + `first_free_slot` 回填逆
Cayley；i2 是**双值**直接 Cayley（z→z1 经 dual_second，随后**落入** i1
分支处理 dual_first；blocks.cpp:575-590 的 fall-through）；槽满报
`"Cayley pair slots"`。cross 表先填 0 再全量覆写（0 初值不可观测，但若
循环边界改变会成静默默认值——2026-10-06 阅读观察）。

## 访问器语义

- `size`/`rank`；`x(z)`/`y(z)`/`length(z)` 返回 `Option`。
- `descent_value(z, s)`（`Block_base::descentValue`）；`status_code(z, s)` 即
  `language_code`。
- `cross(z, s)`：对每个生成元有定义。
- `cayley(z, s)`（blocks.h:143-148）：直接 Cayley 像对，在 weak descent 处
  强制 `Some((None, None))`；内层 `None` 即上游 `UndefBlock`。
- `inverse_cayley(z, s)`（blocks.h:150-155）：与 `cayley` 互补——恰在 weak
  descent 处返回槽原值，非 descent 返回 `Some((None, None))`。
- `element(x, y)`（blocks.cpp:242-248）：先取 `x` 的 `first_z_of_x` 区间，
  再按连续 `y` 偏移定位；找到的坐标会被校验（相当于上游 assert 的角色），
  失败返回 `Err(StructureError)`（越界 `IndexOutOfRange`、纤维不符
  `"element fiber"`）。

## dual() 与 Bruhat Hasse 图

`BlockGraph::dual` 对应上游 `Bare_block::dual`（blocks.cpp:474-509）：同一对
实形式角色互换的块，纯数据变换：元素序反转（$z' = \mathrm{size}-1-z$），
`x`/`y` 坐标互换，长度反射为 $\mathrm{max\_len} - \ell(z)$（
$\mathrm{max\_len} = \ell(\mathrm{size}-1)$，依赖末元素长度最大的排序性质；
空块取 0），每个 descent 状态经 `BlockDescent::dual` 映射，每条 cross/Cayley
链 $c$ 映为 $\mathrm{size}-1-c$（**Cayley 第二像仅在第一像有定义时映射**；
上游双值 Cayley 对在对偶中也不重排，故与原生对偶块比较时按无序集比）。
`first_z_of_x` 按 `max_y + 1` 范围重算。上游被标注 "probably not right" 的
`orbits`/`dd` 字段没有 Rust 对应物。与部分块的 `dual()`（见
[部分公共块](partial-common-block.md)）不同，full block 的 cross 链总有定义。

`bruhat_hasse` 是块的 Bruhat Hasse 图（blocks.cpp:1603-1656
`complete_Hasse_diagram`）：每个元素的直接下邻；递归沿首个严格 good descent
（complex 或 type-I real）进行，与 KGB 情形相同；在 split principal series
处，前驱恰为各 type-II 实 descent 的逆 Cayley 变换。实现委托
`crate::block_access`（见 [块访问/修饰符](block-access-modifier.md)）。
`n_bruhat_comparable` 按 poset.cpp:197-229 的 `n_comparable_from_Hasse`
统计 Bruhat 可比对数（含自身；要求 Hasse 行按拓扑序，直接索引
`closure[j]`，乱序会 panic——2026-10-06 阅读观察）。

## 测试锚点（2026-10-06 重读补充）

7 个测试，全部秩 1（A1）：`dual_involution` 交换恒等/反射；
block(SL(2,R), PGL(2,R))（大小 3，坐标 (0,1),(1,1),(2,0)，状态 i1/i1/r1，
语言码 6/2，两 i1 共享单值 r1 像，inverse 双值）对齐冻结 fixture
capture 3501519；block(PGL(2,R), SL(2,R))（i2 双值 Cayley、r2 单值逆）；
Bruhat-Hasse `[[],[],[0,1]]` 与 `block_access` 自由函数一致；descent dual
静态表逐项 + 对合性；`dual()` 的反转/互换/链接映射逐点核对；`dual()` 与
原生对偶块经 `element` 换编号后一致（覆盖全 KGB 范围时 `dual().dual()`
为恒等，含 `first_z_of_x` 表）。未覆盖：多生成元（两种平铺布局的区分）、
C±/rn/ic 块级状态、全部 20 个 `BlockInvariantViolation` 字面量分支、
空对偶包/空块、`element` 失败分支、`n_bruhat_comparable`。

## 来源与限制

- 源码：[block.rs](../../crates/atlas-real-group/src/block.rs)；阅读快照
  [`2026-10-03-block-graph.json`](snapshots/2026-10-03-block-graph.json)
  （初读）与 [`2026-10-06-block.json`](snapshots/2026-10-06-block.json)
  （重读，同一 SHA-256 `3d3a88fa…`）。
- 关联：[KGB 图结构](kgb-graph-structure.md)、
  [KLV 多项式](kl-polynomial-table.md)、
  [部分公共块](partial-common-block.md)、
  [块访问/修饰符](block-access-modifier.md)、
  [Weyl 身份与共享](weyl-context-identity-and-sharing.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 初读起草经由本地 Kimi probe（exit 0，83.1s）；重读同样经 Kimi probe
  （620s 期限，exit 0，242.7s），其 20 个 `BlockInvariantViolation` 字面量
  普查与未覆盖分支清单均精确，已并入正文。调用记录见两份快照的
  `kimi_assist`。
