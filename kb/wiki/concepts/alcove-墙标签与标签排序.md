---
title: Alcove 墙标签与标签排序
summary: 墙标签取自各墙分量余根间唯一的正本原整数关系，全部墙按标签降序、同标签按 RootNbr 排序。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:39.098Z"
updatedAt: "2026-10-10T00:21:12.534Z"
tags:
  - alcove
  - 线性代数
aliases:
  - alcove-墙标签与标签排序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Alcove 墙标签与标签排序
summary: 墙分量标签来自余根间取正的唯一本原整数关系，可由环境余根坐标求核；全部墙按标签降序排列，同标签按 RootNbr 序排列。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
tags:
  - alcove
  - 整数关系
  - 排序约定
aliases:
  - alcove-墙标签与标签排序
provenanceState: extracted
---

# Alcove 墙标签与标签排序

Alcove 墙标签由每个墙分量中余根之间唯一的正本原整数关系确定。`labels_for_component` 计算分量标签，`sorted_by_label` 将全部墙按标签降序排列，标签相同时按 RootNbr 序排列。^[atlas-core-root-numbering-alcove.md:37-41]

## 墙分量与标签计算

墙集由 `wall_set` 确定，相关筛选见 [[Alcove 墙集与整值墙筛选]]。`root_components` 按非正交关系将根子集划分为连通分量，每个分量内部按 RootNbr 升序排列。原版在新根触及分量时就追加该根，因此最终分量按最大 RootNbr 排序，而非最小 RootNbr；这一顺序在 FPP 乘积向量及其 Weyl 见证中可观察。^[atlas-core-root-numbering-alcove.md:29-36]

`labels_for_component` 求出一个墙分量的余根之间唯一的本原整数关系，并取正。环境余根坐标与上游单余根坐标携带相同的线性关系，因此可由环境余根表计算核，并使用 `gcd`、`lcm` 辅助计算。^[atlas-core-root-numbering-alcove.md:37-39]

## 排序约定

`sorted_by_label` 对全部墙按标签降序排序，并列时按 RootNbr 序排列。全体墙的标签排序与 `root_components` 的分量排序属于不同层次：前者比较标签并以 RootNbr 打破并列，后者依据分量的最大 RootNbr 确定分量顺序。^[atlas-core-root-numbering-alcove.md:33-41]

并列比较所用的编号来自 [[RootNumbering 根编号与 RootNbr 顺序]]。正根按 `(level, root_compare)` 排序，其中 `level` 是坐标和，`root_compare` 从最后一个坐标向前比较；`prefer_coroots` 决定使用单余根坐标还是单根坐标。^[atlas-core-root-numbering-alcove.md:18-20]

## 在 Weyl 词构造中的作用

`from_fundamental_alcove` 构造其 alcove 具有给定墙集的 Weyl 词。它在每个分量中留出一个标签为 1 的墙，将其余墙经 `to_positive_system` 移到单根系，再将步骤逆序并通过最终单值索引得到词。因此，标签也参与确定每个分量中留出的墙。^[atlas-core-root-numbering-alcove.md:42-44]

## 证据边界

本页依据 `crates/atlas-core/src/domain_builtins.rs` 中根编号与 alcove 机器的结构性阅读，不构成数学验收。来源中的上游行号引用属于实现方的移植陈述；编号与 alcove 行为的兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
