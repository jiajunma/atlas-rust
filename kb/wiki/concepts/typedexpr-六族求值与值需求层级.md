---
title: TypedExpr 六族求值与值需求层级
summary: evaluate 按原子、结构、应用、变更、容器和控制六族分派，通过 Level（NoValue/SingleValue）传递值构造需求。
sources:
  - atlas-core-typed-eval.md
kind: concept
createdAt: "2026-10-09T14:37:32.891Z"
updatedAt: "2026-10-09T20:45:26.276Z"
tags:
  - 求值器
  - 类型化表达式
aliases:
  - typedexpr-六族求值与值需求层级
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: TypedExpr 六族求值与值需求层级
summary: TypedExpr::evaluate 按原子、结构、应用、变更、容器和控制六族分派，与转换遍同形，并通过 Level（NoValue/SingleValue）控制值的构造需求。
sources:
  - atlas-core-typed-eval.md
kind: concept
createdAt: "2026-10-09T14:37:32.891Z"
updatedAt: "2026-10-09T14:37:32.891Z"
tags:
  - 表达式求值
  - 类型化表达式
  - 运行时
aliases:
  - typedexpr-六族求值与值需求层级
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# TypedExpr 六族求值与值需求层级

`TypedExpr` 的求值入口为 `evaluate(context, level) -> Result<Option<Value>, Control>`。入口按表达式变体分派到六个求值族，分区与转换遍一致；`Level` 沿求值过程下传，决定每个值生产者需要构造多少值。^[atlas-core-typed-eval.md:14-31]

## 六族求值

六个求值函数均标注 `#[inline(never)]`，采用与转换遍相同的栈帧纪律。六族覆盖如下，可结合 [[类型化转换与求值管线]] 理解其与转换阶段的对应关系。^[atlas-core-typed-eval.md:16-29]

| 求值函数 | 覆盖的表达式变体 |
| --- | --- |
| `evaluate_atom` | Denotation、Captured、GlobalIdent、LocalIdent、Closure、Return、Break、Dont、Die |
| `evaluate_structure` | TupleDisplay、ListDisplay、Conversion、Void、UnionInject、TupleProject |
| `evaluate_application` | LetGroup、Conditional、BuiltinCall、HungryBuiltinCall、FunctionCall、Sequence、Next |
| `evaluate_mutation` | Global/Local/MultiAssignment，以及 Component/Field 的 Assignment/Transform |
| `evaluate_container` | Subscription、Slice、BarList |
| `evaluate_control` | While、Do、For、Case、IntCase、UnionCase、CountedFor |

表中的分组描述求值入口按变体进行的分派。^[atlas-core-typed-eval.md:20-29]

## 值需求层级

`Level` 包含 `NoValue` 与 `SingleValue` 两种层级，作为求值入口的显式参数一路下传，控制值生产者的构造需求。^[atlas-core-typed-eval.md:16-31]

闭包中的 `return` 在当前调用边界被解开：`Err(Control::Return(value))` 经由 `at_level(level, …)` 按本次调用的需求供值，因此返回值处理也遵循传入的 `Level`。相关控制边界见 [[闭包返回控制与调用错误迹]]。^[atlas-core-typed-eval.md:46-53]

## 调用边界与参数处理

`apply_function` 本身不添加调用迹：闭包交给 `apply_closure`，内建函数在参数处理后交给 `builtin.run`。变参数内建的元组会先被解开；bare 变量参数即使收到元组，也将其作为一个值消费。调用节点把被调对象与参数的求值留在调用迹之外，再附加函数值的出处：内建为 `"built-in"`，闭包为 `defined <loc>`。详见 [[函数调用分派与内建参数解包]]。^[atlas-core-typed-eval.md:41-45]

`apply_closure` 按一个值接收参数：多参数按元组拆分，单参数整体接收，无参及全匿名参数表不占帧。递归闭包在新帧的 0 号槽自绑定，但新帧不进入捕获链，从而保持 `Rc` 结构无环。这些规则与分析期的层规则一致，见 [[闭包参数绑定与无环递归帧]]。^[atlas-core-typed-eval.md:46-50]

运行时错误穿过带名槽的闭包调用时，会附加局部变量迹行；无参闭包不压帧，也没有这类迹行。^[atlas-core-typed-eval.md:50-53]

## 容器迭代的构造纪律

容器迭代借用输入，只构造当前所需的分量或键。矩阵迭代不得额外构造列矩阵，多项式迭代则保持 canonical 项序与属主形式，并与领域值的 `loop_terms` 配对。详见 [[容器迭代的借用与规范项序]]。^[atlas-core-typed-eval.md:33-37]

## 证据范围

本页依据对 `crates/atlas-core/src/typed.rs` 第 11565–13741 行的结构性阅读；来源覆盖求值入口、六族求值、调用机器和回溯渲染，不声称语言或数学验收。^[atlas-core-typed-eval.md:9-12]

`EvaluationContext` 的帧与上下文栈，以及 133 个测试，尚待单独分包。来源中的上游行号引用属于实现方的移植陈述，求值兼容性仍以 HPC 语料门为准，参见 [[求值兼容性的证据边界]]。^[atlas-core-typed-eval.md:64-70]

## Sources

- [atlas-core-typed-eval.md](atlas-core-typed-eval.md) — TypedExpr 求值（typed.rs 11565–13741）：六族求值、调用机器与回溯渲染。
