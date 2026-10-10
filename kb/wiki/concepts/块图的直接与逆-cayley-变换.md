---
title: 块图的直接与逆 Cayley 变换
summary: 直接与逆 Cayley 访问器按 weak descent 互补读取共享像槽，i1 构建单值直接像，i2 构建双值直接像，并回填逆像及检查槽容量。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:34.895Z"
updatedAt: "2026-10-10T00:27:48.441Z"
tags:
  - 块图
  - Cayley变换
aliases:
  - 块图的直接与逆-cayley-变换
  - 块C变
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 块图的直接与逆 Cayley 变换
summary: 直接与逆 Cayley 访问器按 weak descent 状态互补读取共享像槽；i1 构建单值直接像，i2 构建双值直接像，并回填逆像、检查槽容量。
sources:
  - block-graph.md
kind: concept
tags:
  - 块图
  - Cayley变换
aliases:
  - 块图的直接与逆-cayley-变换
  - 块C变
---

# 块图的直接与逆 Cayley 变换

完整块图 `BlockGraph` 的直接与逆 Cayley 变换共享两个像槽 `cayley_first`、`cayley_second`，对应上游 `Cayley_image.first`、`.second`。两个访问器依据 weak descent（弱下降）状态互补读取这些槽：直接变换用于 ascent，逆变换用于 weak descent。^[block-graph.md:56-60, block-graph.md:84-87]

## 状态与访问语义

[[BlockDescent 八值状态体系]]的前四种状态是 ascent，后四种是 weak descent；`is_descent` 以 `index & 0x4 != 0` 判定。`cayley(z, s)` 在 weak descent 处强制返回 `Some((None, None))`；`inverse_cayley(z, s)` 在 weak descent 处返回槽原值，在非 descent 处返回 `Some((None, None))`。像对内部的 `None` 对应上游未定义标记 `UndefBlock`。^[block-graph.md:37-44, block-graph.md:84-87]

块级状态涉及两侧 KGB 图。虚非紧根的 cross 若移动元素，则状态为 `i1`，否则为 `i2`；实根或虚紧根则查看对偶侧：对偶虚非紧且 cross 移动时为 `r2`，不移动时为 `r1`，否则按原侧根类型取 `rn` 或 `ic`。^[block-graph.md:49-52]

## 直接像构造与逆像回填

构建时，`i1` 产生单值直接 Cayley 像，并通过 `first_free_slot` 回填逆 Cayley 槽。`i2` 产生双值直接像：先经 `dual_second` 处理一个像，再贯穿进入 `i1` 分支处理 `dual_first`，保留上游的分支贯穿行为。逆像槽已满时，报告 `"Cayley pair slots"` 错误。^[block-graph.md:72-75]

秩一测试展示了两种像数关系。`block(SL(2,R), PGL(2,R))` 中，两个 `i1` 元素共享同一个 `r1` 目标，各自的直接像为单值，目标的逆像为双值；该用例对齐冻结 fixture capture `3501519`。`block(PGL(2,R), SL(2,R))` 则覆盖 `i2` 的双值直接 Cayley 与 `r2` 的单值逆 Cayley。^[block-graph.md:117-120]

## 对偶变换中的像对

[[完整块图的对偶数据变换]]将元素编号反转为 $z'=\mathrm{size}-1-z$，并将 Cayley 链目标 $c$ 映为 $\mathrm{size}-1-c$。第二像仅在第一像有定义时映射。相关状态的对偶配对为 `ImaginaryTypeI ↔ RealTypeII`、`ImaginaryTypeII ↔ RealTypeI`。^[block-graph.md:43-44, block-graph.md:95-101]

双值 Cayley 像对在对偶变换中不重新排序，因此将 `dual()` 的结果与原生构建的对偶块比较时，双值像按无序集合比较。^[block-graph.md:100-101]

## 在 Bruhat 图中的作用

[[块的 Bruhat Hasse 图与可比对计数]]沿首个严格 good descent，即 complex 或 type-I real descent，递归构造元素的直接下邻。在 split principal series 处，前驱恰为各 type-II 实 descent 的逆 Cayley 变换。实现委托 `crate::block_access`。^[block-graph.md:106-110]

## 测试与证据边界

来源记录的七个测试全部为秩一 A1。除上述直接与逆 Cayley 情形外，测试还逐点核对 `dual()` 的反转、坐标互换与链接映射，并通过 `element` 换编号检查其与原生对偶块的一致性；覆盖全 KGB 范围时，还检查 `dual().dual()` 为恒等，包括 `first_z_of_x` 表。^[block-graph.md:117-124]

覆盖缺口包括多生成元情形、C±/rn/ic 块级状态、全部 20 个 `BlockInvariantViolation` 字面量分支、空对偶包或空块、`element` 失败分支，以及 `n_bruhat_comparable`。详见 [[完整块图的测试覆盖与证据边界]]。^[block-graph.md:124-126]

来源属于对 `block.rs` 的结构性阅读，未执行构建、测试或原版运行。块枚举正确性另有 [[HPC 验收证据链]]；本页不扩展该链，也不构成新增的数学验收、性能或并行结论。^[block-graph.md:9-13, block-graph.md:139-139]

## Sources

- [block-graph.md](../../sources/block-graph.md) — 完整块图：实形式与对偶实形式的纤维积。
