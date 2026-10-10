---
title: Alcove 根格顶点双重移植的语义漂移风险
summary: 两份 root_vertex_simple 的核心算法对齐，但错误通道、分配防护及 bracket 失败处理存在差异，尚不能据此判定缺陷。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:35.559Z"
updatedAt: "2026-10-10T01:44:23.053Z"
tags:
  - Alcove
  - 移植一致性
  - 错误处理
aliases:
  - alcove-根格顶点双重移植的语义漂移风险
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Alcove 根格顶点双重移植的语义漂移风险

`root_vertex_simple` 有两份对应同一上游函数的 Rust 移植，分别位于 `atlas-core` 的 `domain_builtins.rs:6002` 和 `atlas-real-group/alcove.rs:643`。两份实现的核心算法对齐，但错误通道、预算纪律和 `bracket` 失败处理不同。来源将这些差异记录为语义漂移风险，并未据此判定缺陷。^[atlas-core-domain-seams.md:63-79, atlas-core-domain-seams.md:115-118]

## 共同算法与对齐依据

算法通过 `labels_for_component` 获取墙分量标签，将第一个标签为 1 的墙作为“最低余根”墙剔除，由其余墙生成有限部分，再用 `inverse_cartan` 求转置子 Cartan 矩阵的精确逆。相关概念见 [[Alcove 墙标签与标签排序]]与 [[Alcove 算法中的精确有理线性代数]]。^[atlas-core-domain-seams.md:63-66]

搜索先以 `numer·floors` 尝试未移位解；若坐标不能全部被 `denom` 整除，则依次针对其余标签为 1 的墙加一列重试。全部失败时，`domain_builtins.rs` 版本报 `no root lattice vertex found for alcove component`；成功时，将系数乘以根向量并累加到结果。^[atlas-core-domain-seams.md:66-69]

两份实现都把 `bracket(gen[j], gen[i])` 放在矩阵的 `[i][j]` 位置，转置构造逐元相同。标签为 1 的墙的重试逻辑也等价：`domain_builtins.rs` 版本省略 `index > chosen` 条件，是因为 `first_one` 已经是第一个标签为 1 的墙。^[atlas-core-domain-seams.md:72-75]

## 三处语义漂移风险

**错误通道不同。** `domain_builtins.rs` 版本使用 `Result<_, String>`，`atlas-real-group/alcove.rs` 版本使用 `StructureError`。相关错误体系见 [[StructureError 统一错误分类学]]。^[atlas-core-domain-seams.md:75-77]

**预算纪律不同。** 来源将 `domain_builtins.rs` 版本记为无对应预算纪律，而 `atlas-real-group/alcove.rs` 版本使用 `try_reserve_exact`。核心数学步骤的对齐并未消除这一分配处理差异。^[atlas-core-domain-seams.md:75-78]

**配对失败处理不同。** `domain_builtins.rs` 版本通过 `unwrap_or(0)` 将 `bracket` 失败静默转为零；`atlas-real-group/alcove.rs` 版本通过 `?` 传播错误。这是两份实现对失败情况采取的不同处理语义。^[atlas-core-domain-seams.md:75-78]

## 调用范围与证据边界

`domain_builtins.rs` 版本的唯一调用方是 `"alcove_root_vertex"` 派发臂，来源标注其位置为 `13275/13315`。该调用关系限定于本次结构性阅读所覆盖的源码。^[atlas-core-domain-seams.md:78-79]

两份实现的注释分别引用 `alcoves.cpp:345-408` 和 `alcoves.cpp:347-412`。这些位置均转述自 Rust 源码注释，来源未独立重读上游，行号可能随版本漂移；因此，所记录的算法对齐不等同于独立核对上游后的兼容性验收。^[atlas-core-domain-seams.md:63-75, atlas-core-domain-seams.md:115-116]

材料属于对当前工作区字节的结构性阅读，不声称语言或数学验收。两份移植的并存及差异是阅读观察，不构成缺陷判定或修复授权。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:115-118]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词/生成元校验。
