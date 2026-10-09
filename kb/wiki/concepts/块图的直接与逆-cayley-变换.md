---
title: 块图的直接与逆 Cayley 变换
summary: 直接与逆 Cayley 访问器共享两个像槽并按 weak descent 互补开放；i1 构建单值直接像，i2 构建双值直接像，同时回填逆像并检查槽容量。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:34.895Z"
updatedAt: "2026-10-09T14:41:34.895Z"
tags:
  - 表示论
  - Cayley变换
  - 访问器语义
aliases:
  - 块图的直接与逆-cayley-变换
  - 块C变
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 块图的直接与逆 Cayley 变换

完整块图 `BlockGraph` 的直接与逆 Cayley 变换共享同一对存储槽 `cayley_first`、`cayley_second`，分别对应上游 `Cayley_image.first`、`.second`。访问器依据生成元处的 weak descent 状态决定是否返回槽中内容；因此，存储的像对与某一方向上可访问的像对需要区分。^[block-graph.md:56-60, block-graph.md:80-87]

## 状态与访问语义

[[BlockDescent 八值状态体系]]以 `index & 0x4 != 0` 判定 weak descent。八种状态的前四种为 ascent，后四种为 weak descent；Cayley 访问器使用这一划分控制直接与逆变换的方向。^[block-graph.md:35-44, block-graph.md:84-87]

`cayley(z, s)` 返回直接 Cayley 像对，但在 weak descent 处强制返回 `Some((None, None))`。`inverse_cayley(z, s)` 与之互补：仅在 weak descent 处返回存储槽原值，在非 descent 处返回 `Some((None, None))`。像对内部的 `None` 对应上游未定义标记 `UndefBlock`；非 descent 状态本身并不保证存在直接像。^[block-graph.md:84-87]

## 构造与逆像回填

构建时，虚根 type I 状态 `i1` 产生单值直接 Cayley 像，并通过 `first_free_slot` 将源元素回填到目标元素的逆 Cayley 槽。虚根 type II 状态 `i2` 产生双值直接像：先通过 `dual_second` 处理一个像，再继续进入 `i1` 分支处理 `dual_first`。回填时若槽已满，则报告 `"Cayley pair slots"` 不变量错误。^[block-graph.md:66-76]

这套构造使直接像与逆像共用两槽表示，同时由访问器区分方向。块级状态中，虚非紧根的 cross 若移动元素，则为 `i1`，否则为 `i2`；实根一侧的 `r1`、`r2` 判定则依赖对偶侧虚非紧根的 cross 行为。^[block-graph.md:49-52, block-graph.md:56-60, block-graph.md:72-87]

## 对偶变换中的像对

[[完整块图的对偶数据变换]]将元素编号反转为
\[
z'=\mathrm{size}-1-z,
\]
并将 Cayley 链的目标编号 \(c\) 映为 \(\mathrm{size}-1-c\)。第二像仅在第一像有定义时映射。对偶状态配对为 `ImaginaryTypeI ↔ RealTypeII`、`ImaginaryTypeII ↔ RealTypeI`，与直接、逆访问方向的互换相配合。^[block-graph.md:43-44, block-graph.md:95-101]

双值 Cayley 像对在对偶变换中不重新排序。因此，将 `dual()` 的结果与原生构建的对偶块比较时，应把双值像视为无序集合进行比较。^[block-graph.md:100-101]

## 在 Bruhat 图中的作用

[[块的 Bruhat Hasse 图与可比对计数]]沿首个严格 good descent（complex 或 type-I real）递归构造直接下邻。在 split principal series 处，前驱恰好是各 type-II 实 descent 的逆 Cayley 变换。^[block-graph.md:106-110]

## 测试与证据边界

源码中的秩一 A1 测试提供了两种互补情形：`block(SL(2,R), PGL(2,R))` 的两个 `i1` 元素共享同一个 `r1` 目标，每个直接像单值，而目标的逆像双值；`block(PGL(2,R), SL(2,R))` 则覆盖 `i2` 的双值直接 Cayley 与 `r2` 的单值逆 Cayley。前一种情形还对齐了冻结 fixture capture 3501519。^[block-graph.md:117-120]

现有测试还逐点检查 `dual()` 的链接映射，并通过坐标重新定位比较其与原生对偶块的一致性。但全部七个测试均为秩一，未覆盖多生成元、全部块不变量错误分支以及空对偶包或空块。来源材料仅报告结构性阅读，未执行构建、测试或原版运行，不能据此扩大数学验收范围。^[block-graph.md:117-126, block-graph.md:139-139]

## Sources

- [block-graph.md](block-graph.md) — 完整块图：实形式与对偶实形式的纤维积。
