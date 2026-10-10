---
title: TypeScheme 与类型变量作用域
summary: TypeScheme 以 body、fixed 和 degree 描述类型方案，为不同洞分配新变量、保留重复变量共享，并保留构造器声明的元数及未用参数。
sources:
  - atlas-core-types.md
kind: concept
createdAt: "2026-10-09T14:38:33.320Z"
updatedAt: "2026-10-10T00:25:20.022Z"
tags:
  - 类型系统
  - 多态
  - 作用域
aliases:
  - typescheme-与类型变量作用域
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: TypeScheme 与类型变量作用域
summary: TypeScheme 以 body、fixed 和 degree 描述类型方案；变量刚性由外围作用域决定，wrap 保留变量共享关系，构造器方案保留声明元数，实例化保持刚性变量不变。
sources:
  - atlas-core-types.md
kind: concept
tags:
  - 多态
  - 类型变量
aliases:
  - typescheme-与类型变量作用域
---

# TypeScheme 与类型变量作用域

`TypeScheme` 是 Atlas 类型模型二阶机器中的类型方案，由 `body`、`fixed` 和 `degree` 组成。类型变量表示为 `Type::Variable(usize)`；变量是否刚性由外围方案的 `fixed` 阈值决定，而非变量节点自身的属性。类型表示背景见 [[Type 类型表示与语义等值]]。^[atlas-core-types.md:24-29, atlas-core-types.md:58-65]

## 作用域与方案构造

在配套的 `TypeAssignment` 中，编号小于 `fixed` 的变量为刚性变量，区间 `[fixed, fixed+degree)` 内的变量参与无环替换。变量编号的含义因此依赖外围作用域。^[atlas-core-types.md:66-70]

`TypeScheme::wrap` 按遍历中的首次出现顺序打包变量：不同的 `Undetermined` 洞获得不同的新变量，重复出现的显式变量号共享同一槽位。构造后的方案不含独立的 `Undetermined` 洞，同时保留显式变量的共享关系。^[atlas-core-types.md:63-65, atlas-core-types.md:76-77]

`TypeScheme::constructor` 保留声明的参数个数（arity）与参数编号，包括类型体中未使用的形式参数。相应地，`Type::Applied(TypeNumber, Vec<Type>)` 即使结构展开涉及未使用的形式参数，也保留构造器名称与实参。^[atlas-core-types.md:27-29, atlas-core-types.md:63-65]

## 实例化与推断约束

`TypeAssignment::instantiate` 导入方案的新鲜实例，并保持刚性变量不变。`append` 导入另一赋值时，连同其待决替换一起导入，否则会静默丢失推断约束。替换属于一次分析或重载试验，不属于全局绑定。^[atlas-core-types.md:66-77]

[[TypeAssignment 与 InferredType 推断机器|InferredType]] 将类型体与赋值配对，提供 `from_scheme`、`wrap`、`bake`、`wring_out`、`raise_floor` 和 `lower_floor` 等操作。二阶机器还提供 `substitute_parameters` 与 `shift(t, fixed, amount)`；相关错误包括 `ScopeCapture`、`VariableOutOfRange` 和 `UndeterminedInAssignment`。^[atlas-core-types.md:60-75]

合一失败可能留下部分变异：`TypeAssignment::unify` 延续这一上游语义，需要回滚时使用 `try_unify`。`Type::specialise` 同样可能在失败时留下部分特化，需要回滚的调用方应先用 `can_specialise` 检查。^[atlas-core-types.md:36-37, atlas-core-types.md:66-70]

## 候选匹配中的作用域隔离

`TypeTable::matching_bindings` 在全部保留定义中查找字段或标签元数据，而不限于活跃名称或当前投影重载。每个候选绑定都使用新鲜的形式构造器应用，保留接收方的刚性下限，并隔离候选的自由变量；保留定义的机制见 [[TypeTable 的稳定身份与活跃绑定]]。^[atlas-core-types.md:51-53]

## 证据边界

源材料记录 `polymorphic` 部分包含 37 个测试，但材料属于结构性阅读，不据此声称语言验收。上游行号引用属于实现方的移植陈述；类型行为兼容仍以 HPC 语言语料门为准。^[atlas-core-types.md:9-18, atlas-core-types.md:84-91]

## Sources

- [atlas-core-types.md](../../sources/atlas-core-types.md) — 类型模型（types.rs + types/）。
