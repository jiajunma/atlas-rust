---
title: Alcove 根格顶点双重移植的语义漂移风险
summary: 两份 root_vertex_simple 的转置 Cartan 构造与 label-1 重试算法对齐，但错误通道、分配防护和 bracket 失败处理不同；差异尚不构成缺陷判定。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:35.559Z"
updatedAt: "2026-10-09T22:15:51.287Z"
tags:
  - alcove
  - Rust移植
  - 错误处理
aliases:
  - alcove-根格顶点双重移植的语义漂移风险
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Alcove 根格顶点双重移植的语义漂移风险
summary: 两份 root_vertex_simple 的转置 Cartan 构造与 label-1 重试算法对齐，但错误通道、分配防护和 bracket 失败处理不同；这些差异属于结构性阅读观察，不构成缺陷判定或数学验收。
sources:
  - atlas-core-domain-seams.md
kind: concept
tags:
  - alcove
  - 重复实现
  - 错误处理
aliases:
  - alcove-根格顶点双重移植的语义漂移风险
---

# Alcove 根格顶点双重移植的语义漂移风险

`root_vertex_simple` 在 Rust 中存在两份移植：一份位于 `atlas-core` 的 `domain_builtins.rs:6002`，另一份位于 `atlas-real-group/alcove.rs:643`。两者对应同一上游函数，转置 Cartan 构造与标签为 1 的墙的重试逻辑对齐，但错误通道、资源预算纪律和 `bracket` 失败处理存在差异，来源将其记录为语义漂移风险。^[atlas-core-domain-seams.md:63-79]

## 共同的顶点搜索算法

算法通过 `labels_for_component` 获取墙分量标签，将第一个标签为 1 的墙作为“最低余根”墙剔除，其余墙生成有限部分。随后用 `inverse_cartan` 求转置子 Cartan 矩阵的精确逆。相关背景见 [[Alcove 墙标签与标签排序]] 与 [[Alcove 算法中的精确有理线性代数]]。^[atlas-core-domain-seams.md:63-66]

搜索先以 `numer·floors` 尝试未移位解；若坐标不能全部被 `denom` 整除，则依次针对其余标签为 1 的墙加一列重试。全部尝试失败时，`domain_builtins.rs` 版本报 `no root lattice vertex found for alcove component`；成功时将系数乘以根向量并累加到结果。^[atlas-core-domain-seams.md:66-69]

两份实现都将 `bracket(gen[j], gen[i])` 放在矩阵的 `[i][j]` 位置，因此转置构造逐元相同。标签为 1 的墙的重试逻辑也等价：`domain_builtins.rs` 版本省略 `index > chosen` 条件，是因为 `first_one` 已经是第一个标签为 1 的墙。^[atlas-core-domain-seams.md:72-75]

## 三处实现差异

**错误通道。** `domain_builtins.rs` 版本返回 `Result<_, String>`，`atlas-real-group/alcove.rs` 版本使用 `StructureError`。两份实现采用不同的错误表示，后者可结合 [[StructureError 统一错误分类学]] 阅读。^[atlas-core-domain-seams.md:75-77]

**资源预算纪律。** 来源将 `domain_builtins.rs` 版本记为无对应预算纪律，而 `atlas-real-group/alcove.rs` 版本使用 `try_reserve_exact`。算法步骤的对齐并不涵盖这一分配防护差异。^[atlas-core-domain-seams.md:75-78]

**配对失败处理。** `domain_builtins.rs` 版本对 `bracket` 使用 `unwrap_or(0)`，失败时静默取零；`atlas-real-group/alcove.rs` 版本使用 `?` 传播错误。这是同一配对失败在两份实现中得到不同处理的直接差异。^[atlas-core-domain-seams.md:75-78]

## 调用范围与证据边界

`domain_builtins.rs` 版本的唯一调用方是 `"alcove_root_vertex"` 派发臂，来源标注的位置为 `13275/13315`。这一记录限定于所读源码中的调用关系。^[atlas-core-domain-seams.md:78-79]

两份实现的注释分别引用 `alcoves.cpp:345-408` 与 `alcoves.cpp:347-412`。这些上游位置均转述自 Rust 源码注释，未独立重读上游，行号可能随版本漂移。^[atlas-core-domain-seams.md:63-72, atlas-core-domain-seams.md:115-116]

本页依据当前工作区字节的结构性阅读，Git base 与哈希由来源所指向的阅读快照记录。两份移植的并存及其差异是阅读观察，不构成缺陷判定、修复授权或语言与数学验收；验收背景可参见 [[HPC 验收证据链]]。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:115-118]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词/生成元校验。
