---
title: per-involution (1-θ)X* 图像基对：播种、传送与坐标
source: atlas-rust/real-projection
ingestedAt: 2026-10-03T10:32:42Z
---

# per-involution (1-θ)X* 图像基对：播种、传送与坐标

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `real_projection.rs` 的基对与传送纪律；其正确性属于它自己的 HPC
证据链（signed-projection 与 fundamental-lattice gate 等），本包不重述也不
扩展。所读字节见
[`snapshots/2026-10-03-real-projection.json`](snapshots/2026-10-03-real-projection.json)
（`real_projection.rs` SHA-256
`bd76f5d1f170cb0ae27f5b5d9ee36a34b6a406448deb068b2d86a052edb3f5e3`，dirty
工作区）。

## 定位：什么是 (lift_mat, M_real)

每个 involution 的 $(1-\theta)X^*$ 图像基对（上游 `InvolutionTable::record`
对，involutions.h:104-105）：`lift_mat` 是 $1-\theta$ 像的列 echelon 基
（$n \times r$），`m_real`（$r \times n$）把像元素表为该基下的坐标。二者
满足 $\mathrm{lift\_mat} \cdot \mathrm{m\_real} = 1 - \theta$
（involutions.h:105）。

## 基的不唯一性与播种/传送纪律

基**并非**由 $\theta$ 唯一决定。上游纪律：在 Cartan 轨道的 canonical
involution 处由 $1-\theta$ 的 echelon 归约播种（`InvolutionTable::add_involution`，
involutions.cpp:196-208），再沿 cross-action BFS 传送（`add_cross`，
involutions.cpp:242-243）。播种端的 echelon 归约逐操作复现上游
`matreduc::column_echelon`（matreduc.h:129）及其 gcd sweep（matreduc.h:70）——
因为被选出的 `lambda-rho` 代表元与 `y_lift` 的符号依赖精确的图像基。

`build(theta)` 是 `column_echelon` 作用于 $1-\theta$ 的移植，增量跟踪列操作
矩阵及其逆；上游随后取该逆矩阵的前 $r$ 行作为 `M_real`。

## 传送与坐标接口

- `transported(reflection)`：沿一条 cross 边（一个单反射）传送——
  `lift_mat' = reflection * lift_mat`，`m_real' = m_real * reflection`，
  保持分解不变式 $(sL)(Ms) = s(1-\theta)s = 1-\theta'$；用 checked i64
  算术，溢出报 `ArithmeticOverflow`。传送所得基与对 $1-\theta'$ 重新 echelon
  归约的结果相差列符号/列次序——这正是上游把基存放在 record 中携带而非
  重算的原因。
- `check_against(theta)`：校验 $\mathrm{lift\_mat} \cdot \mathrm{m\_real}
  = 1 - \theta$。
- `coordinates(weight)`：$(1-\theta)v$ 在图像基下的坐标，即 $M_{real} \cdot
  v$（involutions.h:211）；`lift(coordinates)`：`lift_mat * coordinates`，
  把坐标映回 $X^*$（involutions.cpp:346-356）。

## 来源与限制

- 源码：[real_projection.rs](../../../crates/atlas-real-group/src/real_projection.rs)；
  阅读快照
  [`2026-10-03-real-projection.json`](snapshots/2026-10-03-real-projection.json)。
- 上游行号均转述自源码注释（involutions.h/involutions.cpp/matreduc.h），
  未独立重读上游，随版本演进可能漂移。
- 关联：[Twisted involution 表](involution-table.md)、
  [表示参数上下文](rep-context.md)、[整数格](integer-lattice.md)、
  [Tits 元素](tits-element.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  74.5s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `transported` 的两个矩阵方向、`check_against` 与坐标/lift 接口的
  内容均已按源码落实，其余骨架内容未采用。调用记录见快照的 `kimi_assist`。
