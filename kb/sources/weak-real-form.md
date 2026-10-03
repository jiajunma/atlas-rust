---
title: 弱实形式划分：adjoint fiber 的 W_im 轨道
source: atlas-rust/weak-real-form
ingestedAt: 2026-10-03T10:32:42Z
---

# 弱实形式划分：adjoint fiber 的 W_im 轨道

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `weak_real_form.rs` 的划分与归因；其正确性属于它自己的 HPC 证据链
（Cartan/seed gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-weak-real-form.json`](snapshots/2026-10-03-weak-real-form.json)
（`weak_real_form.rs` SHA-256
`fe22088fea7b955d1a4602891dda07c36b56b5a61d9cb03bae5964393ecb8e49`，dirty
工作区）。

## 定位

`WeakRealFormPartition` 把一个 adjoint Cartan fiber 划分为 $W_{im}$ 轨道；
轨道对应 inner class 在该 Cartan involution 处的弱实形式。划分拥有类表与每类
一个确定性代表元；实形标签住在 `RealFormLabels`，square-class/强实层住在
[强实形式分类](strong-real.md)。

## WeakRealFormId：编号约定

编号按各 $W_{im}$ 轨道在 canonical-coordinate 整数序下的最小元升序指派；
class 0 是 identity 元素的轨道（quasisplit normalization）。stage-(d) 排序审计
（`SEED_X0_DESIGN.md`）**证明**该编号与上游内部 `RealFormNbr` 编号一致（同样的
升序轨道播种、同样的 low-pivot RREF subquotient 基、同样的坐标提取）——这些
id 不在 adapter deferral 之内；只有解释器的外部 `FormNumberMap` 顺序需要
adapter 置换。由 fundamental Cartan 的划分铸造的 id 兼任 crate 的全局实形编号。

`MAX_MASK_BITS = 63`：masks 是 `u64`，adjoint fiber 维数必须使每个
`1 << dimension` 移位保持在范围内。

## 构建与查询

`WeakRealFormPartition::build(grading, max_elements)`：对已校验 grading 表背后
的 adjoint fiber 做划分；`max_elements` 是调用方对枚举规模 $2^{dimension}$ 的
上界。访问器：`class_count`/`classes`（升序 `ExactSizeIterator`）、
`class_of`/`class_of_mask`（canonical 坐标 mask 查询，支撑语言级
`fiber_partition`）、`class_representative`（该类在 canonical-coordinate 整数序
下的最小元）、`quasisplit_class`（class 0）、`adjoint_fiber`。

## weak_real_form_at_representative：代表元级归因内核

上游 `real_form_of` 的代表元级内核（atlas-types.w:3878-3894，
innerclass.cpp:1305-1355）：先施加 dual fixed-point projection
$v \mapsto (v + v\theta)/2$（Atlas 的行向量/右乘约定）；投影值的整数配对使
平方中心化，偶数配对标记 noncompact simple-imaginary roots，grading 由此确定
一个局部 adjoint-fiber 轨道，该 Cartan 的标签把此轨道映到 fundamental
weak-form 编号。

前置条件与门控：`twisted` 必须恰为 `classification` 存储的某个代表元（把一般
twisted involution 移到代表元还需经 table-backed Tits cross actions 一并搬运其
torus factor——本 helper 不会隐式构建或扩展该表）。因为
`CartanClassification` 不保留 `InnerClass` 句柄，本函数先把其第一个代表元
重构为归一化 distinguished involution 作为显式 provenance 门（不匹配报
`DatumMismatch`），再检查 `twisted` 的 datum 与 distinguished-involution
分解（不匹配报 `DistinguishedInvolutionMismatch`）。后续的
`minimal_torus_part` 下降还需要目前仍分离的 inverse-Cayley 操作。

## 来源与限制

- 源码：[weak_real_form.rs](../../../crates/atlas-real-group/src/weak_real_form.rs)；
  阅读快照
  [`2026-10-03-weak-real-form.json`](snapshots/2026-10-03-weak-real-form.json)。
- 上游行号均转述自源码注释（atlas-types.w/innerclass.cpp/cartanclass.cpp），
  未独立重读上游，随版本演进可能漂移。
- 关联：[强实形式分类](strong-real.md)、[Cartan 分类](cartan-classification.md)、
  [KGB 种子](real-form-seed.md)；`AdjointCartanFiber`、
  `CartanGradingData`、`RealFormLabels` 的展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  164.6s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `class_of_mask` 的 mask 语义、`weak_real_form_at_representative`
  的 provenance 门控的内容均已按源码落实，其余骨架内容未采用。调用记录见
  快照的 `kimi_assist`。
