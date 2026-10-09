---
title: TypedExpr 可执行表达式树
summary: 以类型化节点表达值、容器、读写、调用与控制流，并在节点结构中保留求值顺序和诊断信息；本包不覆盖求值实现。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:37:01.550Z"
updatedAt: "2026-10-09T14:37:01.550Z"
tags:
  - Rust
  - 类型化管线
  - 表达式树
aliases:
  - typedexpr-可执行表达式树
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# TypedExpr 可执行表达式树

`TypedExpr` 是类型化管线中的可执行表达式树，其节点涵盖值与容器、标识符读写、赋值、调用及控制流。源材料定位其定义于 `typed.rs` 的 124–425 行；该材料以结构性阅读为基础，不构成语言或数学验收。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:32-60]

## 值与容器

`Denotation`、`Captured`、`TupleDisplay` 和 `ListDisplay` 表示值及容器表达式。其中，`Captured` 为冻结的重载值保留表达式拼写，这与值自身的打印形式不同。`Conversion{tag,…}` 表示已注册的强制转换，打印为 `tag:expr`；`Void` 只求效果，并将结果值替换为 void。^[atlas-core-typed-core.md:34-38]

`BarList` 直接构造矩阵，用户定义的 `^` 或 `mat` 重载无法拦截这一构造。^[atlas-core-typed-core.md:36-38]

## 标识符与赋值

`GlobalIdent` 和 `LocalIdent` 分别通过 cell 或词法坐标访问绑定，并有对应的 `GlobalAssignment`、`LocalAssignment` 节点。全局绑定的定义分配新鲜 cell，转换后的代码保留所捕获的 cell；重定义只更换名称对应的绑定。相关身份语义见 [[IdTable 与 TypeCell 的绑定身份和类型精化]]。^[atlas-core-typed-core.md:40-45, atlas-core-typed-core.md:75-77]

`ComponentAssignment` 先求待赋值的值，再求下标，随后替换分量并产出该值；其 `source` 字段供越界诊断逐字引用。`ComponentTransform` 表示 `op:=`，范围检查发生在合成读取 `a[i]` 时，因此诊断引用选择式，例如 `in subscription v[5]`。字段更新由 `FieldAssignment` 和 `FieldTransform` 表示。^[atlas-core-typed-core.md:40-45]

`MultiAssignment` 先完整求值右侧，再按 `MultiAssignmentPlan` 后序分发：从左到右访问子项，整体目的地最后处理。多重赋值的全局目的地在分析时捕获 cell，局部目的地保留词法坐标，与普通赋值节点一致。^[atlas-core-typed-core.md:24-29, atlas-core-typed-core.md:45-45]

## 访问、调用与闭包

`Subscription` 的求值次序依访问类别而异：领域系数读取先求接收方，再求键；普通位置订阅先求下标。树中还包含切片节点 `Slice`，以及按 `(shape, initializer)` 组织每个绑定的 `LetGroup`。^[atlas-core-typed-core.md:47-48]

`BuiltinCall` 保存注册表索引，并使用 `name@argtype` 作为回溯名称。`HungryBuiltinCall` 在顶层饥饿操作数恰为赋值目的地时立即移出旧值。相关调用机制可参阅 [[函数调用分派与内建参数解包]]。^[atlas-core-typed-core.md:49-50]

`Closure` 求值时将当前帧链捕获为闭包值。`FunctionCall` 的动态被调对象可以是内建函数或闭包，参数始终作为一个值传入，多参数表现为元组；当 `name=None` 时，上游约定在回溯行打印被调表达式。^[atlas-core-typed-core.md:51-53]

## 控制流与循环

`Conditional` 表示条件表达式；`Sequence` 以前项的 `NoValue` 模式只求效果。`While` 和 `Do` 均由词法守卫帧包围，所需结果上下文通过 `WhileMode::Void`、`Count` 或 `Row{reversed}` 选择 while 求值器。相关转换规则见 [[while 转换的上下文模式与循环边界]]。^[atlas-core-typed-core.md:20-23, atlas-core-typed-core.md:49-54]

`For` 表示上游七种聚合形式的分量循环，下标类型随接收方而为 int、KType 或 Param；`CountedFor` 使用每次迭代帧中的可变计数器。`Next` 产出第一个值。^[atlas-core-typed-core.md:55-60]

控制节点还包括 `Return`、`Break`、`Dont` 和 `Die`。`Break` 的层数语义为 `levels+1` 层，分析期仅在 `loop_depth` 非零时允许使用；`Die` 可通过任何所需类型的分析，但求值时抛出 `I die`。^[atlas-core-typed-core.md:51-57, atlas-core-typed-core.md:68-71]

## 联合、投影与分支

`UnionInject` 和 `TupleProject` 用作 injector/projector 的闭包体。`Case` 选择首个 tag 匹配的分支，并按 shape 分发载荷；`UnionCase` 按变体位置应用分支函数。`IntCase` 对负值使用 then 分支，对越界值使用 else 分支，两者皆缺时取模。^[atlas-core-typed-core.md:57-60]

## 转换上下文与证据边界

与表达式树配套的 [[Analysis 转换期上下文与活类型要求]] 保存局部绑定、循环深度及最近函数的活结果要求。函数体与其 `return` 操作数通过 `ConversionType` 共享活转换单元；递归转换时不得持有该单元的 `RefCell` 借用。^[atlas-core-typed-core.md:62-73]

本材料覆盖节点结构，但不覆盖 `convert_expr` 转换遍、内建注册表或 `TypedExpr` 的求值实现，也不覆盖执行层 `EvaluationContext`、`frames.rs` 及领域桥。上游行号属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准；本页不能据此宣称完整运行时兼容。^[atlas-core-typed-core.md:108-114]

## Sources

- [atlas-core-typed-core.md](atlas-core-typed-core.md) — 类型化管线核心数据结构（typed.rs 上部）
