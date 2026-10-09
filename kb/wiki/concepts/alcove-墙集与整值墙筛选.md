---
title: Alcove 墙集与整值墙筛选
summary: wall_set 计算 gamma 经小 dominant 位移所达 alcove 的墙余根，记录整值墙，并依据余根差不是余根的条件筛选。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:34.499Z"
updatedAt: "2026-10-09T20:35:35.224Z"
tags:
  - alcove
  - 余根
aliases:
  - alcove-墙集与整值墙筛选
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Alcove 墙集与整值墙筛选
summary: wall_set 确定 gamma 经小优势方向位移所达 alcove 的墙余根，以余根不可相减条件筛选，并将其中在 gamma 上取整数值的墙收集到 integrals。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
tags:
  - alcove
  - 余根
  - 墙集
aliases:
  - alcove-墙集与整值墙筛选
---

# Alcove 墙集与整值墙筛选

`wall_set` 根据参数 `gamma` 经小 dominant（优势方向）位移所到达的 alcove，确定其墙余根。其中在 `gamma` 上取整数值的墙收集到 `integrals`，对应上游的 `on_wall_coroots`。^[atlas-core-root-numbering-alcove.md:29-32]

## 墙余根与整值墙的筛选

墙集过滤器只保留“余根不能被减去”的根，对应上游 `min_coroots_for` 的成员条件：相关余根之差 \(\alpha^\vee-\beta^\vee\) 不是余根。墙集筛选依据余根差的成员关系，`integrals` 的收集则依据墙在 `gamma` 上的取值是否为整数。^[atlas-core-root-numbering-alcove.md:29-32]

## 墙集的分量与标签

`root_components` 按非正交关系划分根子集的连通分量，每个分量内部按 RootNbr 升序排列。原版在新根触及分量时就追加该根，因此最终分量顺序按最大 RootNbr，而非最小 RootNbr；这一顺序在 FPP 乘积向量及其 Weyl 见证中可观察。编号背景见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[atlas-core-root-numbering-alcove.md:33-36]

`labels_for_component` 为每个墙分量计算余根间唯一的本原整数关系，并取正。环境余根坐标与上游单余根坐标具有相同的线性关系，因此可以从环境余根表计算核，并用 `gcd`、`lcm` 辅助计算，详见 [[墙分量的本原 Coroot 关系]]。`sorted_by_label` 将全部墙按标签降序排列，标签相同时按 RootNbr 序排列，参见 [[Alcove 墙标签与标签排序]]。^[atlas-core-root-numbering-alcove.md:37-41]

## Weyl 词构造与墙数检查

给定墙集后，`from_fundamental_alcove` 在每个分量中留出一个标签为 1 的墙，将其余墙经 `to_positive_system` 移到单根系，再以逆序步骤结合最终单值索引得到 Weyl 词。该词对应具有给定墙集的 alcove。^[atlas-core-root-numbering-alcove.md:42-44]

基本 alcove 的墙由单余根以及每个不可约分量的一条最低余根组成，墙数等于“秩 + 分量数”。在所述 `"Too few walls"` 检查中，只有墙集大小可观察；相关说明见 [[基本 Alcove 的墙数]]。^[atlas-core-root-numbering-alcove.md:45-48]

## 证据边界

本页依据对 `domain_builtins.rs` 中根编号与 alcove 机器的结构性阅读，不构成数学验收。来源中的上游行号引用属于实现方的移植陈述；根编号与 alcove 行为的兼容性仍以 HPC 差分门为准，参见 [[HPC 验收证据链]]。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
