---
title: 有理余权的逐坐标表示与 API 边界
summary: RationalCoweight 采用 Vec<Rational> 逐坐标存储，通过 dimension 与 to_rationals 提供公开坐标访问，将构造限制在 crate 内，且不提供算术或 Hash。
sources:
  - lattice-types.md
kind: concept
createdAt: "2026-10-09T14:57:34.779Z"
updatedAt: "2026-10-09T14:57:34.779Z"
tags:
  - 有理余权
  - 精确算术
  - API设计
aliases:
  - 有理余权的逐坐标表示与-api-边界
  - 有A边
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 有理余权的逐坐标表示与 API 边界

`RationalCoweight` 以 `coordinates: Vec<Rational>` 表示有理余权，每个坐标分别保存为有理数。它与 `RationalWeight` 的表示不同：后者使用 `Vec<i64>` 分子向量和一个公共的 `i64` 分母。^[lattice-types.md:17-20, lattice-types.md:73-79]

## 格类型与表示纪律

权格 `Weight` 对应 character lattice $X^*$，余权格 `Coweight` 对应 cocharacter lattice $X_*$。二者通过 perfect pairing 联系，但即使所选基下的坐标表示相同，也不能互换，因此实现刻意使用两个独立的 newtype。整数坐标保持 checked 固定宽度存储，精确解释器值在领域边界处转换，而不改变每个根系矩阵条目的表示。^[lattice-types.md:17-25]

## API 边界

`RationalCoweight::from_coordinates` 的可见性是 `pub(crate)`，构造入口限于 crate 内部。公开 API 为 `dimension()` 和 `to_rationals()`；后者向 workspace 消费者提供精确坐标视图。第三方 malachite 类型不进入该类型的公开 API，而解释器值层已公开使用 malachite 有理数。这一边界可结合 [[atlas-real-group 的 crate 门面与数学值边界]] 理解。^[lattice-types.md:75-79]

该类型没有提供算术操作，也没有实现 `Hash`。因此，其当前接口范围集中于逐坐标有理表示、维度查询与精确坐标导出。^[lattice-types.md:75-79]

## 分配与证据范围

源文件对按输入长度新建的 `Vec` 使用 `try_reserve_exact`，将预留失败映射为 `StructureError::AllocationFailed`；但 `to_rationals()` 使用裸 `clone()`，属于这项纪律的明确例外。相关错误分类见 [[StructureError 统一错误分类学]]。^[lattice-types.md:35-39]

这些说明来自对 `lattice.rs` 的结构性阅读。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；其正确性仍属于独立的 [[HPC 验收证据链]]。^[lattice-types.md:9-13, lattice-types.md:89-89]

## Sources

- [lattice-types.md](lattice-types.md) — 权格类型层：Weight、Coweight 与有理权。
