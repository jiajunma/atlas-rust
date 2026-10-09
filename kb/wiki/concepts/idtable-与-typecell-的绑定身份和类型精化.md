---
title: IdTable 与 TypeCell 的绑定身份和类型精化
summary: 重定义分配新鲜绑定 cell，使旧代码保留原绑定；TypeCell 按定义处词法下限解释类型，克隆共享精化单元而导入用例不得写入。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:36:52.598Z"
updatedAt: "2026-10-09T22:21:14.914Z"
tags:
  - 绑定身份
  - 类型精化
aliases:
  - idtable-与-typecell-的绑定身份和类型精化
  - I与T的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: IdTable 与 TypeCell 的绑定身份和类型精化
summary: IdTable 为每次定义分配新鲜 cell，使已转换代码保留原绑定；TypeCell 按定义处词法下限解释类型，克隆共享精化单元，导入用例不得写入。
sources:
  - atlas-core-typed-core.md
kind: concept
tags:
  - 符号表
  - 词法绑定
  - 类型精化
aliases:
  - idtable-与-typecell-的绑定身份和类型精化
provenanceState: extracted
---

# IdTable 与 TypeCell 的绑定身份和类型精化

`IdTable` 管理名称与绑定的对应关系，`TypeCell` 关联绑定的类型号与精化单元。核心约定是：**名称重定义不改变旧代码捕获的绑定**，而类型号必须按**绑定定义处的词法下限**解释。^[atlas-core-typed-core.md:75-77]

## 绑定身份与重定义

`IdTable` 对每个名称维护一个绑定，每次定义都分配新鲜的 cell。转换后的代码保留已捕获的 cell；重定义只改变名称指向的绑定，因此旧代码仍持有原来的 cell。^[atlas-core-typed-core.md:75-77]

这一身份约定体现在 [[TypedExpr 可执行表达式树]] 中：`GlobalIdent` 保存 cell，`LocalIdent` 保存词法坐标。多重赋值目的地同样区分二者：全局目的地在分析时捕获 cell，局部目的地保留词法坐标，与普通赋值节点一致。^[atlas-core-typed-core.md:24-29, atlas-core-typed-core.md:40-45]

## 类型解释与精化共享

`TypeCell` 中绑定的类型号按该绑定定义处的词法下限解读。相关的 [[Analysis 转换期上下文与活类型要求]] 通过 `type_floor` 表示词法阈值，阈值以下的类型变量视为刚性变量。^[atlas-core-typed-core.md:70-77]

克隆 `TypeCell` 时共享原有的精化单元，不建立独立的精化副本；导入用例绝不写入该共享单元。这规定了克隆的共享语义与导入时的写入边界。^[atlas-core-typed-core.md:75-77]

## 绑定位置与诊断

[[TypedContext 会话状态与启动初始化]] 的 `type_locations` 将位置关联到当前标识符绑定，而非可能被复用的类型表槽。因此，名称重定义后，诊断必须报告新绑定的位置。^[atlas-core-typed-core.md:100-106]

## 证据范围

本页依据 `typed.rs` 上部数据结构的结构性阅读。来源未覆盖 `convert_expr` 转换遍、`TypedExpr` 求值实现或测试部分，不据此声称语言或数学验收；上游行号对应关系属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:108-114]

## Sources

- [类型化管线核心数据结构（typed.rs 上部）——TypedExpr 树、Analysis/OverloadState 与 TypedContext](../../sources/atlas-core-typed-core.md)
