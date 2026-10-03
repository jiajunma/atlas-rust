---
title: 权格类型层：Weight、Coweight 与有理权
source: atlas-rust/lattice-types
ingestedAt: 2026-10-03T10:32:42Z
---

# 权格类型层：Weight、Coweight 与有理权

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `lattice.rs` 的格类型纪律；其正确性属于它自己的 HPC 证据链
（lattice gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-lattice-types.json`](snapshots/2026-10-03-lattice-types.json)
（`lattice.rs` SHA-256
`3cf4e62cf3e1a14da2e93a773751fd0b1412fc2deac9a1ec7856e591250f8509`，dirty
工作区）。

## 定位与设计决策

四个格元素类型：`Weight(Vec<i32>)`（character lattice $X^*$）、
`Coweight(Vec<i32})`（cocharacter lattice $X_*$）、`RationalWeight`
（公共分母 `numerator: Vec<i64>` + `denominator: i64`）、`RationalCoweight`
（逐坐标 `Rational`）。

设计决策（文档载明）：坐标**刻意保持 checked 固定宽度存储**，精确解释器值在
domain boundary 处转换，而不是改变每个根系矩阵条目的表示。`Weight` 与
`Coweight` 是刻意分离的两个 newtype：两格之间有 perfect pairing 但**不可互换**
——即使所选基给出相同的坐标表示。

## 配对与坐标辅助

- `pair(weight, coweight)`：canonical character–cocharacter pairing；先查秩
  一致（`RankMismatch`），再委托 `pair_coordinates`。
- crate 内部辅助：`pair_coordinates`（坐标级配对）、`checked_add_weights`/
  `checked_sub_weights`（equal-rank 的 checked 逐坐标和/差）、
  `try_copy_coordinates`（把 reservation failure 映射为
  `StructureError::AllocationFailed`）。

## RationalWeight：公共分母与 gcd 归一化

`new(numerator, denominator)`：构造即 gcd 归一化，拒绝非正分母（对照上游
`normalize` 拒绝零分母，ratvec.cpp:172-175）。`from_weight` 把整权视为分母 1
的有理权；`zero(rank)`、`add`/`sub` 返回 `Result`。

## RationalCoweight：逐坐标有理数

`RationalCoweight { coordinates: Vec<Rational> }` 与 `RationalWeight` 的公共
分母表示形成对照；`from_coordinates` 是 `pub(crate)`；公开 API 是
`dimension()` 与 `to_rationals()`（为 workspace 消费者提供精确坐标视图——
解释器的值层已公开使用 malachite 有理数）。

## 来源与限制

- 源码：[lattice.rs](../../../crates/atlas-real-group/src/lattice.rs)；
  阅读快照 [`2026-10-03-lattice-types.json`](snapshots/2026-10-03-lattice-types.json)。
- 上游行号转述自源码注释（ratvec.cpp），未独立重读上游。
- 关联：[根坐标与格坐标](../wiki/math/root-coordinates.md)、
  [整数格](integer-lattice.md)、[表示参数上下文](rep-context.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  215.9s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `pair` 的秩检查、`RationalWeight::new` 的归一化与拒绝条件的
  内容均已按源码落实，其余骨架内容未采用。调用记录见快照的 `kimi_assist`。
