---
title: 共享块存储：reduced 键控复用与 RepTableOwner
source: atlas-rust/rep-table
ingestedAt: 2026-10-03T10:32:42Z
---

# 共享块存储：reduced 键控复用与 RepTableOwner

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `rep_table.rs` 的键控复用与并发约定；块存储的正确性属于它自己的
HPC 证据链（PSp4R/GL2 gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-rep-table.json`](snapshots/2026-10-03-rep-table.json)
（`rep_table.rs` SHA-256
`5c75d719137eab7aab4a47bf3a1b829381a937380e3e18558a5eec9702d1eed7`，dirty
工作区）。

## 定位

这是上游 `gkmod/repr.cpp` 的 `Rep_table` 切片：为**一个实形式**提供共享的
部分/完整公共块存储。locator 切片的规范键控已接入（模块文档称之为 "step 3"）：
reduced 键是上游 `Reduced_param` 值 `(x, int_sys_nr, residue)`，由
`Reduced_param::reduce`（repr.cpp:110-125）计算——`InnerClass::int_item`
在 Weyl 群作用下规范化整数据，srm 由 locator attitude 传输，residue 取自
规范数据的 Smith codec。若查询的积分子系统在某个 Weyl 姿态下与已存块匹配，
就**复用**该已存块；查询到已存块的 `block_modifier`（repr.cpp:338-350 的
`make_relative_to`）记录姿态差，仍假设恒等姿态的消费者被显式门控。reduced
键与其 Smith codec 保持私有；消费者只获得稳定的块句柄与查询相对的代表元。

## ReducedParamKey

`ReducedParamKey { x: KgbId, int_sys: u32, residue: u32 }`（模块私有）是
reduced 参数的哈希稳定身份：`x` 是经 locator attitude 传输**之后**的 KGB
元素（`transform<true>(loc.w, srm)`），`int_sys` 是规范整数据 id
（`locator::int_sys_nr`），`residue` 是规范 codec 各赋值的混合进制打包。

## LocatedBlock：稳定块句柄

- `block()`：`Arc<PartialBlock>`；`raw_row()`：查询在存储块编号中的行号；
  `is_full()`：句柄是否指向完整公共块。
- `prepared_query()`：部分查找存规范化（normalised）查询，完整查找存
  dominant 查询；reduced 键与块相对代表元正是从该参数算得。
- `has_identity_generator_attitude()`：为真当且仅当查询到存储的 block
  modifier 具有恒等 `w` 与恒等 `simple_pi`；只有此时消费者才可用平实中心
  位移直接读存储行。
- `block_modifier()`/`relative_shift()`（repr.h:494 的 `shift`）/
  `adapted_representative()`：查询相对已存块的姿态数据。

## with_kl_table：共享 KL 表与并发约定

`with_kl_table(operation)` 以该记录**惰性构造的共享 KL 表**运行回调：
记录局部互斥锁在整个回调期间保持持有，对同一块的调用被串行化；重入禁令
——KL 回调不得对**任何块**再次调用 `with_kl_table`，同线程嵌套会在获取
另一记录锁**之前**由 `ActiveKlCallback::enter()` 返回稳定的不变量错误
（`RepInvariantViolation`）。

## RepTableOwner

- `new(table, graph)` 校验并绑定自有的 involution 表/KGB 图对；
  `from_shared` 绑定已共享的底层基件；`context()` 借出临时 `RepContext`。
- `lookup(query)` 解析或物化查询**下方最小的部分块**；
  `lookup_full_block(query)` 解析或物化**包含查询的完整公共块**。
- `table()`/`graph()` 是过渡性访问器。
- `k_type_formula(ktype, max_level)`（K_repr.cpp:591-602 的
  `Rep_table::K_type_formula`）：memoized 的 K 型公式。键是该实形式所有者
  内部的**严格 K 型身份** `(x, lambda_rho)`。可能返回缓存的更高截断——
  调用方在导出前必须自行把各项截断到所求高度。并发设计：**生成期间不持有
  共享互斥锁**；先在锁外计算，提交时再复核——若另一调用方在此期间提交了
  更大截断的公式，保留后者。这是并行赛道值得记录的既有模式。

## 来源与限制

- 源码：[rep_table.rs](../../crates/atlas-real-group/src/rep_table.rs)；
  阅读快照 [`2026-10-03-rep-table.json`](snapshots/2026-10-03-rep-table.json)。
- 上游行号均转述自源码注释（repr.h/repr.cpp/K_repr.cpp），未独立重读上游，
  随版本演进可能漂移。
- 关联：[表示参数上下文](rep-context.md)、[部分公共块](partial-common-block.md)、
  [KLV 多项式](kl-polynomial-table.md)、[形变驱动](deformation-drivers.md)；
  `BlockLocator`、`IntegralDatumTable`、`ActiveKlCallback` 的展开属于后续
  来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  112.6s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `with_kl_table` 的互斥/重入约定、`k_type_formula` 的更高截断
  复用与锁外计算、两个 `lookup` 入口语义的内容均已按源码落实，其余骨架
  内容未采用。调用记录见快照的 `kimi_assist`。
