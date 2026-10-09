---
title: Weyl 词对根与权的作用顺序
summary: word_act_root 与 word_act_weight 均采用最后一个字母先作用的约定，这是理解反射词组合与轨道词转换的关键顺序约束。
sources:
  - atlas-core-center-classifier.md
kind: concept
createdAt: "2026-10-09T14:26:42.739Z"
updatedAt: "2026-10-09T14:26:42.739Z"
tags:
  - Weyl群
  - 根系
  - 权
  - 作用约定
aliases:
  - weyl-词对根与权的作用顺序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 词对根与权的作用顺序

Weyl 词作用于根与权时，统一采用**最后一个字母先作用**的约定：`word_act_root` 对应根上的作用，`word_act_weight` 对应权上的作用。源材料分别将它们对应到上游 `RootSystem::permuted_root`（`rootdata.h:313–318`）与 `weyl.cpp:1071–1082`。^[atlas-core-center-classifier.md:51-53]

## 作用顺序

若 Weyl 词写作 \(w=s_1s_2\cdots s_k\)，则对根或权 \(v\)，其作用顺序为
\(w(v)=s_1(s_2(\cdots s_k(v)\cdots))\)：先应用最右侧的 \(s_k\)，最后应用最左侧的 \(s_1\)。例如，词 \(s_1s_2\) 先施加 \(s_2\)，再施加 \(s_1\)。^[atlas-core-center-classifier.md:51-53]

## 反射词与轨道词的构造

`simple_reflect_root_nbr` 实现单根反射在 RootNbr 上的作用。`reflection_word` 沿首个下降将根降到单根，再逆序回溯构造共轭词；这一构造可结合[[反射词的首个下降构造]]理解。^[atlas-core-center-classifier.md:48-50]

`convert_to_words` 经陪集树展开恒等，每一步都把反射词**左乘**到父段条目上。它与反射子群的见证序配对，采用一致的词对权作用约定；相关轨道构造见[[Levi 子商轨道与顶点轨道扩展]]。^[atlas-core-center-classifier.md:38-44]

## 证据边界

本页依据实现的结构性阅读，未声称数学验收。上游函数与行号的对应属于实现方的移植陈述，兼容性仍以 HPC 差分门为准；相关验收要求见[[HPC 验收证据链]]。^[atlas-core-center-classifier.md:9-14, atlas-core-center-classifier.md:55-60]

## Sources

- [atlas-core-center-classifier.md](../../sources/atlas-core-center-classifier.md)
