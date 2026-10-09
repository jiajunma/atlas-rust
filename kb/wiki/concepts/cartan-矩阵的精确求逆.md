---
title: Cartan 矩阵的精确求逆
summary: 使用分数自由消元计算 Cartan 矩阵的精确逆，并以分子与分母的形式表示结果。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:53.655Z"
updatedAt: "2026-10-09T14:32:53.655Z"
tags:
  - Cartan矩阵
  - 精确算术
  - 线性代数
aliases:
  - cartan-矩阵的精确求逆
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cartan 矩阵的精确求逆

Cartan 矩阵的精确求逆是 `crates/atlas-core/src/domain_builtins.rs` 中 alcove 机器所覆盖的功能之一。该实现采用**分数自由消元**，以“分子、分母”二元形式表示精确的 Cartan 逆。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:49-49]

## 实现上下文

源材料将精确 Cartan 求逆与墙集计算、根分量划分、墙标签排序及从基本 alcove 构造 Weyl 词一同列入 alcove 机器。相关概念包括 [[Alcove 墙集与整值墙筛选]]、[[墙分量的本原 Coroot 关系]] 和 [[Alcove 墙标签与标签排序]]。^[atlas-core-root-numbering-alcove.md:27-49]

## 证据边界

源材料仅明确了分数自由消元及“分子、分母”表示，未展开消元步骤、分母规范化规则、奇异矩阵处理或算术溢出行为。因而不能据此补充这些实现细节。^[atlas-core-root-numbering-alcove.md:49-49]

本材料的证据等级为结构性阅读，不构成数学验收；其中上游行号属于实现方的移植陈述，编号与 alcove 兼容性仍以 HPC 差分门为准。可参阅 [[HPC 验收证据链]] 理解这一证据边界。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
