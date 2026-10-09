---
title: 领域值与 Weyl 身份（domain_builtins.rs 上部）——DatumWeylIdentity、RootDatumHandle 与 DomainValue 面
source: atlas-rust/atlas-core-domain-values
ingestedAt: 2026-10-09T12:40:00Z
---

# 领域值与 Weyl 身份（domain_builtins.rs 上部）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/domain_builtins.rs`（22771 行）的上部区域：
`LieTypeValue`（77+）、Weyl 身份机器（150–424）、`*Value` 结构
（424–715）、`DomainValue` 面与其等值（715–1242）、Weyl 兼容机器
（9600–9730）、派发入口（12263–12371）。`coerce`、`InnerClassContext`
impl、`RootNumbering`/`CenterClassifier`/`RootTable`/`validate`/
`print_text`、派发表（12371–18442）与 4077 行测试**不在本包**（各待
分包）。本区域正是 Weyl owner/dual 修复的落点；其语义由 A1 限定 HPC 门
验收（见下方交叉引用），本包只做结构性阅读。

## Weyl 身份机器（已验收修复的核心）

- `WeylIdentityCell<T>`：`OnceLock` 值 + 初始化互斥锁；`get_or_try_init`
  **只在成功时落定**（失败不占用单元）；双检查 + 锁防并发重复初始化；
  毒化/二次初始化 → `StructureError::RepInvariantViolation`。
- `DatumWeylKernel`：该 datum 的枚举根系（owner 局部坐标缓存），每个
  克隆/别名/`root_datum(WeylElt)` 往返共享。
- `AbstractWeylGroup`：抽象 Weyl 群（canonical-word 接口），恰在上游共享
  其 `WeylGroup` 指针时共享。
- `DatumWeylIdentity`：每个根数据 owner 的 Weyl 身份 = 惰性坐标 kernel +
  抽象群身份；两个单元都不回指 handle，**无所有权环**。
- `share_group_into_if_cold`（`root_datum_value::dual` 共享，
  atlas-types.w:1146-1152）：仅当目标 canonical dual 仍冷时把源的抽象群
  装入它；**预热目标永不被覆盖**，与源保持不兼容。并发下的竞态 share
  只会有一个发布，另一个被丢弃不出版。
- `DATUM_WEYL_IDENTITIES`：按**完整内容**弱引用的全局表（上游
  `root_datum_value` 弱 interning，atlas-types.w:977-991）：等值 owner
  活着时新构造复用其单元；强引用全灭后槽位过期、后来等值构造重新开始；
  表过 4096 才扫死槽。

## `RootDatumHandle` 与构造

- `interned` 是唯一构造入口：等值活数据共享一个 Weyl 身份。
- `PartialEq` **刻意忽略** Weyl 身份缓存：只比 datum + lie_type +
  isogeny + prefers_coroots（结构相等）；Debug 同。
- `DatumIsogeny`：SimplyConnected/Adjoint/Both/Other；`description()` 打印
  `root datum of Lie type '…'`（带 isogeny 标签前缀）。

## `*Value` 结构（424–715）

- `InnerClassContext`：每内类的管线，被该类的每个实形共享
  （root_datum + inner_class + classification Arc…）。
- `RealFormContext`：实形管线（`parent` + `internal` + `seed`…）。
- `BlockValue`：`rf` + `dual_rf` + `graph`。
- `WeylEltContext`：一个根数据的 Weyl 侧 = owner 局部坐标 kernel +
  携带内部生成元重编号的抽象群身份（该重编号固定上游 canonical-word
  选择）。**Weyl 兼容是抽象群 `Arc` 身份，永不是结构 handle 或坐标
  kernel**。
- `WeylEltValue`：元素 + 构造上下文 + **构造时冻结的** canonical 既约
  word——Display 与 `word` 是纯读。
- `SplitValue`：对偶数 e+f·s（s²=1），(e,f) 对存储；算术按上游
  `Split_integer`（arithmetic.h:152-213）的机器位宽**回绕**；打印
  `(e±|f|s)`（符号折入分隔符，io/basic_io.cpp:150-154）。
  `split_keeps`：零因子标量（1∓s 的倍数）杀掉在湮灭点取值为零的项，
  其余标量保留所有项（atlas-types.w:5868-5900）。
- `KTypeValue`/`ParamValue`：crate 的 `KType`/`StandardRepr` + 其属主
  实形（K_type_value/module_parameter_value）。
- `KTypePolValue`/`ParamPolValue`：一个实形上的有序 `(Split, KType)`/
  `(Split, StandardRepr)` 项；同类项合并、零系数删项（`SR_poly::add_term`）。

## `DomainValue` 面与等值

13 个变体（LieType/RootDatum/InnerClass/RealForm/KgbElement/Block/
WeylElement/CartanClass/Split/KType/KTypePol/Param/ParamPol）。等值是
**结构的**：独立构造的同一数学对象句柄相等（上游 memoized 句柄的可观察
语义）：

- InnerClass/CartanClass/KgbElement 比内类 + 形式号（+元素 id）；
  RealForm 及 KType/Param/两个多项式比 `same_real_form`
  （realredgp.h:142-149：同内类同形式 + 同基余特征标 + 同初始环面部）
  再比严格分量；`Block` 比两侧实形的内类。
- `WeylElement`：`weyl_elements_equal`——先抽象群 Arc 同一性，再把右
  元素按其 canonical 外生成元 word 在**左**系统重放后比较；
  辫子等价词跨坐标相等。外来根置换**永不**直接比较或复合。
- `same_real_form_owner`：指针同一性，给直接比 `shared_real_form` 字段的
  包装用；规范构造经父弱缓存共享一个上下文，自定义构造即使数学相等也
  总是新建属主。

## Weyl 兼容门（9600–9730）

- `weyl_group_compatible` = `Arc::ptr_eq(group)`；
  `require_weyl_compatible` 在**无值门之前**拒绝二元关系/乘积
  （`Weyl group mismatch`），与上游一致。
- `weyl_elt_value`：构造时算一次 canonical 既约 word（weyl.cpp:944-957）。
- `check_weyl_word`（atlas-types.w:2344-2359）：条目先转 unsigned
  （负数被拒），再必须低于半单秩——"Illegal Weyl word entry i
  (should be <r)"。

## 多项式系数契约（DomainValue 方法）

- `assign_polynomial_coefficient`：只**替换**精确 final 键的系数
  （atlas-types.w::assign_coef）——绝不加到它或展开非 final 键；即使
  零/丢弃写也校验；在触碰目标项**之前**拒绝不兼容属主（刻意偏离上游
  未检查的外来属主插入——证据见多项式系数切片）。
- `polynomial_coefficient`：订阅有自己的校验契约（区别于加/写）；Param
  的 dominant 化只改副本，绝不改调用方存储的参数；即使结果被丢弃也
  校验。
- `loop_terms`：借用 canonical 非零项序；产出的键保留其实形属主，无
  多项式拷贝或整数键替代。

## 派发入口（12263–12371）

- `call` = `call_with_printed` 丢弃打印侧通道。
- `call_owned_with_printed`：三个同结果类型的饥饿乘积
  （LieType×LieType、WeylElement×Vector、Vector×WeylElement）走
  `hungry_product_owned`（先秩校验，再 factors 合并/逐生成元作用），
  其余一切走普通借用适配路径。
- `call_with_printed` 的打印侧通道只有 `partial_extended_KL_block` 使用
  （ext_kl.cpp:945-948）。

## 交叉引用与边界

语义验收（A1 限定）：`tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`；
v8 的原始差异发现与 A1 回归金标在 `tests/math/generics/weyl_context_core_*`。
设计规则原文在根 AGENTS.md 的 "Weyl reuse caller" 条目（兼容 = 抽象群
Arc 身份；跨坐标等值/乘积重放外生成元词；二元关系在无值级也查兼容）。
`weyl_subgroup.rs`（433 行）待自己的包；`coerce`、内类/实形管线实现、
派发表与测试不在本包。
