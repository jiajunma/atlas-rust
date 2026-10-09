---
title: 有理余权的逐坐标表示与 API 边界
summary: RationalCoweight 逐坐标保存有理数，构造限于 crate 内，公开维度和精确坐标访问，不提供算术或 Hash。
sources:
  - lattice-types.md
kind: concept
createdAt: "2026-10-09T14:57:34.779Z"
updatedAt: "2026-10-09T19:32:34.239Z"
tags:
  - 有理余权
  - 数值表示
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

`RationalCoweight` 使用 `coordinates: Vec<Rational>` 表示有理余权，每个坐标分别存储为有理数。它不同于 [[有理权的公共分母表示与归一化|RationalWeight]]：后者使用 `Vec<i64>` 分子向量与一个公共的 `i64` 分母。^[lattice-types.md:17-20, lattice-types.md:75-79]

## 格类型与表示纪律

格类型层将 `Weight`（权格 $X^*$）与 `Coweight`（余权格 $X_*$）定义为独立的 newtype。两格之间存在完美配对，但即使所选基下的坐标相同，也不能互换，参见 [[权格与余权格的类型隔离]]。整数坐标刻意采用带检查的固定宽度存储，精确解释器值在领域边界处转换，而不改变每个根系矩阵条目的表示。^[lattice-types.md:17-25]

## 构造与公开接口

`RationalCoweight::from_coordinates` 的可见性为 `pub(crate)`，构造入口仅限 crate 内部。公开 API 为 `dimension()` 与 `to_rationals()`，分别提供维度查询与面向 workspace 消费者的精确坐标视图。^[lattice-types.md:75-78]

来源明确区分该类型的公开 API 与解释器值层：第三方 malachite 类型不进入前者，而解释器值层已公开使用 malachite 有理数。`RationalCoweight` 本身不提供算术操作，也不实现 `Hash`。^[lattice-types.md:75-79]

## 坐标导出与分配边界

`lattice.rs` 对按输入长度新建的 `Vec` 使用 `try_reserve_exact`，并将预留失败映射为 `StructureError::AllocationFailed`。不过，`to_rationals()` 使用直接的 `clone()`，与 `halve()`、`normalized()` 一同列为这项分配纪律的例外；因此，坐标导出并未采用相同的预留失败映射路径。相关背景见 [[StructureError 统一错误分类学]]。^[lattice-types.md:35-39]

## 证据范围

以上说明来自对 `lattice.rs` 的结构性阅读。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；正确性仍属于独立的 [[HPC 验收证据链]]，包括 lattice gate 等验证。^[lattice-types.md:9-13, lattice-types.md:81-89]

## Sources

- [lattice-types.md](lattice-types.md) — 权格类型层：Weight、Coweight 与有理权。
