---
title: Inner class 层：构造、验证门与 twisted 共轭枚举
source: atlas-rust/inner-class
ingestedAt: 2026-10-03T10:32:42Z
---

# Inner class 层：构造、验证门与 twisted 共轭枚举

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `inner_class.rs` 的层边界与算法；inner class 的正确性属于它自己的
HPC 证据链（capacity/rank6 gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-inner-class.json`](snapshots/2026-10-03-inner-class.json)
（`inner_class.rs` SHA-256
`1ee32f3194b13b4fe6d714018a55595358e1be6eae544c8ffc56fd461fcfd31e`，dirty
工作区）。

## 有意为之的部分实现

模块文档明确声明这是 partial implementation：`InnerClass` 只拥有
一个 validated `BasedRootDatum`、其有限常根系 `RootSystem` 和一个
distinguished `RootInvolutionData`。它能枚举 root-theoretic
twisted-conjugacy 轨道，为 `CayleyCrossDecomposition` 提供
distinguished-involution context，并为 `RealFormLabels` 锚定 provenance；
但**尚未**构建 Atlas Cartan-class fibers、不持有 real-form data，也不含构造
KGB graph 所需的 torus data。

## 构造入口

- `inner_class_with_twisted_involution(datum, involution, root_budget)`：构建由
  `involution` 定义的 inner class，连同其相对所得 distinguished involution
  的 Weyl factor；委托 `InnerClass::from_root_involution_with_factor`。
- `InnerClass::new`：构建共享的 root-theoretic state；root enumeration 是
  deliberate caller-budgeted。成功结果证明 distinguished lattice involution
  permute 该根系统并 transport 其 coroots——但这不是 Atlas real-form
  兼容性的声明。
- `InnerClass::from_root_involution`：镜像上游 `inner_class(RootDatum,mat)`
  入口（atlas-types.w 的 `check_involution`）：接受 unbased root datum 的任意
  involution，验证其 permute 根系并 transport coroots，随后左复合由
  `wrt_distinguished` 从 reflected simple-root images 读出的 Weyl word 使之成为
  based datum 的 involution；该 Weyl word 随后被丢弃，与上游 wrapper 一致。

## 验证门

- `based_involution_twist`：移植上游 `check_based_root_datum_involution`
  （atlas-types.w:2787-2795）：involution 须 permute 本类根系并 transport
  coroots，**且**把每个 simple root 映到 simple root；拒绝为
  `StructureError::InvalidBasedAutomorphism`，成功时返回诱导的 simple-root
  permutation（上游 `rootdata::twist`）。
- `generator_twist()`：distinguished involution 对 simple generators 的置换
  （上游 `TwistedWeylGroup` 的 `weyl::Twist`）；`twist[s]` 是其 simple root
  为 $\alpha_s$ 的 distinguished image 的那个 generator。

## 成员判定：twisted_from_involution

移植上游 `twisted_from_involution`（atlas-types.w:3844-3851）：调用方已检查
square 与 involutive；本方法验证它落在**本** inner class，并返回分解
$\theta = w\cdot\delta$（$\delta$ distinguished）中的 Weyl element $w$。
weight-matrix equality 蕴含 twist 比较；拒绝由
`StructureError::InvalidBasedAutomorphism` 承载。

## 三阶段 canonicalize

`canonicalize` 移植 `InnerClass::canonicalize`（innerclass.cpp:740-832）的
三阶段 Atlas 算法：(i) 使 positive real roots 与 positive imaginary roots 的
sums 均 dominant；(ii) 限制到与两个 sums 都正交的 simple generators；(iii) 使
实际 involution 在残余 complex subsystem 中保持 positivity。返回的
generators 按执行顺序给出；对每个返回的 `s` 反复以 $s\cdot\sigma\cdot
\delta(s)$ 替换 $\sigma$，即把输入 transport 到 canonical representative。
`canonicalize_with_generators` 把算法限制到 `active` 中的 simple generators
（`Rep_context::to_singular_canonical` 以 singular generators 调用之，
repr.cpp:613-620）；phase two 的残余子系统是 `active` 与「正交于两个
dominant sums 的 generators」的交集。

## canonical_involution_expr

移植上游 `TwistedWeylGroup::canonical_involution_expr`（weyl.cpp:1359-1385）：
输出 twisted involution 的 Weyl part 的 reduced twisted-involution
expression，在 **EXTERNAL** generator numbering 下字典序最小。每步一个
signed entry：plain entry `s` 表示 cross（左乘 `s`），按位取反 entry `!s`
表示 twisted conjugation by `s`（上游将两者打包进一个 `int`；
prettyprint.cpp:219-232 以同样方式解码）。实现细节：每步取第一个 descent
（ascending generator order，上游的 external-least 选举，**不是**内部重编号），
再以 `hasTwistedCommutation`（weyl.cpp:1296-1312）区分 cross 与 twisted
conjugation。前置条件是调用方契约：`weyl` 必须是本 inner class 某 twisted
involution 的 Weyl part——循环终止性依赖于此（每步降低 twisted length）。

## twisted 共轭枚举族

- `twisted_involutions`：枚举形如 `w after distinguished` 的 root
  involutions——这是 twisted involutions 的**稳定列表**，尚不是经 twisted
  conjugacy 或 Cayley transforms 商出的 Cartan classes。
- `twisted_conjugacy_classes`：确定性的 Weyl twisted-conjugacy 轨道；
  representative **不是** Atlas-canonical；不构造 Cartan fibers、real forms
  或 Cartan partial order。
- `twisted_conjugacy_partition`：带 membership lookup 的完整 partition，是
  唯一的 orbit 实现；`twisted_conjugacy_classes` 只是其上的 thin wrapper。
- `generated_twisted_conjugacy_partition`：经 generator closure 构建完整
  partition；与 legacy 版本的差异：预算计的是 twisted involutions 而非所有
  Weyl elements，每个类只 materialize 一个 lattice involution，membership
  用 compact root permutations，避免大群构造中为每个 candidate 克隆完整
  lattice datum；外部 Cartan 编号仍由 `CartanClassification` 选举。

## 来源与限制

- 源码：[inner_class.rs](../../crates/atlas-real-group/src/inner_class.rs)；
  阅读快照 [`2026-10-03-inner-class.json`](snapshots/2026-10-03-inner-class.json)。
- 上游行号均转述自源码注释（atlas-types.w/innerclass.cpp/weyl.cpp/
  prettyprint.cpp/repr.cpp），未独立重读上游，随版本演进可能漂移。
- 关联：[Cartan 分类](cartan-classification.md)、
  [Weyl 身份与共享](weyl-context-identity-and-sharing.md)、
  [KGB 图结构](kgb-graph-structure.md)；`CayleyCrossDecomposition`、
  `RealFormLabels`、`TwistedConjugacyPartition` 的定义见
  [Cartan 分类](cartan-classification.md) 与后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  111.5s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 canonicalize 三阶段、`canonical_involution_expr` 的 signed-entry
  编码与 external-least 选举的内容均已按源码落实，其余骨架内容未采用。
  调用记录见快照的 `kimi_assist`。
