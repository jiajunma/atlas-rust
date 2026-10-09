---
title: Rust ladder 成员查询的选择性溢出处理
summary: build_ladder_bottoms 仅将 root/coroot 成员查询中的 ArithmeticOverflow 解释为 false，独立执行两类查询并传播其他错误，同时保持底层减法、排序、布局和公共 API 不变。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:11:14.608Z"
updatedAt: "2026-10-09T15:11:14.608Z"
tags:
  - Rust
  - 错误处理
  - 成员查询
aliases:
  - rust-ladder-成员查询的选择性溢出处理
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Rust ladder 成员查询的选择性溢出处理

Rust 在构造 root/coroot ladder bottom 表时，需要判断坐标差是否仍属于已存根集或余根集。选择性溢出处理将这一特定成员查询中的 `StructureError::ArithmeticOverflow` 解释为“不属于集合”，避免因环境格坐标差超出 `i32` 范围而使整个 `RootSystem` 构造失败；其他错误仍然传播。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:74-83]

## 数学依据

对于完整存储的有限根集 \(R\subseteq\mathbb Z^d\) 和 \(\alpha\in R\)，ladder bottom 集定义为
\[
B_\alpha=\{\beta\in R\mid \beta-\alpha\notin R\}.
\]
Rust 将已存根和余根的坐标表示为 `i32`。如果精确整数差 \(\beta-\alpha\) 的某个坐标超出 `i32` 可表示范围，它就不可能等于任何已存向量，因此成员查询必为 `false`，相应的 \(\beta\) 应进入 bottom 集。参见 [[Root ladder bottom 集与固定宽度成员查询]]。^[root-ladder-overflow-repair.md:41-49]

这一推导依赖于查询对象是完整的 `i32` 存储集合，只适用于精确差的成员判定。它不允许 wrapping 或 saturating 算术，也不能推广到一般向量减法、反射、root combination、seed negation 或构造输入验证。^[root-ladder-overflow-repair.md:51-53]

## 实现与错误传播

生产代码改动仅涉及 `build_ladder_bottoms` 中的两次成员查询：`subtract_coordinates` 成功时，继续执行原来的 root 二分查找或 coroot map 查找；仅当错误恰为 `StructureError::ArithmeticOverflow` 时，将成员关系判为 `false`；`AllocationFailed` 以及任何其他错误继续返回。相关错误分类见 [[StructureError 统一错误分类学]]。^[root-ladder-overflow-repair.md:74-80]

root 与 coroot 查询独立执行，root 查询发生溢出不会跳过 coroot 查询。修复不改变 `subtract_coordinates`、reflection、`combine_roots`、数据布局、排序或 public API，因此溢出解释被限定在 ladder 成员查询的调用位置。^[root-ladder-overflow-repair.md:81-83]

两次查询共享 `difference` 缓冲区。helper 在每次调用开始时清空缓冲区；溢出后留下的部分前缀不会在错误分支中读取，下一次调用也会再次清空。^[root-ladder-overflow-repair.md:85-86]

## 与 original Atlas 的关系

冻结的 original Atlas 版本 `7e1b958c7aa9456769cc9cf09ac1542814b4800a` 先在抽象简单根坐标 `Byte_vector` 中建立 simple-root ladder，再通过 Weyl reflection permutation 将表传送到全部正根和负根。包含 torus factor 的环境格嵌入所产生的大坐标不参与这段 ladder 减法。参见 [[Original Atlas 的抽象坐标 ladder 构造]]。^[root-ladder-overflow-repair.md:57-62]

Rust 使用环境格坐标逐对相减，构造策略与原版不同，但两者的可观测契约仍是同一根/余根成员关系。修复要求固定宽度表示保持该关系，并不要求将 Rust 整体改写为 C++ 的数据结构。^[root-ladder-overflow-repair.md:64-66]

## 验证与适用范围

[[坐标边界修复的 tests-first 验证链]]先由 BEFORE-v3 作业 3873400 确认未修复代码出现两条 domain 回归失败和一条 core 完整流回归失败；AFTER-v3 作业 3875239 的独立检查随后记录限定接受。接受的 `root_system.rs` 字节 SHA-256 为 `cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9`。^[root-ladder-overflow-repair.md:19-28, root-ladder-overflow-repair.md:100-107]

边界 fixture 包含 11 个 A1 加中心环面 case，覆盖 root/coroot 交换、两种 numbering、阈值两侧和 `i32::MAX`，并保留 recovery marker 719。原版 oracle 来自历史 capture 作业 3868832，不能视为 AFTER 作业重新执行原版的结果。参见 [[A1 加中心环面边界 fixture 与历史 oracle]]。^[root-ladder-overflow-repair.md:92-98, root-ladder-overflow-repair.md:124-130]

来源的状态说明存在登记口径差异：开头将 acceptance-index 登记排除在限定接受范围之外，后文则明确记载 entry `0003-a1-torus-root-coroot-ladder-boundary` 已追加，状态为 `acceptance: accepted`、`status: math_pass`，并绑定报告、独立 review 和源文件清单。解读时应保留这一差异，参见 [[Root ladder 修复的限定接受与账本状态歧义]]。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:30-34, root-ladder-overflow-repair.md:115-123]

证据限于 A1 加中心环面坐标边界 fixture、相关 kernel 与解释器回归及完整 Rust 套件守卫，不证明一般 root system 正确性、更高 rank 或 KLV 等后续数学结果，也不构成性能或并行验收。该改动使过去提前失败的极值输入继续完成 \(O(|R|^2)\) 表构造，可能增加耗时；性能与内存结论仍需受控测量。^[root-ladder-overflow-repair.md:85-88, root-ladder-overflow-repair.md:124-130]

## Sources

- [Root ladder 固定宽度坐标溢出修复](root-ladder-overflow-repair.md)
