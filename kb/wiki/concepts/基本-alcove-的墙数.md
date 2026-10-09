---
title: 基本 Alcove 的墙数
summary: 基本 alcove 的墙由单余根及每个不可约分量的一条最低余根组成，墙数为秩加分量数，本实现中仅该大小用于“Too few walls”检查。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:45.918Z"
updatedAt: "2026-10-09T14:32:45.918Z"
tags:
  - alcove
  - 根系
  - 边界检查
aliases:
  - 基本-alcove-的墙数
  - 基A的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 基本 Alcove 的墙数

基本 Alcove 的墙由所有单余根，以及每个不可约分量的一条最低余根组成。因此，若根系的秩为 \(r\)，不可约分量数为 \(c\)，则基本 Alcove 的墙数为
\[
N_{\mathrm{walls}}=r+c.
\]
这一构造对应 `RootSystem::fundamental_alcove_walls`；源材料将其上游实现定位于 `rootdata.cpp:474–481`。^[atlas-core-root-numbering-alcove.md:45-48]

## 实现与可观察行为

`fundamental_alcove_wall_count` 提供基本 Alcove 的墙数，用于 `"Too few walls"` 检查。在这一检查中，可观察的是墙集的大小，而不是具体墙的选择或排列。墙集的其他处理可参见 [[Alcove 墙集与整值墙筛选]] 与 [[Alcove 墙标签与标签排序]]。^[atlas-core-root-numbering-alcove.md:29-48]

## 证据边界

本说明依据对 `domain_builtins.rs` 中根编号与 alcove 机器的结构性阅读，不代表数学验收。上游位置属于实现方的移植陈述；编号与 alcove 行为的兼容性仍以 HPC 差分门为准，参见 [[HPC 验收证据链]]。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](atlas-core-root-numbering-alcove.md)
