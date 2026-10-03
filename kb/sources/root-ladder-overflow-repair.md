---
title: Root ladder 固定宽度坐标溢出修复
source: atlas-rust/root-ladder-overflow-repair
ingestedAt: 2026-10-01T05:23:39Z
---

# Root ladder 固定宽度坐标溢出修复

编辑状态：本来源包的证据窗口止于 BEFORE-v3，尚未纳入仓库中时间上
更新的 root-ladder 记录。下文的“候选”与“尚未”保留为该快照时点的
历史状态；本包在重新读取源码和证据、追加新快照前不得当作当前结论。

本来源包限定一个很窄的数学错误：Rust 在构造 root/coroot ladder bottom
表时，把环境格坐标差的 `i32` 溢出当成整个 `RootSystem` 构造失败。这里把
数学推导、original Atlas 的算法、Rust 候选修复和 HPC 证据分开记录。

该证据窗口冻结时的状态是 **候选修复，AFTER 尚未接受**。已接受的 BEFORE 只证明三条新增
回归在未修改生产代码上按预期失败；它不证明候选代码正确。

## 数学对象与局部推导

对完整存储的有限根集 $R\subseteq\mathbb Z^d$ 和 $\alpha\in R$，ladder
bottom 集为

$$B_\alpha=\{\beta\in R\mid \beta-\alpha\notin R\}.$$

Rust 当前把每个已存根和余根坐标表示为 `i32`。若在数学整数中计算
$\beta-\alpha$ 时某个坐标不属于 `i32` 的可表示区间，那么这个精确差不可能
等于任何已存向量；因此在这个成员查询中答案必为 `false`，相应的 $\beta$
应进入 bottom 集。

这个论证只适用于“精确差是否属于完整 `i32` 存储集合”的查询。它不允许
wrapping/saturating 算术，也不适用于一般向量减法、反射、root combination、
seed negation 或构造输入验证。分配失败以及未来其他错误仍必须传播。

## original Atlas 的构造边界

冻结的 original commit 是
`7e1b958c7aa9456769cc9cf09ac1542814b4800a`。在
`sources/structure/rootdata.cpp:238-317`，原版先用抽象简单根坐标
`Byte_vector` 建立 simple-root ladder，再用 Weyl reflection permutation 把表
传到所有正根和负根。含 torus factor 的环境格嵌入所产生的大坐标不进入
这段 ladder 减法；原版的抽象简单根坐标阶段不读取它。

这与 Rust 逐对相减环境格坐标的实现策略不同，但 observable contract 仍是
同一个根/余根成员关系。这里不主张把 Rust 整体重写成 C++ 数据结构，只要求
固定宽度表示不能改变该成员关系。

## Rust 候选的精确范围

Git base 是 `eba9c7ea080e61de9d4b105fbf54589c44a10b87`；工作区文件
`crates/atlas-real-group/src/root_system.rs` 的候选字节 SHA-256 是
`cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9`。

候选的生产代码增量只修改 `build_ladder_bottoms` 的两次 membership 查询；
tests-first 的 test hunk、session 回归和 fixture 是另一个固定增量，由 BEFORE
证据单独约束：

- `subtract_coordinates` 成功时，保持原来的 root 二分查找或 coroot map 查找；
- 恰为 `StructureError::ArithmeticOverflow` 时，把 membership 解释为 false；
- `AllocationFailed` 和任何其他错误继续返回；
- root 与 coroot 查询独立执行，root 溢出不会跳过 coroot 查询；
- 不修改 `subtract_coordinates`、reflection、`combine_roots`、数据布局、排序或
  public API。

共享的 `difference` 缓冲区在 helper 每次调用开始时清空；溢出后的 partial
prefix 不在错误分支读取，下一次调用会再次清空。候选不是性能优化：过去早退
的极值输入现在会完成 $O(|R|^2)$ 表构造，因而可能使用更多时间；只有受控测量
才能支持性能或内存结论。

## 回归和现有证据

fixture `tests/math/generics/root_ladder_coordinate_boundary.atlas` 包含 11 个
A1+torus case，覆盖 root/coroot 交换、两种 numbering、阈值下方、阈值上方和
`i32::MAX`，并保留 recovery marker 719。original 完整 stdout SHA-256 为
`3a7fdade43c46cf4f3048b52cf81f282db060012951ee559296f017f7d3eab80`。
其 oracle provenance 来自历史 original-backed capture job 3868832，仓库记录
[`math_weyl_context_capture_2026_09_30.json`](../../tests/reference/hpc/math_weyl_context_capture_2026_09_30.json)
绑定 case、oracle binary/source 和 raw stream；这不是候选 AFTER 执行。

HPC BEFORE-v3 job 3873400 的独立检查记录是
`tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json`，其文件
SHA-256 为
`608e996acac10ea1bacca177468fcd83f3ec39c3e579899440e11aab674fc37f`。
该记录接受的范围是：65 个 harness checker 通过，测试 inventory 为
atlas-real-group 521 / atlas-core 630，且未修复生产代码时精确出现 2 个 domain
失败和 1 个 core full-stream 失败，0 ignored。这是 tests-first 证据，不是
AFTER pass。

候选只有在 HPC AFTER 同时证明三条 focused 回归全部通过、两个 crate 的完整
521/630 测试实际全部通过、original 与 Rust 完整流相等、相关溢出负例仍保留、
retained gates 不丢失且源码/补丁/报告哈希闭合后，才具备送交独立 review 的
条件。AFTER execution/report 不能自我验收；独立 review 本身也不能绕过正式
账本。当前 acceptance index 尚未注册 ladder 的 stable claim contract、capture
validator 或 claim-specific independent-review validator，不应临时编造
`claim_id`。只有这些合约和 validator 经审查、独立机器可读 review 通过，并在
hash-chained、checkpointed acceptance index 中追加 `accepted + math_pass` entry
后，wiki 才能把候选改写成已验收行为。

## 精确阅读快照与限制

本次读取字节见
[`sources/snapshots/2026-10-01-root-ladder-repair-candidate-v2.json`](snapshots/2026-10-01-root-ladder-repair-candidate-v2.json)。
它后继但不改写初始 candidate snapshot，并补入 tests-first patch 与历史 capture
provenance。
其中 original 条目使用 `commit:path` 逻辑标识；读取来自只读临时 checkout，
身份由 commit、文件哈希与 clean 状态共同约束，绝对路径不是长期依赖。

本来源包没有执行本地 Atlas 构建或测试，没有提升 rank，没有证明一般 root
system、KGB、KLV、unitarity、Hodge、associated cycle 或 AV-ann 正确，也没有
给出加速或内存节约结论。
