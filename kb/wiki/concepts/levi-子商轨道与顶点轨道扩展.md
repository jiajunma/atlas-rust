---
title: Levi 子商轨道与顶点轨道扩展
summary: basic_orbit_adjoint 构造 Cartan 前 i+1 个生成元的 Levi 子商轨道，vertex_orbit 提供用于 label>1 最终扩展的模变体。
sources:
  - atlas-core-center-classifier.md
kind: concept
createdAt: "2026-10-09T14:26:33.023Z"
updatedAt: "2026-10-09T22:12:45.405Z"
tags:
  - Levi子群
  - 轨道枚举
  - Alcove
aliases:
  - levi-子商轨道与顶点轨道扩展
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Levi 子商轨道与顶点轨道扩展
summary: basic_orbit_adjoint 在 adjoint 坐标下构造 Cartan 前 i+1 个生成元的 Levi 子商轨道；vertex_orbit 提供沿 label>1 最终扩展的模变体，两者共享分层有序 BFS。
sources:
  - atlas-core-center-classifier.md
kind: concept
tags:
  - Levi子商
  - 轨道算法
  - adjoint坐标
aliases:
  - levi-子商轨道与顶点轨道扩展
---

# Levi 子商轨道与顶点轨道扩展

Levi 子商轨道与顶点轨道扩展是 adjoint 坐标轨道机器中的两类构造，分别由 `basic_orbit_adjoint` 和 `vertex_orbit` 实现。它们共享 `adjoint_orbit_bfs` 广度优先搜索核心，轨道元素使用 `AdjOrbitElem` 表示。^[atlas-core-center-classifier.md:31-41]

## Levi 子商轨道

`basic_orbit_adjoint` 使用 `cartan` 的前 `i+1` 个生成元，构造 adjoint 坐标下的 Levi 子商轨道。源材料将其对应到上游 `alcoves.cpp:526–565` 的 `basic_orbit`；`AdjOrbitElem` 对应同文件第 437–445 行的 `orbit_elem`。^[atlas-core-center-classifier.md:33-39]

## 顶点轨道扩展

`vertex_orbit` 是轨道构造的模变体，用于沿 `label > 1` 的最终扩展，对应上游 `alcoves.cpp:458–518`。标签相关背景可参见 [[Alcove 墙标签与标签排序]]。^[atlas-core-center-classifier.md:40-41, atlas-core-center-classifier.md:57-58]

## 共享 BFS 的排序纪律

`adjoint_orbit_bfs` 将新元素插入 `finish` 之后，使尾部保持**递减**；每完成一层，再将该层反转为**递增**。生成过程中的尾部顺序与层完成后的顺序因此不同。这一排序纪律由两类轨道构造共享，详见 [[adjoint 轨道的分层有序 BFS]]。^[atlas-core-center-classifier.md:35-41]

## 轨道到 Weyl 词的转换

同一轨道机器中的 `convert_to_words` 经陪集树展开恒等，每一步都将反射词**左乘**到父段条目上。该方向与反射子群见证序的词权作用约定一致，相关背景见 [[ambient Weyl 见证词与作用方向]]。^[atlas-core-center-classifier.md:42-44]

`reflection_word` 沿首个下降降到单根，再逆序回溯得到共轭词。`word_act_root` 与 `word_act_weight` 均采用**最后一个字母先作用**的约定；相关细节见 [[反射词的首个下降构造]] 与 [[Weyl 词对根与权的作用顺序]]。^[atlas-core-center-classifier.md:48-53]

## 证据边界

本页依据 `crates/atlas-core/src/domain_builtins.rs` 中相关实现的结构性阅读，不构成数学验收。上游函数与行号的对应属于实现方的移植陈述，兼容性仍以 HPC 差分门为准，参见 [[HPC 验收证据链]]。^[atlas-core-center-classifier.md:9-14, atlas-core-center-classifier.md:59-60]

## Sources

- [atlas-core-center-classifier.md](../../sources/atlas-core-center-classifier.md) — 中心分类器与轨道词（domain_builtins.rs 6102–7992）。
