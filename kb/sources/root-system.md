---
title: 普通根系的确定性枚举：RootSystem、RootId 与梯子底表
source: atlas-rust/root-system
ingestedAt: 2026-10-05T21:40:00Z
---

# 普通根系的确定性枚举：RootSystem、RootId 与梯子底表

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/root_system.rs`（1151 行）。它是结构性
阅读，不声称根系层的数学验收；上游引用（rootdata.h 行号、HPC capture
3535636、Atlas 3868832 等）仅转录自代码注释，本包未核对上游字节。梯子底表
的坐标溢出修复另有专属来源包（`root-ladder-overflow-repair.md`），本包记录
其当前形态。

## 概览

本文件从 `BasedRootDatum` 生成有限普通根系：`RootSystem` 存储按环境坐标
字典序升序的 `roots`/`coroots`/`simple_coordinates` 三张索引对齐表，外加
预计算的 `positive` 标志、生成序的 `simple_ids` 与每根一张的梯子底表
`min_roots`/`min_coroots`。`RootId` 是 `usize` newtype，一个 ID 同时索引
三张表；crate 外只能经 `from_usize`/`index()` 构造/读回，所有访问器对越界
返回 `Option`/`Result`。枚举是显式的调用方预算操作，预算不存进
`RootSystem`（文档注释明示 "deliberately not stored"）。

## 裸签名清单（完整）

```rust
pub struct RootId(pub(crate) usize)   // Copy/Ord/Hash; from_usize / index
pub struct RootSystemBudget { max_lattice_rank, max_roots,
    max_coordinate_entries, max_reflection_steps }   // const new / complete_for
pub struct RootSet { blocks, len }    // contains/len/is_empty/iter 为 pub；
                                      // with_capacity/insert 私有
pub struct RootSystem { datum, roots, coroots, simple_coordinates, positive,
    simple_ids, min_roots, min_coroots }             // 字段全私有
impl RootSystem {
    pub fn enumerate(&BasedRootDatum, max_roots: usize) -> Result<Self, _>
    pub fn enumerate_with_budget(&BasedRootDatum, &RootSystemBudget) -> _
    pub fn lattice_rank / datum / roots / root / coroot / entries / bracket /
        simple_coordinates / action_permutation / is_positive / id_of /
        min_roots_for / min_coroots_for
    pub(crate) fn positivity / simple_root_ids
}
pub(crate) fn combine_roots(&RootSystem, left, right, subtract: bool)
    -> Result<Option<RootId>, _>
```

## 枚举流程与顺序保证

`enumerate(datum, max_roots)` 是兼容包装：`RootSystemBudget::complete_for`
把条目/步数限额设为饱和最坏值（只有基数限额能在运行中触发），并把
`RootSystemResourceLimit { resource: "roots" }` 映射回历史
`ResourceLimitExceeded { limit }`，其它错误原样透传。

`enumerate_with_budget`：固定顺序的 `check_budget_consistency`（格秩 →
根数 → 坐标条目 → 反射步数，全部严格 `>` 比较）→ `datum.try_clone()` 自有
快照 → 播种：每个简单根插入正负两条记录（简单坐标 ±e_i，负根经
`try_negate` 的 `checked_neg`）→ BFS：`pending.pop_front()` 逐条弹出，对
每个生成器计算 `coefficient = pair(root, coroot_g)`，简单坐标第 g 位
`checked_sub(coefficient)`，以 `reflect_weight`/`reflect_coweight` 生成候选
并入闭包。最终 `roots` 顺序仅由闭包 `BTreeMap` 的键序决定，与 BFS 发现
顺序无关（A2 六根的字典序锚定）。

`Closure::insert` 的顺序：自配对 `<α, α∨> = 2` 检查**先于**重复键检查
（对重复键也执行）；重复键要求余根与简单坐标逐字节一致（否则
`"coroot agreement"` 不变量错误）；基数在 `seen.len() == max_roots` 时拒绝
（`RootSystemResourceLimit { resource: "roots" }`）；重复候选不计入基数。
文档注释称自配对/一致性两道检查是防御性的，已知公开 datum 构造路径不可达；
测试经直接注入私有 `Closure` 覆盖之。

预算公式（`u128` 饱和算术）：`entry_bound = 2(r² + 2rn) + (R_eff+1)(4n+2r)`
（覆盖借入 datum、自有快照、map、待处理队列与一个在途候选）；
`step_bound = R_eff · r`；纯环面（半单秩 0）的 `R_eff = 0`。静态检查与
运行时基数拒绝是两条不同路径（测试分别锚定）。

## 访问器语义

`bracket(root, coroot)` 是 Atlas 的「根在左、余根在右」配对（任一 ID 越界
报 `IndexOutOfRange`，两侧各用自己的表长作 `upper_bound`）；`id_of` 对有序
`roots` 二分查找；`is_positive` 查预计算表（正根 = 简单坐标任一分量 > 0；
因 `roots` 按环境坐标排序，无「按半劈开」捷径）；`simple_root_ids()` 按
生成器顺序供下降查询免逐次二分；`action_permutation` 先验 datum 一致性再
逐根作用并反查（余根运输不复查：`WeylAction` 是简单反射之词，其余权生成器
与闭包所用对偶反射相同——注释声明）；`combine_roots` 逐坐标 checked 加减
后以 `id_of` 判定成员（`Ok(None)` = 非根）。

## 梯子底表（min_roots / min_coroots）

`min_roots[alpha]` 标记所有使 `β - α` 不是根的 `β`（含 `alpha` 自身），
`min_coroots` 在配对余根上定义同一关系（上游 `d_minRoots`/`d_minCoroots`，
rootdata.h:154-157；`min_roots_for`/`min_coroots_for`，rootdata.h:270-273）。
成员判定：根用二分查找；余根无序（跟随根序），故先建一次坐标→下标映射。
**溢出即非成员**：`subtract_coordinates` 的 `checked_sub` 溢出时该成员查询
按 false 处理——所存坐标皆为 `i32`，精确差若出界则不可能等于任何所存根/
余根；分配等其它错误照常传播。这是 3868832 修复的落点，配套边界测试以
`m ∈ {0, 2³⁰-1, 2³⁰, i32::MAX}` × 两种配置固定「不拒绝、两表均为全集合」。

测试锚点：B2/G2 的 oracle 梯子底成员集合（HPC capture 3535636，经上游
正根序换算）；暴力对照（A2/B2/G2 逐 `(α,β)` 与 `combine_roots` 及逐坐标
余根差对照）；越界 `RootId(6)` → `None`；纯环面空表。注释声称「原 Atlas
3868832 接受全部十一个坐标边界用例」，而本文件实际钉住 4 值 × 2 配置共
8 组——数量差异已记录待核（其余用例在 HPC 证据侧）。

## RootSet 与 panic 面

`RootSet` 是稳定根序上的位图（上游 `RootNbrSet` 的对应物）：`insert` 私有
且直接索引块（越界 panic，但仅在 `build_ladder_bottoms` 内以 `beta < count`
调用，不可达）；`contains` 对越界返回 false；`iter` 升序；构造后只读（无
公开插入途径）。非测试代码无 `unwrap`/`panic!`/`expect`；`debug_assert!` 仅
四处（pending 清空、两表长一致、减法等长）。

## 测试锚点索引（25 个测试 + 3 个辅助）

A2 字典序与简单坐标锚点；正根计数与 `simple_root_ids`；包装器的
`ResourceLimitExceeded` 映射；A2/B2 根-余根配对坐标；中心余根坐标在两符号
下保持；oracle 梯子底三组；暴力对照；越界；坐标边界；纯环面空枚举；
每根 `bracket(id,id)==Ok(2)`；`bracket` 越界错误；Weyl 作用运输一致性
（A2 全部 6 个作用）；注入闭包的两种不变量拒绝；四种命名资源拒绝及限额
边界；运行时基数拒绝；`complete_for` 与包装器结果全等（全字段逐表比较）；
秩 33 动态枚举；种子余根 `i32::MIN` 取反溢出。

## 限制与未覆盖面

- 不做数学/正确性验收；上游对应关系与捕获编号仅为注释声称，需对照上游
  源码与捕获数据另行核对。
- `RootSystemBudget` 无字段读取器、无校验；`RootSet` 无公开构造/修改入口；
  `RootSystem` 不暴露 `coroots` 整表切片（仅 `coroot(id)` 与 `entries()`）。
- 梯子底表总在构造时全量预计算（每对 `(α,β)` 两次坐标差），无惰性选项；
  此处仅陈述结构。
- 未见直接触发的错误路径：`DatumMismatch`、`InvalidRootAutomorphism`、
  `AllocationFailed`、包装器映射中「其它 resource 原样透传」分支。

## 来源与限制

精确读取身份见
[`2026-10-06-root-system.json`](snapshots/2026-10-06-root-system.json)：绑定
Git base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe（无工具
档案）以完整文件字节起草，维护者对照源码逐条核对改写。本次知识维护未执行
Atlas、Cargo、测试或 benchmark。
