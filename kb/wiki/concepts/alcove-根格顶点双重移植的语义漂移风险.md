---
title: Alcove 根格顶点双重移植的语义漂移风险
summary: 两份 root_vertex_simple 的转置 Cartan 构造及 label-1 重试算法对齐，但错误类型、分配防护和 bracket 失败处理不同，其中领域层版本用 unwrap_or(0) 静默置零；此观察不构成缺陷判定。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:35.559Z"
updatedAt: "2026-10-09T20:33:35.559Z"
tags:
  - alcove
  - 重复实现
  - 错误处理
aliases:
  - alcove-根格顶点双重移植的语义漂移风险
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# Alcove 根格顶点双重移植的语义漂移风险

`root_vertex_simple` 在 Rust 中存在两份移植：一份位于 `atlas-core` 的 `domain_builtins.rs:6002`，另一份位于 `atlas-real-group/alcove.rs:643`。两者指向同一上游 alcove 根格顶点函数，核心算法对齐，但错误通道、资源预算与配对失败处理存在差异；这些差异被记录为语义漂移风险。^[atlas-core-domain-seams.md:63-79]

## 共同的顶点搜索算法

算法先通过 `labels_for_component` 获取墙分量标签，将第一个标签为 1 的墙作为“最低余根”墙剔除，其余墙构成有限部分。随后调用 `inverse_cartan`，求转置子 Cartan 矩阵的精确逆。墙标签背景可参见 [[Alcove 墙标签与标签排序]]。^[atlas-core-domain-seams.md:63-66]

搜索先用 `numer·floors` 尝试未移位解；若坐标不能全部被 `denom` 整除，则依次针对其余标签为 1 的墙加一列重试。全部尝试失败时报 `no root lattice vertex found for alcove component`；成功时将系数乘以根向量并累加到结果中。^[atlas-core-domain-seams.md:66-69]

两份实现均将 `bracket(gen[j], gen[i])` 放在矩阵的 `[i][j]` 位置，因此转置构造逐元相同。标签为 1 的墙的重试逻辑也等价：`domain_builtins.rs` 版本省略 `index > chosen` 条件，是因为 `first_one` 已经选取第一个标签为 1 的墙。^[atlas-core-domain-seams.md:72-75]

## 三处漂移风险

**错误通道不同。** `domain_builtins.rs` 版本使用 `Result<_, String>`，领域层版本使用 `StructureError`。这使两份算法通过不同的错误表示向调用方报告失败；后者可结合 [[StructureError 统一错误分类学]] 阅读。^[atlas-core-domain-seams.md:75-77]

**资源预算纪律不同。** `domain_builtins.rs` 版本没有对应的预算纪律，领域层版本使用 `try_reserve_exact`。因此，核心搜索步骤一致并不表示资源分配处理也一致。^[atlas-core-domain-seams.md:75-78]

**配对失败处理不同。** `domain_builtins.rs` 版本对 `bracket` 使用 `unwrap_or(0)`，失败时静默取零；领域层版本使用 `?` 传播错误。这是两份实现对同一失败情形采取不同处理的直接例子。^[atlas-core-domain-seams.md:75-78]

## 调用范围与证据边界

`domain_builtins.rs` 版本的唯一调用方是 `"alcove_root_vertex"` 派发臂，来源标注的位置为 `13275/13315`。这一调用范围限定了该版本在所读源码中的入口。^[atlas-core-domain-seams.md:78-79]

两份实现的注释分别引用 `alcoves.cpp:345-408` 与 `alcoves.cpp:347-412`。这些行号均转述自 Rust 源码注释，未独立重读上游，不能将行号差异本身视为算法差异的证据。^[atlas-core-domain-seams.md:63-72, atlas-core-domain-seams.md:115-116]

本页依据当前工作区字节的结构性阅读，不构成语言或数学验收。两份移植的并存及其差异属于阅读观察，不是缺陷判定或修复授权；源码身份与验收证据应保持区分，参见 [[HPC 验收证据链]]。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:113-118]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词/生成元校验。
