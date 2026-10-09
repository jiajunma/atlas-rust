---
title: TypeTable 的稳定身份与活跃绑定
summary: TypeTable 将保留的类型定义与活跃名称映射分离，forget 仅移除活名，字段与标签匹配仍搜索全部保留定义并隔离候选自由变量。
sources:
  - atlas-core-types.md
kind: concept
createdAt: "2026-10-09T14:38:18.342Z"
updatedAt: "2026-10-09T14:38:18.342Z"
tags:
  - 类型系统
  - 符号表
  - 名称绑定
aliases:
  - typetable-的稳定身份与活跃绑定
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# TypeTable 的稳定身份与活跃绑定

`TypeTable` 将保留的类型绑定、当前活跃的名称映射和修订身份分开管理。`forget` 只移除活跃名称，已存值仍保留旧类型；字段与标签查找也可访问全部保留定义。这使类型名称的当前可见性与既有类型引用的保留相互独立。^[atlas-core-types.md:39-53]

## 存储结构与身份

类型表的 `bindings: Vec<TypeBinding>` 保存绑定名称、定义及位置字段或 injector 名称，其中 `None` 表示匿名分量。`active: BTreeMap<String, TypeNumber>` 单独维护活跃名称到类型编号的映射。`constructors: BTreeMap<usize, (usize, bool)>` 仅记录新构造器项，旧的 tabled 项继续保持名义递归行为。^[atlas-core-types.md:41-44]

[[Type 类型表示与语义等值|Type]] 通过 `Tabled(TypeNumber)` 引用递归类型，并按名字进行名义比较；`Applied(TypeNumber, Vec<Type>)` 则保存构造器名称与实参，即使展开结构中存在未使用的形式参数也不丢弃它们。因此，绑定引用的含义不能仅由展开后的结构决定。^[atlas-core-types.md:24-29]

## 活跃绑定的维护

`add` 与 `update` 支持类型绑定的建立和更新。括号形式的 `set_type` 分两阶段处理：先注册整组名称，再解析各个右侧定义，以支持递归引用。类型表还提供 `add_constructor`；`add_simple` 则按名称、定义与 arity 去重，并搬移字段元数据。^[atlas-core-types.md:48-50]

`forget` 只摘除活跃名称，不清除已存值所持有的旧类型。因此，名称退出当前活跃映射并不意味着所有既有类型引用同时失效。^[atlas-core-types.md:48-50]

## 保留定义与成员查找

`matching_bindings` 在全部保留定义中查找字段或标签元数据，范围不限于活跃名称或当前投影重载。每个候选绑定都使用一个新鲜的形式构造器应用进行尝试，同时保留接收方的刚性下限，并隔离候选的自由变量。这一机制是[[基于保留类型定义的投影解析]]的重要基础。^[atlas-core-types.md:51-53]

## 修订身份与缓存

`revision: Arc<()>` 表示非语义的快照身份：克隆共享这一身份，发生突变时则换用新身份。缓存读者持有该身份以防止地址复用，`Arc` 同时保持 `TypeTable` 的 `Send + Sync`。它与类型本身的语义身份职责不同，相关缓存机制见[[TypeTable 修订身份与缓存失效]]。^[atlas-core-types.md:44-47]

## 构造器应用的校验边界

`validate_applications` 在不展开定义的情况下校验构造器应用，使递归名称保持有限表示。即使进入等值应用或变量绑定的处理路径，也必须执行校验，不能借助快路径接受残缺构造器。`expand_application` 只展开一层，定义体内的递归应用仍保留为引用。^[atlas-core-types.md:54-56]

## 证据范围

来源属于结构性阅读，不据此声称整体语言验收。来源另将修订身份设计关联到已验收的跨命令缓存门；上游行号属于实现方的移植陈述，类型行为兼容性仍以 [[HPC 验收证据链|HPC 语言语料门]]为准。^[atlas-core-types.md:9-18, atlas-core-types.md:86-91]

## Sources

- [atlas-core-types.md](atlas-core-types.md) — 类型模型（types.rs + types/）
