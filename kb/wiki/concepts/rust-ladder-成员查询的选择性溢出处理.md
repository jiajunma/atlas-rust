---
title: Rust ladder 成员查询的选择性溢出处理
summary: build_ladder_bottoms 仅将成员查询中的 ArithmeticOverflow 解释为 false，独立执行 root 与 coroot 查询并传播其他错误，保持底层减法和公共 API 不变。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:11:14.608Z"
updatedAt: "2026-10-09T21:09:27.652Z"
tags:
  - Rust
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Rust ladder 成员查询的选择性溢出处理
summary: build_ladder_bottoms 仅将 root/coroot 成员查询中的 ArithmeticOverflow 解释为 false，独立执行两类查询并传播其他错误，保持底层减法、排序、布局和公共 API 不变。
sources:
  - root-ladder-overflow-repair.md
kind: concept
tags:
  - Rust
  - 错误处理
  - 成员查询
aliases:
  - rust-ladder-成员查询的选择性溢出处理
---

# Rust ladder 成员查询的选择性溢出处理

Rust 在构造 root/coroot ladder bottom 表时，需要判断坐标差是否属于已存根集或余根集。选择性溢出处理仅将这一成员查询中的 `StructureError::ArithmeticOverflow` 解释为“不属于集合”，避免环境格坐标差超出 `i32` 范围导致整个 `RootSystem` 构造失败；其他错误仍然传播。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:74-83]

## 数学依据

对于完整存储的有限根集 \(R\subseteq\mathbb Z^d\) 和 \(\alpha\in R\)，[[Root ladder bottom 集与固定宽度成员查询|ladder bottom 集]]定义为 \(B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}\)。Rust 将已存根和余根的坐标表示为 `i32`；若精确整数差 \(\beta-\alpha\) 的某个坐标超出 `i32` 可表示区间，它就不可能等于任何已存向量。因此，该成员查询必为 `false`，相应的 \(\beta\) 应进入 bottom 集。^[root-ladder-overflow-repair.md:41-49]

这一推导只适用于“精确差是否属于完整的 `i32` 存储集合”的判定。它不允许 wrapping 或 saturating 算术，也不能推广到一般向量减法、反射、root combination、seed negation 或构造输入验证；分配失败及其他错误仍须传播。^[root-ladder-overflow-repair.md:51-53]

## 实现与错误传播

生产代码改动仅涉及 `build_ladder_bottoms` 中的两次成员查询。`subtract_coordinates` 成功时，继续执行原来的 root 二分查找或 coroot map 查找；错误恰为 `StructureError::ArithmeticOverflow` 时，将成员关系判为 `false`；`AllocationFailed` 及任何其他错误继续返回。相关错误类型见 [[StructureError 统一错误分类学]]。^[root-ladder-overflow-repair.md:74-80]

root 与 coroot 查询独立执行：root 查询溢出不会跳过 coroot 查询。修复不改变 `subtract_coordinates`、reflection、`combine_roots`、数据布局、排序或 public API，溢出的特殊解释仅发生在 ladder 成员查询处。^[root-ladder-overflow-repair.md:81-83]

两次查询共享 `difference` 缓冲区。helper 在每次调用开始时清空缓冲区；溢出后留下的部分前缀不会在错误分支中读取，下一次调用会再次清空，因此不会将未完成的差用于查找。^[root-ladder-overflow-repair.md:85-86]

## 与 original Atlas 的关系

冻结的 original Atlas commit `7e1b958c7aa9456769cc9cf09ac1542814b4800a` 在 `sources/structure/rootdata.cpp:238-317` 中，先使用抽象简单根坐标 `Byte_vector` 建立 simple-root ladder，再通过 Weyl reflection permutation 将表传送到所有正根和负根。含中心环面的环境格嵌入所产生的大坐标不进入这一减法阶段。参见 [[Original Atlas 的抽象坐标 ladder 构造]]。^[root-ladder-overflow-repair.md:57-62]

Rust 采用环境格坐标逐对相减，虽然构造策略不同，可观测契约仍是同一根/余根成员关系。修复要求固定宽度表示保持这一关系，并不要求将 Rust 整体改写成 C++ 数据结构。^[root-ladder-overflow-repair.md:64-66]

## 验证与证据边界

[[坐标边界修复的 tests-first 验证链]]由 BEFORE-v3 作业 `3873400` 确认：未修复生产代码精确出现两条 domain 回归失败和一条 core 完整流回归失败。AFTER-v3 作业 `3875239` 随后以 `COMPLETED 0:0` 结束，独立检查记录为 `LADDER_BOUNDARY_AFTER_ACCEPTED`，涵盖 95 项 harness checker、两个 crate 的完整测试及源码完整性复核。接受的 `root_system.rs` 字节 SHA-256 为 `cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9`。^[root-ladder-overflow-repair.md:19-28, root-ladder-overflow-repair.md:100-107]

边界 fixture 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种编号、阈值两侧及 `i32::MAX`，并保留 recovery marker `719`。原版 oracle 来自历史 capture 作业 `3868832`；AFTER 作业没有重新运行原版 oracle。参见 [[A1 加中心环面边界 fixture 与历史 oracle]]。^[root-ladder-overflow-repair.md:92-98, root-ladder-overflow-repair.md:124-130]

来源保留了不同的登记口径：前段将 acceptance-index 登记排除在限定接受范围之外，后段则记载 entry `0003-a1-torus-root-coroot-ladder-boundary` 已追加，状态为 `acceptance: accepted`、`status: math_pass`，并绑定 AFTER-v3 报告、独立 review 和 canonical 源文件清单。这一差异应保留，详见 [[Root ladder 修复的限定接受与账本状态歧义]]。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-123]

证据限定于 A1 加中心环面坐标边界 fixture、相关 kernel 与解释器回归及完整 Rust 套件守卫。它不证明一般 root system 正确性，不覆盖更高 rank、KGB、KLV、unitarity、Hodge、associated cycle 或 AV-ann，也不构成性能或并行验收；完整 Rust 套件属于回归守卫，而非 oracle 证明。^[root-ladder-overflow-repair.md:30-34, root-ladder-overflow-repair.md:124-130]

该修复不是性能优化：过去提前失败的极值输入现在会继续完成 \(O(|R|^2)\) 表构造，可能增加耗时。性能或内存结论仍须由受控测量支持。^[root-ladder-overflow-repair.md:85-88]

## Sources

- [Root ladder 固定宽度坐标溢出修复](../../sources/root-ladder-overflow-repair.md)
