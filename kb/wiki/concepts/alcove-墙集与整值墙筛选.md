---
title: Alcove 墙集与整值墙筛选
summary: wall_set 计算 gamma 经小 dominant 位移所达 alcove 的墙余根，记录整值墙并按余根差的非成员条件筛选。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
createdAt: "2026-10-09T14:32:34.499Z"
updatedAt: "2026-10-10T00:21:10.269Z"
tags:
  - alcove
  - 根系
aliases:
  - alcove-墙集与整值墙筛选
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Alcove 墙集与整值墙筛选
summary: wall_set 确定 gamma 经小优势方向位移所达 alcove 的墙余根，按余根差的成员关系筛选，并将其中在 gamma 上取整数值的墙收集到 integrals。
sources:
  - atlas-core-root-numbering-alcove.md
kind: concept
tags:
  - alcove
  - 根系
aliases:
  - alcove-墙集与整值墙筛选
provenanceState: extracted
---

# Alcove 墙集与整值墙筛选

`wall_set` 确定参数 `gamma` 经小 dominant（优势方向）位移所到达的 alcove 的墙余根。其中，在 `gamma` 上取整数值的墙被收集到 `integrals`，对应上游的 `on_wall_coroots`。^[atlas-core-root-numbering-alcove.md:29-32]

## 筛选条件

墙集过滤器只保留“余根不能被减去”的根，对应上游 `min_coroots_for` 的成员条件：相关余根之差 $\alpha^\vee-\beta^\vee$ 不是余根。墙集筛选依据余根差的成员关系；整值墙的记录则依据墙在 `gamma` 上的取值是否为整数。^[atlas-core-root-numbering-alcove.md:29-32]

## 墙分量与标签

`root_components` 按非正交关系将根子集划分为连通分量，每个分量内部按 RootNbr 升序排列。原版在新根触及分量时就追加该根，最终分量顺序按最大 RootNbr，而非最小 RootNbr；这一顺序在 FPP 乘积向量及其 Weyl 见证中可观察。编号约定见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[atlas-core-root-numbering-alcove.md:33-36]

`labels_for_component` 计算每个墙分量的余根之间唯一的本原整数关系，并取正。环境余根坐标与上游单余根坐标具有相同的线性关系，因此可由环境余根表计算核，并使用 `gcd`、`lcm` 辅助计算。`sorted_by_label` 将全部墙按标签降序排列，标签相同时按 RootNbr 序排列，详见 [[Alcove 墙标签与标签排序]]。^[atlas-core-root-numbering-alcove.md:37-41]

## Weyl 词构造与墙数检查

`from_fundamental_alcove` 构造对应于给定墙集的 alcove 的 Weyl 词：每个分量留出一个标签为 1 的墙，其余墙经 `to_positive_system` 移到单根系，再将步骤逆序并通过最终单值索引得到词。^[atlas-core-root-numbering-alcove.md:42-44]

基本 alcove 的墙由单余根及每个不可约分量的一条最低余根组成，因此墙数等于“秩 + 分量数”。在所述 `"Too few walls"` 检查中，只有墙集大小可观察，详见 [[基本 Alcove 的墙数]]。^[atlas-core-root-numbering-alcove.md:45-48]

## 证据边界

本页依据对 `crates/atlas-core/src/domain_builtins.rs` 中根编号与 alcove 机器的结构性阅读，不构成数学验收。来源中的上游行号引用属于实现方的移植陈述；编号与 alcove 行为的兼容性仍以 [[HPC 验收证据链|HPC 差分门]] 为准。^[atlas-core-root-numbering-alcove.md:9-14, atlas-core-root-numbering-alcove.md:55-56]

## Sources

- [根编号与 alcove 机器（domain_builtins.rs 5005–5924）](../../sources/atlas-core-root-numbering-alcove.md)
