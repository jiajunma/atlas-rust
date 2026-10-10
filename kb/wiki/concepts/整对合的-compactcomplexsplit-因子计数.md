---
title: 整对合的 compact、complex、split 因子计数
summary: 通过 θ+I 的饱和整数核及模二行空间秩计算三类整分解因子数，满足 compact+2·complex+split=rank，但不选取具体分解。
sources:
  - cayley-cross.md
  - twisted-involution-trio.md
kind: concept
createdAt: "2026-10-09T14:43:56.779Z"
updatedAt: "2026-10-10T00:29:48.073Z"
tags:
  - 整对合
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 整对合的 compact、complex、split 因子计数
summary: 通过 θ+I 的饱和整数核与模二秩计算恒等、交换对和取负因子的数量，不选定具体分解。
sources:
  - cayley-cross.md
  - twisted-involution-trio.md
kind: concept
tags:
  - 整数线性代数
  - 对合分类
aliases:
  - 整对合的-compactcomplexsplit-因子计数
  - 整C因
provenanceState: extracted
---

# 整对合的 compact、complex、split 因子计数

`InvolutionClassification` 记录整对合分解中唯一确定的三类因子数量：`compact` 对应恒等作用，`complex` 对应交换对，`split` 对应取负作用。类型实现 `Copy`，字段私有，刻意不选取或存储具体分解；crate 外只能通过 `classify_involution` 获得分类值。^[cayley-cross.md:63-65, twisted-involution-trio.md:41-42]

## 计数公式

设整对合为 \(\theta\)，所在整数格的秩为 \(n\)。`classify_plus_identity` 通过 `saturated_kernel` 求 \(\theta+I\) 的饱和整数核，再计算该矩阵模二后的秩，使用以下公式。^[cayley-cross.md:74-78, twisted-involution-trio.md:49-53]

\[
\begin{aligned}
r_+ &= n-\operatorname{rank}_{\mathbb Z}\ker(\theta+I),\\
\mathrm{complex}
&=\operatorname{rank}_{\mathbf F_2}\bigl((\theta+I)\bmod 2\bigr),\\
\mathrm{compact} &= r_+-\mathrm{complex},\\
\mathrm{split} &= n-r_+-\mathrm{complex}.
\end{aligned}
\]

模二归约以 `entry % 2 != 0` 判断非零坐标，因此负奇数同样计入。计数满足 \(\mathrm{compact}+2\,\mathrm{complex}+\mathrm{split}=n\)：每个交换对贡献两个维度。^[cayley-cross.md:74-78, twisted-involution-trio.md:49-53]

计算中的三处减法使用 `checked_sub`；下溢会返回 `IntegerLatticeInvariantViolation`，不会将结果钳制为零。^[twisted-involution-trio.md:49-53]

## 构造与验证顺序

`classify_involution(matrix, budget)` 首先检查方阵形状，失败时返回 `InvalidIntegerMatrixShape`。随后通过 `IntegerMatrix::from_i32_rows(...)` 检查秩、存储和系数预算，并立即释放临时矩阵，避免抬高后续的 live-entry 记账。预算门在立方复杂度的对合检查之前执行，详见 [[整对合分类的预算门与检查顺序]]。^[cayley-cross.md:67-72, twisted-involution-trio.md:44-48]

通过预算门后，`is_involution` 使用带溢出检查的 `i128` 三重循环计算矩阵平方，与单位矩阵逐元比较，遇到首个不符即短路。确认对合性后，对角元素经 `checked_add(1)` 构造 \(\theta+I\)，再调用 `classify_plus_identity`。^[cayley-cross.md:67-72, twisted-involution-trio.md:44-48]

`classify_plus_identity` 的可见性为 `pub(crate)`，供 [[中心环面的商对合分类]] 在 Smith 基坐标下复用。该内部入口由调用方负责保证对合前提。^[cayley-cross.md:74-78, twisted-involution-trio.md:49-53]

## 示例与测试锚点

以下三元组均按 `(compact, complex, split)` 排列。源材料记录了恒等矩阵 \(I_2\mapsto(2,0,0)\)，以及 A2 反向矩阵 \(\begin{pmatrix}0&-1\\-1&0\end{pmatrix}\mapsto(0,1,0)\) 的测试。^[cayley-cross.md:88-89]

奇偶区分测试给出 \(\begin{pmatrix}1&1\\0&-1\end{pmatrix}\mapsto(0,1,0)\)，而 \(\begin{pmatrix}1&2\\0&-1\end{pmatrix}\mapsto(1,0,1)\)，锚定模二秩对交换对计数的影响。^[cayley-cross.md:89, twisted-involution-trio.md:60-61]

负路径测试包括：非对合 \(2I_2\) 返回 `InvalidInvolution`；不规则行长度矩阵首先报告形状错误；秩预算上限为 1 时，\(I_2\) 返回 `IntegerLatticeResourceLimit { resource: "rank", limit: 1 }`，锚定预算门先于对合检验。^[cayley-cross.md:90-91, twisted-involution-trio.md:60-64]

## 与 fiber_rank 的区别

同文件的 `fiber_rank` 计算的是对偶分量群 `dualPi0(−θᵀ)` 的 \(\mathbf F_2\) 维数，即 `Cartan_info` 打印的纤维大小指数，参见 [[对偶分量群的 fiber rank]]。其末值采用 `saturating_sub` 钳零，而因子分类使用 `checked_sub` 报错；来源将这一差异记为实现观察，未确认其设计原因。^[cayley-cross.md:80-86, twisted-involution-trio.md:54-58, twisted-involution-trio.md:79-84]

## 证据边界

本页依据结构性源码阅读及源材料记录的测试锚点，不构成数学或正确性验收。`classify_plus_identity` 仅经 `classify_involution` 间接覆盖，`fiber_rank` 没有测试；两份来源所述知识维护均未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:95-101, cayley-cross.md:107-111, twisted-involution-trio.md:88-92]

Cayley/Cross 分解与整对合分类虽收录在同一来源包中，但所读取的两个实现文件互不引用；更深调用链超出该来源包范围。^[cayley-cross.md:10-13, cayley-cross.md:102-103]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md)：整对合因子计数、检查顺序、测试锚点与覆盖限制。
- [twisted-involution-trio.md](../../sources/twisted-involution-trio.md)：分类值类型、预算记账、算术错误行为与纤维秩区别。
