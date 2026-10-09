---
title: 组内名解析（typed/type_groups.rs）——递归图构造前的验证与形参转发
source: atlas-rust/atlas-core-type-groups
ingestedAt: 2026-10-10T09:40:00Z
---

# 组内名解析（typed/type_groups.rs）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/typed/type_groups.rs`（90 行，全部）——
`typed` 的私有子模块，在递归图构造（`types/recursive.rs`，见类型模型包）
**之前**做组内名解析。模块自述：把 global.w:1632-1860 的验证顺序与诊断
与普通别名既有契约分开保持（行号转述自模块注释，未独立重读上游）。
对应[阅读快照](snapshots/2026-10-10-atlas-core-type-groups.json)。
结构性阅读，不声称语言或数学验收。

## resolve_spec 的两遍结构

- 第一遍是 **BFS 验证**：根为别名的 RHS 或 struct/union 各字段类型；
  注释明确「原版按广度优先访问每个 RHS，结果先于参数」——`Function`
  先把 result 入队再入队 argument；`Applied` 先把参数入队再校验自身，
  即**先校验应用本身、再校验其参数**。
- 名字判定三分支：`Named` 视为 supplied=0 的非显式应用；`Applied`
  带显式参数；`Row`/`Tuple`/`Union` 只递归。查表失败报
  `Identifier '{name}' does not refer to any type`。
- 正在定义的局部名（locals）**永不接受显式参数**——即使个数恰等于
  arity 也报 `Type '{name}' being defined cannot be given type
  arguments`；非局部名查构造器 arity，不符报
  `Type constructor '{name}' called with {supplied} type arguments,
  expected {expected}`。三类诊断都是 Program 类。
- 第二遍 `resolve`：克隆表达式，先 `forward_formals` 再 `resolve_in`，
  并把 `resolve_in` 的错误种类改写为 Program。
- 落地：Alias 直解；Struct → `Type::tuple`，Union → `Type::union_of`；
  字段名另行为 `Vec<Option<String>>` 返回。

## forward_formals：裸自引用的形参转发

- 命中条件：`Named` 且 arity≠0 且查表所得编号在 locals 内——即组内
  裸自引用（如 `MathGenericList`）。
- 重写为 `Applied`，参数是 `Variable{index}` 的 0..arity——裸名
  **转发所在组的全部形参**（`MathGenericList` 变为
  `MathGenericList<A>`），不是无参构造器调用。这与 generic recursive
  groups 弧的既有记录一致。
- 递归进入 Row/Function/Tuple/Union/Applied 的子表达式；其他叶不变。

## 边界声明

本包是对当前工作区字节的结构性阅读（git base 与哈希见快照）。验证顺序
与诊断措辞锚点的行为权威是 HPC 语料门；泛型递归组的端到端验收属于其
自身的 HPC 证据链（如 544-core 门 3844249/3844321），本包不重述。
