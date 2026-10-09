---
title: 精确 Cramer 求解与矩阵辅助
summary: 矩阵辅助接口提供行主序转置、行数据到 Matrix 值的转换，以及采用分数自由变量消元的精确 Cramer 求解。
sources:
  - atlas-core-deformation-cache.md
kind: concept
createdAt: "2026-10-09T14:27:59.731Z"
updatedAt: "2026-10-09T22:14:11.380Z"
tags:
  - 精确线性代数
  - 矩阵算法
aliases:
  - 精确-cramer-求解与矩阵辅助
  - 精C求
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 精确 Cramer 求解与矩阵辅助
summary: domain_builtins.rs 中的矩阵辅助提供行主序转置、行数据到 Matrix 值的转换，以及采用分数自由变量消元的精确 Cramer 求解。
sources:
  - atlas-core-deformation-cache.md
kind: concept
tags:
  - 线性代数
  - 精确计算
  - 矩阵
aliases:
  - 精确-cramer-求解与矩阵辅助
  - 精C求
---

# 精确 Cramer 求解与矩阵辅助

`crates/atlas-core/src/domain_builtins.rs` 的 2933–3022 行包含三个矩阵辅助函数：`transpose_matrix`、`matrix_value` 和 `cramer_solution`，分别提供矩阵转置、矩阵值构造与精确 Cramer 求解。^[atlas-core-deformation-cache.md:9-14, atlas-core-deformation-cache.md:52-55]

## 矩阵转置与值构造

`transpose_matrix` 实现行主序矩阵的转置；`matrix_value` 将行数据转换为 `Matrix` 值。源材料仅概述这两项功能，没有展开输入维度校验、内部存储细节或错误处理约定。^[atlas-core-deformation-cache.md:52-55]

## 精确 Cramer 求解

`cramer_solution` 提供精确 Cramer 解，源材料将其方法描述为“分数自由变量消元”。材料未给出具体公式、消元步骤、数值表示或奇异矩阵的处理规则，因此本页仅记录功能与方法层面的说明。^[atlas-core-deformation-cache.md:54-55]

## 形变机器上下文

这三个辅助函数与普通全形变、扭曲全形变及缓存函数共同纳入同一源码阅读包，相关背景可参见 [[全形变缓存与递归环检测]] 和 [[扭曲全形变计算流程]]。这种共同收录并未说明形变函数与矩阵辅助函数之间的具体调用关系。^[atlas-core-deformation-cache.md:9-14, atlas-core-deformation-cache.md:17-50]

## 证据边界

源材料属于结构性源码阅读，不声称数学验收。其中的上游行号引用是实现方的移植陈述，形变兼容性仍以 HPC 差分门为准；本材料未提供这三个矩阵辅助函数的独立数学验收结果。^[atlas-core-deformation-cache.md:9-15, atlas-core-deformation-cache.md:57-65]

## Sources

- [形变缓存机器（domain_builtins.rs 2657–2933）——full/twisted full deformation 的递归、缓存纪律与协作截止](../../sources/atlas-core-deformation-cache.md)
