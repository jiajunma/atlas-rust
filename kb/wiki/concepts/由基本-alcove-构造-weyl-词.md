---
title: 由基本 Alcove 构造 Weyl 词
summary: 每个墙分量留出一个单位标签墙，将其余墙移到单根系，再逆序处理步骤并通过最终单值索引构造 Weyl 词。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:46.555Z"
updatedAt: "2026-10-10T00:21:19.738Z"
tags:
  - alcove
  - Weyl群
aliases:
  - 由基本-alcove-构造-weyl-词
  - 由A构W词
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 由基本 Alcove 构造 Weyl 词
summary: from_fundamental_alcove 在每个墙分量留出一个单位标签墙，将其余墙移到单根系，再逆序处理步骤并通过最终单值索引构造 Weyl 词。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
tags:
  - alcove
  - Weyl群
  - 词构造
aliases:
  - 由基本-alcove-构造-weyl-词
provenanceState: extracted
---

# 由基本 Alcove 构造 Weyl 词

`from_fundamental_alcove` 根据给定墙集构造 Weyl 词，使其对应的 alcove 具有该墙集。核心过程是在每个墙分量中留出一个单位标签墙，将其余墙移到单根系，再逆序处理步骤并通过最终单值索引得到词。源材料将该实现对应到上游 `alcoves.cpp:186–236`。^[atlas-core-root-numbering-alcove.md:42-44]

## 墙分量与标签

`root_components` 按根之间的非正交关系划分连通分量，每个分量内部按 RootNbr 升序排列。原版在新根触及分量时追加该根，最终分量之间按最大 RootNbr 排序，而非最小 RootNbr；这一顺序在 FPP 乘积向量及其 Weyl 见证中可观察。编号背景见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[atlas-core-root-numbering-alcove.md:33-36]

`labels_for_component` 计算一个墙分量内余根之间唯一的本原整数关系，并取正，作为墙的标签。环境余根坐标与上游单余根坐标携带相同的线性关系，因此实现可由环境余根表求核，并使用 `gcd`、`lcm` 辅助计算。详见 [[Alcove 墙标签与标签排序]]。^[atlas-core-root-numbering-alcove.md:37-39]

## 构造过程与顺序

构造首先在每个墙分量中留出一面标签为 1 的墙，其余墙经 `to_positive_system` 移到单根系。源材料将这一辅助过程对应到上游 `rootdata.cpp:1329–1347`。随后将步骤逆序，通过最终单值索引得到 Weyl 词；逆序处理是该构造明确规定的顺序。^[atlas-core-root-numbering-alcove.md:42-44]

同一 alcove 机器中的 `sorted_by_label` 将全部墙按分量标签降序排列，标签相同时按 RootNbr 序排列。这是墙的标签排序约定，与按最大 RootNbr 确定的分量顺序分别描述不同层次的排序。^[atlas-core-root-numbering-alcove.md:33-41]

## [[基本-alcove-的墙数|基本 Alcove 的墙数]]

基本 alcove 的墙由单余根以及每个不可约分量的一条最低余根组成，因此墙数为“秩 + 分量数”。在源材料描述的 `fundamental_alcove_wall_count` 用途中，只有墙集大小可观察，用于 `"Too few walls"` 检查。参见 [[基本 Alcove 的墙数]]。^[atlas-core-root-numbering-alcove.md:45-48]

## 证据边界

本页依据 `crates/atlas-core/src/domain_builtins.rs` 中根编号与 alcove 机器的结构性阅读，不构成数学验收。上游函数及行号的对应属于实现方的移植陈述；根编号与 alcove 行为的兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
