---
title: 整对合的 compact、complex、split 因子计数
summary: 通过 θ+I 的饱和整数核及模二像秩确定三类因子数而不选取具体分解；测试锚点以非对角元的奇偶性区分复因子与紧致加分裂因子，本次未执行测试。
sources:
  - twisted-involution-trio.md
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:43:56.779Z"
updatedAt: "2026-10-10T02:22:05.816Z"
tags:
  - 整数对合
  - 整数格
  - 模二线性代数
aliases:
  - 整对合的-compactcomplexsplit-因子计数
  - 整C因
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 整对合的 compact、complex、split 因子计数

`InvolutionClassification` 记录整对合分解中唯一确定的三类因子数量：`compact` 对应恒等因子，`complex` 对应交换对，`split` 对应取负因子。该类型实现 `Copy`，字段私有，只保存计数，刻意不选取或存储具体分解；crate 外只能通过 `classify_involution` 获得分类值。^[twisted-involution-trio.md:45-46, cayley-cross.md:63-65]

## 计数公式

设整对合为 $\theta$，整数格的秩为 $n$。内部函数 `classify_plus_identity` 通过 `saturated_kernel` 求出 $\theta+I$ 的饱和整数核，并计算其模二秩。记 $r_+=n-\operatorname{rank}_{\mathbb Z}\ker(\theta+I)$，则三个计数如下。^[cayley-cross.md:74-78]

$$
\begin{aligned}
\mathrm{complex}
&=\operatorname{rank}_{\mathbf F_2}\bigl((\theta+I)\bmod 2\bigr),\\
\mathrm{compact}
&=r_+-\mathrm{complex},\\
\mathrm{split}
&=n-r_+-\mathrm{complex}.
\end{aligned}
$$

模二非零坐标以 `entry % 2 != 0` 判定，负奇数同样计入。三个计数满足 $\mathrm{compact}+2\,\mathrm{complex}+\mathrm{split}=n$，其中每个交换对贡献两个维度。相关计算见 [[利用饱和核与模二秩计算整对合分类]]。^[twisted-involution-trio.md:53-57, cayley-cross.md:74-78]

计算 `compact` 和 `split` 的三处减法使用 `checked_sub`；下溢返回 `IntegerLatticeInvariantViolation`。`classify_plus_identity` 为 `pub(crate)`，可供 [[中心环面的商对合分类]] 在 Smith 基坐标下复用，但调用方必须自行保证对合前提。^[twisted-involution-trio.md:53-57]

## 构造与验证顺序

`classify_involution(matrix, budget)` 首先检查方阵形状，失败时返回 `InvalidIntegerMatrixShape`；随后通过 `IntegerMatrix::from_i32_rows(...)` 执行秩、存储和系数预算检查，并立即释放临时矩阵，避免抬高后续的存活矩阵条目记账。预算检查发生在立方复杂度的对合检验之前。^[cayley-cross.md:67-72, twisted-involution-trio.md:48-52]

通过预算检查后，`is_involution` 使用带溢出检查的 `i128` 三重循环计算矩阵平方，与单位矩阵逐元比较，遇到首个不符即短路。确认对合性后，对角元素通过 `checked_add(1)` 构造 $\theta+I$，最后调用 `classify_plus_identity`。^[cayley-cross.md:70-72, twisted-involution-trio.md:50-52]

## 示例与测试锚点

源材料记录的测试包括 $I_2\mapsto(2,0,0)$，以及 A2 反向矩阵 $\begin{pmatrix}0&-1\\-1&0\end{pmatrix}\mapsto(0,1,0)$；三元组均按 `(compact, complex, split)` 排列。^[cayley-cross.md:88-89]

[[整对合分类中的奇偶区分]]由以下两个测试体现：$\begin{pmatrix}1&1\\0&-1\end{pmatrix}\mapsto(0,1,0)$，而 $\begin{pmatrix}1&2\\0&-1\end{pmatrix}\mapsto(1,0,1)$。非对角条目的奇偶差异在这些例子中对应交换对与“恒等加取负”两种不同的整分解计数。^[cayley-cross.md:89, twisted-involution-trio.md:64-65]

负路径测试包括：非对合 $2I_2$ 返回 `InvalidInvolution`；行长度不规则的矩阵首先报告形状错误；秩预算上限为 1 时，$I_2$ 返回 `IntegerLatticeResourceLimit { resource: "rank", limit: 1 }`，锚定预算检查先于对合检验。^[cayley-cross.md:90-91, twisted-involution-trio.md:65-68]

## 与纤维秩的区别

同文件的 `fiber_rank` 计算对偶分量群 `dualPi0(−θᵀ)` 的 $\mathbf F_2$ 维数，即 `Cartan_info` 打印的纤维大小指数，详见 [[对偶分量群的 fiber rank]]。它不检查对合前提，末值使用 `saturating_sub` 钳零，而因子分类使用 `checked_sub` 报错；来源将这种防御策略差异记为阅读观察，未确认其设计原因。^[cayley-cross.md:80-86, twisted-involution-trio.md:83-87]

## 证据边界

本页依据结构性源码阅读与源材料记录的测试锚点，不构成数学或正确性验收。`classify_plus_identity` 仅经 `classify_involution` 间接覆盖，`fiber_rank` 没有测试；两份来源所述知识维护均未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:9-13, cayley-cross.md:100-101, cayley-cross.md:107-111, twisted-involution-trio.md:92-96]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md)：计数公式、构造检查、测试锚点与覆盖限制。
- [twisted-involution-trio.md](../../sources/twisted-involution-trio.md)：分类值类型、预算记账与算术错误行为。
