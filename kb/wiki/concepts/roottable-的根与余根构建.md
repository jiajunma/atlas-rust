---
title: RootTable 的根与余根构建
summary: RootTable::build 在 prefer_coroots 模式下先转置 Cartan 矩阵生成正（余）根再换回，并划分分量、转换到环境格基及分别标记根与余根的长根属性。
sources:
  - atlas-core-domain-scc-root-table.md
kind: concept
createdAt: "2026-10-09T14:29:31.333Z"
updatedAt: "2026-10-09T22:15:33.235Z"
tags:
  - 根系
  - 余根
  - Rust实现
aliases:
  - roottable-的根与余根构建
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: RootTable 的根与余根构建
summary: RootTable::build 在 prefer_coroots 模式下先转置 Cartan 矩阵生成正（余）根再换回，并处理分量划分、环境格基表达以及根与余根的长标志。
sources:
  - atlas-core-domain-scc-root-table.md
kind: concept
tags:
  - 根系
  - 余根
  - 根表构建
aliases:
  - roottable-的根与余根构建
provenanceState: extracted
---

# RootTable 的根与余根构建

`RootTable::build` 位于 `crates/atlas-core/src/domain_builtins.rs` 的 9382 行起，构建涉及正（余）根生成、分量划分、环境格基表达，以及根与余根的长标志。^[atlas-core-domain-scc-root-table.md:10-13, atlas-core-domain-scc-root-table.md:34-38]

## 构建方式

启用 `prefer_coroots` 时，构建先转置 Cartan 矩阵，生成正（余）根，再换回。相关概念可参见 [[对偶根映射与 Cartan 矩阵转置]]。^[atlas-core-domain-scc-root-table.md:36-38]

`components` 负责划分分量；`express` 将单坐标向量表达到环境格基；`length_flags` 分别提供根和余根的长标志。^[atlas-core-domain-scc-root-table.md:36-38]

## 证据范围

来源属于结构性源码阅读，不构成数学验收。材料明确限定了阅读深度：包括 RootTable 消费者在内的大块内容仅在区域头部精读，因此不能据此宣称已完整覆盖其后续使用行为。^[atlas-core-domain-scc-root-table.md:9-14, atlas-core-domain-scc-root-table.md:40-43]

来源中的上游行号引用属于实现方的移植陈述，兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。快照的 Git 基点为 `964f0033`，字节数与哈希仅用于标识该快照字节。^[atlas-core-domain-scc-root-table.md:44-45]

## Sources

- [块图 SCC 与根表（domain_builtins.rs 3022–5005 + 7992–9606）](../../sources/atlas-core-domain-scc-root-table.md)
