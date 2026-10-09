---
title: Alcove 墙标签与标签排序
summary: 墙分量标签来自余根间取正的唯一原始整数关系，可由环境余根坐标求核；全部墙按标签降序、同标签按 RootNbr 排序。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:39.098Z"
updatedAt: "2026-10-09T20:35:48.467Z"
tags:
  - alcove
  - 整数关系
  - 排序约定
aliases:
  - alcove-墙标签与标签排序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Alcove 墙标签与标签排序
summary: 墙分量的标签来自余根间取正的唯一本原整数关系，可由环境余根坐标求核；全部墙按标签降序排列，同标签按 RootNbr 序排列。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
tags:
  - alcove
  - 整数关系
  - 排序
aliases:
  - alcove-墙标签与标签排序
provenanceState: extracted
---

# Alcove 墙标签与标签排序

Alcove 墙标签由各墙分量中余根的唯一正本原整数关系确定。`labels_for_component` 计算分量标签，`sorted_by_label` 将全部墙按标签降序排列，标签相同时按 RootNbr 序排列。^[atlas-core-root-numbering-alcove.md:37-41]

## 墙分量与标签计算

墙集由 `wall_set` 确定，相关背景见 [[Alcove 墙集与整值墙筛选]]。`root_components` 按根之间的非正交关系划分连通分量，每个分量内部按 RootNbr 升序排列。原版在新根触及分量时追加该根，最终分量之间按最大 RootNbr 排序，而非最小 RootNbr；这一顺序在 FPP 乘积向量及其 Weyl 见证中可观察。^[atlas-core-root-numbering-alcove.md:29-36]

`labels_for_component` 求出一个墙分量的余根之间唯一的本原整数关系，并取正。环境余根坐标与上游单余根坐标具有相同的线性关系，因此可由环境余根表计算核，并使用 `gcd`、`lcm` 辅助计算。详见 [[墙分量的本原 Coroot 关系]]。^[atlas-core-root-numbering-alcove.md:37-39]

## 排序约定

`sorted_by_label` 对全部墙按分量标签降序排序，并列时按 RootNbr 序排列。分量之间按最大 RootNbr 排列的约定，与全部墙按标签排列的约定作用于不同层次，应分别保留。^[atlas-core-root-numbering-alcove.md:33-41]

并列比较所用的编号来自 [[RootNumbering 根编号与 RootNbr 顺序]]。正根按 `(level, root_compare)` 排序，其中 `level` 是坐标和，`root_compare` 从最后一个坐标向前比较；`prefer_coroots` 决定采用单余根坐标还是单根坐标。^[atlas-core-root-numbering-alcove.md:18-20]

## 在 Weyl 词构造中的作用

`from_fundamental_alcove` 构造对应于给定墙集的 alcove 的 Weyl 词。它在每个分量中留出一个单位标签墙，将其余墙经 `to_positive_system` 移到单根系，再将步骤逆序并通过最终单值索引得到词。因此，标签也参与该构造中每个分量的留墙选择。^[atlas-core-root-numbering-alcove.md:42-44]

## 证据边界

本说明依据 `domain_builtins.rs` 中根编号与 alcove 机器的结构性阅读，不构成数学验收。源材料中的上游行号属于实现方的移植陈述；编号与 alcove 行为的兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
