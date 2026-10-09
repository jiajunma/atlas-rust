---
title: TypeAssignment 与 InferredType 推断机器
summary: TypeAssignment 管理局部无环替换、实例化与合一并连同待决替换导入赋值；InferredType 将类型体与赋值配对，支持作用域调整和类型匹配。
sources:
  - atlas-core-types.md
kind: concept
createdAt: "2026-10-09T14:38:33.600Z"
updatedAt: "2026-10-09T20:46:13.037Z"
tags:
  - 类型推断
  - 多态
  - 合一
aliases:
  - typeassignment-与-inferredtype-推断机器
  - T与I推
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: TypeAssignment 与 InferredType 推断机器
summary: TypeAssignment 管理局部无环替换、实例化与合一，append 连同待决替换导入赋值；InferredType 将类型体与赋值配对，提供作用域调整、合一和匹配接口。
sources:
  - atlas-core-types.md
kind: concept
tags:
  - 类型推断
  - 多态
  - 替换
---

# TypeAssignment 与 InferredType 推断机器

`TypeAssignment` 与 `InferredType` 位于 `types/polymorphic.rs` 的二阶类型机器中。前者管理类型变量的无环替换、实例化与合一，后者将类型体与赋值配对，提供推断和匹配接口。替换属于一次分析或重载试验，不属于全局绑定。^[atlas-core-types.md:58-77]

## 变量作用域与实例化

`TypeAssignment` 表示变量区间 `[fixed, fixed + degree)` 上的无环替换；编号低于 `fixed` 的变量是刚性的。刚性由外围 scheme 的 `fixed` 阈值决定，不存储在 `Type::Variable` 节点上。相关模型见 [[TypeScheme 与类型变量作用域]]。^[atlas-core-types.md:24-25, atlas-core-types.md:66-70]

`instantiate` 导入 scheme 的新鲜用例，保持刚性变量不变。`append` 导入另一赋值时，会连同其待决替换一起导入；遗漏这些替换会静默丢弃已有的推断约束。^[atlas-core-types.md:66-70]

scheme 不保留独立的 `Undetermined` 洞：`wrap` 按遍历和首次出现顺序打包，每个独立洞获得新变量，重复的显式变量号共享槽位。构造器 scheme 则保留声明的参数个数与参数编号，包括未使用的参数。^[atlas-core-types.md:63-65, atlas-core-types.md:76-77]

## 合一与失败语义

`TypeAssignment` 提供 `unify` 与 `try_unify`。普通合一失败时可能已经发生部分变异，这是沿用上游的行为；需要失败回滚的调用方使用 `try_unify`。因此，不能把普通合一的失败理解为赋值保持原状。^[atlas-core-types.md:66-70]

这一失败纪律也适用于 `Type::specialise`：成功时得到最一般合一子（MGU），失败时接收方可能已部分特化；需要回滚的场景先使用 `can_specialise`。参见 [[类型特化与失败回滚语义]]。^[atlas-core-types.md:36-37]

## InferredType 接口

`InferredType` 将类型体与赋值组合为一对。其构造与整理接口包括 `from_scheme`、`wrap`、`bottom`、`wrap_tuple`、`bake` 和 `wring_out`，并提供 `raise_floor` 与 `lower_floor`。^[atlas-core-types.md:71-73]

合一相关接口包括 `unify_to`、`try_unify_to`、`unify` 与 `has_unifier`；函数类型相关接口包括 `function_parts`、`matches_argument` 与 `matches_result`；此外还有 `unify_specialise`、`try_unify_specialise`，以及用于构造器形式匹配的 `matches`。源材料仅列出这些接口，未逐项展开内部算法。^[atlas-core-types.md:71-75]

## 错误与证据边界

二阶机器的 `TypeError` 包括 `IndexOverflow`、`ScopeCapture`、`VariableOutOfRange`、`UndeterminedInAssignment`、`UnknownConstructor` 和 `Arity` 等错误，涵盖索引、作用域、变量范围以及构造器与参数个数等边界。^[atlas-core-types.md:58-61]

源包记录类型模块共 59 个测试，其中 `polymorphic` 部分有 37 个。该材料属于结构性阅读，不声称完成语言验收；上游行号属于实现方的移植陈述，类型行为兼容仍以 HPC 语言语料门为准。^[atlas-core-types.md:9-18, atlas-core-types.md:84-91]

## Sources

- [atlas-core-types.md](../../sources/atlas-core-types.md) — 类型模型（types.rs + types/）。
