---
title: 全局 Tits 元素的精确有理环面表示
summary: GlobalTitsElement 保存含中心坐标的完整有理余特征及扭曲对合，将环面坐标规范化至 [0,2)，纤维模二商归约留待后续处理。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:46:16.460Z"
updatedAt: "2026-10-10T00:31:19.403Z"
tags:
  - Tits群
  - 有理算术
aliases:
  - 全局-tits-元素的精确有理环面表示
  - 全T元
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 全局 Tits 元素的精确有理环面表示
summary: GlobalTitsElement 保存含中心坐标的完整有理余特征与扭曲对合，将环面坐标规范化到 [0,2)，向纤维模二商的规约留待后续处理。
sources:
  - error-global-tits.md
kind: concept
tags:
  - Tits交叉作用
  - 精确有理运算
  - 数据表示
aliases:
  - 全局-tits-元素的精确有理环面表示
  - 全T元
provenanceState: extracted
---

# 全局 Tits 元素的精确有理环面表示

`GlobalTitsElement` 是 Atlas 全局 Tits 交叉作用的精确有理环面传输载体。与 `TitsCoset` 不同，它保留完整有理余特征，包括中心坐标；向纤维模二商的规约发生在后续阶段。环面坐标保存为区间 \([0,2)\) 中的典范代表元。^[error-global-tits.md:58-65]

## 表示与构造

该类型的可见性为 `pub(crate)`，包含两个私有字段：`torus_factor: RationalCoweight` 保存环面因子，`twisted_involution: TwistedInvolution` 保存扭曲对合。^[error-global-tits.md:63-65]

构造器 `new` 按固定顺序检查和处理输入：先检查环面因子的维数是否等于根数据的格秩，不等则返回 `RankMismatch`；再验证根数据与 distinguished involution 的上下文一致性；最后复制环面坐标，逐坐标模 2 规范化，并通过 `RationalCoweight::from_coordinates` 构造存储值。传入的扭曲对合原样存储。^[error-global-tits.md:67-73]

每个有理坐标 \(x\) 按 \(x\mapsto x-2\lfloor x/2\rfloor\) 规范化。例如，测试中的 \((-1/2,9/2)\) 变为 \((3/2,1/2)\)。这种逐坐标规范化与后续向纤维模二商的规约是不同阶段，规范化后的载体仍保存完整有理余特征。^[error-global-tits.md:60-65, error-global-tits.md:87-90, error-global-tits.md:122-122]

## 单生成元交叉作用

`crossed_generator` 实现 \(s*(t,w)*\delta(s)\)。每次调用先重新验证上下文，再检查生成元编号；编号不小于半单秩时返回 `IndexOutOfRange`，无法取得对应单根的根类型时返回 `InvalidRootAutomorphism`。方法接收 `&self` 并返回 `Result<Self>`，不修改原值。^[error-global-tits.md:75-90]

设当前单根为 \(\alpha\)，对应余根为 \(\alpha^\vee\)。按照[[对合下的虚根、实根与复根分类]]，环面分量在再次规范化前按下式更新。^[error-global-tits.md:79-85]

\[
\widetilde t=
\begin{cases}
t-\langle\alpha,t\rangle\alpha^\vee,
& \alpha\text{ 为复根},\\[2pt]
t+(1-\langle\alpha,t\rangle)\alpha^\vee,
& \alpha\text{ 为虚根且 }\langle\alpha,t\rangle\in\mathbb Z,\\[2pt]
t,
& \alpha\text{ 为实根}.
\end{cases}
\]

配对与余根累加采用精确有理运算；虚根配对不为整数时返回 `InvalidStrongTorusFactor`。完成更新后，所有坐标再次模 2 规范化。^[error-global-tits.md:81-87, error-global-tits.md:107-108]

Weyl 分量更新为 \(w'=s_i\circ w\circ s_{\delta(i)}\)，随后通过 `TwistedInvolution::new` 重建。生成元的 distinguished involution 像由 `distinguished_generator_image` 取得；若像缺失或不属于单根集合，则返回 `InvalidBasedAutomorphism`。^[error-global-tits.md:87-90, error-global-tits.md:105-106]

## 上下文与词作用

[[全局 Tits 传输的上下文一致性校验]]要求 Weyl 作用与根对合所用的根数据均等于当前内类的根数据，否则返回 `DatumMismatch`。随后比较 \(w\delta\) 的权与余权矩阵是否分别等于存储的对合矩阵，任一不等即返回 `DistinguishedInvolutionMismatch`。^[error-global-tits.md:101-104]

`crossed_word` 先验证一次上下文，再按输入切片顺序逐个调用 `crossed_generator`，因此每一步仍会重新验证上下文。这一顺序匹配上游 `cross_act(GlobalTitsElement&, const WeylWord&)`；非法生成元在执行到该位置时才报错，入口不预检整个词。前向与逆向执行可能产生不同结果，参见[[Weyl 字的前向交叉作用顺序]]。^[error-global-tits.md:77-79, error-global-tits.md:92-97]

坐标复制通过 `crate::grading::try_capacity` 的预算门槛后进行。容量申请、矩阵复合、Weyl 作用及扭曲对合重建产生的错误经 `?` 传播；来源未枚举这些间接错误的具体变体。错误类型的整体设计见[[StructureError 统一错误分类学]]。^[error-global-tits.md:109-116]

## 测试证据与限制

源码中的测试锚点覆盖逐坐标规范化、零秩与空词、A1 虚根的紧与非紧因子及整性门槛、实根交叉两分量不变、A2 复根的有理反射，以及 B2 沿余根方向更新。A2 词作用测试明确断言前向与逆向结果不同。^[error-global-tits.md:120-133]

带中心环面的 A1 测试给出 \((0,7/3)\mapsto(1,1/3)\)，展示中心坐标以模 2 代表元保留。另有测试精确断言秩、生成元、根数据及 distinguished involution 不匹配时的错误。^[error-global-tits.md:134-138]

未覆盖路径包括 `InvalidRootAutomorphism`、`InvalidBasedAutomorphism`、内部配对与余根累加的秩错误、容量申请失败，以及非空词中的错误传播。直接下标访问依赖生成元小于半单秩的检查，而半单秩与单根数组长度之间的不变量未在本来源中核实；详见[[全局 Tits 传输的测试锚点与证据边界]]。^[error-global-tits.md:140-148]

`GlobalTitsElement` 的 `Eq` 派生自 `RationalCoweight` 与 `TwistedInvolution`，来源未展开其相等语义。本页依据结构性源码阅读，不构成 Tits 传输的数学验收；该次知识维护未执行 Atlas、Cargo、测试或 benchmark，上述测试描述不代表本次执行结果。^[error-global-tits.md:9-14, error-global-tits.md:150-151, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](../../sources/error-global-tits.md)：StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）。
