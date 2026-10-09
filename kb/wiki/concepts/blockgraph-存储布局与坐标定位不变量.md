---
title: BlockGraph 存储布局与坐标定位不变量
summary: 块元素按 x 外层、y 内层编号，descent 与 cross 使用不同平铺布局，坐标定位依赖 xs 弱增、区间哨兵和连续 y 偏移并校验目标坐标。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:24.383Z"
updatedAt: "2026-10-09T22:24:59.459Z"
tags:
  - 块图
  - 数据布局
  - 索引
aliases:
  - blockgraph-存储布局与坐标定位不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: BlockGraph 存储布局与坐标定位不变量
summary: BlockGraph 保留完整 KGB 坐标；descent 与 cross 采用不同平铺布局，坐标定位依赖 xs 弱增、区间哨兵和连续 y 偏移，并校验候选坐标。
sources:
  - block-graph.md
kind: concept
tags:
  - Rust设计
  - 块图
  - 存储布局
aliases:
  - blockgraph-存储布局与坐标定位不变量
---

# BlockGraph 存储布局与坐标定位不变量

`BlockGraph` 表示实形式与对偶实形式的 KGB 图的纤维积，每个块元素保存两侧坐标 `(x, y)`。其存储与定位依赖三个关键约定：保留完整 KGB 编号、使 `xs` 弱增，以及在同一 x 区间内按连续 y 坐标偏移定位。数学背景见 [[完整块的 KGB 纤维积]]。^[block-graph.md:17-24, block-graph.md:26-33, block-graph.md:56-64, block-graph.md:88-91]

## 字段与平铺布局

`rank` 保存半单秩；`xs[z]`、`ys[z]` 分别保存块元素 `z` 的 primal、dual KGB 坐标；`lengths[z]` 保存 primal KGB 长度，供 KL 表进行长度排序。坐标必须沿用各实形式自身的完整 KGB 编号，因为 common Cartans 受限构造会改变编号。^[block-graph.md:26-33, block-graph.md:56-61, block-graph.md:66-70]

`descent` 按 `z * rank + s` 平铺，即元素优先；`cross` 按 `s * size + z` 平铺，即生成元优先，对应上游 `data[s][z]`。这两种索引方向必须分别保留；状态含义见 [[BlockDescent 八值状态体系]]。^[block-graph.md:56-58]

`cayley_first`、`cayley_second` 保存 Cayley 像对，由直接与逆 Cayley 访问器共享。`cayley(z, s)` 在 weak descent 处强制返回 `Some((None, None))`；`inverse_cayley(z, s)` 恰在 weak descent 处返回槽原值，其他状态返回 `Some((None, None))`。内层 `None` 对应上游 `UndefBlock`，详见 [[块图的直接与逆 Cayley 变换]]。^[block-graph.md:58-60, block-graph.md:84-87]

## 元素编号与坐标定位

构建时，各包按原 KGB 对合顺序排列，包内采用 **x 外层、y 内层**的笛卡尔积顺序。实现检查 `xs.len() == size`，不满足时报 `"block size"`；`xs` 弱增是区间表与坐标定位成立的必要不变量。^[block-graph.md:62-71]

`first_z_of_x[x]` 保存满足 \(x(z)\ge x\) 的首个块元素位置。表长为 `xrange + 1`，并带有值为 `size` 的哨兵。这里使用“大于等于”条件，因此表项所指元素的 x 坐标不必恰好等于查询值。^[block-graph.md:62-64]

`element(x, y)` 先取得 x 对应的 `first_z_of_x` 区间，再按连续 y 坐标偏移定位候选元素，随后核验候选坐标。这一检查承担上游断言的校验职责；失败返回 `Err(StructureError)`，包括越界的 `IndexOutOfRange` 与纤维不符的 `"element fiber"`。^[block-graph.md:88-91]

按块元素编号读取时，`x(z)`、`y(z)`、`length(z)` 返回 `Option`；完整块的 `cross(z, s)` 对每个生成元都有定义。^[block-graph.md:80-83, block-graph.md:103-104]

## 构建约束与表初始化

`BlockGraph::build` 要求两图半单秩一致，并要求对偶表与对偶 inner class 的根系统相同，不匹配时报 `DatumMismatch`。对偶 inner class 提供 distinguished twist 与最长元，`weyl_budget` 限定定位最长元的枚举规模。^[block-graph.md:66-70]

Cayley 表构建会回填逆 Cayley 槽，槽满时报 `"Cayley pair slots"`。cross 表先填零，再全量覆写；来源的阅读观察指出，当前零初值不可观测，但若循环边界改变，可能留下静默默认值。^[block-graph.md:72-76]

## 对偶变换中的索引重建

`dual()` 将元素编号反转为 \(z'=\mathrm{size}-1-z\)，交换 x/y 坐标，并变换状态及链接。交换后，`first_z_of_x` 按原块 `max_y + 1` 的范围重新计算，相关变换见 [[完整块图的对偶数据变换]]。^[block-graph.md:95-104]

对偶长度为 \(\mathrm{max\_len}-\ell(z)\)，其中 \(\mathrm{max\_len}\) 取末元素长度，空块取 0。该实现依赖末元素长度最大的排序性质。^[block-graph.md:95-99]

## 测试与证据边界

来源记录的 7 个测试全部为秩 1 的 A1 情形，覆盖具体块坐标、对偶编号反转与坐标交换，以及通过 `element` 换编号后与原生对偶块的一致性。在覆盖完整 KGB 范围时，还检查 `dual().dual()` 恢复原块，包括 `first_z_of_x` 表。^[block-graph.md:117-124]

测试未覆盖多生成元情形，因此未覆盖两种平铺布局的区分；空对偶包、空块和 `element` 失败分支也未覆盖。详见 [[完整块图的测试覆盖与证据边界]]。^[block-graph.md:124-126]

本页依据结构性阅读材料；来源本身未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。块枚举正确性属于独立的 HPC 证据链，不能由此处的存储说明推定。^[block-graph.md:9-13, block-graph.md:139-143]

## Sources

- [block-graph.md](../../sources/block-graph.md) — 完整块图：实形式与对偶实形式的纤维积。
