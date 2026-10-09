---
title: BlockGraph 存储布局与坐标定位不变量
summary: 块按 x 外层、y 内层编号，descent 与 cross 使用不同平铺布局，坐标定位依赖 xs 弱增、区间哨兵及连续 y 偏移并校验结果。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:24.383Z"
updatedAt: "2026-10-09T19:26:31.142Z"
tags:
  - 块图
  - 存储布局
  - 不变量
aliases:
  - blockgraph-存储布局与坐标定位不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# BlockGraph 存储布局与坐标定位不变量

`BlockGraph` 表示实形式与对偶实形式的 KGB 图的纤维积，每个块元素以两侧 KGB 坐标 `(x, y)` 标识。其存储布局支持生成元操作、长度查询和坐标定位；定位成立的关键条件是 `xs` 弱增，以及同一 x 区间内可按连续 y 坐标偏移查找。参见 [[完整块的 KGB 纤维积]]。^[block-graph.md:17-24, block-graph.md:56-64, block-graph.md:88-91]

## 字段与平铺布局

`rank` 保存半单秩；`xs[z]`、`ys[z]` 保存块元素 `z` 的 primal、dual KGB 坐标；`lengths[z]` 保存 primal KGB 长度，供 KL 表进行长度排序。两侧坐标沿用各实形式自身的完整 KGB 编号：若改用 common Cartans 受限的 KGB 构造，编号会发生变化。^[block-graph.md:26-33, block-graph.md:56-61]

`descent` 按 `z * rank + s` 平铺，即元素优先；`cross` 按 `s * size + z` 平铺，即生成元优先，对应上游 `data[s][z]`。两张表的索引方向不能混用；状态含义见 [[BlockDescent 八值状态体系]]。^[block-graph.md:56-58]

`cayley_first`、`cayley_second` 保存 Cayley 像对，由直接与逆 Cayley 访问器共享。`cayley(z, s)` 在 weak descent 处强制返回 `Some((None, None))`；`inverse_cayley(z, s)` 恰在 weak descent 处返回槽原值，其他状态返回空像对。内层 `None` 对应上游 `UndefBlock`。参见 [[块图的直接与逆 Cayley 变换]]。^[block-graph.md:58-60, block-graph.md:84-87]

## 元素编号与区间索引

构建时，各包按原 KGB 对合顺序排列，包内采用 **x 外层、y 内层**的笛卡尔积顺序。实现检查 `xs.len() == size`，不满足时报告 `"block size"`；`xs` 弱增则是区间索引和坐标定位赖以成立的不变量。^[block-graph.md:62-71]

`first_z_of_x[x]` 保存满足 \(x(z)\ge x\) 的首个块元素位置，表长为 `xrange + 1`，并带有值为 `size` 的哨兵。这里的条件是“大于等于”，因此表项并不保证对应元素的 x 坐标恰好等于查询值。^[block-graph.md:62-64]

`element(x, y)` 先取得 x 对应的 `first_z_of_x` 区间，再按连续 y 坐标的偏移定位候选元素。随后核验候选元素的实际坐标，承担上游断言的校验职责；失败返回 `Err(StructureError)`，包括越界的 `IndexOutOfRange` 与纤维不符的 `"element fiber"`。^[block-graph.md:88-91]

按块元素编号读取时，`x(z)`、`y(z)`、`length(z)` 返回 `Option`；完整块的 `cross(z, s)` 对每个生成元都有定义。^[block-graph.md:80-83, block-graph.md:103-104]

## 构建与变换中的维护

`BlockGraph::build` 检查两图半单秩是否一致，以及对偶表与对偶 inner class 的根系统是否相同，不匹配时报 `DatumMismatch`。Cayley 表构建会回填逆 Cayley 槽，槽满时报 `"Cayley pair slots"`。cross 表先填零再全量覆写；来源指出，当前零初值不可观测，但若循环边界改变，可能留下静默默认值。^[block-graph.md:66-76]

`dual()` 将元素编号反转为 \(z'=\mathrm{size}-1-z\)，交换 x/y 坐标，并变换长度、状态及链接。变换后，`first_z_of_x` 按 `max_y + 1` 的范围重新计算。长度反射使用末元素长度作为最大长度，依赖末元素长度最大的排序性质；空块取最大长度为 0。参见 [[完整块图的对偶数据变换]]。^[block-graph.md:95-104]

## 测试与证据边界

来源记录的 7 个测试全部为秩 1 的 A1 情形，覆盖具体块坐标、对偶的编号反转与坐标交换，以及通过 `element` 换编号后与原生对偶块的一致性。在覆盖完整 KGB 范围时，还检查 `dual().dual()` 恢复原块，包括 `first_z_of_x` 表。^[block-graph.md:117-124]

这些测试未覆盖多生成元情形，因而未覆盖两种平铺布局的区分；空对偶包、空块和 `element` 失败分支也未覆盖。来源属于结构性阅读记录，本身未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。参见 [[完整块图的测试覆盖与证据边界]]。^[block-graph.md:124-126, block-graph.md:139-139]

## Sources

- [block-graph.md](block-graph.md) — 完整块图：实形式与对偶实形式的纤维积。
