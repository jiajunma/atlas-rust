---
title: adjoint 轨道的分层有序 BFS
summary: adjoint_orbit_bfs 将新元素插入 finish 之后保持尾部递减，并在每层完成后反转为递增，作为两种轨道构造的共享核心。
sources:
  - atlas-core-center-classifier.md
kind: concept
createdAt: "2026-10-09T14:26:25.988Z"
updatedAt: "2026-10-09T22:12:40.955Z"
tags:
  - 轨道枚举
  - 广度优先搜索
  - 排序
aliases:
  - adjoint-轨道的分层有序-bfs
  - A轨B
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: adjoint 轨道的分层有序 BFS
summary: adjoint_orbit_bfs 是共享轨道遍历核心，在 finish 之后保持新元素尾部递减，并在每层完成后反转为递增。
sources:
  - atlas-core-center-classifier.md
kind: concept
tags:
  - 轨道算法
  - 广度优先搜索
  - 排序不变量
aliases:
  - adjoint-轨道的分层有序-bfs
provenanceState: extracted
---

# adjoint 轨道的分层有序 BFS

`adjoint_orbit_bfs` 是 adjoint 坐标轨道构造的共享广度优先搜索（BFS）核心，供 `basic_orbit` 与 `vertex_orbit` 使用。轨道元素由 `AdjOrbitElem` 表示，对应上游 `alcoves.cpp:437–445` 的 `orbit_elem`。^[atlas-core-center-classifier.md:31-37]

## 分层排序不变量

搜索将新元素插入 `finish` 之后，使尾部保持**递减**；每完成一层，再将该层反转为**递增**。因此，生成期间的尾部顺序与层完成后的顺序不同。源材料将这一共享核心对应到上游 `alcoves.cpp:480–565`。^[atlas-core-center-classifier.md:35-37]

## 两类轨道构造

`basic_orbit_adjoint` 对应上游 `basic_orbit`，使用 `cartan` 的前 `i+1` 个生成元构造 adjoint 坐标下的 Levi 子商轨道。`vertex_orbit` 是模变体，用于沿 `label > 1` 的最终扩展。两者共用上述分层 BFS，相关说明见 [[Levi 子商轨道与顶点轨道扩展]]。^[atlas-core-center-classifier.md:35-41]

## 与 Weyl 词构造的衔接

同一轨道机器中的 `convert_to_words` 经陪集树展开恒等，每一步都将反射词**左乘**到父段条目上。词对根和权的作用均采用“最后一个字母先作用”的约定，参见 [[Weyl 词对根与权的作用顺序]]。^[atlas-core-center-classifier.md:42-53]

`reflection_word` 沿首个下降降到单根，再逆序回溯得到共轭词；其构造细节见 [[反射词的首个下降构造]]。^[atlas-core-center-classifier.md:48-50]

## 证据边界

本页依据 `crates/atlas-core/src/domain_builtins.rs` 中 adjoint 轨道机器的结构性阅读，不构成数学验收。源材料中的上游行号与对应关系属于实现方的移植陈述；兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-center-classifier.md:9-14, atlas-core-center-classifier.md:55-60]

## Sources

- [atlas-core-center-classifier.md](../../sources/atlas-core-center-classifier.md) — 中心分类器与轨道词（domain_builtins.rs 6102–7992）。
