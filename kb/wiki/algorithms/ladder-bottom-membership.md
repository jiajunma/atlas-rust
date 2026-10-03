---
id: ladder-bottom-membership
type: algorithm
note_status: reviewed
aliases: [Root ladder bottoms]
source_snapshot: sources/snapshots/2026-10-03-root-ladder-after-v3.json
---

# Ladder bottom 的成员判定

编辑状态：本页已于 2026-10-03 按 AFTER-v3 限定接受重读。成员查询把
`ArithmeticOverflow` 解释为“不属于存储集合”的修复已在 acceptance index
entry `0003-a1-torus-root-coroot-ladder-boundary` 中登记为
`accepted + math_pass`，范围仅限 A1+中心环面坐标边界 fixture 与两个 crate
的回归套件；不含更高 rank、一般 root system、性能或内存结论。

这里解释 `RootSystem::min_roots_for` 对应的有限集合运算，以及为什么成员查询的中间差值溢出不能直接等同于数学对象无效。

## 输入与目标

给定完整存储的有限根集 $R$ 和其中的根 $\alpha$，计算

$$B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}.$$

余根表使用对应余根的差与余根集合做同样判定，返回的编号仍与根编号对齐。本页只解释这些集合，不据此声称任何更高层操作已经正确。

## 当前计算步骤

当前 `build_ladder_bottoms` 遍历所有有序根对。对每一对先做坐标减法，再在已排序根数组中二分查找；对余根则查预先建立的 `BTreeMap`。未找到时将对应 `RootId` 加入集合。有限的两层循环保证遍历终止；资源和算术错误仍可能提前结束构造。

需要保持的不变量包括：根与余根的编号对齐、根查找使用同一坐标序、集合按稳定编号输出，以及分配失败仍是独立错误。这里描述的遍历不构成性能测量。

## 固定宽度表示的边界

设存储集合 $S\subseteq I^d$，其中 $I$ 是 `i32` 可表示的整数区间。若在数学整数中计算的 $x-y$ 有某个坐标不属于 $I$，则 $x-y\notin S$。理由是 $S$ 中每个向量的每个坐标都属于 $I$，因而不可能与该差相等。

这个推导只支持完整 `i32` 存储集合中的成员判定。它不允许一般减法使用 wrapping 或 saturating 结果，不消除分配错误，也不能推广成反射等其他运算的溢出处理规则。

`subtract_coordinates` 在 `checked_sub` 失败时返回 `ArithmeticOverflow`；修复后
`build_ladder_bottoms` 恰在该错误分支把 membership 解释为 false，其余错误继续
传播。该修复的回归链与限定接受范围见
[修复来源包](../../sources/root-ladder-overflow-repair.md)。

## 来源与关联

- [root_system.rs](../../../crates/atlas-real-group/src/root_system.rs)：`min_roots_for`、`min_coroots_for`、`build_ladder_bottoms`、`subtract_coordinates`。
- [项目审查记录](../../../docs/HANDOFF.md)：`Original-source review at frozen commit` 段落限定修复范围。
- [来源快照](../../sources/snapshots/2026-10-03-root-ladder-after-v3.json)；历史初始快照
  `2026-10-01-initial.json` 保留不改写。
- 前置知识：[根坐标与格坐标](../math/root-coordinates.md)；历史输入和原版对比：[两版实现比较](../comparisons/root-ladder-cpp-rust.md)。
