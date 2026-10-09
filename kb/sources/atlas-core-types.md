---
title: 类型模型（types.rs + types/）——Type 面、TypeTable 与二阶机器
source: atlas-rust/atlas-core-types
ingestedAt: 2026-10-09T11:00:00Z
---

# 类型模型（types.rs + types/）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`types.rs`（961 行）+ `types/polymorphic.rs`（1106 行）+
`types/recursive.rs`（262 行，crate 私有）+ `types/revision_tests.rs`
（78 行）。types.rs 自述：axis 类型模型（语言 phase B stage 1），移植
上游 `type_expr`（axis-types.w:289-388）：tag+payload，void 即空元组，
长度 1 元组/联合不可表示（构造器折叠它们），变体/字段名住在 typedef
表而非类型里，递归类型按名字比较，`specialise` 是唯一允许的变异
（成功时是最一般合一子，**刻意不是** commit-or-rollback——需要回滚的
调用方用 `can_specialise`）。Display 逐字节同上（axis-types.w:1610-1675）。
结构性阅读，不声称语言验收。

## `Type` 面（types.rs）

- `Prim`：20 个原始类型，`Prim::ALL` 按上游 `prim_names` 顺序；
  `Prim::name()` 给上游拼写（`KgbElt` ↔ `"KGBElt"`）。
- `Type`：`Undetermined`（`*`，只被 `specialise` 收窄）、`Variable(usize)`
  （刚性由**外围 scheme 的 fixed 阈值**决定，不在节点上）、`Primitive`、
  `Function(Box<(Type,Type)>)`（多参数函数带元组参数）、`Row`、`Tuple`
  （空 = void）、`Union`、`Tabled(TypeNumber)`（递归项按名字名义比较）、
  `Applied(TypeNumber, Vec<Type>)`（即使结构展开含未用形式参数也保留名
  与实参）。`Type::tuple`/`union_of` 折叠长度 1。
- `expanded`（Cow）：普通类型借用；构造器替换只拥有展开结果；子项保留
  名字（含递归应用）；循环上界 = 绑定数，使**直接自环的残缺占位**也可
  检查而不死循环。
- `equivalent`（语义等值，别于文本/表槽位等值）：命名头可先暴露不同
  形状——先 `validate_applications` 两侧再结构递归；递归名是终止边界
  （两侧皆递归的不同名 → false；同名同槽 → true）。
- `specialise(&mut self, pattern)`：唯一变异路径，成功即 MGU；失败时
  `self` 可能已部分特化（**上游语义**）——回滚场景先 `can_specialise`。

## `TypeTable`：不可变身份 + 活绑定 + 修订身份

- 组成：`bindings: Vec<TypeBinding>`（name + definition + 位置字段/
  injector 名，`None` = 匿名分量）+ `active: BTreeMap<String, TypeNumber>`
  + `constructors: BTreeMap<usize, (usize, bool)>`（仅新构造器项；旧
  tabled 项保留其名义递归行为）+ `revision: Arc<()>`。
- `revision`：**非语义的快照身份**——克隆共享，突变即换；缓存读者持
  有它以阻止地址复用；Arc 保住 TypeTable 的 Send+Sync。这是已验收的
  跨命令缓存设计（见下方交叉引用）。
- `add`/`update`（两阶段括号 set_type 先注册全组名再解析各 RHS——递归）、
  `add_constructor`、`forget`（只摘活名，已存值保留旧类型）、
  `add_simple`（按 name+definition+arity 去重并搬字段元数据）。
- `matching_bindings`：在**全部保留定义**里查字段/标签元数据，不只查活
  名或当前投影重载（axis-types.w:1454）；每个绑定试一个新鲜形式构造器
  应用，保留接收方的刚性下限并隔离候选的自由变量。
- `validate_applications`：**不展开定义**地校验（递归名保持有限）；即使
  等值应用与变量绑定也校验——快路径不是接受残缺构造器的许可。
  `expand_application` 只展开一层：体内的递归应用保持引用。

## `types/polymorphic.rs`：二阶机器（axis-types.w:2264+）

- `TypeError`：IndexOverflow/ScopeCapture/VariableOutOfRange/
  UndeterminedInAssignment/UnknownConstructor/Arity 等。
- `substitute_parameters`、`shift(t, fixed, amount)`。
- `TypeScheme{body, fixed, degree}`：`wrap` 按遍历/首次出现序打包——
  不同洞保持不同，重复变量号共享槽位；`constructor` 保留**声明** arity
  与参数编号（含未用参数，duplicate-formal 探针）。
- `TypeAssignment`：`[fixed, fixed+degree)` 的无环替换，fixed 以下为
  刚性；`append` 导入另一赋值**连同其待决替换**（axis-types.w::append，
  否则静默丢弃推断约束）；`instantiate` 导入 scheme 的新鲜用例（刚性
  变量不动）；`unify`/`try_unify`（失败时部分变异同上游；回滚用
  try_unify）。
- `InferredType`（体 + 赋值对）：`from_scheme`/`wrap`/`bottom`/
  `wrap_tuple`、`bake`、`wring_out`、`raise_floor`/`lower_floor`、
  `unify_to`/`try_unify_to`/`unify`、`has_unifier`、`function_parts`、
  `matches_argument`/`matches_result`、`unify_specialise`/
  `try_unify_specialise`、`matches`（构造器形式匹配）。
- 自述纪律：scheme 永不含独立 `Undetermined` 洞（wrap 给每洞配新变量
  但保留重复显式变量）；替换属于一次分析/重载试验，不属于全局绑定。

## `types/recursive.rs`（crate 私有）

图级递归 typedef 安装（axis-types.w:1640-2010）：命名 RHS 槽位优先；环上
的匿名后代也保留身份；调用方把此表与全部生成成员一起暂存。

## 测试与交叉引用

59 个测试（types 13 + polymorphic 37 + recursive 5 + revision_tests 4）。
`revision` 身份设计对应已验收的跨命令缓存门（命令 AFTER R2：
`tests/reference/hpc/` 的命令链验收记录，见 docs/HANDOFF.md 索引）；
`TypedContext` 对本表的消费见 [会话包](atlas-core-session.md)。
上游行号引用是**实现方移植陈述**；类型行为兼容以 HPC 语言语料门为准。
字节数/哈希只标识本快照字节（git base `964f0033`）。
