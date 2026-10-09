---
title: Weyl 词对根与权的作用顺序
summary: word_act_root 与 word_act_weight 均采用最后一个字母先作用的约定；来源仅提供结构性说明，上游兼容性仍须 HPC 差分验证。
sources:
  - atlas-core-center-classifier.md
kind: concept
createdAt: "2026-10-09T14:26:42.739Z"
updatedAt: "2026-10-09T22:12:58.682Z"
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
summary: word_act_root 与 word_act_weight 均采用最后一个字母先作用的约定；轨道词转换将每步反射词左乘到父段条目上，兼容性仍须 HPC 差分验证。
sources:
  - atlas-core-center-classifier.md
kind: concept
tags:
  - Weyl群
  - 作用顺序
  - 兼容性
aliases:
  - weyl-词对根与权的作用顺序
---

# Weyl 词对根与权的作用顺序

Weyl 词作用于根与权时，统一采用**最后一个字母先作用**的约定。`word_act_root` 对应上游 `RootSystem::permuted_root`（`rootdata.h:313–318`）；`word_act_weight` 对应 `weyl.cpp:1071–1082`，两者的作用顺序一致。^[atlas-core-center-classifier.md:51-53]

## 作用顺序

对于词 \(w=s_1s_2\cdots s_k\) 及根或权 \(v\)，这一约定写作 \(w(v)=s_1(s_2(\cdots s_k(v)\cdots))\)：先施加最右侧的 \(s_k\)，最后施加最左侧的 \(s_1\)。例如，词 \(s_1s_2\) 表示先施加 \(s_2\)，再施加 \(s_1\)。^[atlas-core-center-classifier.md:51-53]

## 反射词的构造

`simple_reflect_root_nbr` 实现单根 \(s\) 的反射在 `RootNbr` 上的作用。`reflection_word` 则沿**首个下降**将根降到单根，再逆序回溯得到共轭词；来源将其对应到上游 `RootSystem::reflection_word`（`rootdata.cpp:601–618`）。详见 [[反射词的首个下降构造]]。^[atlas-core-center-classifier.md:48-50]

## 轨道词转换的乘法方向

`convert_to_words` 经陪集树展开恒等，每一步将反射词**左乘**到父段条目上。来源明确指出，这一方向与反射子群的见证序配对，采用一致的词对权作用约定。^[atlas-core-center-classifier.md:42-44]

该转换属于 adjoint 轨道机器。同一机器中的 `basic_orbit_adjoint` 构造 `cartan` 前 \(i+1\) 个生成元的 Levi 子商轨道；`vertex_orbit` 是用于沿 `label > 1` 最终扩展的模变体。相关背景见 [[Levi 子商轨道与顶点轨道扩展]] 与 [[adjoint 轨道的分层有序 BFS]]。^[atlas-core-center-classifier.md:31-44]

## 证据边界

本页依据 `crates/atlas-core/src/domain_builtins.rs` 相关实现的结构性阅读，不构成数学验收。上游函数及行号的对应属于实现方的移植陈述，兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-center-classifier.md:9-14, atlas-core-center-classifier.md:55-60]

## Sources

- [atlas-core-center-classifier.md](../../sources/atlas-core-center-classifier.md) — 中心分类器与轨道词（domain_builtins.rs 6102–7992）。
