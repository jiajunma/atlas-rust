---
title: Twisted involution 记录的数学不变量
summary: 记录构造检查种子长度公式的奇偶性、BFS 长度步进、(2ρ+θ·2ρ)/2 的整性以及传送投影与新 θ 的相容性。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:27.697Z"
updatedAt: "2026-10-09T19:31:10.361Z"
tags:
  - 扭曲对合
  - 构造不变量
  - 精确算术
aliases:
  - twisted-involution-记录的数学不变量
  - TI记
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Twisted involution 记录的数学不变量

Twisted involution 记录是 KGB 构建流水线 stage b 的基本条目，按 Cartan 类轨道连续存储。每条记录包含词级 Weyl 元素、矩阵级 `TwistedInvolution`（θ 及根分类）、模二去重子空间、$(1+\theta)\rho$、对合长度与 Weyl 长度，以及 $(1-\theta)X^*$ 的图像基对 `lift_mat`、`M_real`。相关存储结构见 [[Twisted involution 表与 Cartan 轨道存储]]。^[involution-table.md:20-24]

## θ 与记录字段的一致性

除图像基对外，所有记录字段均在入表时由 θ 典范导出。B2 测试锚点逐条检查 θ 的根作用满足 `weyl.image(δ.image(root))`，即词级 Weyl 元素与 distinguished involution δ 的合成给出记录中的 θ。^[involution-table.md:26-29, involution-table.md:81-83]

`theta_plus_one_rho` 使用公式 $(1+\theta)\rho=(2\rho+\theta(2\rho))/2$ 计算，其中 $2\rho$ 在建表时从 positivity slice 导出。若分子存在奇坐标，入表过程报 `"theta rho parity"`，从而检查结果的整性。^[involution-table.md:39-42, involution-table.md:57-59]

## 长度公式与 BFS 步进

Cartan 轨道种子的对合长度按 $\ell_{\mathrm{inv}}=(\ell_W+\#\mathrm{Cayley})/2$ 计算；若分子为奇数，则报 `"length parity"`。`CayleyCrossDecomposition` 按 Cartan 类使用，不会为每条记录重新构造；B2 测试还逐条核对这一长度公式。^[involution-table.md:44-49, involution-table.md:81-83]

[[Twisted cross-action 的 BFS 轨道构建|外序 BFS]] 通过 $s_g\,w\,s_{\mathrm{twist}(g)}$ 生成邻居，并以前向根置换去重。对新记录，`stepped_length` 要求 Weyl 长度差恰为 $\pm2$，并相应将对合长度调整为 $\mp1$，否则报 `"twisted length step"`。长度差为 0 表示该边固定对合，已先由去重处理。^[involution-table.md:51-54]

## 图像基的播种、传送与校验

图像基对是由 θ 典范导出规则的例外：它在轨道典范 involution 处通过 $1-\theta$ 的阶梯形约化播种，随后沿 cross-action BFS 传送。该基具有路径依赖性，且 `y_lift` 的符号依赖于它。参见 [[图像基的典范播种与轨道传送纪律]]。^[involution-table.md:26-29]

投影传送使用普通生成元 $s$ 的矩阵，而非 $\mathrm{twist}(s)$，因为 δ 已并入 θ。种子处调用 `RealProjection::build`；后继记录采用传送投影前，先通过 `check_against` 对照当前记录新鲜推导出的 θ，检查传送结果与新对合的相容性。^[involution-table.md:55-59]

B2 投影传送测试给出了路径依赖的具体锚点，涉及以下 θ 与两种图像基结果：^[involution-table.md:84-86]

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

记录保存的基与现场重算结果互为相反数，测试使用 `assert_ne` 固定这一差异。因此，传送投影与 θ 的相容性检查不能被理解为要求它等于现场重算的基。^[involution-table.md:55-59, involution-table.md:84-86]

## 轨道完整性与调用方契约

`add_cartan` 从 classification 取得种子与期望轨道大小，并要求生成轨道恰好填满该大小，否则报 `InvolutionTableInvariantViolation { invariant: "orbit size" }`。重复添加同一 Cartan 类返回已有切片，体现 [[Cartan 轨道的幂等添加与容量约束|轨道添加的幂等性]]。^[involution-table.md:44-47]

跨 Cartan 轨道的键不重叠依赖调用方纪律：种子插入 `index_by_permutation` 时没有碰撞检查，`BTreeMap::insert` 遇到同键会静默覆盖。这一条件并非入表时显式验证的不变量。^[involution-table.md:60-61]

## 测试与证据边界

A1 测试逐字段锚定 fundamental 记录的 `theta_plus_one_rho = [1]`、`mod_space` 秩 0，以及分裂记录的 `[0]`、秩 1；B2 测试覆盖逐记录 θ、长度公式、ρ 公式与投影传送差异。四个不变量错误字面量对应的分支尚未触及，另有 `AllocationFailed` 与 `lookup` 的 `Some` 命中直接断言缺口。参见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:79-88]

来源属于源码结构性阅读，未执行构建、测试或原版运行，不提供新的数学验收、性能或并行结论。上游 C++ 行号转述自源码注释，未独立重读核实，可能随版本变化。^[involution-table.md:9-16, involution-table.md:98-103]

## Sources

- [involution-table.md](involution-table.md) — Twisted involution 表（KGB stage b）
