---
title: 块的 Bruhat Hasse 图与可比对计数
summary: Hasse 图沿首个严格 good descent 递归构造，split principal series 使用 type-II 实下降的逆 Cayley 像；含自身的可比对计数要求 Hasse 行按拓扑序排列。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:33.389Z"
updatedAt: "2026-10-09T22:25:07.321Z"
tags:
  - Bruhat偏序
  - 图算法
aliases:
  - 块的-bruhat-hasse-图与可比对计数
  - 块BH图
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 块的 Bruhat Hasse 图与可比对计数
summary: 完整块的 Hasse 图沿首个严格 good descent 递归构造；可比对计数包含自身，并要求 Hasse 行按拓扑序排列。
sources:
  - block-graph.md
kind: concept
tags:
  - Bruhat偏序
  - Hasse图
  - 图算法
aliases:
  - 块的-bruhat-hasse-图与可比对计数
  - 块BH图
---

# 块的 Bruhat Hasse 图与可比对计数

完整块的 `bruhat_hasse` 返回 Bruhat 偏序的 Hasse 图，以每个块元素的直接下邻表示覆盖关系；`n_bruhat_comparable` 据此统计 Bruhat 可比对数，包含元素与自身的配对。两者分别对应上游的 `complete_Hasse_diagram` 与 `n_comparable_from_Hasse`。^[block-graph.md:106-113]

## Hasse 图的构造

完整块由一个实形式的 KGB 图与其对偶实形式的 KGB 图，按 twisted involution 的对偶关系配对形成纤维积。Hasse 图描述这些块元素之间的 Bruhat 偏序；元素构造背景见 [[完整块的 KGB 纤维积]]。^[block-graph.md:17-24, block-graph.md:106-110]

`bruhat_hasse` 沿首个严格 good descent 递归计算直接下邻；这里的下降是 complex 或 type-I real，递归方式与 KGB 情形相同。在 split principal series 处，前驱恰为各 type-II 实下降的逆 Cayley 变换。相关状态与变换见 [[BlockDescent 八值状态体系]]、[[块图的直接与逆 Cayley 变换]]。^[block-graph.md:106-110]

实现委托给 `crate::block_access`，对应上游 `blocks.cpp:1603-1656` 的 `complete_Hasse_diagram`；相关算法见 [[Bruhat 偏序的 Hasse 图构造]]。^[block-graph.md:106-110]

## 可比对计数与拓扑顺序

`n_bruhat_comparable` 按上游 `poset.cpp:197-229` 的 `n_comparable_from_Hasse` 统计可比对数，计数包含自身。实现要求 Hasse 行按拓扑序排列，并直接索引 `closure[j]`；来源的结构性阅读指出，乱序会导致 panic。拓扑顺序因此是该计数过程必须保持的输入前提。^[block-graph.md:111-113]

## 测试与证据边界

来源列出的 7 个测试全部属于秩 1 的 A1 情形。其中，`block(SL(2,R), PGL(2,R))` 大小为 3，坐标依次为 `(0,1)`、`(1,1)`、`(2,0)`。Bruhat Hasse 图的测试锚点是：元素 0、1 没有直接下邻，元素 2 的直接下邻为 0、1；测试还检查其与 `block_access` 自由函数的结果一致。^[block-graph.md:117-121]

来源明确列出的未覆盖项包括多生成元情形、空对偶包或空块，以及 `n_bruhat_comparable`。因此，已有 Hasse 图测试锚点不能作为可比对计数已获测试覆盖的依据；完整限制见 [[完整块图的测试覆盖与证据边界]]。^[block-graph.md:124-126]

来源属于结构性源码阅读，本身未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。块枚举正确性另有自身的 HPC 证据链，该来源不重述或扩展其结论；所列上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。^[block-graph.md:9-13, block-graph.md:139-143]

## Sources

- [block-graph.md](../../sources/block-graph.md)：完整块图：实形式与对偶实形式的纤维积。
