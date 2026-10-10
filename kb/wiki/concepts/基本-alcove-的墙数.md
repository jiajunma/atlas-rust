---
title: 基本 Alcove 的墙数
summary: 基本 alcove 的墙数为秩加不可约分量数，本实现仅使用该数量进行 Too few walls 检查。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:45.918Z"
updatedAt: "2026-10-10T00:21:18.034Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 基本 Alcove 的墙数
summary: 基本 Alcove 的墙由单余根及每个不可约分量的一条最低余根组成，墙数为秩加分量数，用于 Too few walls 检查。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
tags:
  - alcove
  - 输入校验
---

# 基本 Alcove 的墙数

基本 Alcove 的墙由所有单余根及每个不可约分量的一条最低余根组成。因此，若根系的秩为 \(r\)，不可约分量数为 \(c\)，则基本 Alcove 的墙数为 \(N_{\mathrm{walls}}=r+c\)。源材料将这一构造对应到 `RootSystem::fundamental_alcove_walls`，上游位置为 `rootdata.cpp:474–481`。^[atlas-core-root-numbering-alcove.md:45-48]

## 实现与可观察行为

`fundamental_alcove_wall_count` 提供基本 Alcove 的墙数，用于 `"Too few walls"` 检查。在这一检查中，只有墙集的大小可观察，具体墙的选择与排列不属于该检查的可观察内容。^[atlas-core-root-numbering-alcove.md:45-48]

相关的 `wall_set` 确定由 `gamma` 的小 dominant 位移所达 Alcove 的墙余根，并收集在 `gamma` 上取整的墙，详见 [[Alcove 墙集与整值墙筛选]]。`sorted_by_label` 则将全部墙按分量标签降序排列，并列时按 RootNbr 序排列，详见 [[Alcove 墙标签与标签排序]]。^[atlas-core-root-numbering-alcove.md:29-32, atlas-core-root-numbering-alcove.md:40-41]

## 证据边界

本页依据 `crates/atlas-core/src/domain_builtins.rs` 中根编号与 Alcove 机器的结构性阅读，不构成数学验收。源材料中的上游行号属于实现方的移植陈述；编号与 Alcove 行为的兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
