---
title: Weyl 群层：矩阵作用与词级元素的双层结构
source: atlas-rust/weyl-layer
ingestedAt: 2026-10-03T10:32:42Z
---

# Weyl 群层：矩阵作用与词级元素的双层结构

编辑状态：**结构性阅读（两次：2026-10-03 初读、2026-10-06 重读，字节未变，
SHA-256 相同）；两份草案均经 Kimi probe 起草、维护者逐条对照源码核对后改写合并**。
本包解释 `weyl.rs` 与 `weyl_element.rs` 的双层分工；Weyl 层的正确性属于它
自己的 HPC 证据链（capacity gate、Weyl owner 语义线），本包不重述也不扩展。
所读字节见 [`snapshots/2026-10-03-weyl-layer.json`](snapshots/2026-10-03-weyl-layer.json)
（初读）与 [`snapshots/2026-10-06-weyl-root-involution.json`](snapshots/2026-10-06-weyl-root-involution.json)
（重读，仅 `weyl.rs`，与 root_involution.rs 同包进行）。

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
对偶配对的反射矩阵——`reflection_matrix`：`M[i][j] = δ_ij − reflected[i]·
pairing[j]`，i128 checked + i32 收窄；根用 `.get` 越界报
`IndexOutOfRange { upper_bound: semisimple_rank }`，余根直接下标——长度不
一致时的潜在 panic 面，2026-10-06 阅读观察）、`root_reflection`（任意枚举根
上的反射——replay Cayley/cross 分解需要强正交根中的反射，且符号无关）。
运算：`compose`（`self` after `right`）、`act`/`act_on_coweight`；访问器
`datum_arc`（Arc refcount bump）、`datum`、`rank`、`matrix`、
`coweight_matrix`。`WeylGroup::enumerate_actions(budget)` 带显式基数预算
枚举全部 canonical 作用（刻意与构造分离），结果按 character-lattice 作用
矩阵的字典序排列。

实现细节（2026-10-06 重读补充）：

- **等值 = derive 逐字段比较（含 datum 值）**——与文档「相等即矩阵作用
  相等」存在张力：矩阵相同而 datum 值不同的两作用按 derive 不相等；因
  Arc 的 PartialEq 委派给内层值，同 datum 值时两表述一致。
- `compose_matrices` 用 i64 累加后 **`sum as i32` 无检查截断**（注释论证
  Weyl 矩阵条目 Cartan-bounded；该论证未形式化）；`compose_fast`
  （pub(crate)，枚举热循环用）完全无检查，前置条件违约即 panic 面；
  `apply_matrix` 对 ragged 矩阵行复用 `InvalidRootAutomorphism` 变体。
- `enumerate_actions` 走 `weyl_transducer::CompactWeyl` 紧致表示枚举
  （注释：E6 约 50ms vs 矩阵 BFS 约 1.1s——转述，非本包测量），再 rayon
  `par_iter` 逐元素 `compose_fast` 物化矩阵；字典序来自 CompactWeyl 输出序
  + rayon 保序 collect，**无测试断言**。
- **死代码观察**：私有 `insert_action`（VecDeque BFS 去重助手）在文件内无
  任何调用点，疑为旧矩阵 BFS 路径遗留。

测试锚点（2026-10-06 重读补充，7 个）：全格作用（格秩 2 半单秩 1 的 A1）；
A2 编织关系值相等；非对称 Cartan 下双作用保配对；空半单部分的
IndexOutOfRange；A2 预算 6 成功/5 报 `ResourceLimitExceeded`；同秩异 datum
拒（`DatumMismatch`）；`i32::MAX` 根坐标报 `ArithmeticOverflow`。

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

- 源码：[weyl.rs](../../crates/atlas-real-group/src/weyl.rs)、
  [weyl_element.rs](../../crates/atlas-real-group/src/weyl_element.rs)；
  阅读快照 [`2026-10-03-weyl-layer.json`](snapshots/2026-10-03-weyl-layer.json)
  （初读）与
  [`2026-10-06-weyl-root-involution.json`](snapshots/2026-10-06-weyl-root-involution.json)
  （重读，仅 `weyl.rs`，同一 SHA-256 `222f4249…`）。
- 上游行号均转述自源码注释（weyl.cpp/involutions.cpp），未独立重读上游，
  随版本演进可能漂移。
- 关联：[Weyl 身份与共享](weyl-context-identity-and-sharing.md)（语言层的
  owner/identity 语义）、[KGB 图结构](kgb-graph-structure.md)、
  [Inner class 层](inner-class.md)、[根坐标与格坐标](../wiki/math/root-coordinates.md)；
  `weyl_transducer.rs`（枚举实现）属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 初读起草经由本地 Kimi probe（exit 0，151.4s）；重读同样经 Kimi probe
  （600s 期限，exit 0，327.7s），其等值语义张力、死代码与未形式化截断三处
  观察均属实并已并入正文。调用记录见两份快照的 `kimi_assist`。
