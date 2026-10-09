---
title: StructureError 统一错误分类学
summary: atlas-real-group 以 53 个 StructureError 变体统一表达输入验证、不变量违例、资源限制及算术错误，并通过固定英文 Display 文案呈现错误上下文。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:46:15.652Z"
updatedAt: "2026-10-09T14:46:15.652Z"
tags:
  - Rust
  - 错误处理
  - 系统设计
aliases:
  - structureerror-统一错误分类学
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# StructureError 统一错误分类学

`StructureError` 是 `atlas-real-group` 的统一错误类型，用于表达输入验证失败、结构不变量违例、资源限制、算术错误及尚未移植的代码路径。所提供的源码阅读记录统计了 **53 个变体**，其中 22 个无字段、31 个带字段。^[error-global-tits.md:18-56]

## 类型与依赖边界

该枚举派生 `Clone`、`Debug`、`Eq` 和 `PartialEq`，并以空实现体实现 `std::error::Error`，因此 `source()` 等方法采用默认行为。`error.rs` 内没有构造函数、`From` 转换、测试模块或任何 `pub fn`；错误的实际构造点分散在各子系统中。^[error-global-tits.md:18-21]

`error.rs` 唯一的 crate 内依赖是 `crate::lattice::Weight`。全局 Tits 传输模块通过 crate 根重导出的 `crate::StructureError` 使用该类型，依赖方向为 `global_tits.rs` → `error.rs`。^[error-global-tits.md:10-14]

## 分类与字段约定

### 不变量违例

不变量违例家族共有 14 个变体，统一携带 `{ invariant: &'static str }`。命名前缀标识所属子系统：`RootSystem`、`CayleyCross`、`RealFormLabel`、`CartanClassification`、`StrongReal`、`WeylElement`、`InvolutionTable`、`TitsCoset`、`Seed`、`Kgb`、`Block`、`Rep`、`RealFormOrder` 和 `Layout`。其 `Display` 格式统一为 `<子系统> {invariant} invariant was violated`。^[error-global-tits.md:25-30]

`ModTwoSubquotientInvariantViolation` 虽然也表示不变量违例，却是无字段变体，不属于上述字段形态家族；相关结构见 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[error-global-tits.md:48-51]

### 资源预算与规模上限

资源预算家族中有 7 个变体使用 `{ resource: &'static str, limit: usize }`，分别对应 `RootSystem`、`WeakRealForm`、`CayleyCross`、`StrongReal`、`InvolutionTable`、`Seed` 和 `AdjointFiber` 的 `ResourceLimit`。`IntegerLatticeResourceLimit` 是类型例外：其 `limit` 使用 `u64`。^[error-global-tits.md:31-34]

规模与分配失败另由 `RootSystemTooLarge`、`WeylGroupTooLarge`、`ResourceLimitExceeded { limit }` 和 `AllocationFailed { requested }` 表达。其中 `RootSystemTooLarge` 表示根系闭包超出了有限根系限制。^[error-global-tits.md:35-37]

### 输入与根数据验证

输入验证变体包括 `EmptyRootDatum`、`NonSquareCartan`、`InvalidCartanMatrix`、`RankMismatch { expected, actual }`、`DatumMismatch`、`IndexOutOfRange { index, upper_bound }` 和 `RootPairingMismatch { row, column, expected: i32, actual: i32 }`。`DatumMismatch` 表达操作必须使用同一带基根数据的要求；`RootPairingMismatch` 是唯一含 `i32` 字段的变体。相关构造约束见 [[BasedRootDatum：带基根数据与构造不变量]]。^[error-global-tits.md:38-42]

### 对合与自同构合法性

这一类包括 `InvalidInvolution`、`InvalidBasedAutomorphism`、`InvalidRootAutomorphism`、`InvalidRootDatumAutomorphism`、`SimpleRootImageNotRoot { simple_root }`、`SimpleCorootImageMismatch { simple_root, image_root: Weight }` 和 `DistinguishedInvolutionMismatch`。其中 `SimpleCorootImageMismatch` 是唯一携带 `Weight` 的变体，`Display` 使用 `{image_root:?}` 输出该字段。^[error-global-tits.md:43-47]

### 分级、纤维与强对合

这一类覆盖 `GradingShiftsNotFaithful`、`ImpossibleGrading`、`InvalidStrongTorusFactor`、`ModTwoSubquotientInvariantViolation`、`NotInModTwoSubspace`、`CartanFiberMismatch`、`CartanFiberInvolutionMismatch`、`CartanFiberMapDoesNotDescend { relation }` 和 `RealFormNotDefinedOnCartan`，涉及分级、子商成员资格、纤维一致性及映射下降等边界。相关概念包括 [[Grading shifts 的忠实性不变量]] 和 [[Cartan fiber 的有限域子商模型]]。^[error-global-tits.md:48-52]

### 算术与移植状态

算术相关变体包括 `InvalidIntegerMatrixShape` 和 `ArithmeticOverflow`，后者的显示文案为 `root-system arithmetic overflow`。`NotYetImplemented { feature }` 是唯一带文档注释的变体，表示上游 oracle 已定义、但本 crate 尚未移植的路径：遇到此类路径应明确报错，避免使用错误近似继续计算。^[error-global-tits.md:53-56]

## 全局 Tits 传输中的使用

`global_tits.rs` 直接构造 7 个变体：`RankMismatch`、`IndexOutOfRange`、`InvalidRootAutomorphism`、`InvalidStrongTorusFactor`、`DatumMismatch`、`DistinguishedInvolutionMismatch` 和 `InvalidBasedAutomorphism`。来自 `try_capacity`、`compose_matrices`、`WeylAction::*` 及 `TwistedInvolution::new` 的错误通过 `?` 传播；这些间接错误的具体变体不在该来源包的枚举范围内。^[error-global-tits.md:112-116]

错误检查具有明确顺序：`GlobalTitsElement::new` 先检查环面因子的秩，再验证上下文，最后规范化坐标。上下文验证先检查 datum 一致性，再比较 `w·δ` 的 weight/coweight 矩阵与存储的对合矩阵，分别返回 `DatumMismatch` 或 `DistinguishedInvolutionMismatch`；详见 [[全局 Tits 传输的上下文一致性校验]]。^[error-global-tits.md:69-73, error-global-tits.md:101-104]

单生成元交叉作用重新验证上下文，并检查生成元范围及根类型；虚根分支要求根与环面因子的配对为整数，否则返回 `InvalidStrongTorusFactor`。`crossed_word` 按切片顺序执行，不在入口预检全部生成元，因此非法生成元在折叠执行到该位置时才返回 `IndexOutOfRange`。^[error-global-tits.md:75-97]

## 测试与证据边界

`error.rs` 本身没有测试。`global_tits.rs` 的 10 个测试提供了局部锚点，包括虚根非整配对触发 `InvalidStrongTorusFactor`，以及对 `RankMismatch`、`IndexOutOfRange`、`DatumMismatch` 和 `DistinguishedInvolutionMismatch` 的精确错误断言。它们不代表对全部 53 个变体的覆盖。^[error-global-tits.md:120-138, error-global-tits.md:154-154]

未覆盖的路径包括 `InvalidRootAutomorphism`、`InvalidBasedAutomorphism`、有理配对及余根累加辅助函数内部的 `RankMismatch`、`try_capacity` 失败，以及非空词中的错误传播。该源码阅读记录仅提供结构性证据，不声称错误覆盖完整或 Tits 传输已获数学验收；本次知识维护未运行 Atlas、Cargo、测试或 benchmark。^[error-global-tits.md:13-14, error-global-tits.md:140-142, error-global-tits.md:158-162]

该错误类型没有序列化表示或错误码；`Display` 使用带字段插值的固定英文字符串。除全局 Tits 模块直接构造的 7 个变体外，其余 46 个变体的构造点需要结合各自模块继续检查。^[error-global-tits.md:149-154]

## Sources

- [error-global-tits.md](error-global-tits.md) — StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）
