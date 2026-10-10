---
title: BlockDescent 八值状态体系
summary: 八种块级状态由根类型及两侧 cross 行为决定，以位 0x4 标识 weak descent，并通过固定表实现对偶映射和语言状态码转换。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:18.039Z"
updatedAt: "2026-10-10T00:27:50.328Z"
tags:
  - 块图
  - 下降状态
aliases:
  - blockdescent-八值状态体系
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: BlockDescent 八值状态体系
summary: 块级八种状态由根类型、cross 行为与对偶侧信息确定；内部索引的 0x4 位标识弱下降，固定表提供对偶映射和 Atlas 语言码转换。
sources:
  - block-graph.md
kind: concept
tags:
  - 块图
  - 下降状态
aliases:
  - blockdescent-八值状态体系
---

# BlockDescent 八值状态体系

`BlockDescent` 表示完整块图中每个元素相对于一个生成元的八种状态。分类结合根类型、cross 是否移动元素及对偶侧信息，区分 ascent（上升）与 weak descent（弱下降），并提供对偶状态映射和 Atlas 语言码转换。^[block-graph.md:35-52]

## 八值顺序与语言码

八种状态遵循上游 `descents::DescentStatus::Value` 的顺序，其中上游 `RealNoncompact` 对应 Rust 的 `RealNonparity`。前四值为上升，后四值为弱下降；`is_descent` 按内部索引计算 `index & 0x4 != 0`。语言码则通过固定表 `TAB = [4,5,6,7,1,0,3,2]` 转换，具体对应如下。^[block-graph.md:37-47]

| 内部索引 | 状态 | 简写 | 分类 | Atlas 语言码 |
|---|---|---|---|---|
| 0 | `ComplexAscent` | C+ | 上升 | 4 |
| 1 | `RealNonparity` | rn | 上升 | 5 |
| 2 | `ImaginaryTypeI` | i1 | 上升 | 6 |
| 3 | `ImaginaryTypeII` | i2 | 上升 | 7 |
| 4 | `ImaginaryCompact` | ic | 弱下降 | 1 |
| 5 | `ComplexDescent` | C− | 弱下降 | 0 |
| 6 | `RealTypeII` | r2 | 弱下降 | 3 |
| 7 | `RealTypeI` | r1 | 弱下降 | 2 |

内部索引与语言码必须区分：`0x4` 位判定用于内部索引；Atlas 语言码中，0–3 对应弱下降，4–7 对应上升。^[block-graph.md:40-47]

## 块级状态判定

`descents()` 对复根检查 `is_descent`：为真时取 C−，否则取 C+。对虚非紧根，检查 cross 是否移动当前元素：移动时取 i1，不动时取 i2。^[block-graph.md:49-51]

对实根或虚紧根，还需查看对偶侧：若对偶侧为虚非紧根，其 cross 移动时取 r2、不动时取 r1；否则，实根取 rn，虚紧根取 ic。这一判定使用了[[完整块的 KGB 纤维积]]两侧的信息。^[block-graph.md:17-24, block-graph.md:49-52]

## 对偶映射

`BlockDescent::dual` 按下表成对交换状态；每对连接一个上升与一个弱下降，连续应用两次恢复原状态。^[block-graph.md:37-44]

| 状态 | 对偶状态 |
|---|---|
| `ComplexAscent`（C+） | `ComplexDescent`（C−） |
| `RealNonparity`（rn） | `ImaginaryCompact`（ic） |
| `ImaginaryTypeI`（i1） | `RealTypeII`（r2） |
| `ImaginaryTypeII`（i2） | `RealTypeI`（r1） |

[[完整块图的对偶数据变换]]使用这一映射转换下降状态，同时反转元素顺序、交换两侧 KGB 坐标，并映射 cross/Cayley 链接的目标编号。^[block-graph.md:95-101]

## 存储、访问与 Cayley 方向

在 [[BlockGraph 存储布局与坐标定位不变量]]中，下降状态按 `z * rank + s` 平铺存储，其中 `z` 为块元素索引，`s` 为生成元索引。`descent_value(z, s)` 读取状态，`status_code(z, s)` 返回对应的 Atlas 语言码。^[block-graph.md:56-64, block-graph.md:80-82]

弱下降决定[[块图的直接与逆 Cayley 变换]]访问器的方向。`cayley(z, s)` 在弱下降处强制返回 `Some((None, None))`；`inverse_cayley(z, s)` 在弱下降处返回存储的像对，在非下降处返回 `Some((None, None))`。内层 `None` 对应上游 `UndefBlock`。^[block-graph.md:84-87]

构建时，i1 对应单值直接 Cayley，并通过 `first_free_slot` 回填逆 Cayley 槽；i2 对应双值直接 Cayley，先处理 `dual_second`，随后进入 i1 分支处理 `dual_first`。槽位已满时报 `"Cayley pair slots"`。^[block-graph.md:72-75]

弱下降与 Bruhat 递归中的严格 good descent 用途不同：[[块的 Bruhat Hasse 图与可比对计数]]沿首个 complex 或 type-I real 的严格 good descent 递归；在 split principal series 处，前驱来自各 type-II 实下降的逆 Cayley 变换。^[block-graph.md:106-110]

## 测试覆盖与证据边界

来源记录的七个测试均为秩 1（A1）。`block(SL(2,R), PGL(2,R))` 覆盖 i1/i1/r1 状态、语言码 6/2、单值直接 Cayley 与双值逆 Cayley，并对齐冻结 fixture capture `3501519`；反向配对覆盖 i2 的双值直接 Cayley 与 r2 的单值逆 Cayley。测试还逐项检查下降状态对偶表及其对合性。^[block-graph.md:117-124]

这些测试未覆盖 C±、rn、ic 的块级状态判定，也未覆盖多生成元情形下两种平铺布局的区别。来源属于结构性阅读记录，本身未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；块枚举正确性另属独立的 HPC 证据链。^[block-graph.md:9-13, block-graph.md:124-126, block-graph.md:139-139]

## Sources

- [block-graph.md](../../sources/block-graph.md) — 完整块图：实形式与对偶实形式的纤维积。
