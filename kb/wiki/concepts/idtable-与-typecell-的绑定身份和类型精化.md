---
title: IdTable 与 TypeCell 的绑定身份和类型精化
summary: 重定义分配新鲜绑定 cell，使旧代码保留原绑定；类型按定义处词法下限解释，克隆共享精化单元且导入用例不得写入。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:36:52.598Z"
updatedAt: "2026-10-10T00:24:14.883Z"
tags:
  - 绑定身份
  - 类型推断
aliases:
  - idtable-与-typecell-的绑定身份和类型精化
  - I与T的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: IdTable 与 TypeCell 的绑定身份和类型精化
summary: IdTable 为每次定义分配新鲜 cell，使已转换代码保留原绑定；TypeCell 按定义处词法下限解释类型，克隆共享精化单元，导入用例不得写入。
sources:
  - atlas-core-typed-core.md
kind: concept
tags:
  - 绑定身份
  - 类型精化
aliases:
  - idtable-与-typecell-的绑定身份和类型精化
provenanceState: extracted
---

# IdTable 与 TypeCell 的绑定身份和类型精化

`IdTable` 管理名称与绑定的对应关系，`TypeCell` 保存绑定的类型信息及其精化共享关系。核心约定是：**重定义只改变名称所指向的绑定，已转换代码仍保留原先捕获的 cell**；类型号则始终按绑定定义处的词法下限解释。^[atlas-core-typed-core.md:75-77]

## 绑定身份与重定义

`IdTable` 对每个名称维护一个绑定，每次定义分配新鲜的 cell。转换后的代码保留所捕获的 cell，因此名称重定义不会将旧代码的引用自动转向新绑定。^[atlas-core-typed-core.md:75-77]

这一约定与 [[TypedExpr 可执行表达式树]] 的读写节点一致：`GlobalIdent` 保存 cell，`LocalIdent` 保存词法坐标。多重赋值目的地也采用相同区分，全局目的地在分析时捕获 cell，局部目的地保留词法坐标。^[atlas-core-typed-core.md:24-29, atlas-core-typed-core.md:40-45]

## 类型解释与精化共享

`TypeCell` 中绑定的类型号按**该绑定定义处**的词法下限解读。相关的 [[Analysis 转换期上下文与活类型要求]] 使用 `type_floor` 表示词法阈值，阈值以下的类型变量视为刚性变量。^[atlas-core-typed-core.md:70-77]

克隆 `TypeCell` 时共享原有的精化单元，而导入用例绝不写入该单元。这同时规定了克隆的共享语义和导入时的写入边界。^[atlas-core-typed-core.md:75-77]

## 绑定位置与诊断

[[TypedContext 会话状态与启动初始化]] 的 `type_locations` 将位置关联到当前标识符绑定，而非可能被复用的类型表槽。因此，名称重定义后，诊断必须报告新绑定的位置。^[atlas-core-typed-core.md:100-106]

## 证据范围

本页依据 `typed.rs` 上部数据结构的结构性阅读。来源不覆盖 `convert_expr` 转换遍、内建注册表、`TypedExpr` 求值实现及测试部分，不据此声称语言或数学验收。上游行号对应关系属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:108-114]

## Sources

- [atlas-core-typed-core.md：类型化管线核心数据结构（typed.rs 上部）——TypedExpr 树、Analysis/OverloadState 与 TypedContext](../../sources/atlas-core-typed-core.md)
