---
title: 形变驱动：twisted 与 block 形变
source: atlas-rust/deformation-drivers
ingestedAt: 2026-10-03T10:32:42Z
---

# 形变驱动：twisted 与 block 形变

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `deform.rs` 的移植边界与驱动结构；形变计算的正确性属于它自己的
HPC 证据链（ordinary-deform/G2/PSp4R gate 等），本包不重述也不扩展。所读
字节见
[`snapshots/2026-10-03-deformation-drivers.json`](snapshots/2026-10-03-deformation-drivers.json)
（`deform.rs` SHA-256
`3e5dc2c99fcde6f21b4a0dc341243dfe579250326d51c5ab553a0a1f51d91713`，dirty
工作区）。数学记号沿用 $D(z)=\sum c\,(L(t)+2D(t))$、$F(z)=L(z)+(1-s)D(z)$。

## 移植入口与上游对照

模块移植 gkmod/repr.cpp 的四个驱动：
`twisted_deformation_terms`（repr.cpp:2426-2520）、$s$ 处的 twisted KL 和
（自由函数 repr.cpp:2304-2350 与 `Rep_table` 变体 repr.cpp:2371-2423）、
`block_deformation_to_height`（repr.cpp:2027-2124）、递归
`twisted_deformation`（repr.cpp:2552-2653）。移植遵循冻结的
`domain/deform` 契约，简化清单如下。

## 移植简化清单（frozen domain/deform 契约）

- `lambda_rho` 由调用方一次性提供，而非按块元素从上游 `StandardReprMod` 池
  读取（`common_block::sr`，blocks.cpp:1260-1264）；per-element `lambda_rho`
  在 full block 上确实变化（SL(2,R) 块在 $\gamma = 2\rho$ 时 compact-Cartan
  元素为 `[1]`、split 元素为 `[0]`），故调用方须传入其所有形变项共享该值的
  参数——语言层用 `rc.lambda_rho(p)`，与已验证的 `deform` 分支一致。
- 在 PROPER integral 子系统上，父块是 `PartialBlock`（`common_context` 的
  `common_block`，repr.cpp:2666-2670），每行重构用自己的 stored
  `gamma_lambda` 加 lookup 的 block modifier（`RepContext::sr_with_modifier`，
  repr.cpp:815-823），与上游 `common_block::sr` 一致。
- rank-0 integral 子系统由 `IntegralBlockScope` 检测：common block 是长度 0
  的单例 `{p}`，语言层走该 fast path。
- 上游按 `block_modifier` 索引的 singular-orbit 计算（repr.cpp:2380-2390 与
  2617-2633）在 `bm` 平凡时与 `ExtBlock::singular_orbits`（plain
  simple-coroot singular set）一致；此时 `bm.simp_int` 是 identity-indexed
  simple list、`bm.simple_pi` 是 identity permutation。
- `weyl::alcove_center` 收缩（repr.cpp:2556-2557）在
  `gamma.denominator() > 2^rank` 时先于递归 twisted deformation 应用。
- `Rep_table` memoisation（`deformation_unit`/`alcove_hash`）以朴素重算替代；
  无共享池时 memo-hit flip 调整（repr.cpp:2576-2584）不必要，因为每个结果
  都是为正在报告其 flip 的那个参数本身计算的。

## SplitInteger：$a + b\,s$ 系数类型

`SplitInteger { a, b }` 表示 $a + b\,s$，是 $F(z)=L(z)+(1-s)D(z)$ 中
$s$-系数的载体。字段为 `i32`，所有算术用 `wrapping_*`（沿用上游
arithmetic.h 的环绕语义；溢出防护不属于本类型）。运算与上游一一对应：
`add_int`（arithmetic.h:173，整数加到 `1`-部分）；`times_s`（:197-199，
$(a+bs)\cdot s = b + as$）；`times_1_s`（:200-204，
$(a+bs)(1-s) = (a-b)+(b-a)s$）；`negate`/`mul_int` 逐分量；`Add`/`Mul`
trait（分裂乘法 $(a+bs)(c+ds) = (ac+bd)+(ad+bc)s$）；与 `(i32, i32)` 的
双向 `From`。`Split(c, -c)` 即 $c\,(1-s)$。

## IntegralBlockScope 与奇异集

`integral_block_scope(rc, gamma)` 按 `gamma` 的 integral root system 分类
（blocks.cpp:701-709 为 singular set 读取同样的 pairings）：coroot 与
`gamma` 整配对当且仅当 `coroot · gamma.numerator()` 被
`gamma.denominator()` 整除。三个变体：

- `Singleton`：无根整配对——rank-0 子系统，common block 是长度 0 的单例
  `{p}`；其形变项为空（`block.length(y) == 0`，repr.cpp:2435-2436），
  twisted KL 和在 $s$ 处为 `1*p`。
- `Full`：每个 simple coroot 都整配对——带平凡 block modifier 的完整块，
  各驱动接受 `BlockGraph`。
- `ProperSubsystem`：真积分子系统——common-block 支撑不在此移植面；调用方
  必须显式失败而不是在完整块上默默计算（A1 在 $\nu=[1]/2$ 的陷阱：full
  block 大小为 3 而积分块是单例）。

`simple_singular_flags` 是平凡 block modifier 下的 `common_block::singular(gamma)`
（blocks.cpp:701-709）；`singular_orbits_at` 给出 extended block 在 `gamma`
处的 singular-orbit flags（repr.cpp:2380-2390 取平凡 `bm`，或 wrapper 的
plain fold，atlas-types.w:8138-8141）。partial parent 以同样方式折叠其子系统
生成元的 singular set（`common_context::singular`）。

## 父块抽象

`KlSumParent<'a>` 是 twisted KL 和与 twisted deformation 读取的父 common
block：`Full { block: &BlockGraph, lambda_rho: &Weight }`（调用方提供的常量
`lambda_rho`），或 `Partial { block: &PartialBlock, modifier:
Option<&BlockModifier> }`（每行按 `common_block::sr` 重构自己的
`lambda_rho`）。`DeformParent` 是递归 twisted deformation 中
reducibility-point lookup 返回的 owned 版本（`Full { Box<BlockGraph>, Weight }`
或 `Partial { Arc<PartialBlock>, BlockModifier }`）；递归驱动必须在
`twisted_deformation_terms` 借用其块视图期间保持所选 parent 存活，经
`as_kl_sum_parent` 降级借用。

## 两个 twisted KL 和：长度函数的差异

- `twisted_kl_sum`：自由函数 `twisted_KL_sum`（repr.cpp:2304-2350）——
  EXTENDED-block 元素 `y` 在 $q = s$ 处的交错 twisted KL 列和，符号取自
  EXTENDED block 自身的长度函数（`eblock.length`，repr.cpp:2339-2344）。
- `twisted_kl_column_at_s`：`Rep_table::twisted_KL_column_at_s`
  （repr.cpp:2371-2423）——PARENT 块元素 `y0` 处参数的交错和，符号取自
  PARENT 块的长度函数（`parent.length(eblock.z(x))`，repr.cpp:2416）。
  这是两者的关键差异。

## 形变项与块形变

- `twisted_deformation_terms`：`Rep_table::twisted_deformation_terms`
  （repr.cpp:2426-2520）的平凡 block modifier 版——final、delta-fixed 父块
  元素 `y`（PARENT 编号）的形变项，以 `(StandardRepr, int)` 对按
  reverse-accumulated finals 顺序返回；wrapper 把每个系数 `c` 映射为
  `Split(c, -c)` 并按 `SR_poly` 顺序排序。
- `common_deformation_terms`：对 `RepTable::lookup` 返回的 partial block 求
  common-block 形变项（repr.cpp:1933-2025），走上游
  `contributions(block, block.singular(bm,gamma), y)` 路径。
- `block_deformation_to_height`：`Rep_table::block_deformation_to_height`
  （repr.cpp:2027-2124）的平凡 block modifier 版——形变 `p` 的 full block 中
  height 不超过 `height_bound` 的项（`u32::MAX` 对应上游负界的 "maximal
  level"）；返回 downward（reversed）block order 的 `(StandardRepr,
  SplitInteger)` 项，以及与 `accumulator` 平行的已消费项 flags（上游用
  `queue.erase` 原地改队列；Atlas 值不可变，改为报告该 split）。上游在
  height bound 之上用 `plug_hole` 填对偶块的 KL 表；holes 只跳过工作而
  多项式值不依赖它们，故本移植填整张表（repr.cpp:2057）。

## 递归 twisted_deformation

`Rep_table::twisted_deformation`（repr.cpp:2552-2653），不含 memoisation：
final、delta-fixed 参数 `z` 在 `ctx` 的 delta 上的完整递归 twisted
deformation，输出 `(KType, SplitInteger)` 项，外加 shrink-wrap 到最后
reducibility point 所记录的 net flip（无 shrink-wrap 时 `flip == false`，
repr.cpp:2561）。`lookup` 在 INTEGRAL `gamma` 的 reducibility-point 参数处
扮演 `rt.lookup(zi, index, bm)` + `block.extended_block(bm, ...)` 的角色；
rank-0 单例不贡献形变项且不调用 `lookup`。可取消变体
`twisted_deformation_with_cancel` 在每次递归或块级操作之间检查取消 probe，
取消时返回 `Ok(None)` 且不发布部分多项式。

## 来源与限制

- 源码：[deform.rs](../../crates/atlas-real-group/src/deform.rs)；阅读快照
  [`2026-10-03-deformation-drivers.json`](snapshots/2026-10-03-deformation-drivers.json)。
- 上游行号均转述自源码注释（repr.cpp/blocks.cpp/arithmetic.h/
  atlas-types.w），未独立重读上游，随版本演进可能漂移。
- 关联：[KLV 多项式](kl-polynomial-table.md)、
  [部分公共块](partial-common-block.md)、
  [完整块图](block-graph.md)；`ExtBlock`/`ExtKlTable`、`RepContext`、
  `BlockModifier` 的展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  168.5s）。草案由维护者对照源码逐条核对改写；其「待源码核对」项中涉及
  `SplitInteger` 算术、`IntegralBlockScope` 三变体、`KlSumParent`/
  `DeformParent` 变体、两个 twisted KL 和的长度函数差异的内容均已按源码
  落实，其余骨架内容未采用。调用记录见快照的 `kimi_assist`。
