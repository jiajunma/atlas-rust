---
title: BlockDescent 八值状态体系
summary: BlockDescent 根据两侧根类型及 cross 行为区分八种状态，以索引位 0x4 标识 weak descent，并提供对偶状态映射与 Atlas 语言码转换。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:18.039Z"
updatedAt: "2026-10-09T14:41:18.039Z"
tags:
  - 表示论
  - 下降状态
  - 兼容性
aliases:
  - blockdescent-八值状态体系
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# BlockDescent 八值状态体系

`BlockDescent` 表示完整块图中每个元素相对于一个生成元的八种下降状态。状态由根类型、cross 作用及对偶侧信息共同决定；内部枚举顺序区分 ascent 与 weak descent，而 Atlas 语言使用另一套状态码。^[block-graph.md:35-52]

## 内部顺序与下降判定

八值按上游 `descents::DescentStatus::Value` 的顺序排列。前四值属于 ascent，后四值属于 weak descent；`is_descent` 使用 `index & 0x4 != 0` 判定。来源中的上游名称 `RealNoncompact` 对应此处的 `RealNonparity`。^[block-graph.md:37-42]

| 内部索引 | 状态 | 简写 | 分类 | Atlas 语言码 |
|---|---|---|---|---|
| 0 | `ComplexAscent` | C+ | ascent | 4 |
| 1 | `RealNonparity` | rn | ascent | 5 |
| 2 | `ImaginaryTypeI` | i1 | ascent | 6 |
| 3 | `ImaginaryTypeII` | i2 | ascent | 7 |
| 4 | `ImaginaryCompact` | ic | weak descent | 1 |
| 5 | `ComplexDescent` | C− | weak descent | 0 |
| 6 | `RealTypeII` | r2 | weak descent | 3 |
| 7 | `RealTypeI` | r1 | weak descent | 2 |

表中的语言码由 `language_code` 按 `TAB = [4,5,6,7,1,0,3,2]` 重编号得到。因此，按位判断下降时使用的是内部索引，不能直接套用于语言码。^[block-graph.md:37-47]

## 块级状态的判定

`descents()` 对复根检查 `is_descent`：为真时取 C−，否则取 C+。对虚非紧根，检查 cross 是否移动当前元素：移动时取 i1，不动时取 i2。^[block-graph.md:49-51]

对实根或虚紧根，判定依赖对偶侧：若对偶侧为虚非紧且 cross 移动，取 r2；若对偶侧为虚非紧且 cross 不动，取 r1。其余情形分别将实根记为 rn、虚紧根记为 ic。这一规则联系了[[完整块的 KGB 纤维积]]两侧的数据。^[block-graph.md:17-24, block-graph.md:49-52]

## 对偶映射

`BlockDescent::dual` 按以下配对交换状态，每一对都连接一个 ascent 与一个 weak descent。^[block-graph.md:37-44]

| 状态 | 对偶状态 |
|---|---|
| `ComplexAscent`（C+） | `ComplexDescent`（C−） |
| `RealNonparity`（rn） | `ImaginaryCompact`（ic） |
| `ImaginaryTypeI`（i1） | `RealTypeII`（r2） |
| `ImaginaryTypeII`（i2） | `RealTypeI`（r1） |

[[完整块图的对偶数据变换]]通过这一映射转换每个下降状态，同时反转元素顺序、交换两侧 KGB 坐标并映射 cross/Cayley 链接。^[block-graph.md:95-101]

## 与块图访问及 Cayley 变换的关系

在 [[BlockGraph 存储布局与坐标定位不变量]]中，下降状态按 `z * rank + s` 平铺存储，其中 `z` 为块元素、`s` 为生成元。`descent_value(z, s)` 读取状态，`status_code(z, s)` 返回对应的 Atlas 语言码。^[block-graph.md:56-64, block-graph.md:80-82]

weak descent 决定 Cayley 访问器的方向：`cayley(z, s)` 在 weak descent 处强制返回 `Some((None, None))`；`inverse_cayley(z, s)` 恰在 weak descent 处返回存储的像对，在非下降处返回 `Some((None, None))`。内层 `None` 对应上游 `UndefBlock`。详见[[块图的直接与逆 Cayley 变换]]。^[block-graph.md:84-87]

构建时，i1 对应单值直接 Cayley，并回填逆 Cayley 槽；i2 对应双值直接 Cayley，先处理 `dual_second`，再进入 i1 分支处理 `dual_first`。^[block-graph.md:72-75]

weak descent 也需与 Bruhat 递归采用的严格 good descent 区分：[[块的 Bruhat Hasse 图与可比对计数]]沿首个 complex 或 type-I real 的严格 good descent 递归；在 split principal series 处，前驱则来自各 type-II 实下降的逆 Cayley 变换。^[block-graph.md:106-110]

## 测试覆盖与证据边界

来源记录的七个测试均为秩 1（A1）。其中，`block(SL(2,R), PGL(2,R))` 覆盖 i1/i1/r1 状态、语言码 6/2 及单值直接与双值逆 Cayley；反向配对覆盖 i2 的双值直接 Cayley 与 r2 的单值逆 Cayley。测试还逐项检查下降状态的对偶表及其对合性。^[block-graph.md:117-124]

这些测试未覆盖 C±、rn、ic 的块级状态判定，也未覆盖多生成元情形。来源包属于结构性阅读记录，未执行构建、测试或原版运行，不能据此扩展为数学验收或性能结论。^[block-graph.md:124-126, block-graph.md:139-139]

## Sources

- [block-graph.md](block-graph.md) — 完整块图：实形式与对偶实形式的纤维积。
