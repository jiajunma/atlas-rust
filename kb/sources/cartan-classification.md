---
title: Cartan 分类：编号、预算与实形式归属
source: atlas-rust/cartan-classification
ingestedAt: 2026-10-03T10:32:42Z
---

# Cartan 分类：编号、预算与实形式归属

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `cartan_classification.rs` 与 `cartan_class.rs` 的构造与归属机制；
Cartan 分类的正确性属于它自己的 HPC 证据链（Cartan3868252、rank1
class/dual-incidence 覆盖），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-cartan-classification.json`](snapshots/2026-10-03-cartan-classification.json)
（两个文件均为 dirty 工作区字节）。

## 总览

`CartanClassification::build` 建立一个 inner class 的全部 Cartan 类并聚合四类
事实：per-form Cartan sets、most-split classes、twisted-involution 总数、严格
Cartan 偏序。内部的 `partition: Arc<TwistedConjugacyPartition>` 会被 dual
correspondence 与 dual-side classification 复用而不重建。

## CartanId：Atlas 顺序的编号

`CartanId(usize)` 的编号遵循 Atlas Cartan 顺序（innerclass.cpp:218-291，
task 1）：fundamental class 是 `CartanId(0)`；其后按 BFS 发现顺序——parents
按升序编号，每个 parent 的 positive imaginary roots 按上游 `RootNbr` 顺序
（先 height，再 simple 坐标的 reverse-lexicographic）；Cayley successor 在
比较与存储**之前**先做 `InnerClass::canonicalize`（innerclass.cpp:252-263）。
外部消费者经 `cartan_ids()` 升序迭代；`cartan_class(id)` 返回
`Option<&CartanClass>`。

## 预算分层

`CartanClassificationBudget` 拥有六个字段：`integer_lattice`、
`adjoint_fiber`（per-Cartan fiber chains 的预算）、`weyl_budget`、
`involution_budget: Option<usize>`、`max_fiber_elements`、`max_peeling_steps`。
默认走 legacy full-Weyl 枚举预算；`with_generated_involutions(limit)` 切换为
canonical representative discovery（direct classification），用自己的精确
twisted-involution 计数上限（计数含 identity），不改动 `weyl_budget` 与
legacy 构造模式。各 getter 均标注对应值是 classification cache key 的一部分。

## 严格 Cayley 偏序

`is_below(a, b)`：`a` 越界返回 `None`；否则查 `below[b][a]`。为真当且仅当
`a != b` 且 `b` 的 fixed torus 的 identity component Weyl-conjugate 进 `a`
的——`a` 位于一条非空 single-root Cayley links 链进入 `b` 的 more-compact 端。
fundamental class 位于其他每个类之下。不可反身性是构造不变量，故
`is_below(x, x)` 恒为 `Some(false)`。

## 弱实形式事实与 real_form_of

- `weak_real_form_count()`：弱实形式数量（from the fundamental partition）。
- `cartan_set(form)`：该形式所处的 Cartan classes，升序。
- `most_split(form)`：该形式**唯一**的 most-split Cartan class。
- `real_form_of(inner_class, twisted, factor)`（innerclass.cpp:1305-1355，上游
  唯一调用者是 synthetic `real_form(InnerClass,mat,ratvec)` wrapper，
  atlas-types.w:3851-3871）：返回认领强对合数据 `(twisted, factor)` 的弱实
  形式；`factor` 是调用方投影的 theta-fixed rational coweight（wrapper 中
  doubled、centrality-checked、再 halved 的 torus factor）。上游的 `coch`
  输出只喂 `minimal_torus_part` 与 default-seed 比较；本 crate 的
  `RealFormSeed` 仅由 form id 重算 elected seed，故此处不返回 cocharacter。
- canonicalize conjugator 只用 COMPLEX letters（原因记录在
  innerclass.cpp:760-766），因此 `complex_cross_act`（tits.h:191-192，factor
  的一次 simple reflection）是 `real_form_of` 内唯一会触发的 transport；行走
  只在 complex simple roots 上分枝：imaginary 或 real simple root 精确固定
  Weyl 部分（当 $\theta(\alpha_s) = \pm\alpha_s$ 时 $s\theta s = \theta$），
  故 complex 步骤 alone 即穷尽 Weyl-part cross orbit；遇到的第一个 class
  representative 即为该数据的 Cartan class。
- 在命中的类处测量 grading：bit 置位（noncompact）当且仅当 factor 与
  simple-imaginary root 的配对是**偶数**整数——上游
  `gr.set(i, not a.torus_part().negative_at(...))`；配对的整数性上游在
  `negative_at` 内部断言，本 crate 以一个命名不变量门控。
- `real_form_of_detailed` = `real_form_of` 加上 `twisted` 所属的 Cartan
  class：synthetic wrapper 需要该类，以便在调用 `minimal_torus_part` 前把
  involution table 扩展到它之下的每个类（atlas-types.w:3902-3907）。

## TwistedConjugacyClass 与 CartanClass

`TwistedConjugacyClass`（cartan_class.rs）是 Weyl twisted conjugacy 下的一条
确定性轨道：字段 `representative: TwistedInvolution` 与
`twisted_involution_count: usize`。代表元有两种来源：
`TwistedConjugacyPartition` 产出的类以本 crate 确定性 Weyl 枚举中的第一个
action 为代表；`CartanClassification` 消费时会用 Atlas-canonical 代表元重建
（`InnerClass::canonicalize` 在编号前作用于 Cayley successor）。无论哪种代表
元，类都经 `CayleyCrossDecomposition` 分解，实形式标签在同一代表元处经
`RealFormLabels` 关联；fiber groups、实形式归属与实 Cartan 分量数据住在
`CartanClass`（owns 一个 `TwistedConjugacyClass` 值），其访问器面为
`representative`/`twisted_involution_count`/`decomposition`/`grading`/
`partition`/`labels`。

## 来源与限制

- 源码：[cartan_classification.rs](../../../crates/atlas-real-group/src/cartan_classification.rs)、
  [cartan_class.rs](../../../crates/atlas-real-group/src/cartan_class.rs)；
  阅读快照
  [`2026-10-03-cartan-classification.json`](snapshots/2026-10-03-cartan-classification.json)。
- 上游行号均转述自源码注释（innerclass.cpp/atlas-types.w/tits.h），未独立
  重读上游，随版本演进可能漂移。
- 关联：[KGB 图结构](kgb-graph-structure.md)、
  [表示参数上下文](rep-context.md)、
  [Weyl 身份与共享](weyl-context-identity-and-sharing.md)；
  `CayleyCrossDecomposition`、`CartanGradingData`、`RealFormLabels`、
  `WeakRealFormPartition`、`TwistedConjugacyPartition` 的展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  210.1s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `is_below` 语义、`real_form_of` 的 complex-only 行走与 grading
  规则的内容均已按源码落实，其余骨架内容未采用。调用记录见快照的
  `kimi_assist`。
