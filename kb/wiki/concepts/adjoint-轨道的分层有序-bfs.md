---
title: adjoint 轨道的分层有序 BFS
summary: adjoint_orbit_bfs 是共享轨道遍历核心，通过将新元素插入 finish 之后保持尾部递减，并在每层完成后反转为递增来维护遍历顺序。
sources:
  - atlas-core-center-classifier.md
kind: concept
createdAt: "2026-10-09T14:26:25.988Z"
updatedAt: "2026-10-09T14:26:25.988Z"
tags:
  - 轨道算法
  - 广度优先搜索
  - 排序不变量
aliases:
  - adjoint-轨道的分层有序-bfs
  - A轨B
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# adjoint 轨道的分层有序 BFS

`adjoint_orbit_bfs` 是 adjoint 坐标轨道构造的共享广度优先搜索（BFS）核心，供 `basic_orbit` 与 `vertex_orbit` 使用。其关键排序纪律是在生成一层时保持尾部递减，再在该层完成后反转为递增。轨道元素由 `AdjOrbitElem` 表示，对应上游 `alcoves.cpp` 中的 `orbit_elem`。^[atlas-core-center-classifier.md:31-37]

## 分层排序纪律

搜索把新元素插入 `finish` 之后，并保持这一尾部区间递减；每完成一层，便将该层反转为递增。因此，层内生成期间的排列方向与该层完成后的排列方向不同，不能把这一过程概括为始终按递增顺序插入。源材料将这套共享 BFS 对应到 `alcoves.cpp:480–565`。^[atlas-core-center-classifier.md:35-37]

## 两类轨道构造

`basic_orbit_adjoint` 使用 `cartan` 的前 `i+1` 个生成元，构造 adjoint 坐标下的 Levi 子商轨道；`vertex_orbit` 则是模变体，用于沿 `label > 1` 的最终扩展。二者共享上述分层排序核心，相关构造见 [[Levi 子商轨道与顶点轨道扩展]]。^[atlas-core-center-classifier.md:35-41]

## 轨道到 Weyl 词的衔接

相关轨道机器中的 `convert_to_words` 经陪集树展开恒等，每一步都将反射词左乘到父段条目上。这一方向与 [[Weyl 词对根与权的作用顺序]] 相配：`word_act_root` 与 `word_act_weight` 均约定最后一个字母先作用。^[atlas-core-center-classifier.md:42-53]

## 证据边界

本说明依据 `domain_builtins.rs` 中 adjoint 轨道机器的结构性阅读，不构成数学验收。源材料中的上游位置与对应关系属于实现方的移植陈述；兼容性仍须以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-center-classifier.md:9-14, atlas-core-center-classifier.md:55-60]

## Sources

- [atlas-core-center-classifier.md](../../sources/atlas-core-center-classifier.md)
