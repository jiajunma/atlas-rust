---
title: ByLastCoordinate 逆坐标字典序
summary: ByLastCoordinate 通过 iter().rev() 从末坐标向前作字典序比较，与上游 root_compare 的一致性属于移植陈述，尚需 HPC 差分验证。
sources:
  - atlas-core-domain-scc-root-table.md
kind: concept
createdAt: "2026-10-09T14:29:18.003Z"
updatedAt: "2026-10-09T20:33:03.561Z"
tags:
  - 排序
  - 根系
aliases:
  - bylastcoordinate-逆坐标字典序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# ByLastCoordinate 逆坐标字典序

`ByLastCoordinate` 采用**从末坐标向前比较的字典序**，实现通过 `iter().rev()` 逆序遍历坐标进行比较。“逆坐标”描述的是坐标的比较次序。^[atlas-core-domain-scc-root-table.md:24-25]

## 与上游实现的对应

来源将这一顺序对应到 C++ 上游 `rootdata.cpp:118–129` 中 `root_compare` 的“末坐标向前”规则。该对应属于实现方的移植陈述；兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-domain-scc-root-table.md:24-25, atlas-core-domain-scc-root-table.md:44-44]

## 实现位置与证据范围

该比较规则位于 `crates/atlas-core/src/domain_builtins.rs` 的阅读范围内，来源将其归入 7992–9382 区域的序、矩阵辅助与对合校验部分。同一来源还介绍了 [[RootTable 的根与余根构建]]，包括按分量处理、环境格基表达及根与余根的长标志。^[atlas-core-domain-scc-root-table.md:9-13, atlas-core-domain-scc-root-table.md:22-38]

本页依据结构性阅读记录，不构成数学验收。来源明确指出，SCC 细节及 RootTable 消费者等大块内容只在头部精读；快照字节数和哈希仅标识对应字节，所记 Git base 为 `964f0033`。^[atlas-core-domain-scc-root-table.md:9-14, atlas-core-domain-scc-root-table.md:40-45]

## Sources

- [atlas-core-domain-scc-root-table.md](../../sources/atlas-core-domain-scc-root-table.md)
