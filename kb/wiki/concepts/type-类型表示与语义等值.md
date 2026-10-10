---
title: Type 类型表示与语义等值
summary: Type 采用 tag+payload 表示，折叠单元素元组与联合；语义等值先校验构造器应用，再以递归类型的名义身份作为比较终止边界。
sources:
  - atlas-core-types.md
kind: concept
createdAt: "2026-10-09T14:38:06.029Z"
updatedAt: "2026-10-10T00:25:20.091Z"
tags:
  - 类型系统
  - 语义等值
aliases:
  - type-类型表示与语义等值
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Type 类型表示与语义等值
summary: Type 采用 tag+payload 表示并折叠单元素元组与联合；语义等值先校验构造器应用，再以递归名义身份作为比较终止边界。
sources:
  - atlas-core-types.md
kind: concept
tags:
  - 类型系统
  - 类型表示
  - 语义等值
aliases:
  - type-类型表示与语义等值
provenanceState: extracted
---

# Type 类型表示与语义等值

`Type` 是 Atlas Rust 语言层的类型表示，移植上游 `type_expr` 的 tag+payload 模型。空元组表示 `void`，单元素元组与联合由构造器折叠；字段名与变体名保存在 typedef 表中，递归类型按名字比较。^[atlas-core-types.md:12-17]

## 类型表示

`Type` 包含 `Undetermined`、`Variable(usize)`、`Primitive`、`Function(Box<(Type, Type)>)`、`Row`、`Tuple`、`Union`、`Tabled(TypeNumber)` 和 `Applied(TypeNumber, Vec<Type>)`。`Undetermined` 显示为 `*`，仅由 `specialise` 收窄；变量是否刚性由外围 scheme 的 `fixed` 阈值决定，不记录在变量节点上，参见 [[TypeScheme 与类型变量作用域]]。^[atlas-core-types.md:24-29]

`Prim` 提供 20 个原始类型，`Prim::ALL` 保持上游 `prim_names` 的顺序，`Prim::name()` 返回上游拼写，例如 `KgbElt` 对应 `"KGBElt"`。函数类型以元组承载多个参数；`Type::tuple` 与 `union_of` 折叠长度为 1 的输入。^[atlas-core-types.md:22-29]

`Tabled` 表示按名字进行名义比较的递归项。`Applied` 保留构造器的名字与实参，即使展开后的结构没有使用某个形式参数，也仍保留相应实参。字段与变体名称由 [[TypeTable 的稳定身份与活跃绑定|TypeTable]] 保存，而非嵌入类型结构。^[atlas-core-types.md:13-15, atlas-core-types.md:27-29, atlas-core-types.md:41-44]

## 展开与应用校验

`expanded` 通过 `Cow` 返回结果：普通类型借用原值，构造器替换时才拥有展开结果。展开保留子项的名字，包括递归应用；循环次数以绑定数为上界，使直接自环的残缺占位也能接受检查而不陷入死循环。^[atlas-core-types.md:30-32]

构造器应用的校验与展开分开处理。`validate_applications` 不展开定义，使递归名称的检查保持有限；`expand_application` 只展开一层，定义体内的递归应用仍保留为引用。^[atlas-core-types.md:54-56]

## 语义等值与递归边界

`equivalent` 判断语义等值，区别于文本等值或表槽位等值。由于命名头可能暴露不同形状，比较先对两侧执行 `validate_applications`，再进行结构递归。递归名称是终止边界：两侧都是递归项而名字不同时返回 `false`，同名同槽时返回 `true`。^[atlas-core-types.md:33-35]

应用校验也适用于等值应用和变量绑定。因此，即使比较或绑定可以走快路径，也不能绕过对残缺构造器应用的检查。^[atlas-core-types.md:54-56]

## 特化与失败行为

`specialise(&mut self, pattern)` 是唯一允许的变异路径，成功时得到最一般合一子（MGU）。失败时，`self` 可能已部分特化；这是上游语义，不提供“提交或回滚”保证。需要回滚的调用方应先使用 `can_specialise`。^[atlas-core-types.md:15-17, atlas-core-types.md:36-37]

进一步的替换与合一由 [[TypeAssignment 与 InferredType 推断机器]] 承担。`TypeAssignment` 在 `[fixed, fixed+degree)` 范围维护无环替换，`fixed` 以下的变量保持刚性；替换属于一次分析或重载试验，不属于全局绑定。^[atlas-core-types.md:66-77]

## 证据边界

来源覆盖 `types.rs`、`types/polymorphic.rs`、crate 私有的 `types/recursive.rs` 与 `types/revision_tests.rs`，属于结构性阅读，不声称语言验收。源码包记录了 59 个测试，其中 `types` 部分有 13 个；上游行号属于实现方的移植陈述，类型行为兼容仍以 HPC 语言语料门为准。^[atlas-core-types.md:9-18, atlas-core-types.md:84-91]

## Sources

- [atlas-core-types.md](../../sources/atlas-core-types.md) — 类型模型（types.rs + types/）。
