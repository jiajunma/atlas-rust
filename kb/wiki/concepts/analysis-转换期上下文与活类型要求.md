---
title: Analysis 转换期上下文与活类型要求
summary: Analysis 管理局部绑定、常量标记、循环深度和刚性类型阈值，函数体与 return 操作数通过 ConversionType 共享活类型要求且递归转换时不得持有 RefCell 借用。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:37:01.054Z"
updatedAt: "2026-10-09T14:37:01.054Z"
tags:
  - 类型分析
  - 转换上下文
  - Rust
aliases:
  - analysis-转换期上下文与活类型要求
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Analysis 转换期上下文与活类型要求

`Analysis` 是类型化管线的转换期上下文，位于 `typed.rs` 的 476–555 行。它集中保存类型表、全局绑定与重载状态的引用，以及局部绑定、函数返回类型要求、循环深度和类型变量的词法阈值。^[atlas-core-typed-core.md:62-73]

## 上下文与词法约束

`types`、`globals`、`overloads` 提供转换所需的共享状态；`locals` 记录局部绑定，`constant_locals` 记录以 `!x` 形式声明的常量名。普通重绑可以解除常量标记。`type_floor` 则规定类型变量的词法阈值，低于该阈值的变量视为刚性变量。^[atlas-core-typed-core.md:64-71]

`loop_depth` 控制 `break` 的合法性：只有其非零时才允许使用 `break`，游离的 `break` 在分析期即被拒绝。这使循环控制的合法性成为转换期检查的一部分。^[atlas-core-typed-core.md:70-71]

## 活返回类型要求

`return_type` 表示最近一层函数的**活结果要求**，独立于当前表达式上下文；其上游对应物是 `layer::current_return_type`。分析 `return` 时所需的函数结果要求，因此不会被当前子表达式的上下文替代。^[atlas-core-typed-core.md:68-69]

`ConversionType` 的表示为 `Rc<RefCell<(Type, usize)>>`，是函数体与其 `return` 操作数共享的活转换单元。实现要求在递归转换时绝不持有 `RefCell` 借用，这是该共享单元的明确使用约束。^[atlas-core-typed-core.md:72-73]

## 重载视图缓存

`overload_views` 是通过 `Rc` 共享的、未移位的有序签名视图缓存。它只缓存结构签名，绝不缓存推断类型或跨命令解析结果；相关重载管理机制见 [[OverloadState 有序重载管理]]。^[atlas-core-typed-core.md:64-67]

公开的 `Analysis` 引用可以在克隆对象上重新绑定，因此缓存使用 `std::ptr::eq` 检查类型表与重载表的同一性；检查不符时清空缓存。该规则约束缓存与所引用上下文之间的一致性。^[atlas-core-typed-core.md:66-67]

## 绑定身份与类型精化

与 `Analysis` 相邻的 [[IdTable 与 TypeCell 的绑定身份和类型精化]] 说明了绑定的延续方式：`IdTable` 每个名字对应一个绑定，每次定义分配新鲜 cell；转换后的代码保留捕获的 cell，重定义只替换名字所指的绑定。`TypeCell` 中的类型号按定义处的词法下限解读，克隆共享旧精化单元，而导入用例不写入该单元。^[atlas-core-typed-core.md:75-77]

## 证据范围

本页依据对类型化管线核心数据结构的结构性阅读。源码包不覆盖 [[convert_expr 的 in/out 类型模式与单遍转换]] 所涉及的转换遍、内建注册表或 `TypedExpr` 的求值实现，也不声称完成语言或数学验收。上游位置引用属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:108-114]

## Sources

- atlas-core-typed-core.md — 类型化管线核心数据结构（typed.rs 上部）
