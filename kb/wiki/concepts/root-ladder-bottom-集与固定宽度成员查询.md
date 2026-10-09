---
title: Root ladder bottom 集与固定宽度成员查询
summary: 对于完整存储的 i32 根集，精确差 β−α 若超出坐标表示范围则必非成员，因此 β 属于 bottom 集；此结论不适用于一般向量运算。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:10:39.774Z"
updatedAt: "2026-10-09T22:47:54.680Z"
tags:
  - 根系
  - 整数算术
  - 成员查询
aliases:
  - root-ladder-bottom-集与固定宽度成员查询
  - RLB集
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Root ladder bottom 集与固定宽度成员查询
summary: 对完整存储的 i32 根或余根集合，精确坐标差若超出表示范围，必不属于集合，因此对应元素应进入 ladder bottom 集；这一判据仅适用于成员查询。
sources:
  - root-ladder-overflow-repair.md
kind: concept
tags:
  - 根系
  - 成员查询
  - 整数边界
aliases:
  - root-ladder-bottom-集与固定宽度成员查询
  - RLB集
---

# Root ladder bottom 集与固定宽度成员查询

Root ladder bottom 集由根之间的差是否仍属于根集决定。固定宽度整数实现必须保持这一成员关系：当精确坐标差超出已存向量的表示范围时，该差必不属于集合，不能仅因此让整个 `RootSystem` 构造失败。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:41-49]

## 数学定义与判据

对完整存储的有限根集 \(R\subseteq\mathbb Z^d\) 和 \(\alpha\in R\)，ladder bottom 集定义为 \(B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}\)。其中，\(\beta-\alpha\) 是数学整数中的精确差。^[root-ladder-overflow-repair.md:41-49]

Rust 将每个已存根和余根的坐标表示为 `i32`。若精确差的任一坐标超出 `i32` 可表示区间，它就不可能等于任何已存向量。因此，该次成员查询必为 `false`，相应的 \(\beta\) 应进入 bottom 集；同一判据也适用于余根集合。^[root-ladder-overflow-repair.md:46-49]

这一推导仅适用于“精确差是否属于完整的 `i32` 存储集合”。它不允许 wrapping 或 saturating 算术，也不能推广到一般向量减法、反射、root combination、seed negation 或构造输入验证。分配失败及其他错误仍须传播，不能统一解释为“不属于集合”。^[root-ladder-overflow-repair.md:51-53]

## Rust 实现的局部修复

修复仅调整 `build_ladder_bottoms` 中的两次成员查询。`subtract_coordinates` 成功时，保持原有的 root 二分查找或 coroot map 查找；仅当错误恰为 `StructureError::ArithmeticOverflow` 时，将成员关系解释为 `false`。`AllocationFailed` 和其他错误继续返回。root 与 coroot 查询独立执行，root 溢出不会跳过 coroot 查询，详见 [[Rust ladder 成员查询的选择性溢出处理]]。^[root-ladder-overflow-repair.md:74-83]

共享的 `difference` 缓冲区在每次 helper 调用开始时清空。溢出留下的部分坐标前缀不会在错误分支中读取，下次调用会再次清空缓冲区。修复不改变 `subtract_coordinates`、reflection、`combine_roots`、数据布局、排序或 public API。^[root-ladder-overflow-repair.md:78-86]

## 与 original Atlas 的关系

冻结于 commit `7e1b958c7aa9456769cc9cf09ac1542814b4800a` 的 original Atlas，在 `sources/structure/rootdata.cpp:238-317` 中先用抽象简单根坐标 `Byte_vector` 建立 simple-root ladder，再通过 Weyl reflection permutation 将表传送到所有正根和负根。含环面因子的环境格嵌入所产生的大坐标不参与这一阶段的减法，参见 [[Original Atlas 的抽象坐标 ladder 构造]]。^[root-ladder-overflow-repair.md:57-62]

Rust 采用环境格坐标逐对相减的实现策略，但两者的可观测契约都是同一个根／余根成员关系。修复要求固定宽度表示保持该关系，并不要求将 Rust 整体改写为 C++ 数据结构。^[root-ladder-overflow-repair.md:64-66]

## 回归证据与适用范围

边界 fixture `tests/math/generics/root_ladder_coordinate_boundary.atlas` 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种编号、阈值下方、阈值上方及 `i32::MAX`，并保留 recovery marker `719`。原版 oracle 来自历史 capture job `3868832`，详见 [[A1 加中心环面边界 fixture 与历史 oracle]]。^[root-ladder-overflow-repair.md:92-98]

BEFORE-v3 job `3873400` 在未修复生产代码上确认两条 domain 回归和一条 core full-stream 回归按预期失败。AFTER-v3 job `3875239` 以 `COMPLETED 0:0` 结束，其[独立检查记录](../../tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json)标记为 `LADDER_BOUNDARY_AFTER_ACCEPTED`，记录了 harness、两个 crate 测试及源码完整性检查通过。相关过程见 [[坐标边界修复的 tests-first 验证链]]。^[root-ladder-overflow-repair.md:19-28, root-ladder-overflow-repair.md:100-107]

来源保留了账本登记状态的口径差异：前部将 acceptance-index 登记排除在接受范围外，后部则记载 entry `0003-a1-torus-root-coroot-ladder-boundary` 已追加，状态为 `acceptance: accepted`、`status: math_pass`，并绑定 AFTER-v3 报告、独立 review 与 canonical 源文件清单。这一差异应予保留，参见 [[Root ladder 修复的限定接受与账本状态歧义]]。^[root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-123]

证据仅覆盖 A1+中心环面坐标边界 fixture 及相关回归守卫。AFTER 作业没有重跑原版 oracle；完整 Rust 套件不是 oracle 证明，也不能据此推断一般根系正确性、更高 rank 或 KLV、unitarity、Hodge、associated cycle、AV-ann 等结果。^[root-ladder-overflow-repair.md:124-130]

该修复不构成性能优化：过去提前失败的极值输入现在会完成 \(O(|R|^2)\) 表构造，可能使用更多时间。性能或内存结论仍需受控测量。^[root-ladder-overflow-repair.md:85-88]

## Sources

- [Root ladder 固定宽度坐标溢出修复](../../sources/root-ladder-overflow-repair.md)
