---
title: 有理余权的逐坐标表示与 API 边界
summary: RationalCoweight 逐坐标保存有理数，构造仅限 crate 内，公开维度及精确坐标访问，不提供算术或 Hash。
sources:
  - lattice-types.md
kind: concept
createdAt: "2026-10-09T14:57:34.779Z"
updatedAt: "2026-10-10T02:07:36.815Z"
tags:
  - 有理余权
  - 接口设计
aliases:
  - 有理余权的逐坐标表示与-api-边界
  - 有A边
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 有理余权的逐坐标表示与 API 边界

`RationalCoweight` 以 `coordinates: Vec<Rational>` 表示有理余权，每个坐标分别保存为有理数。它与 `RationalWeight` 的内部表示不同：后者使用 `Vec<i64>` 分子向量和一个公共的 `i64` 分母，参见 [[有理权的公共分母表示与归一化]]。^[lattice-types.md:17-20, lattice-types.md:76-82]

## 格类型与表示纪律

格类型层将 `Weight`（特征格 $X^*$）与 `Coweight`（余特征格 $X_*$）定义为两个独立的 newtype。两格之间存在完美配对，但即使所选基下的坐标表示相同，也不可互换，参见 [[权格与余权格的类型隔离]]。整数坐标刻意采用带检查的固定宽度存储；精确解释器值在领域边界处转换，而不改变每个根系矩阵条目的表示。^[lattice-types.md:17-25]

## 构造与公开接口

`RationalCoweight::from_coordinates` 的可见性为 `pub(crate)`，构造入口限于 crate 内部。公开 API 为 `dimension()` 与 `to_rationals()`，分别提供维度查询和面向 workspace 消费者的精确坐标视图。^[lattice-types.md:78-81]

第三方 malachite 类型不进入该类型的公开 API；解释器值层则已公开使用 malachite 有理数。`RationalCoweight` 本身不提供算术操作，也不实现 `Hash`。^[lattice-types.md:78-82]

## 坐标导出与分配边界

`lattice.rs` 对按输入长度新建的 `Vec` 使用 `try_reserve_exact`，将预留失败映射为 `StructureError::AllocationFailed`，相关错误体系见 [[StructureError 统一错误分类学]]。`to_rationals()` 使用直接的 `clone()`，与 `halve()`、`normalized()` 一同列为这项分配纪律的例外。^[lattice-types.md:35-39]

## 证据范围

本页依据对 `lattice.rs` 的两次结构性阅读，两次阅读确认源文件字节及 SHA-256 未变。其正确性属于独立的 [[HPC 验收证据链]]，包括 lattice gate 等验证；来源包不重述或扩展这些证据，也未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。^[lattice-types.md:9-13, lattice-types.md:86-92]

## Sources

- [lattice-types.md](../../sources/lattice-types.md) — 权格类型层：Weight、Coweight 与有理权。
