---
title: atlas-real-group 的 crate 门面与数学值边界
summary: lib.rs 组织 60 个模块声明及 52 条根部再导出，定位为实约化群数学值与解释器领域值的适配边界，不承担语法或输出策略。
sources:
  - lib-root.md
kind: concept
createdAt: "2026-10-09T14:58:34.376Z"
updatedAt: "2026-10-09T21:00:34.938Z"
tags:
  - Rust架构
  - 模块接口
aliases:
  - atlas-real-group-的-crate-门面与数学值边界
  - A的C门
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# atlas-real-group 的 crate 门面与数学值边界

`atlas-real-group` 提供实约化群的结构数据。`crates/atlas-real-group/src/lib.rs` 是 crate 的统一对外门面，同时保留 crate 私有的 A1 迁移原型层。其定位是承载数学值，不包含 Atlas 语法或输出策略，作为解释器未来领域值（domain values）的适配边界；构造器负责校验 C++ 中隐含的根数据不变量。^[lib-root.md:10-14]

## 模块组织与公开接口

crate 根共有 60 条模块声明，其中 `deform`、`ext_block`、`ext_kl`、`ext_param`、`real_weyl` 为公开模块，其余 55 个为私有模块。私有模块中的 `global_tits` 与 `weyl_size` 带有 `#[allow(dead_code)]` 及说明注释：前者的消费者位于合成实形构建器，后者关联停放的 task #9 与 `ON_DEMAND_PARTITION_DESIGN`。^[lib-root.md:18-21]

根部以 52 条 `pub use` 集中组织再导出接口。其中，`topology::dual_component_group_rank` 的再导出位于模块声明区；`integer_lattice` 有两条再导出声明；`deform` 同时通过 `pub mod` 和 `pub use` 暴露。^[lib-root.md:22-24]

`matreduc`、`real_projection`、`root_reflection`、`global_tits`、`weyl_size` 均已声明，但没有根部再导出。格模块只导出 `pair` 与 4 个格类型，`lattice::pair_coordinates` 供内部使用，不对外导出。^[lib-root.md:24-28]

## 数学值与错误边界

根部错误类型包括 `StructureError`、`RelationError` 和 `InnerClassLetterError`，其中 `StructureError` 是唯一错误汇聚点；相关分类见 [[StructureError 统一错误分类学]]。^[lib-root.md:26-28]

原型根数据构造展示了具体校验职责：`RootDatum::from_basis` 先逐个检查维数，再按行主序验证根与余根的配对是否等于 Cartan 矩阵对应条目，不一致时报 `RootPairingMismatch`。根闭包计算使用 `i128` 中间精度并收窄到 `i32`，溢出返回 `ArithmeticOverflow`。这些行为属于原型层，详见 [[原型 RootDatum 的构造校验与配对约定]]。^[lib-root.md:34-41]

## A1 原型层与迁移边界

A1 原型层中的类型全部为 `pub(crate)`，实现注释标记其等待替换（pending replacement）。其中 `LatticeVector(Vec<i32>)` 是无校验的 newtype，声明的迁移方向是由 `Weight`／`Coweight` 在编译期区分对偶格；这一计划不表示替换已经完成。相关背景见 [[A1 迁移原型层与对偶格类型设计]]。^[lib-root.md:30-33]

原型层还包含 `PrototypeWeylGroup`、`RootType`、`CartanInvolution` 和 `RealReductiveGroup`。`CartanInvolution` 校验 \(M^2=I\) 及单根像属于根系，但其 `compact_imaginary` 标志是未经校验的调用方断言，来源说明它已被 `Grading` 取代。`RealReductiveGroup::simple_real_rank` 的含义刻意窄于 real rank。^[lib-root.md:42-49]

原型 `RootDatum` 与 `root_datum` 模块中的 `BasedRootDatum` 名称相近，但并非同一类型。解释门面时应保留这一区分，不能将原型行为直接归于 [[BasedRootDatum：带基根数据与构造不变量]]。^[lib-root.md:57-58]

## 测试与证据范围

本文件包含三个测试锚点：A1 反射取负、固定根按标志判为非紧虚根，以及 `i32::MAX` 坐标配对返回 `ArithmeticOverflow` 而非回绕。它们覆盖门面文件中的局部原型行为，不代表其他模块的完整验证；参见 [[原型层的测试锚点与证据边界]]。^[lib-root.md:51-58]

来源属于结构性阅读，不声称数学验收。其覆盖范围限于 crate 门面与原型层，不包括各模块内部实现或 `weyl_size`／`global_tits` 的消费者。来源通过快照记录 Git base、文件字节 SHA-256 与草案调用信息，并明确本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[lib-root.md:9-14, lib-root.md:55-66]

## Sources

- [lib-root.md](../../sources/lib-root.md) — crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）
