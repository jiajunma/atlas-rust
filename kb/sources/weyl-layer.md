---
title: Weyl 群层：矩阵作用与词级元素的双层结构
source: atlas-rust/weyl-layer
ingestedAt: 2026-10-03T10:32:42Z
---

# Weyl 群层：矩阵作用与词级元素的双层结构

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `weyl.rs` 与 `weyl_element.rs` 的双层分工；Weyl 层的正确性属于它
自己的 HPC 证据链（capacity gate、Weyl owner 语义线），本包不重述也不扩展。
所读字节见 [`snapshots/2026-10-03-weyl-layer.json`](snapshots/2026-10-03-weyl-layer.json)
（两个文件均为 dirty 工作区字节）。

## 双层分工与互查

- `weyl.rs` 是矩阵级作用层：`WeylAction` 在 character/cocharacter 两个全格上
  同时作用，自带 datum（"provenance-bearing"）。相等即矩阵作用相等，因此等价
  的生成子词给出相同值，无需枚举 Weyl 群。
- `weyl_element.rs` 是词级组合层（KGB map 的 stage (a)）：元素是一个 ambient
  `RootSystem` 的枚举根的置换，提供 O(1) 长度与 descent 查询、乘法、逆、
  twisted conjugation 与按需约化词。
- 两层经 `RootSystem::action_permutation` 互查；反向由
  `WeylElement::from_action` 桥接。

## WeylAction：矩阵级作用

字段：`datum: Arc<BasedRootDatum>`、`weight_matrix`、`coweight_matrix`。
构造器：`identity`（`lattice_rank` 阶单位矩阵）、`simple_reflection`（根/余根
对偶配对的反射矩阵）、`root_reflection`（任意枚举根上的反射——replay
Cayley/cross 分解需要强正交根中的反射，且符号无关）。运算：`compose`（
`self` after `right`）、`act`/`act_on_coweight`；访问器 `datum_arc`（Arc
refcount bump）、`datum`、`rank`、`matrix`、`coweight_matrix`。
`WeylGroup::enumerate_actions(budget)` 带显式基数预算枚举全部 canonical
作用（刻意与构造分离），结果按 character-lattice 作用矩阵的字典序排列。

## WeylElement：词级元素与不变量纪律

元素由 ambient `RootSystem` 的枚举根的置换表示；每次操作唯一可表达的
provenance 检查是根数匹配；单一 ambient system 纪律是调用方契约，由 KGB
stages 负责；置换的 antisymmetry 靠「构造器是唯一入口」保证。

- 构造：`identity`、`simple_reflection`、`from_action`（双层桥梁）。
- 查询：`length`/`is_identity`/`image`/`image_permutation`；
  `has_left_descent` 判定 $l(s w) < l(w)$（条件 $w^{-1}(\alpha_s) < 0$，
  读**逆**向量）；`has_right_descent` 判定 $l(w s) < l(w)$（条件
  $w(\alpha_s) < 0$，读**正向**置换）。
- 运算：`multiply`（与 `WeylAction::compose` 对齐；同一趟内以
  $(uv)^{-1} = v^{-1} u^{-1}$ 维护逆；长度从 positivity slice 重算——
  操作数长度不相加）；`left_multiply_simple`/`right_multiply_simple`
  报告长度变化（$-1$ 即 `sigma_mult` 分支，`+1$ 对应 `sigma_inv_mult`）；
  `inverse` 直接返回逆。
- `twisted_conjugate`：计算 $s_{\mathrm{gen}} \cdot \mathit{self} \cdot
  s_{\mathrm{twist}(\mathrm{gen})}$，是 Tits 扭曲共轭的 Weyl 影子；`twist`
  必须是生成子上的对合置换，其与 distinguished involution 的单根作用一致
  是调用方契约。stage (b) 消耗的长度变化（$d \in \{0, \pm 2\}$）由调用点
  做 cached-length 减法获得。

## 约化词与 canonical 选择

- `reduced_word`：按 lowest-left-descent peeling 生成，左到右复合。
- `canonical_word`：复刻上游 transducer 的 canonical reduced word
  （`WeylGroup::word`，weyl.cpp:944-957），在 `WeylInterface` 携带的
  INTERNAL 生成子序下字典序最小（上游记为 "minimal for ShortLex"，
  weyl.cpp:911-926）：能作为约化表达式首字母的生成子恰为左 descent，故逐次
  剥离最小内部左 descent 使每位字母最小。实现带不变量检查：剥离步长度必须
  恰减一，否则报 `WeylElementInvariantViolation`。
- `WeylInterface::new(cartan)` 持有上游 `WeylGroup` 构造器的内部生成子
  重编号（weyl.cpp:495-527）：Dynkin 分量按分类顺序，每个分量的 Bourbaki
  `position` 对 A/E/F/G 型直取、对 B/C/D 型反转；`outward()` 即上游
  `d_out`（internal → datum 生成子）。本移植只保留其可观察效果：
  `canonical_word` 的选词与 `ParabolicPieces` 的内部序 piece 索引。
- `ParabolicPieces` 复刻上游 transducer 的 `EltPiece` 索引
  （weyl.cpp:289-416）：`key` 返回元素按 internal-level 序的 piece 列表，
  对应唯一分解 $w = w_1\cdots w_n$（$w_i$ 是右陪集 $W_{i-1}.w$ 的最小代表
  元）；这些列表的字典序比较即上游的 `WeylElt::operator<`，也是上游
  involution 排序（`Cartan_orbits::comparer`，involutions.cpp:420-428）的
  tie-break，供 [KGB 图](kgb-graph-structure.md) 的重编号原样消费。

## 来源与限制

- 源码：[weyl.rs](../../../crates/atlas-real-group/src/weyl.rs)、
  [weyl_element.rs](../../../crates/atlas-real-group/src/weyl_element.rs)；
  阅读快照 [`2026-10-03-weyl-layer.json`](snapshots/2026-10-03-weyl-layer.json)。
- 上游行号均转述自源码注释（weyl.cpp/involutions.cpp），未独立重读上游，
  随版本演进可能漂移。
- 关联：[Weyl 身份与共享](weyl-context-identity-and-sharing.md)（语言层的
  owner/identity 语义）、[KGB 图结构](kgb-graph-structure.md)、
  [Inner class 层](inner-class.md)、[根坐标与格坐标](../wiki/math/root-coordinates.md)；
  `weyl_transducer.rs`（枚举实现）属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  151.4s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 descent 读取方向、`canonical_word` 的不变量检查、
  `WeylInterface` 重编号规则的内容均已按源码落实，其余骨架内容未采用。
  调用记录见快照的 `kimi_assist`。
