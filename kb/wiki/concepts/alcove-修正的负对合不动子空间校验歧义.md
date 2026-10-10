---
title: Alcove 修正的负对合不动子空间校验歧义
summary: 来源将重心修正检查称为 −θ 不动子空间校验，却描述为验证 θ·(centered_gamma−gamma)=0；该等式与不动条件 θ·差值=−差值不同，需核对源码与数学意图。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-10T00:13:31.175Z"
updatedAt: "2026-10-10T00:13:31.175Z"
tags:
  - alcove
  - 对合
  - 证据歧义
aliases:
  - alcove-修正的负对合不动子空间校验歧义
confidence: 0.99
provenanceState: ambiguous
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

# Alcove 修正的负对合不动子空间校验歧义

`alcove_center` 在求出新的 infinitesimal character 后，执行名为“−θ 不动子空间校验”的检查。但来源记录的实际判定是 \(\theta(\Delta)=0\)，其中 \(\Delta=\mathrm{centered\_gamma}-\gamma\)。检查名称与所述矩阵条件之间存在需要核对的语义歧义。^[alcove.md:84-88]

## 校验在算法中的位置

[[Alcove 重心计算与标准参数重建]]先通过墙方程与 radical basis 约束求解重心，再通分得到 `centered_gamma`。随后调用 `rc.theta(z)?` 获取 `theta`，逐行检查 `theta.weight_matrix()` 与修正量的乘积是否为零；失败时返回 `RepInvariantViolation`，错误文本为 `"alcove correction lies outside the -theta fixed subspace"`。校验通过后，才调用 `rc.sr_gamma(z.x(), &lambda_rho, &centered_gamma)` 重建标准参数，保留 KGB 坐标与 `lambda_rho`。^[alcove.md:65-89]

## 名称与判定条件的差异

按“−θ 的不动子空间”的通常数学含义，修正量应满足
\[
(-\theta)\Delta=\Delta
\quad\Longleftrightarrow\quad
(\theta+I)\Delta=0.
\]
这与 \(\theta\Delta=0\) 是不同的条件。如果 `weight_matrix()` 表示的确实是对合 \(\theta\) 本身，则 \(\theta^2=I\) 蕴含 \(\theta\) 可逆，因而后一个条件只允许 \(\Delta=0\)。这是依据术语和来源所述检查作出的数学推论，不是已经核实的实现结论。

来源没有覆盖 `RepContext` 等外部类型的契约，也没有独立核对上游文件字节。因此，仅凭本包不能确认 `weight_matrix()` 的完整语义，也不能确定歧义来自实现、注释还是结构性阅读的转述。^[alcove.md:9-13, alcove.md:185-190]

## 核对重点与证据边界

消解这一歧义需要核对 `rc.theta(z)`、`weight_matrix()` 的实际契约，以及检查中矩阵和修正量的构造方式；若矩阵确为 \(\theta\)，还需确认预期条件是否应为 \((\theta+I)\Delta=0\)。这些属于后续核对方向，不能据此宣称代码已被证明错误或已经修复。

来源将这一步关联到 `alcoves.cpp:317-321`，但该上游位置仅来自代码注释。材料属于结构性阅读，不构成 alcove 计算的数学验收；`alcove_center` 端到端没有单元测试，本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。相关覆盖限制见 [[Alcove 算法的测试覆盖与失败边界]]。^[alcove.md:84-88, alcove.md:171-176, alcove.md:185-190]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。
