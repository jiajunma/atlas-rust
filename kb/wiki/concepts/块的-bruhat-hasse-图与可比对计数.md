---
title: 块的 Bruhat Hasse 图与可比对计数
summary: Bruhat Hasse 图沿首个严格 good descent 递归构造直接下邻，在 split principal series 处使用 type-II 实下降的逆 Cayley 像；含自身的可比对计数要求 Hasse 行按拓扑序排列。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:33.389Z"
updatedAt: "2026-10-09T14:41:33.389Z"
tags:
  - Bruhat序
  - 偏序
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 块的 Bruhat Hasse 图与可比对计数

完整块的 `bruhat_hasse` 返回 Bruhat 偏序的 Hasse 图，以每个块元素的直接下邻表示覆盖关系；`n_bruhat_comparable` 据此统计可比对数，计数包含元素与自身的配对。实现分别对应上游 `complete_Hasse_diagram` 与 `n_comparable_from_Hasse`。^[block-graph.md:106-113]

## Hasse 图的构造

完整块来自一个实形式的 KGB 图与其对偶实形式的 KGB 图在 twisted involution 对偶关系上的纤维积。这里的 Hasse 图描述这些块元素之间的 Bruhat 偏序；块元素的构造背景见 [[完整块的 KGB 纤维积]]。^[block-graph.md:17-24, block-graph.md:106-110]

`bruhat_hasse` 沿首个严格 good descent 递归构造直接下邻；这种下降是 complex 或 type-I real，递归方式与 KGB 情形相同。在 split principal series 处，前驱恰为各 type-II 实 descent 的逆 Cayley 变换。完整块的实现委托给 `crate::block_access`；相关状态与变换见 [[BlockDescent 八值状态体系]]、[[块图的直接与逆 Cayley 变换]]，相关偏序构造见 [[Bruhat 偏序的 Hasse 图构造]]。^[block-graph.md:106-110]

## 可比对计数与输入顺序

`n_bruhat_comparable` 按上游 `poset.cpp:197-229` 的 `n_comparable_from_Hasse` 统计 Bruhat 可比对数，包含自身。该实现要求 Hasse 行按拓扑序排列，并直接索引 `closure[j]`；来源的结构性阅读指出，乱序输入会导致 panic。因此，行的拓扑顺序是此计数过程的调用前提。^[block-graph.md:111-113]

## 测试与证据边界

来源列出的测试全部为秩 1 的 A1 情形。其中，`block(SL(2,R), PGL(2,R))` 大小为 3，坐标依次为 `(0,1)`、`(1,1)`、`(2,0)`；测试核对的 Bruhat Hasse 图为 `[[], [], [0,1]]`，并检查它与 `block_access` 自由函数的结果一致。^[block-graph.md:117-121]

这些测试锚点未覆盖多生成元情形、空对偶包或空块，也未覆盖 `n_bruhat_comparable`。来源包仅记录结构性阅读，未执行构建、测试或原版运行，因此不能将上述测试描述当作本次执行所得的验证结果，也不能据此扩展数学验收结论。更多覆盖限制见 [[完整块图的测试覆盖与证据边界]]。^[block-graph.md:124-126, block-graph.md:139-143]

## Sources

- [block-graph.md](block-graph.md)：完整块图：实形式与对偶实形式的纤维积。
