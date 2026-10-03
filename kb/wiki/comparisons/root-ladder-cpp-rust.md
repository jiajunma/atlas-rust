---
id: root-ladder-cpp-rust
type: comparison
note_status: reviewed
aliases: [Ladder coordinate boundary]
source_snapshot: sources/snapshots/2026-10-03-root-ladder-after-v3.json
---

# Ladder 的 C++ 与 Rust 实现比较

编辑状态：本页已于 2026-10-03 按 AFTER-v3 限定接受重读。历史上的
`REJECTED_RUNTIME` 差异已被限定修复闭合：AFTER-v3（job 3875239）独立接受，
acceptance index entry `0003-a1-torus-root-coroot-ladder-boundary` 为
`accepted + math_pass`；范围与限制以该 entry 的 limitations 为准，不能外推。

两版的数学目标都是判断根或余根的差是否属于相应集合，但计算坐标不同。这里把当前 Rust 源码观察、已有 C++ 审查记录和历史执行报告分开说明。

| 层面 | 原版 | Rust |
| --- | --- | --- |
| 坐标选择 | 已有审查记录称使用压缩的抽象简单根坐标构造 ladder，并经 Weyl 反射传递 | 所读 `build_ladder_bottoms` 对环境 `i32` 坐标逐对相减 |
| 数据生成顺序 | 记录称环境格中的根与余根在之后才构造 | 当前 `from_closure` 先形成环境根、余根数组，再构造 ladder 表 |
| 边界影响 | 上述构造不在 ladder 减法中使用大的环面坐标 | 精确差可能超出 `i32`；修复后恰在溢出分支解释为不属于存储集合（限定接受） |

原版列依据 [HANDOFF](../../../docs/HANDOFF.md) 中对冻结 `7e1b958c7aa9456769cc9cf09ac1542814b4800a` 的审查记录；本次没有重新执行原版或独立重读其完整 C++ 构造流程。Rust 列来自 [root_system.rs](../../../crates/atlas-real-group/src/root_system.rs) 的直接阅读。

## 已有反例

历史 [capture 报告](../../../tests/reference/hpc/math_weyl_context_capture_2026_09_30.json) 的 `generic_root_ladder_coordinate_boundary` case 记录原版接受输入并产生 22 条 `LADDER_ROW` 记录，Rust 仅产生 10 条且被分类为 `REJECTED_RUNTIME`，比较状态为 `RUST_POSITIVE_DIFFERENCE`。该计数不是完整 stdout 的行数。这些是报告记录的历史执行结果，不是本次运行或当前工作区的完整验收。

触发输入保存在 [fixture](../../../tests/math/generics/root_ladder_coordinate_boundary.atlas)，其 [原版 stdout](../../../tests/math/generics/root_ladder_coordinate_boundary.oracle.stdout) 与 [stderr](../../../tests/math/generics/root_ladder_coordinate_boundary.oracle.stderr) 也有保留。查阅精确 case、源 pin、原始流和结果限制时以报告为准。

## 设计含义与后续边界

[成员判定的推导](../algorithms/ladder-bottom-membership.md)说明，不可表示的精确差不可能等于存储集合中的任何元素。该依据支持限定于 ladder 成员查询的修复；不能改写成“所有坐标溢出都可忽略”。

验收决策以 acceptance index entry `0003-a1-torus-root-coroot-ladder-boundary`
及其独立 review 为准；本页只是编辑性对齐。其余阶段与提交门槛仍以
[项目规则](../../../AGENTS.md)和 [交接记录](../../../docs/HANDOFF.md)为准。
KB 不重新安排 HPC gate。

本页的阅读范围由 [来源快照](../../sources/snapshots/2026-10-03-root-ladder-after-v3.json)标识；历史初始快照
`2026-10-01-initial.json` 保留不改写。背景概念见 [根坐标与格坐标](../math/root-coordinates.md)。
