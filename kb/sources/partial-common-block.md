---
title: 部分公共块：Bruhat 区间上的块构造
source: atlas-rust/partial-common-block
ingestedAt: 2026-10-03T10:32:42Z
---

# 部分公共块：Bruhat 区间上的块构造

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `partial_block.rs` 的五个构件及其上游对应；partial block 的正确性
属于它自己的 HPC 证据链（PSp4R/GL2/G2 修复 gate 等），本包不重述也不扩展。
所读字节见
[`snapshots/2026-10-03-partial-common-block.json`](snapshots/2026-10-03-partial-common-block.json)
（`partial_block.rs` SHA-256
`cfa526973c1fef46336456174cd40c86f373dd83845d9c899499d109cd72052e`，dirty
工作区）。

## 定位与五个构件

本模块移植 Atlas `print_partial_block` wrapper（interpreter/atlas-types.w:6700-6711）
背后的机制：积分子系统上的 Bruhat 区间与部分公共块。五个构件：

- `StandardReprMod`：上游 `repr::StandardReprMod`（repr.cpp:52-67）——KGB 元素
  `x` 加经 $(1-\theta)X^*$ 约化并规范化的 `gamma_lambda`；值相等即上游哈希表
  相等。两条构造路径：`build`（repr.cpp:61-67，经 `RepContext::build_srm`）与
  `mod_reduce`（repr.cpp:52-58，经 `RepContext::mod_reduce`，即打印 wrapper 的
  种子计算，atlas-types.w:6704）。
- `IntegralSubsystem`：上游 `subsystem::SubSystem`（structure/subsystem.cpp:35-101）
  的单根数据，由 `integrality_simples`（structure/rootdata.cpp:1494-1500）构造；
  只移植 `common_context` 用到的按生成元编号的访问器
  （`parent_nr_simple`/`simple`/`to_simple`/`reflection`），Bruhat 生成器不需要
  完整的子系统根闭包。
- `CommonContext`：`repr::common_context`（repr.cpp:2666-2677），携带 srm 层面
  的五个生成元操作（下节）。
- `bruhat_below`：`Rep_table::Bruhat_below`（repr.cpp:1565-1573），经
  `Bruhat_generator::block_below`（repr.cpp:1476-1563）产出种子的 Bruhat 区间
  srm 列表，交给 `PartialBlock::build` 消费。
- `PartialBlock`：部分公共块（下两节）。

错误处理约定：上游保护调用方契约、oracle 仅在 NDEBUG 下检查的 `assert` 在此
省略并逐处留注释（commit f668589 先例）；真正的内部不一致以 `StructureError`
报告而非 panic。

## CommonContext 的 srm 层面操作

五个操作均把 KGB 层面的生成元作用转运到共轭的父单根上：

- `status`（repr.cpp:2679-2692）：返回 `(KgbStatus, bool)`；布尔标志在 real
  情形是 `isDoubleCayleyImage`，complex 情形是 `isDescent`，非紧致 imaginary
  区分 type-1/type-2 cross-move。
- `cross`（repr.cpp:2694-2708）：先按反射词对 `x` 做 cross，并以 `pos_to_neg`
  实根修正平移 `gamma_lambda`，再按父根反射。
- `is_parity`（repr.cpp:2711-2722）、`down_cayley`（:2724-2742）、
  `up_cayley`（:2744-2773；提升后的 `gamma_lambda` 不满足奇偶条件时加
  $\alpha_s/2$ 的奇偶修正）。
- `singular_flags`（blocks.cpp:701-708 的 `common_block::singular`）：逐子系统
  生成元判断其余根是否在 `gamma` 的分子上取零。

这些上游断言属调用方契约，本模块不重复检查。

## PartialBlock 的构造与编号

- `build_full`：完整公共块构造器（blocks.cpp:733-1081）；空积分子系统是单元素
  块；同一套 packet 构造同时处理 ambient-full 与真积分子系统两种情形。
- `build`：部分块构造器（blocks.cpp:1086-1248），消费 `bruhat_below` 产出的
  按 `x` 排序的 srm 区间；构造结束时再按 `(length, x, y)` 排序
  （blocks.cpp:1488-1517），因此元素编号与 oracle 打印的行号一致。

## 访问器语义

- `size`/`rank`；`element`/`x`/`y`/`length`/`gamma_lambda` 均返回 `Option`；
  `lookup`（blocks.cpp:1250+ 的 `common_block::lookup`）给出 srm 的块元素，
  区间外返回 `None`（上游 `UndefBlock`）。
- `descent(z, s)`：逐生成元的 `BlockDescent` 状态。
- `cross(s, z)`：`None` 即上游 `UndefBlock`（链离开区间或未设置）。
- `cayley(s, z)`：Cayley 像对——imaginary ascent 的前向 Cayley 目标、real
  descent 的逆 Cayley（上游 `block_fields::Cayley_image`；打印层经
  `isWeakDescent` 选择）。
- 打印支持：`highest_x`/`highest_y`（block_io.cpp:58-59 的 `max_x`/`max_y`）、
  `survives`（blocks.cpp:323-330 的 `Block_base::survives`：没有任何 singular
  生成元是 `z` 的 descent）。

## dual()：纯数据变换及其部分块限制

`dual()` 对应上游 `Bare_block::dual`（gkmod/blocks.cpp:474-507），是
`BlockGraph::dual` 的公共块 analogue：交换每行的 primal/dual 角色。细则：
元素顺序反转（$z' = \mathrm{size}-1-z$）；`x`/`y` 坐标互换；长度反射为
$\mathrm{max\_len} - \ell(z)$（$\mathrm{max\_len}=\ell(\mathrm{size}-1)$）；
每个 descent 状态经 `BlockDescent::dual` 映射；每条 cross/Cayley 链 $c$ 映为
$\mathrm{size}-1-c$（Cayley 第二像仅在第一像有定义时映射）。

限制（文档载明）：上游只对 full block 对偶化（cross 链总有定义）；部分块可能
有离开区间的链（`UndefBlock`），这些链保持未定义。KL 递归只对 link-closed 源
（full block）的对偶有效：部分区间的外出 complex ascent 对偶化为带未定义
cross 的 complex descent，`KlTable` 会拒绝。返回值是 `BareBlock`：对偶块的行
不是本块 context 的标准模参数，参数池与查找表没有对偶对应物。

## 来源与限制

- 源码：[partial_block.rs](../../../crates/atlas-real-group/src/partial_block.rs)；
  阅读快照
  [`2026-10-03-partial-common-block.json`](snapshots/2026-10-03-partial-common-block.json)。
- 上游行号均转述自源码注释，未独立重读上游，随版本演进可能漂移。
- 关联：[KGB 图结构](kgb-graph-structure.md)、
  [KLV 多项式](kl-polynomial-table.md)、
  [根坐标与格坐标](../wiki/math/root-coordinates.md)；`RepContext::mod_reduce`/
  `build_srm`、`BlockDescent`、`BruhatGenerator` 的展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  232.8s，300 秒期限足够）。草案由维护者对照源码逐条核对改写；其「待源码
  核对」项中涉及 `bruhat_below` 签名、Cayley 像对语义、`dual()` 变换细则与
  限制、`highest_x/highest_y` 的内容均已按源码落实，其余骨架内容未采用。
  调用记录见快照的 `kimi_assist`。
