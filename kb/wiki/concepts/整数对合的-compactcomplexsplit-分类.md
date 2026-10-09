---
title: 整数对合的 compact/complex/split 分类
summary: 整数对合的恒等、交换对及取负因子数量唯一确定；以 plus_rank=rank−dim ker(θ+I) 和 complex=rank_F₂((θ+I) mod 2) 推出 compact 与 split，无须选取或存储具体分解。
sources:
  - twisted-involution-trio.md
kind: concept
createdAt: "2026-10-09T15:14:41.758Z"
updatedAt: "2026-10-09T15:14:41.758Z"
tags:
  - 整数格
  - 对合分类
  - 线性代数
aliases:
  - 整数对合的-compactcomplexsplit-分类
  - 整C分
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 整数对合的 compact/complex/split 分类

整数对合的 compact/complex/split 分类记录其整分解中恒等因子、交换对和取负因子的三个唯一确定的秩。`InvolutionClassification` 是字段私有的 `Copy` 类型；它只保存分类计数，刻意不选择或存储具体分解。^[twisted-involution-trio.md:39-42]

## 分类公式

设整数对合为 \(\theta\)，底层格的秩为 \(n\)。实现通过 \(\theta+I\) 的饱和核与模二像计算分类：先令 \(p=n-\operatorname{rank}\ker(\theta+I)\)，再令 \(c=\operatorname{rank}_{\mathbf F_2}((\theta+I)\bmod 2)\)，得到以下三个计数。^[twisted-involution-trio.md:49-53]

\[
\begin{aligned}
\mathrm{complex}&=c,\\
\mathrm{compact}&=p-c,\\
\mathrm{split}&=n-p-c.
\end{aligned}
\]

其中核由 `saturated_kernel` 计算；模二归约以 `entry % 2 != 0` 判断奇坐标，因此负奇数也计入。三个分类计数的减法使用 `checked_sub`，下溢时返回 `IntegerLatticeInvariantViolation`。相关计算可参见 [[利用饱和核与模二秩计算整对合分类]]。^[twisted-involution-trio.md:49-53]

## 构造流程与调用前提

公开入口 `classify_involution` 按固定顺序执行：形状检查、资源预算检查、对合性检查、构造 \(\theta+I\)，最后调用 `classify_plus_identity`。预算检查通过 `IntegerMatrix::from_i32_rows` 记账，并立即释放临时矩阵，以免抬高后续的 live-entry 记账；因此预算门控先于对合性检查。^[twisted-involution-trio.md:44-48]

对合性检查使用 i128 checked 算术的三重循环，发现首个不符即短路；构造 \(\theta+I\) 时，对角元使用 `checked_add(1)`。这套顺序及其失败行为与 [[对合分类的资源预算与验证顺序]] 相关。^[twisted-involution-trio.md:44-48]

内部入口 `classify_plus_identity` 为 `pub(crate)`，供中心环面商在 Smith 基坐标下复用。该入口由调用方负责保证对合前提，相关背景见 [[中心环面的商对合分类]]。^[twisted-involution-trio.md:49-53]

## 奇偶性与测试示例

现有测试以 `(compact, complex, split)` 为计数顺序：秩二恒等对合得到 `(2,0,0)`，A2 反对合得到 `(0,1,0)`。以下两个整数矩阵则展示了奇偶性对分类的影响。^[twisted-involution-trio.md:60-61]

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

拒绝路径的测试包括非对合矩阵 \(2I\)、行长不一致的矩阵，以及秩预算上限为 1 时的恒等矩阵输入。后者返回 `IntegerLatticeResourceLimit{resource:"rank",limit:1}`；这些测试锚定了形状检查最先执行、预算检查先于对合检验的顺序。^[twisted-involution-trio.md:60-64]

## 与纤维秩的关系及证据边界

同一模块还提供 `fiber_rank`，计算 `dualPi0(−θᵀ)` 的 \(\mathbf F_2\) 维数，即 `Cartan_info` 打印的纤维大小指数。令 \(q=-\theta^\mathsf T\)，其计算为 \(\dim\ker((q+I)\bmod 2)-\dim\operatorname{span}(\mathrm{plusBasis}(q))\bmod 2\)，相关主题见 [[对偶分量群的 fiber rank]]。^[twisted-involution-trio.md:54-56]

`fiber_rank` 的防御策略与上述分类计算不同：最终差值用 `saturating_sub` 钳零，核维数使用普通减法，取负与加减 1 使用普通 i32 算术，极端输入可能在 debug 模式下触发溢出 panic。该函数没有测试覆盖，不能将分类入口的测试视为其验证。^[twisted-involution-trio.md:56-64]

本页依据源码结构性阅读材料，不代表数学验收。材料中的上游引用仅转录自代码注释；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[twisted-involution-trio.md:9-13, twisted-involution-trio.md:88-92]

## Sources

- [twisted-involution-trio.md](../sources/twisted-involution-trio.md)
