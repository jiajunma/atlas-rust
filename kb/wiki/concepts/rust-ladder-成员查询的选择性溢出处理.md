---
title: Rust ladder 成员查询的选择性溢出处理
summary: build_ladder_bottoms 仅将 ArithmeticOverflow 解释为非成员，独立执行 root 与 coroot 查询，并继续传播分配失败及其他错误。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:11:14.608Z"
updatedAt: "2026-10-10T00:51:08.617Z"
tags:
  - rust
  - 错误处理
  - 根系
aliases:
  - rust-ladder-成员查询的选择性溢出处理
  - RL成
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Rust ladder 成员查询的选择性溢出处理
summary: build_ladder_bottoms 仅将 root/coroot 成员查询中的 ArithmeticOverflow 解释为非成员，独立执行两类查询并传播其他错误，保持底层减法与公共接口不变。
sources:
  - root-ladder-overflow-repair.md
kind: concept
tags:
  - Rust
  - 错误处理
  - 根系
aliases:
  - rust-ladder-成员查询的选择性溢出处理
---

# Rust ladder 成员查询的选择性溢出处理

Rust 构造 root/coroot ladder bottom 表时，需要判断两个已存向量的坐标差是否属于对应集合。选择性溢出处理仅在这两处成员查询中，将 `StructureError::ArithmeticOverflow` 解释为“不属于集合”，避免环境格坐标差超出 `i32` 范围导致整个 `RootSystem` 构造失败；其他错误继续传播。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:74-83]

## 数学依据与适用边界

对完整存储的有限根集 \(R\subseteq\mathbb Z^d\) 和 \(\alpha\in R\)，[[Root ladder bottom 集与固定宽度成员查询|ladder bottom 集]]定义为 \(B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}\)。Rust 将已存根和余根的坐标表示为 `i32`；若精确整数差 \(\beta-\alpha\) 的任一坐标超出 `i32` 可表示范围，该差就不可能等于任何已存向量。因此成员查询必为 `false`，相应的 \(\beta\) 应进入 bottom 集。^[root-ladder-overflow-repair.md:41-49]

这个结论只适用于“精确差是否属于完整的 `i32` 存储集合”的查询。它不允许 wrapping 或 saturating 算术，也不适用于一般向量减法、反射、root combination、seed negation 或构造输入验证。分配失败及未来其他错误仍须传播。^[root-ladder-overflow-repair.md:51-53]

## 实现与错误传播

生产代码增量只修改 `build_ladder_bottoms` 的两次成员查询：`subtract_coordinates` 成功时，保留原来的 root 二分查找或 coroot map 查找；错误恰为 `StructureError::ArithmeticOverflow` 时，成员关系取 `false`；`AllocationFailed` 及任何其他错误继续返回。相关错误类型见 [[StructureError 统一错误分类学]]。^[root-ladder-overflow-repair.md:74-80]

root 与 coroot 查询独立执行，root 溢出不会跳过 coroot 查询。修复不修改 `subtract_coordinates`、reflection、`combine_roots`、数据布局、排序或公共 API，特殊错误解释局限于 ladder 成员查询。^[root-ladder-overflow-repair.md:81-83]

两次查询共享 `difference` 缓冲区。helper 在每次调用开始时清空缓冲区；溢出后留下的部分前缀不会在错误分支中读取，下次调用会再次清空，因此不会拿未完成的差执行成员查找。^[root-ladder-overflow-repair.md:85-86]

## 与 original Atlas 的关系

冻结的 original Atlas commit 为 `7e1b958c7aa9456769cc9cf09ac1542814b4800a`。其 `sources/structure/rootdata.cpp:238-317` 先用抽象简单根坐标 `Byte_vector` 建立 simple-root ladder，再通过 Weyl reflection permutation 将表传送到所有正根和负根。含环面因子的环境格嵌入所产生的大坐标不进入该减法阶段，详见 [[Original Atlas 的抽象坐标 ladder 构造]]。^[root-ladder-overflow-repair.md:57-62]

Rust 采用逐对相减环境格坐标的实现策略，但两者的可观测契约仍是同一个根/余根成员关系。修复要求固定宽度表示保持这一关系，无须将 Rust 整体改写为 C++ 数据结构。^[root-ladder-overflow-repair.md:64-66]

## 验证与证据范围

[[坐标边界修复的 tests-first 验证链]]由 BEFORE-v3 作业 `3873400` 确认：未修复生产代码时，精确出现两条 domain 回归失败和一条 core 完整流回归失败，且 `0 ignored`。这是测试能够暴露原错误的证据，不是修复后的通过结果。^[root-ladder-overflow-repair.md:100-107]

AFTER-v3 作业 `3875239` 以 `COMPLETED 0:0` 结束，[独立检查记录](../../tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json) 状态为 `LADDER_BOUNDARY_AFTER_ACCEPTED`。95 项 harness checker、完整 stager 清点 `70/67`、domain `521/2/521`、core `630/1/630` 以及源码和最终完整性复核全部通过。接受的 `root_system.rs` 字节 SHA-256 为 `cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9`。^[root-ladder-overflow-repair.md:19-28]

边界 fixture 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种编号、阈值两侧及 `i32::MAX`，并保留 recovery marker `719`。原版 oracle 来自历史 capture 作业 `3868832`；AFTER 作业没有重跑原版 oracle。相关出处见 [[A1 加中心环面边界 fixture 与历史 oracle]]。^[root-ladder-overflow-repair.md:92-98, root-ladder-overflow-repair.md:124-130]

来源前段仍将 acceptance-index 登记排除在限定接受范围之外，后段则记载已追加条目 `0003-a1-torus-root-coroot-ladder-boundary`，状态为 `acceptance: accepted`、`status: math_pass`，并绑定 AFTER-v3 报告、独立 review 和 canonical 源文件清单。这两种登记口径在来源中并存，详见 [[Root ladder 修复的限定接受与账本状态歧义]]。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-123]

证据限于 A1 加中心环面坐标边界 fixture、相关 kernel 与解释器回归、两个 crate 的完整 Rust 套件及目录治理 harness。它不证明一般 root system 或 KGB 正确性，不覆盖更高 rank、KLV、unitarity、Hodge、associated cycle 或 AV-ann，也不构成性能或并行验收；完整 Rust 套件属于回归守卫，而非 oracle 证明。^[root-ladder-overflow-repair.md:30-34, root-ladder-overflow-repair.md:124-130]

该修复不是性能优化：过去提前失败的极值输入现在会完成 \(O(|R|^2)\) 表构造，可能增加耗时。性能或内存结论仍须受控测量支持。^[root-ladder-overflow-repair.md:85-88]

## Sources

- [Root ladder 固定宽度坐标溢出修复](../../sources/root-ladder-overflow-repair.md)
