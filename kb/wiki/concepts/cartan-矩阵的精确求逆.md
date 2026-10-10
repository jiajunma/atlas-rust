---
title: Cartan 矩阵的精确求逆
summary: 通过分数自由消元计算 Cartan 矩阵的精确逆，并以分子与分母表示；来源仅提供结构性阅读证据。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:53.655Z"
updatedAt: "2026-10-10T00:21:23.553Z"
tags:
  - 线性代数
  - 精确计算
aliases:
  - cartan-矩阵的精确求逆
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
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

该功能位于 `crates/atlas-core/src/domain_builtins.rs` 的根编号与 alcove 机器阅读范围内。来源将精确 Cartan 逆与墙集计算、根分量划分、墙标签计算及排序、从基本 alcove 构造 Weyl 词等功能一同列出；相关说明见 [[Alcove 墙集与整值墙筛选]] 和 [[Alcove 墙标签与标签排序]]。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:29-49]

## 证据边界

来源对求逆算法仅明确了分数自由消元及“分子、分母”表示，未说明具体消元步骤、分母规范化规则、奇异矩阵处理或算术溢出行为，因此不能据此确定这些实现细节。^[atlas-core-root-numbering-alcove.md:49-49]

本材料属于结构性阅读，不构成数学验收。来源中的上游行号引用属于实现方的移植陈述；根编号与 alcove 的兼容性仍以 HPC 差分门为准，参见 [[HPC 验收证据链]]。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
