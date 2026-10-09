---
title: Weyl 词对根与权的作用顺序
summary: word_act_root 与 word_act_weight 均采用最后一个字母先作用的约定；来源仅支持结构性说明，兼容性仍须 HPC 差分验证。
sources:
  - atlas-core-center-classifier.md
kind: concept
createdAt: "2026-10-09T14:26:42.739Z"
updatedAt: "2026-10-09T20:30:26.481Z"
tags:
  - Weyl群
  - 作用顺序
  - 兼容性
aliases:
  - weyl-词对根与权的作用顺序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Weyl 词对根与权的作用顺序
summary: word_act_root 与 word_act_weight 均采用最后一个字母先作用的约定，轨道词转换则将每步反射词左乘到父段条目上。
sources:
  - atlas-core-center-classifier.md
kind: concept
tags:
  - Weyl群
  - 根系
  - 权
  - 作用约定
aliases:
  - weyl-词对根与权的作用顺序
---

# Weyl 词对根与权的作用顺序

Weyl 词作用于根与权时，统一采用**最后一个字母先作用**的约定。`word_act_root` 实现根上的词作用，对应上游 `RootSystem::permuted_root`（`rootdata.h:313–318`）；`word_act_weight` 实现权上的词作用，对应 `weyl.cpp:1071–1082`，两者采用相同顺序。^[atlas-core-center-classifier.md:51-53]

## 作用顺序

若 Weyl 词写作 \(w=s_1s_2\cdots s_k\)，则对根或权 \(v\)，有 \(w(v)=s_1(s_2(\cdots s_k(v)\cdots))\)。也就是说，先应用最右侧的 \(s_k\)，最后应用最左侧的 \(s_1\)；例如，词 \(s_1s_2\) 先施加 \(s_2\)，再施加 \(s_1\)。^[atlas-core-center-classifier.md:51-53]

## 反射词与轨道词构造

`simple_reflect_root_nbr` 实现单根 \(s\) 的反射在 `RootNbr` 上的作用。`reflection_word` 沿首个下降将根降到单根，再逆序回溯得到共轭词；源材料将其对应到上游 `RootSystem::reflection_word`（`rootdata.cpp:601–618`）。相关构造见 [[反射词的首个下降构造]]。^[atlas-core-center-classifier.md:48-50]

轨道词转换函数 `convert_to_words` 经陪集树展开恒等，每一步将反射词**左乘**到父段条目上。该组合方向与反射子群见证序配对，使用一致的词对权作用约定；它所在的轨道机器还包括 [[Levi 子商轨道与顶点轨道扩展]]。^[atlas-core-center-classifier.md:38-44]

## 证据边界

本页依据 `domain_builtins.rs` 中相关实现的结构性阅读，不构成数学验收。上游函数与行号的对应属于实现方的移植陈述，兼容性仍以 HPC 差分门为准；参见 [[HPC 验收证据链]]。^[atlas-core-center-classifier.md:9-14, atlas-core-center-classifier.md:55-60]

## Sources

- [atlas-core-center-classifier.md](../../sources/atlas-core-center-classifier.md) — 中心分类器与轨道词（domain_builtins.rs 6102–7992）。
