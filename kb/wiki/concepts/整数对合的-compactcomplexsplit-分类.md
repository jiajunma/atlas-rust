---
title: 整数对合的 compact/complex/split 分类
summary: 通过 θ+I 的饱和整数核及模二行空间秩计算恒等、交换对和取负因子数量，满足 compact+2·complex+split=rank，但不选取具体分解。
sources:
  - cayley-cross.md
  - twisted-involution-trio.md
kind: concept
createdAt: "2026-10-09T15:14:41.758Z"
updatedAt: "2026-10-09T20:49:38.534Z"
tags:
  - 整数线性代数
  - 对合分类
aliases:
  - 整数对合的-compactcomplexsplit-分类
  - 整C分
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 整数对合的 compact/complex/split 分类
summary: 整数对合的恒等、交换对与取负因子数量唯一确定；通过 θ+I 的饱和核及模二秩计算，无须选择或存储具体分解。
sources:
  - cayley-cross.md
  - twisted-involution-trio.md
kind: concept
tags:
  - 整数格
  - 对合分类
  - 线性代数
aliases:
  - 整数对合的-compactcomplexsplit-分类
  - 整C分
provenanceState: extracted
---

# 整数对合的 compact/complex/split 分类

整数对合的 compact/complex/split 分类记录其整分解中恒等因子、交换对因子与取负因子的数量。这三个计数唯一确定，但具体分解刻意不选取、不存储。Rust 类型 `InvolutionClassification` 实现 `Copy`，字段私有；crate 外只能通过 `classify_involution` 获得分类值。^[cayley-cross.md:63-65, twisted-involution-trio.md:41-42]

## 分类公式

设整数对合为 \(\theta\)，底层格的秩为 \(n\)。实现先计算 \(\theta+I\) 的饱和核，得到 `plus_rank`，再计算其模二像秩，按以下公式确定三个计数。^[cayley-cross.md:74-78]

\[
\begin{aligned}
p&=n-\operatorname{rank}_{\mathbf Z}\ker(\theta+I),\\
c&=\operatorname{rank}_{\mathbf F_2}\bigl((\theta+I)\bmod 2\bigr),\\
\mathrm{complex}&=c,\\
\mathrm{compact}&=p-c,\\
\mathrm{split}&=n-p-c.
\end{aligned}
\]

交换对因子占两个格维度，因此计数满足 \(\mathrm{compact}+2\,\mathrm{complex}+\mathrm{split}=n\)。这里的 `complex` 是交换对的个数。^[cayley-cross.md:63-65, cayley-cross.md:74-78]

核由 `saturated_kernel` 计算；模二归约使用 `entry % 2 != 0` 判断奇坐标，负奇数同样计入。分类计算中的三处 `checked_sub` 在下溢时返回 `IntegerLatticeInvariantViolation`。^[twisted-involution-trio.md:49-53]

## 构造流程与调用前提

公开入口 `classify_involution(matrix, budget)` 按固定顺序执行方阵形状检查、预算门控、对合性检查、构造 \(\theta+I\)，最后调用 `classify_plus_identity`。形状错误首先返回 `InvalidIntegerMatrixShape`；预算门在立方复杂度的对合检查前强制执行秩、存储与系数预算，相关主题见 [[整数格计算预算（IntegerLatticeBudget）]]。^[cayley-cross.md:67-72]

预算检查通过 `IntegerMatrix::from_i32_rows` 构造临时矩阵后立即 `drop`，避免抬高后续的 live-entry 记账。`is_involution` 使用 i128 checked 算术计算 \(M^2\)，与单位阵逐元比较，发现首个不符即短路；构造 \(\theta+I\) 时，对角元使用 `checked_add(1)`。这些检查的先后关系见 [[对合分类的资源预算与验证顺序]]。^[cayley-cross.md:67-72, twisted-involution-trio.md:44-48]

内部入口 `classify_plus_identity` 为 `pub(crate)`，供 [[中心环面的商对合分类]] 在 Smith 基坐标下复用。此入口由调用方负责保证对合前提，不能把它视为再次执行完整输入验证的公开分类入口。^[twisted-involution-trio.md:49-53]

## 奇偶性与测试锚点

来源列出六个分类测试，以 `(compact, complex, split)` 为结果顺序。秩二恒等矩阵得到 `(2,0,0)`；A2 反向矩阵 \(\begin{pmatrix}0&-1\\-1&0\end{pmatrix}\) 得到 `(0,1,0)`。以下两个矩阵展示了奇偶性对整分类的影响。^[cayley-cross.md:88-91]

\[
\begin{pmatrix}
1&1\\
0&-1
\end{pmatrix}
\longmapsto (0,1,0),
\qquad
\begin{pmatrix}
1&2\\
0&-1
\end{pmatrix}
\longmapsto (1,0,1).
\]

失败路径测试包括非对合矩阵 \(2I_2\) 返回 `InvalidInvolution`、行长不一致的矩阵首先触发形状错误，以及秩预算上限为 1 时输入 \(I_2\) 返回 `IntegerLatticeResourceLimit{resource:"rank",limit:1}`。这些测试锚定了形状检查最先执行、预算门先于对合检查的顺序。^[cayley-cross.md:88-91, twisted-involution-trio.md:60-64]

## 与 fiber rank 的区别

同一模块的 `fiber_rank` 计算 `dualPi0(−θᵀ)` 的 \(\mathbf F_2\) 维数，即 `Cartan_info` 打印的纤维大小指数。令 \(q=-\theta^{\mathsf T}\)，其公式如下，其中 `plusBasis(q)` 由 \(q-I\) 的饱和核取得，再经 `reduce_basis_mod_two` 归约。详见 [[对偶分量群的 fiber rank]]。^[cayley-cross.md:80-86]

\[
\operatorname{fiber\_rank}(\theta)
=
\dim_{\mathbf F_2}\ker\bigl((q+I)\bmod 2\bigr)
-
\dim_{\mathbf F_2}
\operatorname{span}\bigl(\mathrm{plusBasis}(q)\bmod 2\bigr).
\]

`fiber_rank` 不检查对合前提，最终差值使用 `saturating_sub` 钳零，与分类计算的 checked 下溢报错不同。其核维数使用普通减法，取负与加减 1 使用普通 i32 算术，极端输入在 debug 模式下可能触发溢出 panic。该函数没有测试覆盖，不能将分类入口的测试视为对它的验证。^[cayley-cross.md:80-86, twisted-involution-trio.md:54-64]

## 证据边界

两个来源均为源码结构性阅读材料，不构成数学或正确性验收。`classify_plus_identity` 仅通过 `classify_involution` 间接获得测试覆盖；来源中的上游位置仅转录自代码注释，本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:95-111, twisted-involution-trio.md:9-13, twisted-involution-trio.md:88-92]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md)：Cayley/Cross 分解与整对合分类。
- [twisted-involution-trio.md](../../sources/twisted-involution-trio.md)：扭对合、对合分类与环境根反射字。
