---
title: 权格类型层：Weight、Coweight 与有理权
source: atlas-rust/lattice-types
ingestedAt: 2026-10-03T10:32:42Z
---

# 权格类型层：Weight、Coweight 与有理权

编辑状态：**结构性阅读（两次：2026-10-03 初读、2026-10-06 重读，字节未变，
SHA-256 相同）；两份草案均经 Kimi probe 起草、维护者逐条对照源码核对后改写合并**。
本包解释 `lattice.rs`（434 行）的格类型纪律；其正确性属于它自己的 HPC
证据链（lattice gate 等），本包不重述也不扩展。上游行号（ratvec.cpp、
repr.cpp 等）转述自源码注释。

## 定位与设计决策

四个格元素类型：`Weight(Vec<i32>)`（character lattice $X^*$）、
`Coweight(Vec<i32>)`（cocharacter lattice $X_*$）、`RationalWeight`
（公共分母 `numerator: Vec<i64>` + `denominator: i64`）、`RationalCoweight`
（逐坐标 `Rational`）。

设计决策（文档载明）：坐标**刻意保持 checked 固定宽度存储**，精确解释器值在
domain boundary 处转换，而不是改变每个根系矩阵条目的表示。`Weight` 与
`Coweight` 是刻意分离的两个 newtype：两格之间有 perfect pairing 但**不可互换**
——即使所选基给出相同的坐标表示。

## 配对与坐标辅助

- `pair(weight, coweight)`：canonical character–cocharacter pairing；先查秩
  一致（`RankMismatch { expected: weight.rank(), actual: coweight.rank() }`），
  再委托 `pair_coordinates`（i128 checked 累加 + `i32::try_from` 收窄）。
- crate 内部辅助：`pair_coordinates`（坐标级配对，zip 截断靠调用方保证）、
  `checked_add_weights`/`checked_sub_weights`（equal-rank 的 checked 逐坐标
  和/差；`i32::MIN * -1` 一类情形被 `checked_mul(sign)` 拦截）、
  `try_copy_coordinates`（把 reservation failure 映射为
  `StructureError::AllocationFailed`）。
- 预算纪律（2026-10-06 重读补充）：凡按输入长度新建 `Vec` 一律
  `try_reserve_exact` 并映射 `AllocationFailed`；例外是 `halve`/`normalized`/
  `to_rationals` 的裸 `clone()`。

## RationalWeight：公共分母与 gcd 归一化

`new(numerator, denominator)`：构造即 gcd 归一化，拒绝非正分母（对照上游
`normalize` 拒绝零分母，ratvec.cpp:172-175；`denominator <= 0` 报
`RepInvariantViolation { invariant: "rational weight denominator" }`）。
`from_weight` 把整权视为分母 1 的有理权；`zero(rank)`、`add`/`sub` 返回
`Result`（`combine` 交叉相乘公分母后再归一）。

其余方法（2026-10-06 重读补充）：

- `apply_matrix`：保持分母、只作用分子（ratvec.cpp:190）；
- `halve`：只翻倍分母，**刻意不归一**——归一时机交给调用方，与上游一致；
  `normalized` 重跑构造器的 gcd 归一；
- `integral_coordinates`：上游 `assert(entry%denominator==0)` 的可检查化
  （repr.cpp:194-199、768-774；`"rational weight integrality"`）；
- `scale(numerator, denominator)`：标量乘后归一
  （`"rational weight scale denominator"`；K_repr.cpp 的 height_bound/单项式
  算术一族）；
- `dot_coroot`：`dot_Q`（ratvec.h:167-175），返回已约分的 `(i64, i64)`。

复核备注（2026-10-06 重读标记，均属实并保留）：

- `dot_coroot` 的 `RankMismatch` 字段顺序是 `expected: coroot.rank(),
  actual: self.rank()`，与 `pair`/`combine` 的惯例相反；
- `apply_matrix` 行长失配也报 `actual: matrix.len()`（与 compose_matrices
  的填报怪癖一族）；
- `new` 里 `i64::try_from(gcd)`/`checked_div` 两个溢出口在当前不变量下
  不可达（防御性写法）；
- 本文件 `gcd_u64` 返回 `left.max(1)`（故 `gcd(0,0)=1`），与
  `global_kgb.rs` 同名私有副本返回 `left`（`gcd(0,0)=0`）不同——两处语义在
  各自调用点都安全（分母恒正），但重复实现已开始漂移。

## RationalCoweight：逐坐标有理数

`RationalCoweight { coordinates: Vec<Rational> }` 与 `RationalWeight` 的公共
分母表示形成对照；第三方 malachite 类型不进公开 API。`from_coordinates` 是
`pub(crate)`；公开 API 是 `dimension()` 与 `to_rationals()`（为 workspace
消费者提供精确坐标视图——解释器的值层已公开使用 malachite 有理数）。
无算术、无 `Hash`。

## 来源与限制

- 源码：[lattice.rs](../../crates/atlas-real-group/src/lattice.rs)；
  阅读快照 [`2026-10-03-lattice-types.json`](snapshots/2026-10-03-lattice-types.json)
  （初读）与 [`2026-10-06-lattice-kl-polynomial.json`](snapshots/2026-10-06-lattice-kl-polynomial.json)
  （重读，同一 SHA-256 `3cf4e62c…`，重读与 kl_polynomial.rs 同包进行）。
- 关联：[根坐标与格坐标](../wiki/math/root-coordinates.md)、
  [整数格](integer-lattice.md)、[表示参数上下文](rep-context.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 初读起草经由本地 Kimi probe（exit 0，215.9s）；重读同样经 Kimi probe
  （600s 期限，exit 0，326.1s；其 r1 在 360s 期限超时留有草稿残片——超时
  规则由此上修为 ~24s/KB），草案的四条复核备注均属实并已并入正文。调用
  记录见两份快照的 `kimi_assist`。
