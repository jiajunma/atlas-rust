---
title: 类型化管线核心数据结构（typed.rs 上部）——TypedExpr 树、Analysis/OverloadState 与 TypedContext
source: atlas-rust/atlas-core-typed-core
ingestedAt: 2026-10-09T12:00:00Z
---

# 类型化管线核心数据结构（typed.rs 上部）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/typed.rs` 的 59–2757 行（该文件 18519 行、770895
字节）：控制/级别枚举、`TypedExpr` 可执行树、`Analysis`（转换期上下文）、
`IdTable`/`TypeCell`、`OverloadState`、`TypedCommandEvent`、`TypedContext`
字段面。转换遍 `convert_expr`（2757–5973）、内建注册表（5973–11565）、
`TypedExpr` 的 impl（11565–13741）与 5118 行测试**不在本包**（各待分包）。
结构性阅读，不声称语言或数学验收。

## 控制与级别枚举

- `Control`：`Break(usize)`/`Dont`/`Return(Value)`/`Runtime(Diagnostic)`。
- `Level`：`NoValue`/`SingleValue`——上下文要多少值（上游
  `expression_base::level`）。
- `WhileMode`：`Void`/`Count`/`Row{reversed}`——所需结果上下文选择 while
  求值器（axis.w:5948-6004）。
- 赋值目的地族：`MultiAssignmentDestination`（Global 在**分析时**捕获
  cell，Local 留词法坐标——与两个普通赋值节点一致）、
  `MultiAssignmentPlan`（整体目的地刻意单列：求值先左到右访问子项、
  整体最后，axis.w `thread_assign`）、`AssignTarget`（共享 axis.w:6863+
  的层查找）、`TransformOperation`（`op:=` 的解析结果：内建索引或用户
  重载闭包——上游为用户操作重组成普通调用，可观察上即脱糖应用）。
- `PilferDestination`：饥饿内建调用的目的地（Global/Local）。

## `TypedExpr`（124–425）：可执行表达式树

值与容器：`Denotation`、`Captured`（冻结重载值带表达式拼写，别于值自身
的打印——axis.w `capture_expression`）、`TupleDisplay`/`ListDisplay`、
`Conversion{tag,…}`（注册的强转，`tag` 打印为 `tag:expr`）、`Void`
（只求效果、值替换为 void）、`BarList`（直接构造矩阵——用户 `^`/`mat`
重载无法拦截，见 syntax 包）。

读写与赋值：`GlobalIdent`/`LocalIdent`（cell 或词法坐标）、
`GlobalAssignment`/`LocalAssignment`、`ComponentAssignment`（先值后下标再
替换分量并产出该值；`source` 字段供越界诊断逐字引用，axis.w:7953）、
`ComponentTransform`（`op:=`：范围检查在对**合成读** `a[i]` 时触发——
oracle 引用选择式 "in subscription v[5]"）、`FieldAssignment`/
`FieldTransform`、`MultiAssignment`（值完整求值后按 plan 后序分发）。

调用与控制：`Subscription`（**领域系数读先求接收方再求键**；普通位置
订阅先求下标）、`Slice`、`LetGroup`（每绑定一个 (shape, initializer)）；
`Conditional`、`BuiltinCall`（注册表索引 + `name@argtype` 回溯名）、
`HungryBuiltinCall`（顶层饥饿操作数恰是赋值目的地时立即移出旧值）、
`Closure`（求值时把当前帧链捕获成闭包值）、`Return`、`FunctionCall`
（动态被调：内建或闭包，参数按**一个**值传入——多参数为元组；
`name=None` 时上游在回溯行打印被调表达式，axis.w:1913-1915）、
`Sequence`（前者 NoValue 求效果）、`While`/`Do`（词法守卫帧包住两者）、
`For`（七种上游聚合的分量循环；下标按接收方为 int/KType/Param）、
`Break`（levels+1 层）、`Dont`、`Die`（分析通过任何所需类型，求值抛
`I die`）、`UnionInject`/`TupleProject`（injector/projector 闭包体）、
`Case`（首个 tag 匹配分支按 shape 分发载荷）、`IntCase`（负值→then、
越界→else、皆缺则取模）、`UnionCase`（按变体位置应用分支函数）、
`Next`（产出**第一个**值）、`CountedFor`（可变的每迭代帧计数器）。

## `Analysis`（476–555）：转换期上下文

持有：`types`/`globals`/`overloads` 引用 + `overload_views`（Rc 共享的
**未移位**有序签名视图缓存：只缓存结构签名，永不缓存推断类型或跨命令
解析结果；公开 Analysis 引用可在克隆上重绑——`std::ptr::eq` 检查类型表
与重载表同一性，不符则清缓存）+ `locals` + `constant_locals`（`!x`
常量名，普通重绑可解除标记）+ `return_type`（最近函数的**活**结果要求，
独立于当前表达式上下文，axis.w `layer::current_return_type`）+
`loop_depth`（非零才允许 `break`——上游在**分析期**拒绝游离 break）+
`type_floor`（词法阈值以下为刚性变量）。
`ConversionType(Rc<RefCell<(Type, usize)>>)`：函数体与其 return 操作数
共享的活转换单元；**递归转换时绝不持有 RefCell 借用**。

`IdTable`：每名一绑定，定义持其分配的新鲜 cell（转换后的代码保留捕获
的 cell；重定义只换名）。`TypeCell`：绑定的类型号按**其定义处**的词法
下限解读；克隆共享旧精化单元，导入用例绝不写它。

## `OverloadState`（641–1055）：每上下文的重载状态

上游 overload 表（global.w:446-560）的对应物：启动注册表是静态的，
`forget name @ type` 记移除、`set` 记用户变体，解析时合并为**一个有序
列表**。缓存纪律（已验收的跨命令设计）：`views` 以 `TypeTable` 的
`revision` Arc 守护（指针不等即清空），只放出不可变、未移位的签名与
来源索引；`Clone` 的事务克隆**不带**缓存（可选缓存是一次性的）。
`add_user` 按合并视图重放上游单表 `add`（global.w:1004-1023）：精确参数
孪生原地替换（启动孪生经 forgotten 隐藏）、过近邻即上游歧义错误
（保留其完整措辞）、其余按有序位插入；返回值是前/后变体数，用于选择
报告措辞。

## `TypedCommandEvent` 与 `TypedContext` 字段面

- `TypedCommandEvent`：`Diagnostic`/`Value{value,type_,span}`（**携带**
  类型——会话层据此取 `is_void`）/`ReportLine`/`ReportBytes`/`Output`；
  `report()` 的 UTF-8 分流（非法字节进 `ReportBytes`，值不被改写）。
- `STARTUP_COMPLETION_NAMES`：启动补全名（buffer.w:1175-1192，上游
  `main_hash_table` 顺序：35 关键字、21 原始类型名、再按上游注册序）；
  三个启动系统变量（main.w:408-435）**不在其中**——它们是会话全局，
  由 `TypedContext::new` 播种。
- `TypedContext` 字段：`types`、`globals`、`evaluation`
  （EvaluationContext）、`overloads`、`verbosity`（`set quiet`=0/
  `set verbose`=1，main.w `verbosity`）、`type_locations`（位置属于当前
  标识符绑定而非可能被复用的类型表槽——重定义必须报新位置）。
  `Default` 播种启动补全清单（成功的 forget 只更新受影响名，不隐藏
  无关未移植启动名）；`new()` 再播种系统变量并把 `prelude_log` 标为
  const。

## 边界与限制

- `convert_expr`、内建注册表、`TypedExpr` 的求值 impl 各待分包；执行层
  `EvaluationContext`/`frames.rs` 与领域桥 `domain_builtins.rs` 同。
- 上游行号引用是**实现方移植陈述**；行为兼容以 HPC 语料门为准。
- 字节数/哈希只标识本快照字节（git base `964f0033`，typed.rs sha256
  `614975c5…`——即已验收 after-v5 清单中的同一字节）。
