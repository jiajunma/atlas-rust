---
title: TypedExpr 求值（typed.rs 11565–13741）——六族求值、调用机器与回溯渲染
source: atlas-rust/atlas-core-typed-eval
ingestedAt: 2026-10-09T15:00:00Z
---

# TypedExpr 求值（typed.rs 11565–13741）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/typed.rs` 的 11565–13741 行：`TypedExpr::evaluate`
与六个求值族、调用机器（`apply_function`/`apply_closure`）、回溯渲染
（`trace_location`/`frame_dump`）。结构性阅读，不声称语言或数学验收。

## 求值入口与六族（与 convert_expr 同形分区）

`evaluate(context, level) -> Result<Option<Value>, Control>` 按变体分到
**与转换遍相同的族**（各自 `#[inline(never)]`，栈帧纪律同
[convert-expr 包](atlas-core-convert-expr.md)）：

- `evaluate_atom`：Denotation/Captured/GlobalIdent/LocalIdent/Closure/
  Return/Break/Dont/Die。
- `evaluate_structure`：TupleDisplay/ListDisplay/Conversion/Void/
  UnionInject/TupleProject。
- `evaluate_application`：LetGroup/Conditional/BuiltinCall/
  HungryBuiltinCall/FunctionCall/Sequence/Next。
- `evaluate_mutation`：Global/Local/MultiAssignment、Component/Field 的
  Assignment/Transform。
- `evaluate_container`：Subscription/Slice/BarList。
- `evaluate_control`：While/Do/For/Case/IntCase/UnionCase/CountedFor。

`Level`（NoValue/SingleValue）一路下传，决定每个生产者构造多少值。

## 迭代借用纪律（12763+）

借输入而只造当前所需的分量/键：**矩阵迭代不得再造一个列矩阵**；多项式
迭代保持 canonical 项序与属主形式（与
[领域值包](atlas-core-domain-values.md)的 `loop_terms` 配对）。

## 调用机器（12836+）

- `apply_function`（不加调用迹）：闭包走 `apply_closure`；内建先解开
  **变参数内建的元组**（bare 变量参数：即使值是元组也按一个值消费，
  global.w:2990+）再 `builtin.run`。调用节点把被调与参数的求值留在迹
  外，然后附上值的出处（`function_origin`：内建 = `"built-in"`，闭包 =
  `defined <loc>`）。
- `apply_closure`（上游 `apply`，axis.w:3222-3571）：参数按**一个值**
  传入——多参数按元组拆分、单参数整体取、无参不压帧（空层规则）；
  全匿名参数表也不占帧（与分析期的层规则一致）；递归闭包在 0 号槽
  自绑（上游 `maybe_push`，axis.w:3548-3560）——新帧**不在**捕获链上，
  Rc 结构保持无环。`return` 解开到本调用边界并供值（
  `Err(Control::Return(value))` → `at_level(level, …)`）；运行时错误
  穿过带名槽的调用时附加局部变量迹行（axis.w:3525-3533；
  无参闭包不压帧也无此行）。

## 回溯渲染（12952+）

- `trace_location`：上游源码位置渲染（parsetree.w:173-180）——
  `at NAME:LINE:COL-COL`（行 1 基、列 0 基；Rust span 是 1 基），单行
  时结束 exclusive，跨行时 `at NAME:LINE:COL--ENDLINE:ENDCOL`（双破折
  号）。
- `frame_dump`：一个 let 组的被追踪帧（let_expression::evaluate 的
  catch）——按绑定序打印帧槽名。

## 边界与限制

- `EvaluationContext`（frames.rs 的帧/上下文栈）与 133 个测试各待分包；
  转换遍见 [convert-expr 包](atlas-core-convert-expr.md)，数据结构见
  [typed-core 包](atlas-core-typed-core.md)，内建注册表见
  [注册表包](atlas-core-builtin-registry.md)。
- 上游行号引用是**实现方移植陈述**；求值兼容以 HPC 语料门为准。
- 字节数/哈希只标识本快照字节（git base `964f0033`，typed.rs sha256
  `614975c5…`）。
