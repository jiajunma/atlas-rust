---
title: Twisted involution 记录的数学不变量
summary: 记录构造检查种子长度公式的奇偶性、BFS 长度步进、(2ρ+θ·2ρ)/2 的整性及传送投影与新 θ 的相容性；这些检查不构成本包的数学验收。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:27.697Z"
updatedAt: "2026-10-10T00:36:17.863Z"
tags:
  - 对合
  - 构造不变量
  - 错误处理
aliases:
  - twisted-involution-记录的数学不变量
  - TI记
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Twisted involution 记录的数学不变量
summary: 记录构造检查长度公式的奇偶性、BFS 长度步进、ρ 表达式的整性及传送投影与 θ 的相容性；图像基保留路径依赖。
sources:
  - involution-table.md
kind: concept
tags:
  - 扭曲对合
  - 构造校验
aliases:
  - twisted-involution-记录的数学不变量
  - TI记
provenanceState: extracted
---

# Twisted involution 记录的数学不变量

Twisted involution 记录是 KGB 构建流水线 stage b 的条目，按 Cartan 类轨道连续存储。每条记录包含词级 Weyl 元素、矩阵级 `TwistedInvolution`（对合 θ 与根分类）、模二去重子空间、$(1+\theta)\rho$、对合长度与 Weyl 长度，以及 $(1-\theta)X^*$ 的图像基对 `lift_mat`、`M_real`。存储组织见 [[Twisted involution 表与 Cartan 轨道存储]]。^[involution-table.md:20-24, involution-table.md:33-35]

## θ 与记录字段的一致性

除图像基对外，记录字段均在入表时由 θ 典范导出。B2 测试逐条核对 θ 的根作用为 `weyl.image(δ.image(root))`，即先施加 distinguished involution δ，再施加记录中的 Weyl 元素。^[involution-table.md:26-29, involution-table.md:81-83]

`theta_plus_one_rho` 按 $(1+\theta)\rho=(2\rho+\theta(2\rho))/2$ 计算，其中 $2\rho$ 在建表时从 positivity slice 导出。若分子出现奇坐标，`push_record` 报 `"theta rho parity"`，以检查逐坐标除以 2 的整性。^[involution-table.md:39-42, involution-table.md:57-59]

## 长度公式与 BFS 步进

Cartan 轨道种子先经 `WeylElement::from_action` 从矩阵级代表元转换为词级元素，再按 $\ell_{\mathrm{inv}}=(\ell_W+\#\mathrm{Cayley})/2$ 计算对合长度。分子为奇数时报 `"length parity"`。`CayleyCrossDecomposition` 按 Cartan 类构造，不会为每条记录重建；B2 测试逐条核对该长度公式。^[involution-table.md:44-49, involution-table.md:81-83]

[[Twisted cross-action 的 BFS 轨道构建|外序 BFS]] 以 $s_g\,w\,s_{\mathrm{twist}(g)}$ 生成邻居，并以前向根置换去重。对新邻居，`stepped_length` 要求 Weyl 长度差恰为 $\pm2$，按来源记载的差值约定将对合长度相应调整为 $\mp1$；否则报 `"twisted length step"`。长度差为 0 表示该边固定对合，这种情况已由先行去重处理。^[involution-table.md:51-54]

## 图像基的路径依赖与相容性

图像基对是 θ 典范导出规则的例外。在轨道典范 involution 处，它由 $1-\theta$ 的阶梯形约化播种；随后沿 cross-action BFS 传送。该基具有路径依赖性，且 `y_lift` 的符号依赖于它，相关纪律见 [[图像基的典范播种与轨道传送纪律]]。^[involution-table.md:26-29]

种子投影由 `RealProjection::build` 构造。沿边传送使用普通生成元 $s$ 的矩阵，而非 $\mathrm{twist}(s)$，因为 δ 已并入 θ。传送投影在被新记录采用前，须通过 `check_against` 与该记录新鲜推导的 θ 核对。^[involution-table.md:55-59]

B2 投影传送测试记录了以下差异：^[involution-table.md:84-86]

\[
\theta=
\begin{pmatrix}
-1&0\\
2&1
\end{pmatrix},
\qquad
\mathrm{lift\_mat}_{\text{传送}}=
\begin{pmatrix}
2\\-2
\end{pmatrix},
\qquad
\mathrm{lift\_mat}_{\text{现场重算}}=
\begin{pmatrix}
-2\\2
\end{pmatrix}.
\]

来源将此案例标为 arm64 oracle 锚点；测试用 `assert_ne` 固定传送所得 `lift_mat` 与现场重算结果的差异。^[involution-table.md:84-86]

因此，与 θ 相容不意味着必须等于现场重算的图像基；以重新构造替换路径传送，可能改变基的符号。

## 轨道完整性与调用方契约

`add_cartan` 从 classification 取得种子和期望轨道大小，要求生成轨道恰好填满该大小，否则报 `InvolutionTableInvariantViolation { invariant: "orbit size" }`。重复添加同一 Cartan 类返回已有切片，详见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:44-47]

不同 Cartan 轨道的置换键互不重叠属于调用方纪律。种子插入 `index_by_permutation` 时没有碰撞检查，`BTreeMap::insert` 遇到同键会静默覆盖，因此这一条件并未在种子入表时显式验证。^[involution-table.md:60-61]

## 测试与证据边界

A1 测试逐字段锚定 fundamental 记录的 `theta_plus_one_rho = [1]`、`mod_space` 秩 0，以及分裂记录的 `[0]`、秩 1。B2 测试核对全表与分类的一致性、两次独立建表的逐切片可复现性，并检查逐记录 θ、长度公式、ρ 公式及投影传送差异。^[involution-table.md:79-86]

来源列出的未触及分支包括四个不变量错误字面量对应的错误路径、`AllocationFailed`，以及 `lookup` 返回 `Some` 的直接命中断言。相关覆盖范围见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:87-88]

本页依据源码结构性阅读。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；上述测试描述是源码中的测试锚点，而非本次执行结果。上游 C++ 行号转述自源码注释，未独立重读上游，可能随版本变化。^[involution-table.md:9-16, involution-table.md:98-103]

## Sources

- [involution-table.md](../../sources/involution-table.md) — Twisted involution 表（KGB stage b）。
