---
title: IdTable 与 TypeCell 的绑定身份和类型精化
summary: IdTable 为新定义分配新鲜 cell，使旧代码保留原绑定；TypeCell 按定义处词法下限解释类型，克隆共享精化单元而导入用例不得写入。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:36:52.598Z"
updatedAt: "2026-10-09T14:36:52.598Z"
tags:
  - 符号表
  - 词法绑定
  - 类型精化
aliases:
  - idtable-与-typecell-的绑定身份和类型精化
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# IdTable 与 TypeCell 的绑定身份和类型精化

`IdTable` 管理名称与绑定的对应关系，`TypeCell` 保存绑定的类型信息及其精化单元。两者的关键约定是：重定义名称不会替换已被转换代码捕获的旧 cell；绑定的类型号始终按其定义处的词法下限解释。^[atlas-core-typed-core.md:75-77]

## 绑定身份与重定义

`IdTable` 对每个名称维护一个绑定，每次定义都分配新鲜的 cell。转换后的代码保留其捕获的 cell，重定义只更换名称所指向的绑定，因此旧代码仍持有原来的 cell。^[atlas-core-typed-core.md:75-77]

[[TypedExpr 可执行表达式树]] 的标识符访问采用两种表示：`GlobalIdent` 保存 cell，`LocalIdent` 保存词法坐标。多重赋值的目的地也遵循这一约定：全局目的地在分析时捕获 cell，局部目的地保留词法坐标，与普通赋值节点一致。^[atlas-core-typed-core.md:24-29, atlas-core-typed-core.md:40-45]

## 类型解释与精化共享

`TypeCell` 中绑定的类型号按**该绑定定义处**的词法下限解读。相关的 [[Analysis 转换期上下文与活类型要求]] 使用 `type_floor` 表示词法阈值，阈值以下的类型变量视为刚性变量。^[atlas-core-typed-core.md:70-77]

克隆 `TypeCell` 会共享原有的精化单元，而不是建立独立的精化副本；导入用例绝不写入该共享单元。这同时规定了克隆后的共享关系与导入时的写入边界。^[atlas-core-typed-core.md:75-77]

## 绑定位置与诊断

绑定身份也影响诊断位置。[[TypedContext 会话状态与启动初始化]] 的 `type_locations` 将位置关联到当前标识符绑定，而不是可能复用的类型表槽，因此重定义后必须报告新绑定的位置。^[atlas-core-typed-core.md:100-106]

## 证据范围

上述说明来自 `typed.rs` 上部数据结构的结构性阅读。源材料未覆盖 `convert_expr` 转换遍或 `TypedExpr` 的求值实现，也不据此声称语言或数学验收；行为兼容仍以 HPC 语料门为准。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:108-114]

## Sources

- [类型化管线核心数据结构（typed.rs 上部）——TypedExpr 树、Analysis/OverloadState 与 TypedContext](atlas-core-typed-core.md)
