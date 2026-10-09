---
title: 表示参数上下文：StandardRepr 与 RepContext
source: atlas-rust/rep-context
ingestedAt: 2026-10-03T10:32:42Z
---

# 表示参数上下文：StandardRepr 与 RepContext

编辑状态：**结构性阅读；Kimi probe 起草的骨架在超时处截断，已采纳部分经维护者
对照源码逐条核对，其余由维护者按源码补齐**。本包解释 `rep_context.rs` 的参数层
数学；参数层的正确性属于它自己的 HPC 证据链（orientation/deform/unitarity
gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-rep-context.json`](snapshots/2026-10-03-rep-context.json)
（`rep_context.rs` SHA-256
`18d0112e97a35869224b642512c8a1120f6df112798322b942d062be39475345`，dirty
工作区）。

## 定位与只读投影依赖

本模块移植上游 `gkmod/repr.cpp` 的参数层数学与 `structure/involutions.cpp`
的 `K_type` 归一化。`StandardRepr` 即 `repr.h:76-110` 的 $(x, y, \gamma,
\mathrm{height})$ 四元组，只能经 `RepContext::sr_gamma`/`RepContext::sr`
构造。每个 involution 的 $(1-\theta)X^*$ 图像基对（`lift_mat`、`M_real`）
存于 involution 表的记录中（上游 `InvolutionTable::record`，
involutions.h:104-105），沿 cross-action BFS 传送（involutions.cpp:242-243）；
context 只读它。阶梯归约逐步复刻上游 `matreduc::column_echelon`
（matreduc.h:129）及其 gcd sweep（matreduc.h:70），因为当选的
`lambda-rho` 代表元依赖精确的图像基。

## StandardRepr：字段、访问器与相等性

字段：`x: KgbId`、`y_bits: ModTwoVector`（`lambda` 的打包挠部分，上游
`y()`）、`gamma: RationalWeight`（无穷小特征，gcd-归一化）、
`height: u32`；外加缓存 `undefined_print_weights`——仅 `UndefKGB` twist
携带，使打印不依赖图索引，普通参数自行派生两个权重。相等性对应上游
`StandardRepr::operator==`（repr.cpp:36-40）：比较 `x`、打包挠部分与
`gamma`；派生的 `height` 不参与比较。对 undefined 参数的操作经
`ensure_defined` 报 `RepInvariantViolation`。

## RepContext：借用视图与派生常量

`RepContext<'a>` 借用 inner class、involution 表与该形式的 KGB 图——与图
构建时同一个 substrate 三元组——并持有 `Arc<RepContextDerived>`（根 datum
常量 $2\rho$、$2\rho^\vee$、$\rho$）。`new` 有两道一致性闸门：表的 inner
class 不同报 `DatumMismatch`；表与图的 `Arc` 指针不同也报
`DatumMismatch`。`from_derived`（`pub(crate)`）用 `debug_assert!` 复核三方
`Arc` 指针一致。

## 构造入口与派生链

- `sr_gamma(x, lambda_rho, gamma)`（repr.cpp:756-784）：打包 `lambda_rho`
  的挠部分，存入 `gamma` 与 $(1+\theta)\gamma$ 的 height。
- `sr(x, lambda_rho, nu)`（repr.h:242-244）：先算 `gamma` 再转 `sr_gamma`；
  `sr_of_ktype`（repr.h:252-253）以 $\nu=0$ 扩张 K-type；
  `sr_k_of_standard`（repr.h:232-233）反向由标准参数得 `KType`。
- `lambda_rho(z)`：`gamma - rho` 与其 $\theta$ 像相加，取整坐标后与
  `y_lift` 的挠提升逐坐标相加并减半；坐标和为奇数时报
  `RepInvariantViolation`（"lambda-rho halving"）。
- `lambda(z) = rho + lambda_rho`（repr.h:304）；`nu(z) = (\gamma -
  \theta\gamma)/2`（repr.cpp:239-245），即 $-\theta$-不动投影。
- `y_pack`（involutions.h:211-213）：`lambda_rho` 的 `M_real` 坐标 mod 2，
  作为图像基上的 `ModTwoVector`；`y_lift`（involutions.cpp:346-356）对打包
  挠部分计算 $(1-\theta)\lambda_\rho$。
- `lambda_unique`/`real_unique`/`gamma_lambda` 把代表元归一化；
  `lambda_unique` 用欧几里得除法 `div_euclid(2)`（arithmetic.h:249-253）
  取半——注释强调这不是 C++ 内建有符号除法：对负奇数截断会选出同一陪集的
  不同代表元，使公式项无法合并。

## 奇偶、朝向与 reducibility

- `is_parity(s, x, lambda_rho, gamma)`：把生成元 `s` 在 `x` 处的 KGB 状态
  转运到父单根，比较 $\theta_1\lambda_\rho + 2\rho_{\text{non-real}}$ 与
  $\langle\gamma, \alpha_s^\vee\rangle$ 的奇偶（repr.cpp:249 的补集）。
- `orientation_number(z)`：先 `made_dominant`，再按 real 正根的
  $2\rho_{\text{real}}$ 与 $\gamma - \rho + \rho_{\text{real}}$ 计算朝向数。
- `is_fixed`/`is_delta_fixed`：参数在（delta-）twist 下不变。
- `mod_reduce(z)`（repr.cpp:52-58 路径）：`gamma - rho - lambda_rho` 经
  `real_unique` 归一，返回 $(x, \gamma_\lambda)$——打印 wrapper 的种子计算。
- `build_srm(x, gamma_lambda)`（repr.cpp:61-67）：使 `gamma_lambda` 在该
  involution 处 `real_unique` 并规范化。
- `reducibility_points(z)`（repr.cpp:825-925）：按分子/分母对升序返回可约
  分数。
- `finals_for(z)`：以栈驱动的 final 化（dominant 检查 + 奇偶/长度下降），
  返回 `(StandardRepr, i32)` 系数对。
- `deformation_terms(block, y, gamma, lambda_rho, kl_table)`
  （repr.cpp:1933-2025）：`block.length(y) == 0` 的平凡情形返回空；空奇异
  集时每元素皆 final，reverse-accumulated 列表为 `[y, y-1, ..., 0]`。

## 来源与限制

- 源码：[rep_context.rs](../../crates/atlas-real-group/src/rep_context.rs)；
  阅读快照 [`2026-10-03-rep-context.json`](snapshots/2026-10-03-rep-context.json)。
- 上游行号均转述自源码注释（repr.h/repr.cpp/involutions.h/involutions.cpp/
  matreduc.h/arithmetic.h/basic_io.cpp），未独立重读上游，随版本演进可能漂移。
- 关联：[KLV 多项式](kl-polynomial-table.md)、
  [形变驱动](deformation-drivers.md)、[完整块图](block-graph.md)、
  [KGB 图结构](kgb-graph-structure.md)、
  [根坐标与格坐标](../wiki/math/root-coordinates.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`）；300 秒期限
  对 30KB 摘录仍不足，进程在写完代表元归一化一节后超时被 SIGTERM 清理、无
  残留进程组成员。已采纳部分由维护者对照源码逐条核对；其后各节由维护者按
  源码补齐。调用记录见快照的 `kimi_assist`。
