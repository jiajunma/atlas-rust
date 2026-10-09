---
title: Levi 子商轨道与顶点轨道扩展
summary: basic_orbit_adjoint 构造 Cartan 前 i+1 个生成元的 Levi 子商轨道，vertex_orbit 则提供用于沿 label>1 最终扩展的模变体。
sources:
  - atlas-core-center-classifier.md
kind: concept
createdAt: "2026-10-09T14:26:33.023Z"
updatedAt: "2026-10-09T14:26:33.023Z"
tags:
  - Levi子商
  - 轨道算法
  - adjoint坐标
aliases:
  - levi-子商轨道与顶点轨道扩展
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Levi 子商轨道与顶点轨道扩展

Levi 子商轨道与顶点轨道扩展由 adjoint 坐标轨道机器中的 `basic_orbit_adjoint` 和 `vertex_orbit` 实现。两者共享 `adjoint_orbit_bfs` 广度优先搜索核，分别承担 Levi 子商轨道构造和沿 `label > 1` 的最终扩展。^[atlas-core-center-classifier.md:31-41]

## Levi 子商轨道

`basic_orbit_adjoint` 对应上游 `alcoves.cpp:526–565` 的 `basic_orbit`，使用 `cartan` 的前 `i+1` 个生成元构造 Levi 子商轨道，轨道坐标采用 adjoint 坐标。轨道元素由 `AdjOrbitElem` 表示，对应上游的 `orbit_elem`。^[atlas-core-center-classifier.md:33-39]

## 顶点轨道扩展

`vertex_orbit` 对应上游 `alcoves.cpp:458–518`，是轨道构造的模变体，用于沿 `label > 1` 的最终扩展。它与 `basic_orbit_adjoint` 复用同一搜索核；标签相关背景可参见 [[Alcove 墙标签与标签排序]]。^[atlas-core-center-classifier.md:35-41, atlas-core-center-classifier.md:57-58]

## 共享 BFS 的层内顺序

`adjoint_orbit_bfs` 在 `finish` 之后插入新元素，使尾部保持递减；每完成一层，再将该层反转为递增。因此，共享搜索核同时规定分层遍历和层内排列方式，详见 [[adjoint 轨道的分层有序 BFS]]。^[atlas-core-center-classifier.md:35-37]

## 轨道到 Weyl 词的转换

`convert_to_words` 经陪集树展开恒等元素，每一步都把反射词**左乘**到父段条目上。这一转换与反射子群见证序的词权作用约定配对，相关作用方向可参见 [[ambient Weyl 见证词与作用方向]]。^[atlas-core-center-classifier.md:42-44]

用于构造反射词的 `reflection_word` 沿首个下降降到单根，再逆序回溯得到共轭词。得到的 Weyl 词作用于根或权时，均由**最后一个字母先作用**；可参见 [[反射词的首个下降构造]] 与 [[Weyl 词对根与权的作用顺序]]。^[atlas-core-center-classifier.md:48-53]

## 证据边界

本说明依据对 Rust 实现的结构性阅读，不构成数学验收。上游函数与行号的对应属于实现方的移植陈述，兼容性仍以 [[HPC 验收证据链]] 中的差分门为准。^[atlas-core-center-classifier.md:9-14, atlas-core-center-classifier.md:59-60]

## Sources

- [atlas-core-center-classifier.md](../../sources/atlas-core-center-classifier.md)
