---
title: 陪集树到 Weyl 词的转换
summary: convert_to_words 沿陪集树展开，将每步反射词左乘到父段条目上，并与子群见证序及词对权的作用约定配对。
sources:
  - atlas-core-center-classifier.md
kind: concept
createdAt: "2026-10-09T14:26:35.066Z"
updatedAt: "2026-10-10T00:15:09.616Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 陪集树到 Weyl 词的转换
summary: convert_to_words 从恒等元沿陪集树展开，每步将反射词左乘到父段条目上，与 Weyl 词最后一个字母先作用的约定一致。
sources:
  - atlas-core-center-classifier.md
kind: concept
tags:
  - Weyl群
  - 陪集树
  - 词转换
aliases:
  - 陪集树到-weyl-词的转换
  - 陪W词
---

# 陪集树到 Weyl 词的转换

`convert_to_words` 是 adjoint 轨道机器中沿陪集树展开 Weyl 词的过程：从恒等元出发，每一步将反射词**左乘**到父段条目上。源材料将其对应到上游 `alcoves.cpp:746–774`，并说明该规则与子群见证序配对，保持词对权的作用约定一致。^[atlas-core-center-classifier.md:42-44]

## 左乘规则与作用顺序

若父段条目对应 Weyl 词 $w$，当前步骤的反射词为 $r$，则新条目对应 $rw$。这将来源中的“左乘到父段条目上”写成乘积形式；相关见证背景见 [[反射子群轨道与见证的独立验证]]。^[atlas-core-center-classifier.md:42-44]

`word_act_root` 与 `word_act_weight` 分别实现词对根和权的作用，两者均采用**最后一个字母先作用**的约定。前者对应 `RootSystem::permuted_root`（`rootdata.h:313–318`），后者对应 `weyl.cpp:1071–1082`，参见 [[Weyl 词对根与权的作用顺序]]。^[atlas-core-center-classifier.md:51-53]

由这两项规则可知，乘积 $rw$ 的作用顺序是先施加父词 $w$，再施加当前反射词 $r$。

## 轨道构造与反射词背景

轨道元素 `AdjOrbitElem` 使用 adjoint 坐标。共享核心 `adjoint_orbit_bfs` 将新元素插在 `finish` 之后，使尾部保持递减，并在每层完成后反转为递增；这一排序纪律见 [[adjoint 轨道的分层有序 BFS]]。^[atlas-core-center-classifier.md:33-37]

`basic_orbit_adjoint` 构造 `cartan` 前 $i+1$ 个生成元的 Levi 子商轨道；`vertex_orbit` 是用于沿 `label > 1` 进行最终扩展的模变体。两者的背景见 [[Levi 子商轨道与顶点轨道扩展]]。^[atlas-core-center-classifier.md:38-41]

反射词由 `reflection_word` 沿首个下降将根降到单根，再逆序回溯得到共轭词；`simple_reflect_root_nbr` 提供单根反射在 `RootNbr` 上的作用。具体构造见 [[反射词的首个下降构造]]。^[atlas-core-center-classifier.md:48-50]

## 证据边界

本页依据 `crates/atlas-core/src/domain_builtins.rs` 相关实现的结构性阅读，不构成数学验收。来源中的上游行号及对应关系属于实现方的移植陈述，兼容性仍以 HPC 差分门为准，参见 [[HPC 验收证据链]]。^[atlas-core-center-classifier.md:9-14, atlas-core-center-classifier.md:55-60]

## Sources

- [atlas-core-center-classifier.md](../../sources/atlas-core-center-classifier.md) — 中心分类器与轨道词（domain_builtins.rs 6102–7992）。
