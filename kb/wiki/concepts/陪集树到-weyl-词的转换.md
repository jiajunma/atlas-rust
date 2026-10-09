---
title: 陪集树到 Weyl 词的转换
summary: convert_to_words 沿陪集树展开，每一步将反射词左乘到父段条目上，并与子群见证序及词对权的作用约定配对。
sources:
  - atlas-core-center-classifier.md
kind: concept
createdAt: "2026-10-09T14:26:35.066Z"
updatedAt: "2026-10-09T14:26:35.066Z"
tags:
  - Weyl群
  - 陪集树
  - 词转换
aliases:
  - 陪集树到-weyl-词的转换
  - 陪W词
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 陪集树到 Weyl 词的转换

陪集树到 Weyl 词的转换由 `convert_to_words` 实现，属于 adjoint 轨道机器。该过程通过陪集树从恒等元展开，并在每一步将反射词**左乘**到父段条目上；来源将其对应到上游 `alcoves.cpp:746–774`。^[atlas-core-center-classifier.md:31-44]

## 转换规则与作用顺序

转换的关键是保留左乘顺序：若父段条目对应 Weyl 词 \(w\)，当前步骤的反射词为 \(r\)，则新条目对应 \(rw\)。这一顺序与反射子群见证词的权作用约定配对，相关概念见 [[反射子群轨道与见证的独立验证]]。^[atlas-core-center-classifier.md:42-44]

Weyl 词作用于根或权时，均为**最后一个字母先作用**：`word_act_root` 与 `word_act_weight` 使用同一约定。因此，转换中的 \(rw\) 先施加父词 \(w\)，再施加当前反射词 \(r\)。理解或使用转换结果时，应同时保留词的左乘构造顺序与实际作用顺序，参见 [[Weyl 词对根与权的作用顺序]]。^[atlas-core-center-classifier.md:42-53]

## 轨道与反射词上下文

adjoint 轨道机器使用 `AdjOrbitElem` 表示 adjoint 坐标下的轨道元素。共享的 `adjoint_orbit_bfs` 在每层生成过程中保持新增尾部递减，层完成后反转为递增；`basic_orbit_adjoint` 构造 Cartan 前 \(i+1\) 个生成元的 Levi 子商轨道，`vertex_orbit` 则用于沿标签大于 1 的最终扩展。相关背景见 [[adjoint 轨道的分层有序 BFS]] 与 [[Levi 子商轨道与顶点轨道扩展]]。^[atlas-core-center-classifier.md:31-41]

反射词由 `reflection_word` 沿首个下降将根降到单根，再逆序回溯构造共轭词；`simple_reflect_root_nbr` 提供单根反射在 `RootNbr` 上的作用。这些操作的相关说明见 [[反射词的首个下降构造]]。^[atlas-core-center-classifier.md:46-50]

## 证据边界

本说明依据结构性阅读，不能视为数学验收结果。来源中的上游位置属于实现方的移植陈述；兼容性仍以 HPC 差分门为准，参见 [[HPC 验收证据链]]。^[atlas-core-center-classifier.md:9-14, atlas-core-center-classifier.md:55-60]

## Sources

- [atlas-core-center-classifier.md](atlas-core-center-classifier.md)
