---
title: Twisted involution 记录的数学不变量
summary: 记录构建检查种子长度 (W_length+#Cayley)/2 的奇偶性、BFS 长度步进、(2ρ+θ·2ρ)/2 的整性，以及传送投影与当前 θ 的相容性。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:27.697Z"
updatedAt: "2026-10-09T14:52:27.697Z"
tags:
  - 数学不变量
  - 长度函数
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Twisted involution 记录的数学不变量

Twisted involution 记录是 KGB 构建流水线 stage b 的基本条目，按 Cartan 类轨道连续存储。每条记录包含词级 Weyl 元素、矩阵级 `TwistedInvolution`（对合 θ 及根分类）、模二去重子空间、$(1+\theta)\rho$、对合长度与 Weyl 长度，以及 $(1-\theta)X^*$ 的图像基对 `lift_mat`、`M_real`。^[involution-table.md:20-24]

## θ 与记录字段的一致性

除图像基对外，记录字段均在入表时由 θ 典范导出。B2 测试锚点逐条检查 θ 的根作用满足 `weyl.image(δ.image(root))`，即词级 Weyl 元素与 distinguished involution δ 合成后给出记录中的 θ。相关背景见 [[扭曲对合（TwistedInvolution）]]。^[involution-table.md:26-29, involution-table.md:79-83]

记录中的 `theta_plus_one_rho` 按下式计算：

\[
(1+\theta)\rho=\frac{2\rho+\theta(2\rho)}{2}.
\]

其中 $2\rho$ 在建表时从 positivity slice 导出；若分子存在奇坐标，入表过程报不变量错误 `"theta rho parity"`，确保除以 2 后仍得到整坐标。^[involution-table.md:39-42, involution-table.md:57-59]

## 长度公式与跨边约束

Cartan 轨道种子使用下式计算对合长度：

\[
\ell_{\mathrm{inv}}
=\frac{\ell_W+\#\mathrm{Cayley}}{2}.
\]

若分子为奇数，则报 `"length parity"`。`CayleyCrossDecomposition` 是每个 Cartan 类使用的工具，不会为每条记录重新构造；B2 测试还对每条记录核对这一长度公式。^[involution-table.md:44-49, involution-table.md:81-83]

外序 BFS 通过 $s_g\,w\,s_{\mathrm{twist}(g)}$ 生成邻居，并以前向根置换去重。对新记录，`stepped_length` 要求 Weyl 长度差恰为 $\pm2$，并相应将对合长度调整为 $\mp1$；否则报 `"twisted length step"`。长度差为 0 的边固定该对合，会先被去重处理。参见 [[Twisted cross-action 的 BFS 轨道构建]]。^[involution-table.md:51-54]

## 图像基的路径依赖与校验

$(1-\theta)X^*$ 的图像基对是“由 θ 典范导出”规则的例外。它在轨道典范 involution 处通过 $1-\theta$ 的阶梯形约化播种，随后沿 cross-action BFS 传送。该基具有路径依赖性，且 `y_lift` 的符号依赖于它，因此记录必须保留传送所得的基。参见 [[图像基的典范播种与轨道传送纪律]]。^[involution-table.md:26-29]

投影传送使用普通生成元 $s$ 的矩阵，而非 $\mathrm{twist}(s)$，因为 δ 已并入 θ。种子使用 `RealProjection::build`；后继记录采用传送投影前，先以 `check_against` 对照本记录新鲜推导出的 θ，检查传送结果与新对合的一致性。^[involution-table.md:55-59]

B2 投影传送测试给出了符号差异的具体锚点：对于下列 θ，记录保存的 `lift_mat` 与现场重算结果互为相反数，测试使用 `assert_ne` 固定这一差异。^[involution-table.md:84-86]

\[
\theta=
\begin{pmatrix}
-1&0\\
2&1
\end{pmatrix},
\qquad
\mathrm{lift\_mat}_{\mathrm{传送}}=
\begin{pmatrix}
2\\-2
\end{pmatrix},
\qquad
\mathrm{lift\_mat}_{\mathrm{重算}}=
\begin{pmatrix}
-2\\2
\end{pmatrix}.
\]

## 轨道完整性与调用方契约

记录所属轨道的种子及期望大小来自 Cartan classification。`add_cartan` 要求生成轨道恰好填满期望大小，否则报 `InvolutionTableInvariantViolation { invariant: "orbit size" }`；重复添加同一 Cartan 类则返回已有切片。^[involution-table.md:44-47]

不同 Cartan 轨道的前向根置换键不得重叠，但种子插入 `index_by_permutation` 时没有碰撞检查，`BTreeMap::insert` 会静默覆盖同键值。因此，这一跨轨道条件依赖调用方纪律，并非入表时显式验证的不变量。^[involution-table.md:60-61]

## 测试与证据边界

A1 测试逐字段锚定了 fundamental 记录的 `theta_plus_one_rho = [1]`、`mod_space` 秩 0，以及分裂记录的 `[0]`、秩 1；B2 测试覆盖逐记录 θ、长度公式、ρ 公式与投影传送差异。但四个不变量错误字面量对应的分支尚未触及，另有 `AllocationFailed` 与 `lookup` 的 `Some` 命中直接断言缺口。参见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:79-88]

上述内容来自源码结构性阅读及测试锚点整理；来源包未执行构建、测试或原版运行，不构成新的数学验收或性能结论。上游 C++ 行号仅转述自源码注释，未独立重读核实。^[involution-table.md:9-16, involution-table.md:98-103]

## Sources

- [involution-table.md](involution-table.md) — Twisted involution 表（KGB stage b）
