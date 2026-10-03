---
title: KGB 图的结构与构造（每个弱实形式一张图）
source: atlas-rust/kgb-graph-structure
ingestedAt: 2026-10-03T10:32:42Z
---

# KGB 图的结构与构造（每个弱实形式一张图）

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `kgb_graph.rs` 的数据布局与构造算法；不声称 KGB 枚举的数学正确性
（那属于它自己的 HPC 证据链），不含性能或兼容性结论。所读字节见
[`snapshots/2026-10-03-kgb-graph.json`](snapshots/2026-10-03-kgb-graph.json)
（`kgb_graph.rs` SHA-256
`a82e9d7cabaa7cb21f2512486ffb34134d4eee9bddc6603248ff19e16f97380d`，dirty
工作区）。

## 数学对象

Atlas 中的 KGB 集合是 $K$ 在旗簇 $G/B$ 上的轨道集合；在本实现中，一张
`KgbGraph` 对应**一个弱实形式**（模块文档称之为 KGB stage e，由 stage-(d) 的
种子 `RealFormSeed` 生成）。图的元素是该形式各 involution 之上的 Tits 元素；
每个元素在每个单生成元（simple generator）处携带一个状态、一条 cross 链接，
以及可能的 Cayley 与 inverse-Cayley 链接。

## 状态与分类

`KgbStatus` 是四值枚举：`Complex`、`ImaginaryCompact`、`Real`、
`ImaginaryNoncompact`，表示一个单生成元在一个 KGB 元素处的状态。分类分两步：
先由 involution 表的 `simple_root_kind` 给出 `RootKind`
（Complex/Real/Imaginary），再对 imaginary 情形用 coset 的
`simple_grading_pregated` 区分 compact 与 noncompact。

`is_descent` 的规则直接从状态读出：real 生成元恒为 descent，imaginary 恒不是，
complex 则比较 cross 链接两端元素的 involution 长度（目标更短者为 descent）。

## 构建：门控、BFS 与不变量

`KgbGraph::build` 的输入是 `InnerClass`、Cartan 分类、强实形式分类、可变的
`InvolutionTable` 和种子。前置检查链依次为：表与 inner class 不匹配报
`DatumMismatch`；`strong.kgb_size(form)` 或 `classification.cartan_set(form)`
缺失报 `IndexOutOfRange`（两处 `upper_bound` 的回退构造不同：一处经
`strong_real_data(CartanId(0))`，一处用 `weak_real_form_count()`——现状如此，
未见注释说明是否有意）。随后是该形式的升序幂等 Cartan 添加（mutation
phase），再验证种子-表绑定：恒等 `WeylElement` 的表内 lookup 必须等于种子的
involution，且种子的 `torus_bits` 必须已是 `mod_space` 的商代表元，否则报
`KgbInvariantViolation { invariant: "seed element" }`。`TitsCoset` 用同一个
inner class 一次门控，覆盖整个 BFS。

BFS 本身是**分窗两相**的：以 64 个元素为一窗，第一相用 Rayon
`into_par_iter` 对窗内元素做纯计算（状态分类、cross、Cayley 目标；表与 coset
在此期间只读），第二相顺序 `intern`（按 `TitsElement` 去重、分配新 id）。
关键不变量：状态槽 write-once（同一 `(x, s)` 写入两次报
`KgbInvariantViolation`）；非紧致 imaginary 的 Cayley 目标必须存在且其
involution 长度恰比源多一（`"Cayley length step"`）；BFS 结束时元素总数必须
等于强实形式分类预言的 `kgb_size`（`"kgb size"`）。数学套件的计时运行强制
`RAYON_NUM_THREADS=1`；这里的并行结构存在不等于已有多核加速证据。

## 上游一致的元素编号

模块声明其编号精确复现上游 `KGB::KGB`（kgb.cpp:489-683）：先按上游排序键
排列该形式的 involution——(involution 长度, Weyl 长度, `WeylElt::pieces`
字典序)，即 `Cartan_orbits::comparer`（involutions.cpp:420-428；源码注释指出
上游 kgb.cpp/involutions.h 中"internal number"注释已过时，实际比较的是
`TwistedInvolution` 的值）——再用**计数排序**把 BFS 发现顺序标准化到各 tau
packet 内：packet 间按排序后的 involution 位置排列，packet 内保持 BFS 发现
顺序。排序键是严格全序，因此 `sort_unstable` 的稳定性无关紧要；真正承载语义的
稳定性全在计数排序。`positions` 记录每个排序位置的
`(InvolutionId, involution 长度, CartanId)`，`first_of_tau` 是长度
`positions.len()+1` 的累计计数；`tau_packet(position)` 返回该 packet 的
`(首元素, 大小)`。

## 链接与存储

- `statuses`、`cross`、`cayley`、`inverse_cayley` 均按 `x * rank + s` 平铺。
- `cross(x, s)` 返回 `Option<KgbId>`（越界为 `None`）；`cayley(x, s)` 返回
  `Result<Option<KgbId>, _>`：`Ok(None)` 表示该生成元在此元素不是非紧致
  imaginary（无 Cayley 链接），`Err` 只来自下标检查。
- `inverse_cayley(x, s)`：`Ok(None)` 表示生成元在此元素不是 real；
  `(first, None)` 是 II 型，`Some((first, Some(second)))` 是 I 型且
  `first < second`。它由标准化之后的升序后处理安装。
- HYBRID self-contained：每个 involution 位置的数据和 cocharacter 都复制进图，
  除 `torus_factor` 外所有访问器都不需要 involution 表。`torus_factor` 计算
  `(g_rho_check - lift(bits) + theta^T 作用) / 2`（精确有理数），是唯一仍需要
  表的访问器，因为 theta 是逐 involution 的。
- `base_grading` 对应上游 `KGB_base::base_grading`（kgb.h:339），即
  `var_print_KGB` 输出的 `Base grading: [...]` 头部。

## 来源与限制

- 源码：[kgb_graph.rs](../../../crates/atlas-real-group/src/kgb_graph.rs)；
  阅读快照
  [`2026-10-03-kgb-graph.json`](snapshots/2026-10-03-kgb-graph.json)。
- 上游引用行号来自源码注释，未经独立重读，随上游演进可能漂移。
- 前置概念：[根坐标与格坐标](../wiki/math/root-coordinates.md)、
  [Weyl 身份与共享](weyl-context-identity-and-sharing.md)；相关治理与验收边界见
  [项目规则](../../../AGENTS.md) 与 [交接记录](../../../docs/HANDOFF.md)。
- 本包未执行任何构建、测试或原版运行；KGB 枚举的正确性证据（rank6 640/640
  inventory 等）属于它自己的 gate 链，本包不重述也不扩展。
- 起草经由已验证的本地 Kimi probe 路由（无工具、文本提案；模型
  `kimi-code/k3-256k`）；进程在产出草案后未在 180 秒期限内退出，被 runner 按
  SIGTERM 清理，无残留进程组成员。草案由维护者对照源码逐条核对并改写；其
  "待源码核对" 项中涉及 Cayley/inverse-Cayley 语义、tau packet 返回值、
  descent 规则、两相 BFS 和排序稳定性的内容均已按源码落实，其余不确切的骨架
  内容未采用。详见快照的 `kimi_assist` 记录。
