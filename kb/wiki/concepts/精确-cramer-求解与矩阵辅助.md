---
title: 精确 Cramer 求解与矩阵辅助
summary: 矩阵辅助函数提供行主序转置、行数据到 Matrix 值的转换，以及采用分数自由变量消元的精确 Cramer 求解。
sources:
  - atlas-core-deformation-cache.md
kind: concept
createdAt: "2026-10-09T14:27:59.731Z"
updatedAt: "2026-10-09T14:27:59.731Z"
tags:
  - 线性代数
  - 精确计算
  - 矩阵
aliases:
  - 精确-cramer-求解与矩阵辅助
  - 精C求
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 精确 Cramer 求解与矩阵辅助

形变机器源码中的矩阵辅助位于 `crates/atlas-core/src/domain_builtins.rs` 的 2933–3022 行，包含 `transpose_matrix`、`matrix_value` 与 `cramer_solution`，分别承担矩阵转置、矩阵值构造和精确求解。^[atlas-core-deformation-cache.md:52-55]

## 矩阵转置与值构造

`transpose_matrix` 实现行主序矩阵的转置；`matrix_value` 将行数据转换为 `Matrix` 值。来源仅概述这两个函数的用途，未展开其输入校验或错误处理。^[atlas-core-deformation-cache.md:52-55]

## 精确 Cramer 求解

`cramer_solution` 提供精确 Cramer 解，来源将其方法描述为“分数自由变量消元”。该概述未给出具体公式、消元步骤或奇异矩阵的处理约定，因此不能据此补充这些算法细节。^[atlas-core-deformation-cache.md:54-55]

## 上下文与证据边界

这组辅助函数与普通、扭曲全形变的计算及缓存函数一起纳入同一源码阅读包；相关主题包括 [[全形变缓存与递归环检测]] 和 [[扭曲全形变计算流程]]。来源并未说明各形变函数如何调用这些矩阵辅助。^[atlas-core-deformation-cache.md:9-15]

现有证据属于结构性阅读，不构成数学验收。来源还明确指出，上游行号引用属于实现方的移植陈述，形变兼容性需以 HPC 差分门为准；这一限制也应在理解相关辅助函数时保留。^[atlas-core-deformation-cache.md:15-15, atlas-core-deformation-cache.md:62-64]

## Sources

- [形变缓存机器（domain_builtins.rs 2657–2933）——full/twisted full deformation 的递归、缓存纪律与协作截止](atlas-core-deformation-cache.md)
