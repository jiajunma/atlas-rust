---
title: TypeTable 的稳定身份与活跃绑定
summary: TypeTable 分离保留定义与活跃名称，forget 仅移除活名，字段和标签匹配仍遍历全部保留定义并隔离候选自由变量。
sources:
  - atlas-core-types.md
kind: concept
createdAt: "2026-10-09T14:38:18.342Z"
updatedAt: "2026-10-10T00:25:19.359Z"
tags:
  - 类型系统
  - 名称绑定
aliases:
  - typetable-的稳定身份与活跃绑定
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: TypeTable 的稳定身份与活跃绑定
summary: TypeTable 分离保留定义与活跃名称，成员匹配遍历全部保留定义，独立的修订身份用于标识缓存依赖的快照。
sources:
  - atlas-core-types.md
kind: concept
tags:
  - 类型系统
  - 绑定管理
aliases:
  - typetable-的稳定身份与活跃绑定
---

# TypeTable 的稳定身份与活跃绑定

`TypeTable` 分别管理保留的类型绑定、当前活跃名称和修订身份。`forget` 只摘除活跃名称，已存值仍保留旧类型；字段与标签匹配继续搜索全部保留定义。因此，名称当前是否可见与既有类型是否保留是分开管理的。^[atlas-core-types.md:39-53]

## 存储结构与类型身份

`bindings: Vec<TypeBinding>` 保存名称、定义以及位置字段或 injector 名称，其中 `None` 表示匿名分量。`active: BTreeMap<String, TypeNumber>` 保存活跃名称映射；`constructors: BTreeMap<usize, (usize, bool)>` 仅记录新构造器项，旧的 tabled 项保留名义递归行为。^[atlas-core-types.md:41-44]

在 [[Type 类型表示与语义等值]]中，`Tabled(TypeNumber)` 表示按名字进行名义比较的递归项；`Applied(TypeNumber, Vec<Type>)` 保留构造器名称和实参，即使结构展开包含未使用的形式参数。变体与字段名称存放在 typedef 表中，而非类型节点内。^[atlas-core-types.md:13-15, atlas-core-types.md:24-29]

## 绑定维护与递归定义

`add` 与 `update` 用于维护绑定。括号形式的 `set_type` 分两阶段处理：先注册整组名称，再解析各个右侧定义，以支持递归。类型表还提供 `add_constructor`；`add_simple` 按名称、定义和参数个数（arity）去重，并搬移字段元数据。^[atlas-core-types.md:48-50]

`forget` 仅移除活跃名称，已存值继续保留旧类型。图级递归 typedef 安装优先安排命名右侧定义的槽位，并为环上的匿名后代保留身份；调用方将类型表与全部生成成员一起暂存。^[atlas-core-types.md:48-50, atlas-core-types.md:79-82]

## 保留定义与成员匹配

`matching_bindings` 在**全部保留定义**中查找字段或标签元数据，范围不限于活跃名称或当前投影重载。每个候选绑定都用一个新鲜的形式构造器应用进行尝试，同时保留接收方的刚性下限，并隔离候选的自由变量；相关主题见 [[基于保留类型定义的投影解析]]。^[atlas-core-types.md:51-53]

类型变量的刚性由外围 scheme 的 `fixed` 阈值决定，不是变量节点自身的属性。替换属于一次分析或重载试验，不属于全局绑定；相关机制见 [[TypeScheme 与类型变量作用域]]及 [[TypeAssignment 与 InferredType 推断机器]]。^[atlas-core-types.md:24-25, atlas-core-types.md:63-77]

## 修订身份与缓存

`revision: Arc<()>` 是**非语义的快照身份**：克隆共享该身份，突变时更换身份。缓存读者持有它以阻止地址复用，`Arc` 同时保持 `TypeTable` 的 `Send + Sync`。这一身份用于缓存快照识别，相关说明见 [[TypeTable 修订身份与缓存失效]]。^[atlas-core-types.md:44-47]

## 构造器应用的校验边界

`validate_applications` 在不展开定义的情况下校验应用，使递归名称保持有限表示。即使处理等值应用或变量绑定，也仍须校验，快路径不能成为接受残缺构造器的理由。`expand_application` 只展开一层，定义体内的递归应用继续保留为引用。^[atlas-core-types.md:54-56]

## 证据范围

来源属于结构性阅读，不据此声称整体语言验收。来源将修订身份设计关联到已验收的跨命令缓存门，并指向 `tests/reference/hpc/` 的命令链验收记录及 `docs/HANDOFF.md` 索引；上游行号属于实现方的移植陈述，类型行为兼容性仍以 HPC 语言语料门为准。^[atlas-core-types.md:9-18, atlas-core-types.md:86-91]

## Sources

- [atlas-core-types.md](../../sources/atlas-core-types.md) — 类型模型（types.rs + types/）。
