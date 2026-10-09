---
title: RootTable 的根与余根构建
summary: RootTable::build 在 prefer_coroots 模式下先转置 Cartan 矩阵生成正（余）根再换回，并通过 components 划分分量、length_flags 分别标记根与余根的长根属性。
sources:
  - atlas-core-domain-scc-root-table.md
kind: concept
createdAt: "2026-10-09T14:29:31.333Z"
updatedAt: "2026-10-09T14:29:31.333Z"
tags:
  - 根系
  - 余根
  - 根表构建
aliases:
  - roottable-的根与余根构建
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# RootTable 的根与余根构建

`RootTable::build` 负责根与余根的构建。源码阅读记录将其定位于 `crates/atlas-core/src/domain_builtins.rs` 的 9382 行起，涉及 Cartan 矩阵的转置、分量处理、环境格基表达以及根与余根的长标志。^[atlas-core-domain-scc-root-table.md:34-38]

## 构建方式

启用 `prefer_coroots` 时，构建过程先转置 Cartan 矩阵，生成正（余）根，再换回。这一处理与[[对偶根映射与 Cartan 矩阵转置]]相关。^[atlas-core-domain-scc-root-table.md:36-38]

`components` 用于分量处理；`express` 将单坐标向量表达为环境格基中的向量；`length_flags` 分别提供根与余根的长标志。^[atlas-core-domain-scc-root-table.md:36-38]

## 证据范围

当前材料属于结构性阅读，不代表数学验收。RootTable 的消费者仅在区域头部精读范围内有所涉及；上游行号引用属于实现方的移植陈述，兼容性仍以 HPC 差分门为准，可参见[[HPC 验收证据链]]。^[atlas-core-domain-scc-root-table.md:9-14, atlas-core-domain-scc-root-table.md:40-45]

## Sources

- [块图 SCC 与根表](atlas-core-domain-scc-root-table.md)
