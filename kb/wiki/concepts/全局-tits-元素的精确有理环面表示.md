---
title: 全局 Tits 元素的精确有理环面表示
summary: GlobalTitsElement 保存完整有理余特征和扭曲对合，将含中心坐标的环面分量规范化到 [0, 2)，向纤维 mod-two 商的规约留待后续处理。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:46:16.460Z"
updatedAt: "2026-10-09T14:46:16.460Z"
tags:
  - Tits交叉作用
  - 有理环面
  - 数据表示
aliases:
  - 全局-tits-元素的精确有理环面表示
  - 全T元
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 全局 Tits 元素的精确有理环面表示

`GlobalTitsElement` 是全局 Tits 交叉作用的精确有理环面传输载体。与 `TitsCoset` 不同，它保留完整有理余特征，包括中心坐标；向纤维 mod-two 商的规约在后续阶段进行。其环面坐标统一取模 2，并保存为区间 \([0,2)\) 中的典范代表元。^[error-global-tits.md:58-65]

## 表示与构造

该类型的可见性为 `pub(crate)`，包含两个私有字段：`torus_factor: RationalCoweight` 表示有理环面因子，`twisted_involution: TwistedInvolution` 表示[[扭曲对合（TwistedInvolution）]]。^[error-global-tits.md:63-65]

构造器 `new` 首先检查环面因子的维数是否等于根数据的格秩，不等则返回 `RankMismatch`；随后验证上下文中的根数据与 distinguished involution 是否一致；最后复制环面坐标，逐坐标模 2 规范化，再构造 `RationalCoweight`。传入的 twisted involution 原样存储。^[error-global-tits.md:67-73]

对每个有理坐标 \(x\)，规范化使用
\[
x\longmapsto x-2\left\lfloor x/2\right\rfloor.
\]
测试示例为 \((-1/2,9/2)\mapsto(3/2,1/2)\)。这种坐标规范化与后续向纤维 mod-two 商的规约属于不同阶段。^[error-global-tits.md:60-65, error-global-tits.md:87-90, error-global-tits.md:122-122]

## 单生成元交叉作用

`crossed_generator` 实现 \(s*(t,w)*\delta(s)\)。每次调用都会重新验证上下文，并检查生成元编号与单根类型的有效性；方法返回新的 `Result<Self>`，不修改原值。^[error-global-tits.md:75-90]

设当前单根为 \(\alpha\)，对应余根为 \(\alpha^\vee\)。环面因子的更新按[[对合下的虚根、实根与复根分类]]分为三种情况：
\[
t'=
\begin{cases}
t-\langle\alpha,t\rangle\alpha^\vee,
& \alpha\text{ 为复根},\\[2pt]
t+(1-\langle\alpha,t\rangle)\alpha^\vee,
& \alpha\text{ 为虚根且 }\langle\alpha,t\rangle\in\mathbb Z,\\[2pt]
t,
& \alpha\text{ 为实根}.
\end{cases}
\]
配对与余根累加采用精确有理运算；虚根分支若配对不为整数，则返回 `InvalidStrongTorusFactor`。^[error-global-tits.md:79-85, error-global-tits.md:107-108]

更新后，所有环面坐标再次模 2 规范化。Weyl 分量更新为
\[
w'=s_{\mathrm{generator}}\circ w\circ
s_{\delta(\mathrm{generator})},
\]
并通过 `TwistedInvolution::new` 重建。^[error-global-tits.md:87-90]

## 上下文与词作用

[[全局 Tits 传输的上下文一致性校验]]要求 Weyl 作用和根对合均属于当前内类的根数据，否则返回 `DatumMismatch`。此外，\(w\delta\) 的权与余权矩阵必须分别等于存储的对合矩阵，否则返回 `DistinguishedInvolutionMismatch`。^[error-global-tits.md:101-104]

`crossed_word` 先验证一次上下文，再按输入切片顺序逐个调用 `crossed_generator`，与上游 `cross_act(GlobalTitsElement&, const WeylWord&)` 的顺序一致。非法生成元在执行到该位置时才报错，入口不对整个词预检；前向与逆向执行可能得到不同结果，参见[[Weyl 字的前向交叉作用顺序]]。^[error-global-tits.md:92-97]

## 测试证据与边界

测试锚定了虚根配对的整性门槛、实根交叉的不变性、复根分支的有理反射，以及 B2 中使用余根方向的约定。中心环面示例 \((0,7/3)\mapsto(1,1/3)\) 表明，中心坐标以模 2 代表元保留。^[error-global-tits.md:124-135]

这些测试与结构性阅读不构成 Tits 传输的数学验收。尚未覆盖的路径包括 `InvalidRootAutomorphism`、`InvalidBasedAutomorphism`、内部配对与余根累加的秩错误、容量申请失败，以及非空词中的错误传播；详见[[全局 Tits 传输的测试锚点与证据边界]]。^[error-global-tits.md:9-14, error-global-tits.md:140-142]

`GlobalTitsElement` 的 `Eq` 来自 `RationalCoweight` 与 `TwistedInvolution`，但该来源未展开其相等语义。本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[error-global-tits.md:150-151, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](error-global-tits.md)：StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）。
