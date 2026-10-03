---
title: 扩展块：delta-不动部分与折叠生成元
source: atlas-rust/extended-block
ingestedAt: 2026-10-03T10:32:42Z
---

# 扩展块：delta-不动部分与折叠生成元

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `ext_block.rs` 的结构切片；扩展块的正确性属于它自己的 HPC 证据链
（ext-KL/unitarity gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-extended-block.json`](snapshots/2026-10-03-extended-block.json)
（`ext_block.rs` SHA-256
`dd48cc3bee5987f4e6eb0d9f9c9c9b7363109a87ceb47aa95a230ac5d06be098`，dirty
工作区）。

## 定位与切片边界

扩展块是普通块的 $\delta$-不动部分，生成元折叠进 $\delta$-轨道（上游
`gkmod/ext_block.{h,cpp}`）。本模块覆盖其**结构切片**；`ext_param`/`star`
属于后续切片，经 `StarOracle` trait 在此注入。元素编号、cross/Cayley 语义与
`UndefBlock` 编码（此处为 `None`）沿用 [完整块图](block-graph.md)。

## DescValue：32 值扩展下降分类

32 个变体按 One*/Two*/Three* 三族排列（ext_block.h:38-74）；`is_descent`
对应奇数枚举值（ext_block.h:77）。十二个谓词（`is_complex`、
`is_unique_image`、`has_double_image`、`is_like_noncompact` 等，
ext_block.cpp:42-160）外加 `generator_length`（折叠生成元长度，按族返回
1/2/3）与 `link_count`（:165-206；零链接类型如 `OneRealNonparity`/
`OneImaginaryCompact` 不记录 cross action）。`has_october_surprise` 按定义式
`generator_length == if has_defect { 3 } else { 2 }`（:134-139），注释说明
这是"links with an even length difference, singled out since October 2016"。

## 生成元折叠：ExtGen 与 fold_orbits

`ExtGen { kind: ExtGenKind, s0, s1 }`：`ExtGenKind::One/Two/Three` 对应轨道
大小；`s1` 在 `One` 时取 `usize::MAX`（上游 `~0`）；`length()` 返回 1/2/3。
`fold_orbits(cartan, twist)` 移植上游 `rootdata::fold_orbits(rd, delta)`
（rootdata.cpp:1532-1551），特化到父块简单生成元：`twist` 是 $\delta$ 诱导的
简单根置换，`cartan[i][j] = <alpha_i, alpha_j^v>`（crate 约定）；轨道按 `s0`
递增发出；非对合 `twist` 对应上游 "Not a distinguished involution"。

## extended_type 与两种构造

`extended_type`（ext_block.cpp:343-503）是父块上的纯组合局部类型识别。
`ExtBlock::build` 是全父块构造，block modifier 平凡（ext_block.cpp:618-668，
`bm` 为恒等，此时 `transformed_twisted`（:597-616）退化为对两个 KGB 坐标的
`kgb.twisted`），经 `complete_construction`（:696-856）与诱导置换 `induced`
（:670-693）完成。

`ExtBlock::build_partial` 在 `PartialBlock` 父块上构造（真积分子系统上的公共
块，blocks.cpp:733-1081）：不动点测试是 `transformed_twisted` 的
`x + gamma_lambda` 形式（partial block 的 `y` 是合成的子系统 y-计数，不是
对偶 KGB 元素，故全块路径的对偶 `kgb.twisted` 测试不适用）；
`fold_orbits` 运行在**子系统** Cartan 矩阵与子系统生成元 twist 上
（common_block::fold_orbits，blocks.cpp:1288-1292 经由 rootdata.cpp:1553-1577）；
生成元姿态在 `complete_construction` 之后 cofold（ext_block.cpp:636-663）：
diagram、轨道、链接表被 `induced(orbits, bm.simple_pi)` 置换，各轨道成员编号
经 `bm.simple_pi` 重写；恒等姿态是恒等置换，因而在既有路径上是无操作。
只有恒等生成元姿态被移植：非恒等的 `bm.simple_pi` 会显式失败。

## tune_signs 与调试门

`ExtBlock::tune_signs`（ext_block.cpp:1707-1876）泛型移植于 `StarOracle`
trait：逐生成元的 `star` 计算与其比较的 `ext_param` 值属于后续
`ext_param`/`star` 切片，故在此注入。调试验证门 `check_quadratic`/
`check_braid`（ext_block.cpp:2140-2245）已移植为同名项，并在 `tune_signs`
内部于 `debug_assertions` 下运行，恰如上游的 `#ifndef NDEBUG` 块。

## 访问器

`rank`（轨道数）、`size`、`orbit(s)`、`folded_generators`、`folded_cartan`
（上游 `folded_diagram` 的整数矩阵形式）、`descent_type(s, n)`、`z(n)`
（扩展元素 `n` 的父索引）、`element(zz)`（ext_block.cpp:1877-1891：最小使
`z(n) >= zz` 的 `n`，无则 `size()`）、`is_present(zz)`（ext_block.h:181-182）、
`length(n)`（同 `parent.length(z(n))`，ext_block.cpp:1894-1907）。

## 来源与限制

- 源码：[ext_block.rs](../../../crates/atlas-real-group/src/ext_block.rs)；
  阅读快照
  [`2026-10-03-extended-block.json`](snapshots/2026-10-03-extended-block.json)。
- 上游行号均转述自源码注释（ext_block.h/ext_block.cpp/blocks.cpp/
  rootdata.cpp），未独立重读上游，随版本演进可能漂移。
- 关联：[完整块图](block-graph.md)、[部分公共块](partial-common-block.md)、
  [KLV 多项式](kl-polynomial-table.md)、
  [形变驱动](deformation-drivers.md)；`ext_param`/`StarOracle`/`ext_kl` 的
  展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  180.0s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `DescValue` 32 值分类、`build_partial` 的不动点测试与 cofold
  的内容均已按源码落实，其余骨架内容未采用。调用记录见快照的
  `kimi_assist`。
