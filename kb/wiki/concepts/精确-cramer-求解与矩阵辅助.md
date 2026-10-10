---
title: 精确 Cramer 求解与矩阵辅助
summary: 矩阵辅助接口提供行主序转置、行数据到 Matrix 值的转换，以及采用分数自由变量消元的精确 Cramer 求解。
sources:
  - atlas-core-deformation-cache.md
kind: concept
createdAt: "2026-10-09T14:27:59.731Z"
updatedAt: "2026-10-10T00:16:28.648Z"
tags:
  - 线性代数
  - 精确算术
aliases:
  - 精确-cramer-求解与矩阵辅助
  - 精C求
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 精确 Cramer 求解与矩阵辅助
summary: domain_builtins.rs 的矩阵辅助提供行主序转置、行数据到 Matrix 值的转换，以及采用分数自由变量消元的精确 Cramer 求解。
sources:
  - atlas-core-deformation-cache.md
kind: concept
tags:
  - 精确线性代数
  - 矩阵算法
aliases:
  - 精确-cramer-求解与矩阵辅助
  - 精C求
---

# 精确 Cramer 求解与矩阵辅助

`crates/atlas-core/src/domain_builtins.rs` 的 2933–3022 行包含三个矩阵辅助函数：`transpose_matrix`、`matrix_value` 和 `cramer_solution`，分别负责矩阵转置、矩阵值构造与精确 Cramer 求解。^[atlas-core-deformation-cache.md:9-14, atlas-core-deformation-cache.md:52-55]

## 矩阵转置与值构造

`transpose_matrix` 提供行主序矩阵的转置；`matrix_value` 将行数据转换为 `Matrix` 值。源材料仅概述这两项功能，未展开输入维度校验、内部存储细节或错误处理约定。^[atlas-core-deformation-cache.md:52-55]

## 精确 Cramer 求解

`cramer_solution` 提供精确 Cramer 解，源材料将其方法描述为“分数自由变量消元”。材料未给出具体公式、消元步骤、数值表示或奇异矩阵的处理规则，因此本页仅记录功能与方法层面的说明。^[atlas-core-deformation-cache.md:54-55]

## 与形变机器的关系

这三个辅助函数与普通全形变、扭曲全形变及缓存函数共同纳入同一源码阅读包，相关背景可参见 [[全形变缓存与递归环检测]] 和 [[扭曲全形变计算流程]]。共同收录本身并未说明形变函数与矩阵辅助函数之间的具体调用关系。^[atlas-core-deformation-cache.md:9-14, atlas-core-deformation-cache.md:17-50]

## 证据边界

源材料属于结构性源码阅读，不声称数学验收，也未提供这三个矩阵辅助函数的独立验收结果。材料中的上游行号引用属于实现方的移植陈述；形变兼容性仍以 HPC 差分门为准，不能据此认定矩阵辅助函数已获得数学验收。^[atlas-core-deformation-cache.md:9-15, atlas-core-deformation-cache.md:52-65]

## Sources

- [atlas-core-deformation-cache.md — 形变缓存机器与矩阵辅助](../../sources/atlas-core-deformation-cache.md)
