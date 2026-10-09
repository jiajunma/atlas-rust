---
title: Alcove 墙标签与标签排序
summary: 墙分量的标签来自余根间取正的唯一原始整数关系，可由环境余根坐标求核；全部墙按标签降序排列，同标签按 RootNbr 排序。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:39.098Z"
updatedAt: "2026-10-09T14:32:39.098Z"
tags:
  - alcove
  - 整数关系
  - 排序
aliases:
  - alcove-墙标签与标签排序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Alcove 墙标签与标签排序

Alcove 墙标签由各墙分量中余根的唯一正本原整数关系确定；`sorted_by_label` 再按标签降序排列全部墙，标签相同时按 RootNbr 序排列。标签因此既描述墙余根的线性关系，也决定墙的排序。^[atlas-core-root-numbering-alcove.md:37-41]

## 墙分量与标签

标签计算以 `wall_set` 得到的墙集为基础。`root_components` 按根之间的非正交关系划分连通分量，每个分量内部按 RootNbr 升序排列。原版在新根触及分量时追加该根，最终分量顺序按最大 RootNbr，而非最小 RootNbr；这一顺序在 FPP 乘积向量及其 Weyl 见证中可观察。相关背景见 [[Alcove 墙集与整值墙筛选]]。^[atlas-core-root-numbering-alcove.md:29-36]

`labels_for_component` 为一个墙分量求出余根间唯一的本原整数关系，并取正系数作为标签。环境余根坐标与上游单余根坐标具有相同的线性关系，因此实现可从环境余根表计算核，并使用 `gcd`、`lcm` 辅助计算。参见 [[墙分量的本原 Coroot 关系]]。^[atlas-core-root-numbering-alcove.md:37-39]

## 标签排序

`sorted_by_label` 对全部墙按所属分量给出的标签降序排序，并在标签相同时按 RootNbr 序排列。这里的分量顺序与标签排序是两个不同约定：前者按分量最大 RootNbr 确定，后者以标签为首要排序依据。^[atlas-core-root-numbering-alcove.md:33-41]

并列时使用的 RootNbr 序来自 [[RootNumbering 根编号与 RootNbr 顺序]]。正根首先按坐标和 `level` 排序，再由 `root_compare` 从最后一个坐标向前比较；`prefer_coroots` 决定使用单余根坐标还是单根坐标。^[atlas-core-root-numbering-alcove.md:18-20]

## 在 Weyl 词构造中的作用

标签还参与 `from_fundamental_alcove` 的 Weyl 词构造：每个墙分量留出一个单位标签墙，其余墙经 `to_positive_system` 移到单根系，再将步骤逆序并通过最终单值索引得到词。该词对应具有给定墙集的 alcove。^[atlas-core-root-numbering-alcove.md:42-44]

## 证据边界

上述说明来自实现的结构性阅读，不构成数学验收。源材料中的上游行号属于实现方的移植陈述；根编号与 alcove 行为的兼容性仍以 HPC 差分门为准。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](atlas-core-root-numbering-alcove.md)
