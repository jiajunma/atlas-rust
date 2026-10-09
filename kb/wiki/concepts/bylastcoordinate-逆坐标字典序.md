---
title: ByLastCoordinate 逆坐标字典序
summary: ByLastCoordinate 通过 iter().rev() 从末坐标向前进行字典序比较，来源称其与 C++ root_compare 的顺序一致，但兼容性仍须 HPC 差分验证。
sources:
  - atlas-core-domain-scc-root-table.md
kind: concept
createdAt: "2026-10-09T14:29:18.003Z"
updatedAt: "2026-10-09T14:29:18.003Z"
tags:
  - 排序规则
  - 根数据
  - 兼容性
aliases:
  - bylastcoordinate-逆坐标字典序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# ByLastCoordinate 逆坐标字典序

`ByLastCoordinate` 采用**从末坐标向前比较的字典序**：实现通过 `iter().rev()` 逆序遍历坐标进行比较。这里的“逆序”指坐标的比较顺序。^[atlas-core-domain-scc-root-table.md:24-25]

## 与上游实现的对应

该比较顺序与 C++ 上游 `rootdata.cpp:118–129` 中 `root_compare` 的“末坐标向前”规则一致。这一对应属于实现方的移植陈述；兼容性仍以 [[HPC 验收证据链|HPC 差分门]]为准。^[atlas-core-domain-scc-root-table.md:24-25, atlas-core-domain-scc-root-table.md:44-44]

## 证据范围

来源将该规则列入 `crates/atlas-core/src/domain_builtins.rs` 的 7992–9606 区域阅读范围，同一来源还涉及矩阵辅助、对合构造校验及 [[RootTable 的根与余根构建]]。这份材料属于结构性阅读记录，不声称数学验收。^[atlas-core-domain-scc-root-table.md:9-14]

## Sources

- [atlas-core-domain-scc-root-table.md](../../sources/atlas-core-domain-scc-root-table.md)
