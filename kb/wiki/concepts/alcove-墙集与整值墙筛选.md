---
title: Alcove 墙集与整值墙筛选
summary: wall_set 计算 gamma 经小 dominant 位移所达 alcove 的墙余根，记录在 gamma 上取整的墙，并按余根不可相减条件筛选。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:34.499Z"
updatedAt: "2026-10-09T14:32:34.499Z"
tags:
  - alcove
  - 余根
  - 墙集
aliases:
  - alcove-墙集与整值墙筛选
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Alcove 墙集与整值墙筛选

`wall_set` 根据参数 `gamma` 的小 dominant（优势方向）位移所到达的 alcove，确定其墙余根；其中在 `gamma` 上取整数值的墙另行收集到 `integrals`，对应上游的 `on_wall_coroots`。因此，墙集的确定与整值墙的收集是相关但不同的步骤。^[atlas-core-root-numbering-alcove.md:29-32]

## 墙余根的筛选条件

墙集过滤器只保留“余根不能被减去”的根，对应上游 `min_coroots_for` 的成员条件：相关余根之差 \(\alpha^\vee-\beta^\vee\) 不是余根。这个余根差条件用于墙集筛选；`integrals` 则依据墙在 `gamma` 上的取值是否为整数进行收集。^[atlas-core-root-numbering-alcove.md:29-32]

## 墙集的后续处理

墙集确定后，`root_components` 按非正交关系划分连通分量。每个分量内部按 RootNbr 升序排列，最终分量之间按最大 RootNbr 排序，而非最小 RootNbr；该顺序在 FPP 乘积向量及其 Weyl 见证中可观察。编号背景见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[atlas-core-root-numbering-alcove.md:33-36]

每个墙分量的标签来自余根之间唯一的正本原整数关系，详见 [[墙分量的本原 Coroot 关系]]。`sorted_by_label` 将全部墙按标签降序排列，标签相同时按 RootNbr 序排列，参见 [[Alcove 墙标签与标签排序]]。^[atlas-core-root-numbering-alcove.md:37-41]

给定墙集后，`from_fundamental_alcove` 在每个分量中留出一个标签为 1 的墙，将其余墙经 `to_positive_system` 移到单根系，再以逆序步骤结合最终单值索引得到相应的 Weyl 词。^[atlas-core-root-numbering-alcove.md:42-44]

## 基本 alcove 的墙数检查

基本 alcove 的墙由单余根以及每个不可约分量的一条最低余根组成，墙数等于“秩 + 分量数”。在所述 `"Too few walls"` 检查中，只有这个大小可观察；相关概念见 [[基本 Alcove 的墙数]]。^[atlas-core-root-numbering-alcove.md:45-48]

## 证据边界

本说明依据对 `domain_builtins.rs` 中根编号与 alcove 机器的结构性阅读，不代表数学验收。源码包中的上游位置引用属于实现方的移植陈述；编号与 alcove 的兼容性仍以 HPC 差分门为准，参见 [[HPC 验收证据链]]。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [atlas-core-root-numbering-alcove.md](atlas-core-root-numbering-alcove.md)
