---
title: Analysis 转换期上下文与活类型要求
summary: Analysis 管理词法绑定、常量、循环深度和类型下限，函数体与 return 操作数共享活转换单元，递归转换不得持有其 RefCell 借用。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:37:01.054Z"
updatedAt: "2026-10-09T22:21:18.814Z"
tags:
  - 类型检查
  - 词法作用域
aliases:
  - analysis-转换期上下文与活类型要求
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Analysis 转换期上下文与活类型要求
summary: Analysis 保存转换期绑定与词法约束，通过 ConversionType 共享函数体和 return 的活类型要求，并限制重载视图缓存的内容与上下文身份。
sources:
  - atlas-core-typed-core.md
kind: concept
tags:
  - 类型分析
  - 转换上下文
  - Rust
aliases:
  - analysis-转换期上下文与活类型要求
---

# Analysis 转换期上下文与活类型要求

`Analysis` 是类型化管线的转换期上下文，位于 `typed.rs` 的 476–555 行。它保存类型表、全局绑定和重载状态的引用，并管理局部绑定、常量标记、函数返回类型要求、循环深度与类型变量的词法阈值。^[atlas-core-typed-core.md:62-73]

## 上下文与词法约束

`types`、`globals`、`overloads` 提供转换所需的共享状态；`locals` 保存局部绑定，`constant_locals` 记录以 `!x` 形式声明的常量名，普通重绑可以解除常量标记。`type_floor` 规定词法阈值，低于该阈值的类型变量视为刚性变量。^[atlas-core-typed-core.md:64-71]

`loop_depth` 控制 `break` 的合法性：只有其非零时才允许 `break`，游离的 `break` 在分析期即被拒绝。^[atlas-core-typed-core.md:70-71]

## 活返回类型要求

`return_type` 表示最近一层函数的**活结果要求**，独立于当前表达式上下文；其上游对应物是 `layer::current_return_type`。因此，函数返回类型要求与当前子表达式的类型上下文需要分别保存。^[atlas-core-typed-core.md:68-69]

`ConversionType` 表示为 `Rc<RefCell<(Type, usize)>>`，是函数体与其 `return` 操作数共享的活转换单元。其明确使用约束是：**递归转换时绝不持有 `RefCell` 借用**。^[atlas-core-typed-core.md:72-73]

## 重载视图缓存

`overload_views` 是通过 `Rc` 共享的、未移位的有序签名视图缓存。它只缓存结构签名，不缓存推断类型或跨命令解析结果；相关状态管理见 [[OverloadState 有序重载管理]]。^[atlas-core-typed-core.md:64-67]

公开的 `Analysis` 引用可以在克隆对象上重新绑定，因此缓存通过 `std::ptr::eq` 检查类型表与重载表的同一性；若不匹配，就清空缓存。^[atlas-core-typed-core.md:66-67]

这一检查与 `OverloadState` 自身的缓存纪律应分别理解：后者的 `views` 由 `TypeTable` 的 `revision` Arc 守护，指针不同即清空，只提供不可变、未移位的签名与来源索引；其事务性 `Clone` 不携带缓存。^[atlas-core-typed-core.md:81-85]

## 绑定身份与类型精化

与转换上下文相邻的 `IdTable` 为每个名字保存一个绑定，每次定义分配新鲜 cell。转换后的代码保留捕获的 cell，重定义只更换名字所指的绑定。`TypeCell` 中的类型号按定义处的词法下限解读；克隆共享旧精化单元，导入用例不写入该单元。详见 [[IdTable 与 TypeCell 的绑定身份和类型精化]]。^[atlas-core-typed-core.md:75-77]

## 证据范围

本页依据类型化管线核心数据结构的结构性阅读，不声称完成语言或数学验收。来源包不覆盖 [[convert_expr 的 in/out 类型模式与单遍转换|convert_expr 转换遍]]、内建注册表或 `TypedExpr` 的求值实现；执行层与领域桥也另待分包。上游行号引用属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:108-112]

## Sources

- [atlas-core-typed-core.md](../../sources/atlas-core-typed-core.md) — 类型化管线核心数据结构（typed.rs 上部）。
