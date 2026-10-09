---
title: BlockGraph 存储布局与坐标定位不变量
summary: BlockGraph 以 x 外层、y 内层编号，分别按元素优先和生成元优先平铺 descent 与 cross，并依赖 xs 弱增、区间索引及连续 y 偏移实现带校验的坐标定位。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:24.383Z"
updatedAt: "2026-10-09T14:41:24.383Z"
tags:
  - Rust设计
  - 数据结构
  - 算法不变量
aliases:
  - blockgraph-存储布局与坐标定位不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# BlockGraph 存储布局与坐标定位不变量

`BlockGraph` 表示实形式 KGB 图与对偶实形式 KGB 图的纤维积。每个块元素由一对 primal/dual KGB 坐标 `(x, y)` 标识；存储布局同时支持生成元操作、长度查询和从坐标定位块元素。参见 [[完整块的 KGB 纤维积]]。^[block-graph.md:17-24, block-graph.md:56-64, block-graph.md:80-91]

## 字段与平铺布局

`rank` 保存半单秩，`xs[z]`、`ys[z]` 保存块元素 `z` 的两侧 KGB 坐标，`lengths[z]` 保存 primal KGB 长度，供 KL 表的长度排序使用。坐标沿用各实形式的完整 KGB 编号；若先限制到 common Cartans 再构建 KGB，编号可能改变。^[block-graph.md:26-33, block-graph.md:56-61]

`descent` 以 `z * rank + s` 索引，按块元素组织各生成元的状态；`cross` 以 `s * size + z` 索引，按生成元组织各块元素的像，对应上游 `data[s][z]`。两者的平铺方向不同。状态含义见 [[BlockDescent 八值状态体系]]。^[block-graph.md:56-58]

`cayley_first` 与 `cayley_second` 保存 Cayley 像对，由直接与逆 Cayley 访问器共享。`cayley(z, s)` 在 weak descent 处返回 `Some((None, None))`；`inverse_cayley(z, s)` 恰在 weak descent 处返回原始槽值，在其他状态返回空像对。内层 `None` 对应上游 `UndefBlock`。参见 [[块图的直接与逆 Cayley 变换]]。^[block-graph.md:58-60, block-graph.md:84-87]

## 元素编号与定位表

构建时，包按原 KGB 对合顺序排列，包内采用 **x 外层、y 内层**的笛卡尔积顺序。`xs` 弱增是 `first_z_of_x` 与坐标定位成立的关键不变量。构建还检查 `xs.len() == size`，不满足时报 `"block size"`。^[block-graph.md:62-71]

`first_z_of_x[x]` 保存满足 \(x(z)\ge x\) 的首个块元素位置。该表长度为 `xrange + 1`，并带有值为 `size` 的哨兵。这里采用“大于等于”条件，而非要求某个块元素的 x 坐标恰好等于查询值。^[block-graph.md:62-64]

## 从坐标定位元素

`element(x, y)` 先通过 `first_z_of_x` 取得 x 对应的区间，再依据连续的 y 坐标偏移定位候选元素。实现随后核验候选元素的实际坐标，承担上游断言的校验职责；失败返回 `Err(StructureError)`，包括越界的 `IndexOutOfRange` 与纤维不符的 `"element fiber"`。^[block-graph.md:88-91]

按块索引读取时，`x(z)`、`y(z)` 和 `length(z)` 返回 `Option`；`cross(z, s)` 对每个生成元都有定义。坐标查询与链接查询因此依赖同一套块编号及其存储布局。^[block-graph.md:80-91]

## 构建与对偶变换中的维护

`BlockGraph::build` 检查两图的半单秩，以及对偶表与对偶 inner class 的根系统是否一致，不匹配时报 `DatumMismatch`。Cayley 构建会回填逆像槽，槽满时报 `"Cayley pair slots"`；cross 表先填零再全量覆写，当前零初值不可观测，但来源指出，若循环边界改变，它可能成为静默默认值。^[block-graph.md:66-76]

`dual()` 将元素编号反转为 \(z'=\mathrm{size}-1-z\)，交换 x/y 坐标，映射长度、状态与链接，并按 `max_y + 1` 的范围重算 `first_z_of_x`。因此，对偶块的定位表需要随坐标变换重新建立。参见 [[完整块图的对偶数据变换]]。^[block-graph.md:95-104]

## 测试与证据边界

来源记录的 7 个测试全部为秩 1 的 A1 情形，覆盖具体块坐标、对偶的反转与坐标交换，以及通过 `element` 换编号后与原生对偶块比较。在覆盖完整 KGB 范围时，测试还检查 `dual().dual()` 恢复原块，包括 `first_z_of_x` 表。^[block-graph.md:117-124]

现有测试未覆盖多生成元情形，因而未验证两种平铺方向的区分；也未覆盖空对偶包、空块和 `element` 失败分支。来源属于结构性阅读记录，未执行构建、测试或原版运行，不提供数学验收或性能结论。参见 [[完整块图的测试覆盖与证据边界]]。^[block-graph.md:124-126, block-graph.md:139-139]

## Sources

- [block-graph.md](block-graph.md) — 完整块图：实形式与对偶实形式的纤维积。
