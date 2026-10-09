---
title: 由基本 Alcove 构造 Weyl 词
summary: from_fundamental_alcove 在每个墙分量留出一个单位标签墙，将其余墙经 to_positive_system 移到单根系，再逆序处理步骤并通过最终单值索引构造 Weyl 词。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:46.555Z"
updatedAt: "2026-10-09T14:32:46.555Z"
tags:
  - alcove
  - Weyl群
  - 词构造
aliases:
  - 由基本-alcove-构造-weyl-词
  - 由A构W词
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 由基本 Alcove 构造 Weyl 词

`from_fundamental_alcove` 根据给定墙集构造一个 Weyl 词，使其对应的 alcove 具有该墙集。源材料将此实现对应到上游 `alcoves.cpp:186–236`，核心步骤是按墙分量留出单位标签墙，将其余墙移到单根系，再由逆序步骤生成词。^[atlas-core-root-numbering-alcove.md:42-44]

## 墙分量与标签

墙集按非正交关系划分为连通分量，每个分量内部按 RootNbr 升序排列。分量本身的顺序遵循原版追加规则，最终按最大 RootNbr 排列，而非最小 RootNbr；这一顺序在 FPP 乘积向量及其 Weyl 见证中可观察，因此不能随意重排。根编号约定可参见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[atlas-core-root-numbering-alcove.md:33-36]

每个墙分量的标签来自其余根之间唯一的正本原整数关系。环境余根坐标与上游单余根坐标具有相同的线性关系，因此实现通过环境坐标表计算核，并使用 `gcd`、`lcm` 辅助处理。相关概念见 [[墙分量的本原 Coroot 关系]]。^[atlas-core-root-numbering-alcove.md:37-39]

## 构造步骤

构造时，每个分量留出一面标签为 1 的墙，其余墙通过 `to_positive_system` 移到单根系。随后将步骤逆序，并通过最终单值索引得到 Weyl 词。这里的“逆序步骤”是源材料明确给出的词构造约定；相关主题见 [[Alcove 根格顶点与基本 Alcove 约化]]。^[atlas-core-root-numbering-alcove.md:42-44]

同一 alcove 机器还提供 `sorted_by_label`：全部墙按分量标签降序排列，标签相同时按 RootNbr 序排列。这一排序规则可与 [[Alcove 墙标签与标签排序]] 对照阅读。^[atlas-core-root-numbering-alcove.md:40-41]

## [[基本-alcove-的墙数|基本 Alcove 的墙数]]

基本 alcove 的墙由单余根以及每个不可约分量的一条最低余根组成，因此墙数为“秩 + 分量数”。在源材料覆盖的实现中，此处只有墙集大小可观察，用于 `"Too few walls"` 检查；参见 [[基本 Alcove 的墙数]]。^[atlas-core-root-numbering-alcove.md:45-48]

## 证据边界

本页依据的是实现的结构性阅读，不代表数学验收。上游函数与行号的对应属于实现方的移植陈述；根编号及 alcove 行为的兼容性仍以 HPC 差分门为准，相关证据纪律见 [[HPC 验收证据链]]。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](atlas-core-root-numbering-alcove.md)
