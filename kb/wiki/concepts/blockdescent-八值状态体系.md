---
title: BlockDescent 八值状态体系
summary: 块级八种状态由两侧根类型与 cross 是否移动决定，位 0x4 标识 weak descent，并通过固定表转换对偶状态和 Atlas 语言码。
sources:
  - block-graph.md
kind: concept
createdAt: "2026-10-09T14:41:18.039Z"
updatedAt: "2026-10-09T20:48:23.820Z"
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: BlockDescent 八值状态体系
summary: 块级状态由根类型、cross 行为与对偶侧信息确定，内部索引的 0x4 位标识弱下降，并通过固定表转换对偶状态与 Atlas 语言码。
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

`BlockDescent` 表示完整块图中每个元素相对于一个生成元的八种状态。它结合根类型、cross 是否移动元素以及对偶侧信息进行分类，以内部索引区分 ascent（上升）与 weak descent（弱下降），并提供对偶状态映射和 Atlas 语言码转换。^[block-graph.md:35-52]

## 八值顺序与下降判定

八种状态遵循上游 `descents::DescentStatus::Value` 的顺序，其中上游名称 `RealNoncompact` 对应 Rust 的 `RealNonparity`。前四值属于 ascent，后四值属于 weak descent；`is_descent` 的判定式为 `index & 0x4 != 0`。各状态及语言码如下。^[block-graph.md:37-47]

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

`language_code` 使用 `TAB = [4,5,6,7,1,0,3,2]` 将内部索引转换为 Atlas 语言码。`0x4` 位判定针对内部索引；转换后的语言码以 0–3 表示弱下降状态，以 4–7 表示上升状态。^[block-graph.md:40-47]

## 块级状态判定

`descents()` 对复根检查 `is_descent`：为真时取 C−，否则取 C+。对虚非紧根，检查 cross 是否移动当前元素：移动时取 i1，不动时取 i2。^[block-graph.md:49-51]

对实根或虚紧根，判定依赖对偶侧：若对偶侧为虚非紧根且 cross 移动，取 r2；若对偶侧为虚非紧根且 cross 不动，取 r1。其余情形将实根记为 rn、虚紧根记为 ic。因此，[[完整块的 KGB 纤维积]]两侧的数据共同参与块级状态判定。^[block-graph.md:17-24, block-graph.md:49-52]

## 对偶映射

`BlockDescent::dual` 按下表交换状态，每一对都连接一个 ascent 与一个 weak descent；连续应用两次恢复原状态。^[block-graph.md:37-44]

| 状态 | 对偶状态 |
|---|---|
| `ComplexAscent`（C+） | `ComplexDescent`（C−） |
| `RealNonparity`（rn） | `ImaginaryCompact`（ic） |
| `ImaginaryTypeI`（i1） | `RealTypeII`（r2） |
| `ImaginaryTypeII`（i2） | `RealTypeI`（r1） |

[[完整块图的对偶数据变换]]通过这一映射转换每个下降状态，同时反转块元素顺序、交换两侧 KGB 坐标，并转换 cross/Cayley 链接的目标编号。^[block-graph.md:95-101]

## 存储、访问与 Cayley 方向

在 [[BlockGraph 存储布局与坐标定位不变量]]中，下降状态按 `z * rank + s` 平铺存储，`z` 表示块元素，`s` 表示生成元。`descent_value(z, s)` 读取状态，`status_code(z, s)` 返回对应的 Atlas 语言码。^[block-graph.md:56-64, block-graph.md:80-82]

weak descent 决定[[块图的直接与逆 Cayley 变换]]访问器的方向：`cayley(z, s)` 在 weak descent 处强制返回 `Some((None, None))`；`inverse_cayley(z, s)` 恰在 weak descent 处返回存储的像对，在非下降处返回 `Some((None, None))`。内层 `None` 对应上游的 `UndefBlock`。^[block-graph.md:84-87]

构建时，i1 对应单值直接 Cayley，并通过 `first_free_slot` 回填逆 Cayley 槽；i2 对应双值直接 Cayley，先处理 `dual_second`，随后进入 i1 分支处理 `dual_first`。槽位已满时报 `"Cayley pair slots"`。^[block-graph.md:72-75]

weak descent 与 Bruhat 递归采用的严格 good descent 用途不同：[[块的 Bruhat Hasse 图与可比对计数]]沿首个 complex 或 type-I real 的严格 good descent 递归；在 split principal series 处，前驱来自各 type-II 实下降的逆 Cayley 变换。^[block-graph.md:106-110]

## 测试覆盖与证据边界

来源记录的七个测试均为秩 1（A1）。`block(SL(2,R), PGL(2,R))` 覆盖 i1/i1/r1 状态、语言码 6/2，以及单值直接 Cayley 和双值逆 Cayley，并对齐冻结 fixture capture `3501519`；反向配对覆盖 i2 的双值直接 Cayley 与 r2 的单值逆 Cayley。测试还逐项检查下降状态对偶表及其对合性。^[block-graph.md:117-124]

这些测试未覆盖 C±、rn、ic 的块级状态判定，也未覆盖多生成元情形下两种平铺布局的区别。来源属于结构性阅读记录，本身未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；块枚举正确性另有独立的 HPC 证据链。^[block-graph.md:9-13, block-graph.md:124-126, block-graph.md:139-139]

## Sources

- [block-graph.md](block-graph.md) — 完整块图：实形式与对偶实形式的纤维积。
