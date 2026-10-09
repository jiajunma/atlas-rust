---
title: RootNumbering 根编号与 RootNbr 顺序
summary: 正根按坐标和及从末坐标向前的比较排序，prefer_coroots 选择坐标系，并通过正负根配对与 signed(nbr)=nbr−npos 提供编号转换。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:27.809Z"
updatedAt: "2026-10-09T14:32:27.809Z"
tags:
  - 根系
  - 编号规则
  - Rust实现
aliases:
  - rootnumbering-根编号与-rootnbr-顺序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# RootNumbering 根编号与 RootNbr 顺序

`RootNumbering` 为根系建立确定的 `RootNbr` 编号，将正根排序、正负根配对及带符号索引转换统一起来。相关实现位于 `crates/atlas-core/src/domain_builtins.rs` 的 5005–5092 行。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:16-25]

## 正根排序

`RootNumbering::new(root_system, prefer_coroots)` 按 `(level, root_compare)` 对正根排序。`level` 是坐标之和；层级相同时，`root_compare` 从最后一个坐标向前比较。`prefer_coroots` 决定使用单余根坐标还是单根坐标，因此坐标系选择参与排序规则。相关逆坐标比较概念见 [[ByLastCoordinate 逆坐标字典序]]。^[atlas-core-root-numbering-alcove.md:18-20]

## 编号与正负配对

设 `npos` 为正根数，正根的 `RootNbr` 占据区间 `[npos, total)`，负根的编号小于 `npos`。对于排序后位置为 `p` 的正根，其负根编号为 `npos - 1 - p`，使负根与正根的位置反向配对；每个负根都有对应的正根。^[atlas-core-root-numbering-alcove.md:21-25]

带符号索引通过 `signed(nbr) = nbr - npos` 得到，对应上游的 `convert_to_signed_root_index`；负根判定为 `is_negative(nbr) = nbr < npos`。此外，实现使用按坐标组织的 `BTreeMap`，建立坐标到正根位置的索引。^[atlas-core-root-numbering-alcove.md:23-25]

## 在 alcove 算法中的可观察顺序

`RootNbr` 顺序参与 alcove 墙集的组织。`root_components` 按非正交关系划分连通分量，每个分量内部按 `RootNbr` 升序排列。原版在新根触及分量时追加该根，最终分量顺序因此按最大 `RootNbr` 确定，而非最小值；这一顺序会影响 FPP 乘积向量及其 Weyl 见证。^[atlas-core-root-numbering-alcove.md:33-36]

`sorted_by_label` 将全部墙按分量标签降序排列，标签相同时按 `RootNbr` 顺序排列。因此，根编号也是 [[Alcove 墙标签与标签排序]] 中的并列排序依据。^[atlas-core-root-numbering-alcove.md:40-41]

## 证据边界

本说明依据结构性源码阅读，不代表数学验收。源材料中的上游行号属于实现方的移植陈述；根编号与 alcove 行为的兼容性仍以 HPC 差分门为准，相关证据要求见 [[HPC 验收证据链]]。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:51-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](atlas-core-root-numbering-alcove.md)
