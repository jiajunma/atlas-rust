---
title: 转换遍 convert_expr（typed.rs 中部）——in/out 类型模式、族划分与赋值助手契约
source: atlas-rust/atlas-core-convert-expr
ingestedAt: 2026-10-09T14:00:00Z
---

# 转换遍 convert_expr（typed.rs 中部）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/typed.rs` 的 2757–5973 行：`convert_expr` 入口、
`convert_expr_context` 调度器、12 个表达式族转换函数与赋值助手群。
文件自述（typed.rs 头部）："convert_expr 镜像上游 axis.w:272-487——一遍
同时做检查与合成，针对只经 `specialise` 变异的 in/out 类型模式；
`conform_types` = 先特化、否则强转、否则类型错误；非行上下文的列表显示
查 `row_coercion`（故 `mat: [[1,2]]` 把元素定型为 `vec`）；转换节点求值
注册的转换函数；整数收窄用上游**逐字**错误文本（含笔误，
bigint.cpp:142-162）"。结构性阅读，不声称语言验收。

## 入口与调度器

- `convert_expr(expression, required: &mut Type, analysis)`：把 `required`
  装进共享 `ConversionType`（当前 `analysis.type_floor`），转换后写回。
- `convert_expr_context`：**type_floor 调整**——return 操作数在类型抽象
  内时可引用外层函数的要求；其变量按要求的**实际** fixed 下限解读（同
  上游 convert_expr）。
- 族划分纪律（栈教训 original3839541：GDB 陷阱命中的是这个**分析**帧，
  不是求值器）：12 个族助手全部 `#[inline(never)]`，保留相同 arm 体与
  已调整的共享转换上下文——机械分区、不改语义。全文件 19 处
  `#[inline(never)]`。

## 12 个族（按 Expr 种类）

`atom`（标量/lambda/OperatorCast/return/group/标识符/break/dont/die）、
`type_context`（TypeAbstraction/Cast）、`display`（tuple/list/barlist）、
`binding`（let）、`assignment`（6 种赋值形）、`subscription`、`slice`、
`application`（OperatorCall/Call）、`control`（conditional/binary/unary/
sequence/do/next）、`loop`（while/for/counted-for）、`tagged_case`
（Case）、`case`（IntCase/UnionCase）。

while 转换的代表性细节：axis.w 把**循环层装在整棵 do 树外**（压平守卫
不得移出此边界）；条件先做 a-priori 转换再查 bool，措辞为上游
"found … while … was needed."；`WhileMode` 由所需上下文决定（void →
体取 void 上下文且**允许异构分支**；int → count；否则 row，带 reversed
标志），row 路径先试 `[*]` 特化、失败则回退 `row_coercion`。

## 赋值助手群（4305+）

- `convert_simple_assignment`：`set x := value` 与裸 `x := value`
  **刻意共享**同一路径。
- `lookup_assignable`（axis.w:8148-8160, 8216-8228）：局部遮蔽全局；两者
  都报赋值专用的未定义/常量诊断，引用整个表达式的紧凑渲染。
- `component_type_for_assignment`（axis.w:8163-8172, 8531-8546 的
  `subscr_base::index_kind` 受 `assignable` 门控）：行/vec/mat（列或双
  下标项）以及 KTypePol[KType]/ParamPol[Param] 允许分量赋值；
  **ratvec 上游只读**。
- `resolve_projector`：当前 axis.w:8824+ 用**保留的类型定义**而非投影
  函数的当前值；复制的或泛型定义也可匹配——故具名接收方本身**不能**
  唯一确定字段选择。
- `factor_transform_call`：本转换器**永不**应用上游 `x+1`→`succ(x)` 的
  丢参数优化——二元调用始终保留两个操作数。

## 边界与限制

- 内建注册表（5973–11565）、`TypedExpr` 的求值 impl（11565–13741）与
  133 个测试（13741+）各待分包；核心数据结构见
  [typed-core 包](atlas-core-typed-core.md)。
- 上游行号引用是**实现方移植陈述**；转换行为兼容以 HPC 语料门为准。
- 字节数/哈希只标识本快照字节（git base `964f0033`，typed.rs sha256
  `614975c5…`——与已验收 after-v5 清单同字节）。
