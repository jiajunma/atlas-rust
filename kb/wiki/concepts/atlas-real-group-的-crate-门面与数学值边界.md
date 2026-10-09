---
title: atlas-real-group 的 crate 门面与数学值边界
summary: lib.rs 通过 60 个模块声明、5 个公开模块与 52 条根部再导出组织接口，专注实约化群的数学值，不承担 Atlas 语法或输出策略，并作为未来解释器 domain values 的适配边界。
sources:
  - lib-root.md
kind: concept
createdAt: "2026-10-09T14:58:34.376Z"
updatedAt: "2026-10-09T14:58:34.376Z"
tags:
  - Rust架构
  - 公共接口
  - 实约化群
aliases:
  - atlas-real-group-的-crate-门面与数学值边界
  - A的C门
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# atlas-real-group 的 crate 门面与数学值边界

`atlas-real-group` 提供实约化群的结构数据。其 `crates/atlas-real-group/src/lib.rs` 是 crate 的统一对外门面，同时保留 crate 私有的 A1 迁移原型层。该 crate 刻意只承载数学值，不包含 Atlas 语法或输出策略，作为解释器未来领域值的适配边界；构造器负责校验 C++ 中隐含的根数据不变量。^[lib-root.md:10-14]

## 模块组织与公开接口

crate 根共有 60 条模块声明，其中 `deform`、`ext_block`、`ext_kl`、`ext_param`、`real_weyl` 是 5 个公开模块，其余 55 个为私有模块。私有模块中，`global_tits` 和 `weyl_size` 带有 `#[allow(dead_code)]` 及说明：前者的消费者位于合成实形构建器，后者关联停放的 task #9 与 `ON_DEMAND_PARTITION_DESIGN`。^[lib-root.md:18-21]

根部通过 52 条 `pub use` 集中组织再导出接口。其中，`topology::dual_component_group_rank` 的再导出位于模块声明区，`integer_lattice` 有两条再导出声明，`deform` 则同时通过 `pub mod` 和 `pub use` 暴露。^[lib-root.md:22-24]

`matreduc`、`real_projection`、`root_reflection`、`global_tits`、`weyl_size` 声明后没有根部再导出。格模块的边界同样明确：`lattice::pair_coordinates` 仅供内部使用，对外只导出 `pair` 和 4 个格类型。^[lib-root.md:24-28]

## 数学值与错误边界

根部公开的错误类型包括作为唯一错误汇聚点的 `StructureError`，以及 `RelationError`、`InnerClassLetterError`。这一门面将数学结构及其构造失败纳入公开接口；相关错误分类可参见 [[StructureError 统一错误分类学]]。^[lib-root.md:26-28]

根数据构造的校验体现了数学值边界的具体要求。例如，原型 `RootDatum::from_basis` 先检查维数，再按行主序核对根与余根的配对是否等于 Cartan 矩阵条目；其根闭包计算使用 `i128` 中间精度并收窄到 `i32`，遇到溢出返回 `ArithmeticOverflow`。这些是原型层的具体行为，相关细节见 [[原型 RootDatum 的构造校验与配对约定]]。^[lib-root.md:34-41]

## A1 原型层与类型区分

A1 原型层全部限定为 `pub(crate)`，实现注释明确标记其等待替换。原型 `LatticeVector(Vec<i32>)` 是没有校验的 newtype，计划由能够在编译期区分对偶格的 `Weight`／`Coweight` 类型取代；这部分迁移背景见 [[A1 迁移原型层与对偶格类型设计]]。^[lib-root.md:30-33]

原型层还包含 `PrototypeWeylGroup`、`RootType`、`CartanInvolution` 和 `RealReductiveGroup`。其中，`CartanInvolution` 的 `compact_imaginary` 标志是未经校验的调用方断言，已被 `Grading` 取代；`RealReductiveGroup::simple_real_rank` 的含义刻意窄于 real rank。^[lib-root.md:42-49]

原型 `RootDatum` 与 `root_datum` 模块中的 `BasedRootDatum` 名称相近，但并非同一类型。阅读门面时应保持这一区分，后者可参见 [[BasedRootDatum：带基根数据与构造不变量]]。^[lib-root.md:57-58]

## 证据范围

本来源属于结构性阅读，不声称数学验收。文件中的 3 个测试锚点覆盖 A1 反射取负、依据标志判定固定根为非紧虚根，以及 `i32::MAX` 坐标配对准确返回 `ArithmeticOverflow` 而非回绕；这些锚点不能代表全部模块的验证覆盖。^[lib-root.md:14-14, lib-root.md:51-58]

本页依据仅覆盖 crate 门面与原型层，不涵盖各模块内部实现，也不包含 `weyl_size`／`global_tits` 的消费者。来源记录了源码字节与 Git base 的快照身份，并明确本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[lib-root.md:55-66]

## Sources

- [lib-root.md](../../sources/lib-root.md) — crate 根：60 模块组织、52 条再导出与 A1 原型层（lib.rs）
