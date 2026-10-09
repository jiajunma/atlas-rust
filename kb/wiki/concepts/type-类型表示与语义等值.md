---
title: Type 类型表示与语义等值
summary: Type 采用 tag+payload 表示，折叠单元素元组与联合；语义等值比较先校验构造器应用，并以递归类型的名义身份作为终止边界。
sources:
  - atlas-core-types.md
kind: concept
createdAt: "2026-10-09T14:38:06.029Z"
updatedAt: "2026-10-09T14:38:06.029Z"
tags:
  - 类型系统
  - 类型表示
  - 语义等值
aliases:
  - type-类型表示与语义等值
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Type 类型表示与语义等值

`Type` 是 Atlas Rust 语言层的类型表示，采用上游 `type_expr` 的 tag+payload 模型。其表示约定包括：空元组表示 `void`，单元素元组与联合由构造器折叠，字段名与变体名保存在 typedef 表中，递归类型按名字比较。该源码包完成了结构性阅读，不据此声称语言验收。^[atlas-core-types.md:9-18]

## 类型表示

`Type` 包含以下形式：`Undetermined`、`Variable(usize)`、`Primitive`、`Function(Box<(Type, Type)>)`、`Row`、`Tuple`、`Union`、`Tabled(TypeNumber)` 和 `Applied(TypeNumber, Vec<Type>)`。其中，`Undetermined` 显示为 `*`，仅由 `specialise` 收窄；变量是否刚性由外围 scheme 的 `fixed` 阈值决定，不记录在变量节点上。相关作用域规则见 [[TypeScheme 与类型变量作用域]]。^[atlas-core-types.md:24-29]

`Primitive` 使用 `Prim` 表示的 20 个原始类型；`Prim::ALL` 保持上游 `prim_names` 的顺序，`Prim::name()` 返回上游拼写，例如 `KgbElt` 对应 `"KGBElt"`。函数类型以元组承载多个参数；空元组表示 `void`，`Type::tuple` 与 `union_of` 会折叠长度为 1 的输入。^[atlas-core-types.md:22-29]

`Tabled` 表示按名字进行名义比较的递归项。`Applied` 同时保存类型编号和实参，即使展开后的结构没有使用某个形式参数，也仍保留构造器名字及其全部实参。类型中的字段与变体名称由 [[TypeTable 的稳定身份与活跃绑定|TypeTable]] 管理，而非嵌入类型结构。^[atlas-core-types.md:13-15, atlas-core-types.md:27-29, atlas-core-types.md:41-44]

## 展开与递归边界

`expanded` 通过 `Cow` 返回结果：普通类型以借用形式返回，构造器替换时才拥有展开结果。展开保留子项的名字，包括递归应用；循环次数以绑定数为上界，使直接自环的残缺占位也能接受检查而不陷入死循环。^[atlas-core-types.md:30-32]

构造器应用的校验与展开分开处理。`validate_applications` 不展开定义，从而保持递归名称的检查有限；`expand_application` 只展开一层，定义体内的递归应用仍保留为引用。^[atlas-core-types.md:54-56]

## 语义等值

`equivalent` 判断语义等值，区别于文本相同或表槽位相同。由于命名头展开后可能暴露不同形状，比较会先对两侧执行 `validate_applications`，再进行结构递归。递归名称构成终止边界：两侧都是递归项而名字不同时返回 `false`，同名同槽时返回 `true`。^[atlas-core-types.md:33-35]

构造器应用校验也适用于等值应用和变量绑定，因此比较或绑定中的快路径不能绕过残缺构造器检查。这使结构比较能够保留递归引用，同时执行应用合法性检查。^[atlas-core-types.md:54-56]

## 特化与变异契约

`specialise(&mut self, pattern)` 是唯一允许的变异路径，成功时得到最一般合一子（MGU）。失败时，`self` 可能已经部分特化；这延续上游语义，并不提供“提交或回滚”保证。需要回滚语义的调用方应先使用 `can_specialise`。进一步的赋值与推断机制见 [[TypeAssignment 与 InferredType 推断机器]]。^[atlas-core-types.md:15-17, atlas-core-types.md:36-37]

## 证据边界

源码包记录了 59 个测试，其中 `types` 部分有 13 个。上游行号引用属于实现方的移植陈述，类型行为兼容仍以 HPC 语言语料门为准，不能仅凭结构性阅读或测试数量认定兼容性验收完成。^[atlas-core-types.md:84-91]

## Sources

- [atlas-core-types.md](atlas-core-types.md)
