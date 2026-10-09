---
title: 领域派发与强转（domain_builtins.rs 中部）——call 路径、coerce 与 166 臂派发匹配
source: atlas-rust/atlas-core-domain-dispatch
ingestedAt: 2026-10-09T13:00:00Z
---

# 领域派发与强转（domain_builtins.rs 中部）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`domain_builtins.rs` 的 `coerce`（2301–2527）、派发入口与巨型
`match name`（12263–18442 的**组织方式**），以及 `build_real_form` 的
规范弱缓存（2404+）。166 个派发臂的逐臂内容不在本包（每臂的上游出处
注释就地保留在源里，共 697 行注释）。结构性阅读，不声称语言或数学验收。

## 派发入口（12263–12371）

- `call(name, arguments, span)` = `call_with_printed` 丢弃打印侧通道。
- `call_owned_with_printed`：三个同结果类型的饥饿乘积
  （`LieType×LieType`、`WeylElement×Vector`、`Vector×WeylElement`）走
  `hungry_product_owned`——先秩校验（`RANK_MAX` 组合秩 / 权与余权大小
  匹配），再就地消费（factors 合并 / `word_act_weight` / 逐生成元
  `simple_coreflect`）；其余一切走普通借用适配路径。这是与
  `typed.rs` 的 `HungryBuiltinCall` 配对的饥饿求值纪律。
- `call_with_printed` 携带 `printed: &mut Vec<String>` 侧通道——只有
  `partial_extended_KL_block` 使用它（ext_kl.cpp:945-948 的中途 stdout）。

## `match name` 派发（12377+，166 臂）

每臂同一契约形状：`arity(name, arguments, n, span)?` → 按类型提取
（`as_lie_type`/`as_integer`/`as_matrix`…）→ 调用 → 包装成 `Value`；
错误经 `type_error`/`runtime`/`structure_diagnostic`/`relation_diagnostic`
发出。臂内注释给出上游出处与**校验顺序契约**（例如
`build_KGB_element_wrapper` 在无值门**之前**跑全部构造检查，
atlas-types.w:4580-4607；`KL_block_wrapper` 先 `test_standard`；
`classify_involution_wrapper` 先验平方与 M²=I 再做整数格分类）。
实整数收窄一律先 signed32 再映到 unsigned32（`Integer value too big for
conversion`）。多个名字由参数个数重载派发（real_form 两包装器按
(InnerClass,int) 与 (InnerClass,mat,ratvec) 区分）。

## `coerce(tag, value, span)`（2301–2527）：强转标签的领域侧

- `LT`→`Lie_type`、`IcRf`→`inner_class`（调用同名的派发臂）。
- `RdIc`/`RdRf`：从 InnerClass/RealForm 句柄**导航**到其根数据（不
  经函数调用）。
- `SpI`/`Sp(I,I)`：int 或 (int,int) → `Split`，先收窄机器位宽
  （atlas-types.w:5113-5125）。
- `KpolK`：`KType` → `KTypePol`——经 `finals_for`（K_repr）展开成 final
  组分，逐项 `merge_ktype_term` 合并，再按 canonical `K_type_pol` 项序
  排序（atlas-types.w:5608-5617）。
- `PolP`：`Param` → `ParamPol`——`expand_final`（repr.cpp:1299-1306）
  展开成 final 标准组分，合并并按 `SR_poly` 项序排序
  （atlas-types.w:7710-7717）。
- 未知 tag → 运行时错误 `conversion '<tag>' is not implemented`。

## `build_real_form` 与规范弱缓存（2404+）

`parent.order.internal(external)` 把外部形式号翻译成内部号（非法号 →
`Illegal real form number: n`）；`canonical_forms` 是**父级弱缓存**：
规范构造共享一个上下文（与 `same_real_form_owner` 的指针规则配对——
自定义构造即使数学相等也新建属主，见
[领域值包](atlas-core-domain-values.md)）。

## 边界与限制

- 每臂的数学内容、 adapters 的实现（`relation_*`/`merge_*`/`sort_*`）、
  `validate`/`print_text`/`coerce` 之外的区域（InnerClassContext impl、
  RootNumbering、CenterClassifier、RootTable）各待分包。
- 166 臂的计数与 697 行注释只标识本快照字节（git base `964f0033`）；
  校验顺序契约是**实现方陈述**，其行为以 HPC 语料门为准。
- 语言层的重载解析在 `typed.rs`（见 [typed-core 包](atlas-core-typed-core.md)），
  强转表的**语言侧**注册在 `coercions.rs`（见 [支撑层包](atlas-core-support-layer.md)）——
  本包是它们的领域侧配对。
