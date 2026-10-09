---
title: RootNumbering 根编号与 RootNbr 顺序
summary: 正根按坐标和及从末坐标向前的字典序排列，prefer_coroots 选择坐标系，负根采用镜像编号，signed(nbr)=nbr−npos。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:27.809Z"
updatedAt: "2026-10-09T22:18:01.350Z"
tags:
  - 根系
  - 编号约定
aliases:
  - rootnumbering-根编号与-rootnbr-顺序
  - R根R顺
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: RootNumbering 根编号与 RootNbr 顺序
summary: 正根按坐标和及从末坐标向前的比较排序，prefer_coroots 选择坐标系；正负根反向配对，RootNbr 顺序参与 alcove 分量和墙标签排序。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
tags:
  - 根系
  - 编号约定
  - Rust实现
aliases:
  - rootnumbering-根编号与-rootnbr-顺序
provenanceState: extracted
---

# RootNumbering 根编号与 RootNbr 顺序

`RootNumbering` 为根系建立 `RootNbr` 编号，规定正根排序、正负根配对和带符号索引转换。实现位于 `crates/atlas-core/src/domain_builtins.rs` 的 5005–5092 行。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:16-25]

## 正根排序与坐标选择

`RootNumbering::new(root_system, prefer_coroots)` 按 `(level, root_compare)` 对正根排序。`level` 是坐标之和；层级相同时，`root_compare` 从最后一个坐标向前比较，对应上游 `rootdata.cpp:118–129`。相关比较约定见 [[ByLastCoordinate 逆坐标字典序]]。^[atlas-core-root-numbering-alcove.md:18-20]

`prefer_coroots` 选择单余根坐标或单根坐标，因此坐标系选择是编号规则的一部分。实现还使用按坐标组织的 `BTreeMap`，建立坐标到正根位置的索引。^[atlas-core-root-numbering-alcove.md:18-25]

## 正负根配对与带符号索引

设 `npos` 为正根数，正根的 `RootNbr` 位于 `[npos, total)`。若 `p` 表示正根排序后的零基位置，则其对应负根的编号为 `npos - 1 - p`，形成反向配对；这里的 `p` 是正根位置，而非完整的正根 `RootNbr`。每个负根都有对应正根。^[atlas-core-root-numbering-alcove.md:21-25]

负根判定为 `is_negative(nbr) = nbr < npos`；带符号索引转换为 `signed(nbr) = nbr - npos`，对应上游 `convert_to_signed_root_index`（`atlas-types.w:1478–1485`）。^[atlas-core-root-numbering-alcove.md:23-24]

## 在 alcove 算法中的可观察顺序

`root_components` 按非正交关系将根子集划分为连通分量，每个分量内部按 `RootNbr` 升序排列。原版在新根触及分量时就追加该根，因此最终分量之间按最大 `RootNbr` 排序，而非最小值；这一顺序在 FPP 乘积向量及其 Weyl 见证中可观察。^[atlas-core-root-numbering-alcove.md:33-36]

`sorted_by_label` 将全部墙按分量标签降序排列，并列时按 `RootNbr` 序排列。因此，根编号也是 [[Alcove 墙标签与标签排序]] 的并列排序依据；这与分量之间按最大 `RootNbr` 排列的规则属于不同层次。^[atlas-core-root-numbering-alcove.md:33-41]

## 证据边界

本页依据结构性源码阅读，不代表数学验收。源材料中的上游行号属于实现方的移植陈述；根编号与 alcove 行为的兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。快照的字节数与哈希仅标识所读字节，来源记载的 Git base 为 `964f0033`。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [atlas-core-root-numbering-alcove.md — 根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
