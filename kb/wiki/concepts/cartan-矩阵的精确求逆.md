---
title: Cartan 矩阵的精确求逆
summary: 采用分数自由消元计算 Cartan 矩阵的精确逆，并以分子与分母表示结果；来源仅支持结构性说明，不构成数学验收。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:53.655Z"
updatedAt: "2026-10-09T22:18:17.634Z"
tags:
  - Cartan矩阵
  - 精确线性代数
aliases:
  - cartan-矩阵的精确求逆
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cartan 矩阵的精确求逆
summary: 使用分数自由消元计算 Cartan 矩阵的精确逆，以分子与分母表示结果；现有来源仅支持结构性说明，不构成数学验收。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
tags:
  - Cartan矩阵
  - 精确线性代数
aliases:
  - cartan-矩阵的精确求逆
provenanceState: extracted
---

# Cartan 矩阵的精确求逆

Cartan 矩阵的精确求逆采用**分数自由消元**，结果以“分子、分母”二元形式表示。^[atlas-core-root-numbering-alcove.md:49-49]

## 实现上下文

该功能属于 `crates/atlas-core/src/domain_builtins.rs` 中 alcove 机器的结构性阅读范围。来源将精确 Cartan 逆与墙集计算、根分量划分、墙标签计算与排序，以及从基本 alcove 构造 Weyl 词一同列出；相关主题包括 [[Alcove 墙集与整值墙筛选]]、[[墙分量的本原 Coroot 关系]] 和 [[Alcove 墙标签与标签排序]]。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:27-49]

## 证据边界

来源对求逆算法仅明确了分数自由消元及“分子、分母”表示，未给出具体消元步骤、分母规范化规则、奇异矩阵处理或算术溢出行为，因而不能据此确定这些实现细节。^[atlas-core-root-numbering-alcove.md:49-49]

本材料属于结构性阅读，不构成数学验收。来源中的上游行号引用属于实现方的移植陈述；根编号与 alcove 的兼容性仍以 HPC 差分门为准，相关主题见 [[HPC 验收证据链]]。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
