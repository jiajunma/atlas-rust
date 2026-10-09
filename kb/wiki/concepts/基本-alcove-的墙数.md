---
title: 基本 Alcove 的墙数
summary: 基本 alcove 的墙由单余根与每个不可约分量的一条最低余根组成，数量为秩加分量数，本实现仅以该数量进行 Too few walls 检查。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:45.918Z"
updatedAt: "2026-10-09T22:18:17.889Z"
tags:
  - alcove
  - 输入校验
aliases:
  - 基本-alcove-的墙数
  - 基A的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 基本 Alcove 的墙数
summary: 基本 Alcove 的墙由单余根及每个不可约分量的一条最低余根组成，墙数为秩加分量数；该数量用于 Too few walls 检查。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
tags:
  - alcove
  - 根系
  - 边界检查
---

# 基本 Alcove 的墙数

基本 Alcove 的墙由所有单余根，以及每个不可约分量的一条最低余根组成。若根系的秩为 \(r\)，不可约分量数为 \(c\)，则墙数为 \(N_{\mathrm{walls}}=r+c\)。源材料将这一构造对应到 `RootSystem::fundamental_alcove_walls`，上游位置为 `rootdata.cpp:474–481`。^[atlas-core-root-numbering-alcove.md:45-48]

## 实现与可观察行为

`fundamental_alcove_wall_count` 提供基本 Alcove 的墙数，用于 `"Too few walls"` 检查。就这一检查而言，只有墙集的大小可观察，具体墙的选择与排列不属于该检查的可观察内容。^[atlas-core-root-numbering-alcove.md:45-48]

相关的墙集构造由 `wall_set` 完成：它确定由 `gamma` 的小 dominant 位移所达 Alcove 的墙余根，并收集在 `gamma` 上取整的墙，详见 [[Alcove 墙集与整值墙筛选]]。墙的排序则由 `sorted_by_label` 完成，按分量标签降序排列，并列时按 RootNbr 序排列，详见 [[Alcove 墙标签与标签排序]]。^[atlas-core-root-numbering-alcove.md:29-32, atlas-core-root-numbering-alcove.md:40-41]

## 证据边界

本说明依据 `crates/atlas-core/src/domain_builtins.rs` 中根编号与 Alcove 机器的结构性阅读，不构成数学验收。源材料中的上游行号属于实现方的移植陈述；编号与 Alcove 行为的兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
