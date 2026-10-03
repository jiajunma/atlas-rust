---
title: KType 层：标准表示的 K-限制
source: atlas-rust/ktype
ingestedAt: 2026-10-03T10:32:42Z
---

# KType 层：标准表示的 K-限制

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `ktype.rs` 的表示不变量与判定链；其正确性属于它自己的 HPC 证据链
（K-type formula 与 unitarity gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-ktype.json`](snapshots/2026-10-03-ktype.json)
（`ktype.rs` SHA-256
`04af728e4934f8fabc8ea004c807b4f56ce9d49cdc05e68272d38e8af74181db`，dirty
工作区）。

## 定位与表示不变量

`KType` 是标准表示的 K-限制，即 K 的不可约表示：参数数据是
[StandardRepr](rep-context.md) 去掉 `nu` 部分（K_repr.h:25-30）。存储的是
**选定的** `lambda-rho` 代表——模 $(1-\theta_x)X^*$ 的归一化只在
`KType::sr_k`（`Rep_context::sr_K`，K_repr.cpp:25-32）中发生一次——因此值
相等是上游的严格分量相等（K_repr.h:56-60），而 `equivalent` 关系经移到
canonical involution 计算（K_repr.cpp:159-171）。字段：
`x: KgbId`、`lam_rho: Weight`、`height: u32`（构造时预计算，
K_repr.h:36-44）。原始构造器是 `pub(crate)`；归一化是 `sr_k` 的职责。

## 性质判定族

- `is_standard`（K_repr.cpp:46-57）：`lambda` 在 simply-imaginary coroots
  上弱支配。
- `is_dominant`（:59-69）：$(1+\theta_x)\lambda$ 在每个单余根上弱支配。
- `is_nonzero`（:71-83）：不存在 singular 的 compact simply-imaginary
  root；与上游一致地假定 `is_standard` 成立。
- `is_semifinal`（:85-100）：没有 really-simple root 在测试权
  $2(\lambda-\rho) + 2\rho - 2\rho_R$ 上取奇值。
- `is_normal`（:102-123）：不存在 singular complex descent；上游断言前四个
  谓词成立且只在形容词链中调用——本移植因计算本身是 total 的而不带前置条件
  求值。
- `is_final`（:126-157）：$(1+\theta_x)\lambda$ 的支配性加上不存在任何
  singular descent。

## 等价关系与规范化链

`equivalent`：先判定是否属于同一 Cartan class（经 `graph().cartan_of`），
再双方将 `to_canonical_fiber` 后做严格相等比较。

规范化链：
- `made_dominant`（K_repr.cpp:174-204）：沿 $(1+\theta)\lambda$ 取负值的
  complex simple roots 做 cross 直至支配；非 standard 输入报错（对应上游
  "Non standard K-type in make_dominant"）。存储的 height 不变（该权重只按
  Weyl 共轭移动），与上游一致。终止由 `weight_defect` 预算保证。
- `made_theta_stable`（:207-233）：穷尽该 involution 的 simple complex
  descents；每次 cross 都是 descent，故以图大小作为宽松的终止上限。
- `to_canonical_fiber`（:236-256）：沿 `InnerClass::canonicalize` 的字 cross
  到该 Cartan class 的选定 fiber。
- `normalised`（:262-289）：先移到 canonical involution，再穷尽 singular
  complex descents（以及负的 complex 评估），在存在时得到一个 final 的类成员。

## 展开

- `finals_for`（K_repr.cpp:290-396）：把（可能非 final 的）K-type 展开为其
  等价类中 final K-types 的带符号重数列表：经 complex/noncompact-imaginary
  crosses 与 Cayley transforms 使 $(1+\theta_x)\lambda$ 支配，丢弃 singular
  compact 因子，并沿 parity real roots 分裂。返回表**无序**；系数合并发生在
  语言层、按多项式的规范项序进行。边界情形：final K-type 恰好产生自身、重数
  为 1。
- `kgp_set`（:398-464）：作用于（final 或 semifinal 的）K-type，给出从
  theta-stable 代表出发、经 inverse-Cayley splits 与沿 real-simple Levi
  生成元的 complex crosses 可达的 K-types；按上游的 BFS 发现顺序。
  semifinal 前置条件由调用方负责（wrapper 在调用前检查）。

## 来源与限制

- 源码：[ktype.rs](../../../crates/atlas-real-group/src/ktype.rs)；阅读快照
  [`2026-10-03-ktype.json`](snapshots/2026-10-03-ktype.json)。
- 上游行号均转述自源码注释（K_repr.h/K_repr.cpp），未独立重读上游，随版本
  演进可能漂移。
- 关联：[表示参数上下文](rep-context.md)、[KGB 图结构](kgb-graph-structure.md)、
  [Inner class 层](inner-class.md)、[形变驱动](deformation-drivers.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  122.4s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `equivalent` 的同 Cartan 类门控、`made_dominant` 的 height 不变量
  与终止预算的内容均已按源码落实，其余骨架内容未采用。调用记录见快照的
  `kimi_assist`。
