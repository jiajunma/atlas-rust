---
title: StructureError 统一错误分类学
summary: 统一错误枚举包含 53 个变体，覆盖输入验证、不变量违例、资源限制与算术错误，Display 使用固定英文文案及字段插值。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:46:15.652Z"
updatedAt: "2026-10-09T22:28:15.600Z"
tags:
  - 错误处理
  - Rust设计
aliases:
  - structureerror-统一错误分类学
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: StructureError 统一错误分类学
summary: StructureError 以 53 个变体统一表达输入验证、不变量违例、资源限制、算术错误及尚未移植的路径，并使用含字段插值的固定英文 Display 文案。
sources:
  - error-global-tits.md
kind: concept
tags:
  - 错误处理
  - Rust设计
  - 系统设计
aliases:
  - structureerror-统一错误分类学
provenanceState: extracted
---

# StructureError 统一错误分类学

`StructureError` 是 `atlas-real-group` 的统一错误类型。来源所记录的版本共有 **53 个变体**，其中 22 个无字段、31 个带字段，涵盖输入验证、结构不变量、资源限制、算术错误和尚未移植的代码路径。^[error-global-tits.md:18-56]

## 类型与依赖边界

该枚举派生 `Clone`、`Debug`、`Eq` 和 `PartialEq`，以空实现体实现 `std::error::Error`，因此 `source()` 等方法采用默认行为。`error.rs` 没有构造函数、`From` 转换、测试模块或任何 `pub fn`；各子系统在自身模块中构造错误。^[error-global-tits.md:18-21]

`error.rs` 唯一的 crate 内依赖是 `crate::lattice::Weight`。全局 Tits 传输模块经 crate 根重导出的 `crate::StructureError` 使用该类型，依赖方向为 `global_tits.rs` → `error.rs`。^[error-global-tits.md:10-14]

## 错误分类与字段约定

### 不变量违例

不变量违例家族包含 14 个变体，统一携带 `{ invariant: &'static str }`。命名前缀标识子系统：`RootSystem`、`CayleyCross`、`RealFormLabel`、`CartanClassification`、`StrongReal`、`WeylElement`、`InvolutionTable`、`TitsCoset`、`Seed`、`Kgb`、`Block`、`Rep`、`RealFormOrder` 和 `Layout`。其 `Display` 格式统一为 `<子系统> {invariant} invariant was violated`。^[error-global-tits.md:25-30]

`ModTwoSubquotientInvariantViolation` 则是无字段变体，不属于上述字段形态家族；相关结构见 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[error-global-tits.md:48-51]

### 资源预算与规模上限

7 个资源预算变体采用 `{ resource: &'static str, limit: usize }`，分别对应 `RootSystem`、`WeakRealForm`、`CayleyCross`、`StrongReal`、`InvolutionTable`、`Seed` 和 `AdjointFiber` 的 `ResourceLimit`。另有 `IntegerLatticeResourceLimit`，其 `limit` 使用 `u64`，是全枚举中的唯一例外。^[error-global-tits.md:31-34]

规模与分配失败另由 `RootSystemTooLarge`、`WeylGroupTooLarge`、`ResourceLimitExceeded { limit }` 和 `AllocationFailed { requested }` 表达。其中 `RootSystemTooLarge` 表示根系闭包超出了有限根系限制。^[error-global-tits.md:35-37]

### 输入与根数据验证

输入验证变体包括 `EmptyRootDatum`、`NonSquareCartan`、`InvalidCartanMatrix`、`RankMismatch { expected, actual }`、`DatumMismatch`、`IndexOutOfRange { index, upper_bound }` 和 `RootPairingMismatch { row, column, expected: i32, actual: i32 }`。`DatumMismatch` 表达操作需要同一带基根数据的要求；`RootPairingMismatch` 是唯一含 `i32` 字段的变体。相关概念见 [[BasedRootDatum：带基根数据与构造不变量]]。^[error-global-tits.md:38-42]

### 对合与自同构合法性

这一类包括 `InvalidInvolution`、`InvalidBasedAutomorphism`、`InvalidRootAutomorphism`、`InvalidRootDatumAutomorphism`、`SimpleRootImageNotRoot { simple_root }`、`SimpleCorootImageMismatch { simple_root, image_root: Weight }` 和 `DistinguishedInvolutionMismatch`。其中 `SimpleCorootImageMismatch` 是唯一携带 `Weight` 的变体，其 `Display` 通过 `{image_root:?}` 以 Debug 格式输出该字段。^[error-global-tits.md:43-47]

### 分级、纤维与强对合

这一类包括 `GradingShiftsNotFaithful`、`ImpossibleGrading`、`InvalidStrongTorusFactor`、`ModTwoSubquotientInvariantViolation`、`NotInModTwoSubspace`、`CartanFiberMismatch`、`CartanFiberInvolutionMismatch`、`CartanFiberMapDoesNotDescend { relation }` 和 `RealFormNotDefinedOnCartan`。相关主题包括 [[Grading shifts 的忠实性不变量]] 与 [[Ambient 映射的子商下降验证]]。^[error-global-tits.md:48-52]

### 算术与移植状态

算术相关变体包括 `InvalidIntegerMatrixShape` 和 `ArithmeticOverflow`，后者的显示文案为 `root-system arithmetic overflow`。`NotYetImplemented { feature }` 是唯一带文档注释的变体，表示上游 oracle 已定义、但本 crate 尚未移植的代码路径：遇到此类路径明确报错，避免以错误近似继续计算。^[error-global-tits.md:53-56]

## 全局 Tits 传输中的检查顺序

`global_tits.rs` 直接构造 7 个错误变体：`RankMismatch`、`IndexOutOfRange`、`InvalidRootAutomorphism`、`InvalidStrongTorusFactor`、`DatumMismatch`、`DistinguishedInvolutionMismatch` 和 `InvalidBasedAutomorphism`。`try_capacity`、`compose_matrices`、`WeylAction::*` 及 `TwistedInvolution::new` 的错误经 `?` 传播，来源没有枚举这些间接错误的具体变体。^[error-global-tits.md:112-116]

`GlobalTitsElement::new` 先检查环面因子的维数是否等于根数据的格秩，不符时返回 `RankMismatch`；随后验证上下文，最后将环面坐标逐项模 2 规范化并存储。^[error-global-tits.md:69-73]

上下文验证先检查 Weyl 作用与根对合的 datum 是否等于 `inner_class.datum()`，不符时返回 `DatumMismatch`；然后比较 \(w\delta\) 的 weight/coweight 矩阵与存储的对合矩阵，任一不等时返回 `DistinguishedInvolutionMismatch`。详见 [[全局 Tits 传输的上下文一致性校验]]。^[error-global-tits.md:101-104]

`crossed_generator` 每次重新验证上下文。生成元下标达到或超过半单秩时返回 `IndexOutOfRange`，无法取得单根类型时返回 `InvalidRootAutomorphism`。虚根分支要求根与环面因子的配对为整数，否则返回 `InvalidStrongTorusFactor`；distinguished 对合的生成元像缺失或不属于单根集合时，辅助函数返回 `InvalidBasedAutomorphism`。^[error-global-tits.md:77-85, error-global-tits.md:105-106]

`crossed_word` 先验证上下文，再按切片顺序逐个调用 `crossed_generator`。它不在入口预检整个词中的生成元，而是在执行到非法生成元时返回 `IndexOutOfRange`。这一行为与 [[Weyl 字的前向交叉作用顺序]] 相联系。^[error-global-tits.md:92-97]

## 显示与证据边界

`StructureError` 没有序列化表示或错误码；`Display` 使用含字段插值的固定英文字符串。除全局 Tits 模块直接构造的 7 个变体外，其余 46 个变体的构造点分散在其他模块，来源没有逐一枚举。^[error-global-tits.md:149-154]

`error.rs` 本身没有测试。`global_tits.rs` 的 10 个测试提供局部锚点，其中包括非整虚根配对精确触发 `InvalidStrongTorusFactor`，以及对 `RankMismatch`、`IndexOutOfRange`、`DatumMismatch` 和 `DistinguishedInvolutionMismatch` 的精确错误断言。^[error-global-tits.md:120-138]

未覆盖路径包括 `InvalidRootAutomorphism`、`InvalidBasedAutomorphism`、`rational_pair` 和 `add_scaled_coroot` 内部的 `RankMismatch`、`try_capacity` 失败，以及非空词的错误传播。辅助函数内部的秩错误在现有调用点已由同秩条件排除，详见 [[全局 Tits 传输的测试锚点与证据边界]]。^[error-global-tits.md:140-142]

来源还指出，单根表等直接下标访问依赖 `generator < semisimple_rank` 的保护；半单秩与单根表长度之间的不变量未在该包中核查，因此这里保留一个潜在 panic 面的阅读观察。^[error-global-tits.md:146-148]

本页依据结构性源码阅读，不声称错误覆盖完整或 Tits 传输已获数学验收。来源草案经过维护者对照源码逐条核对，该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[error-global-tits.md:9-14, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](../../sources/error-global-tits.md) — StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）。
