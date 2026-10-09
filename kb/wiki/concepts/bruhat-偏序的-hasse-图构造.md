---
title: Bruhat 偏序的 Hasse 图构造
summary: 按首个严格良下降构造直接下邻并插入上升像，无此下降时取各 RealTypeII 的第一逆 Cayley 分量；索引和 expect 依赖构造不变量。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:19.978Z"
updatedAt: "2026-10-09T22:24:06.566Z"
tags:
  - Bruhat偏序
  - 图算法
aliases:
  - bruhat-偏序的-hasse-图构造
  - B偏H图
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Bruhat 偏序的 Hasse 图构造
summary: bruhat_hasse 根据首个严格 good descent 的 cross 或逆 Cayley 像及上升扩展构造直接下邻；无此下降时收集 RealTypeII 的第一逆 Cayley 分量。算法依赖块拓扑的构造不变量。
sources:
  - block-access-modifier.md
kind: concept
tags:
  - Bruhat偏序
  - 图算法
  - 结构不变量
aliases:
  - bruhat-偏序的-hasse-图构造
---

# Bruhat 偏序的 Hasse 图构造

`bruhat_hasse` 基于只读块拓扑构造 Bruhat 偏序的 Hasse 图，输出的第 `z` 行列出元素 `z` 的直接下邻。算法根据下降类型选取 cross 或逆 Cayley 像，并在相应分支中通过 `insert_ascents` 补充上升像。^[block-access-modifier.md:27-35]

## 拓扑接口与构造前提

[[BlockTopology 只读块拓扑接口|BlockTopology]] 是 KL 实现消费的最小只读块接口。它采用密封设计，禁止下游 crate 实现，因为 KL 算法依赖方法签名之外的结构不变量：秩不超过 32、元素按非降长度排序、格子存在、链接目标小于块大小。KL 构造在递归前校验这些条件。^[block-access-modifier.md:17-25]

接口区分两层无效性：外层 `None` 表示无效的元素／生成元格子；`cayley` 和 `inverse_cayley` 的内层 `None` 表示相应 Cayley 链未定义。^[block-access-modifier.md:18-20]

## 逐行构造规则

构造第 `z` 行时，算法首先寻找首个严格 good descent，其类型为 `ComplexDescent` 或 `RealTypeI`；只有找不到这两类下降时，才进入遍历所有生成元的回退分支。^[block-access-modifier.md:27-32]

对于 `ComplexDescent`，插入 cross 像，并对该像对应的行调用 `insert_ascents`。^[block-access-modifier.md:28-29]

对于 `RealTypeI`，插入逆 Cayley 的第一像（必需）和第二像（可选），然后**仅对第一像对应的行**调用 `insert_ascents`。第二像即使存在，也不用于这一步上升扩展。^[block-access-modifier.md:29-31]

若不存在严格 good descent，则遍历所有生成元；对每个 `RealTypeII`，只插入逆 Cayley 的第一分量，不取第二分量。^[block-access-modifier.md:31-32]

## 上升像的补充

`insert_ascents` 按上升类型选取链接：`ComplexAscent` 使用 cross 像，`ImaginaryTypeI` 使用 Cayley 第一分量，`ImaginaryTypeII` 使用 Cayley 两个分量。相关链接见 [[Cross、Cayley 与逆 Cayley 链接]]。^[block-access-modifier.md:32-34]

## 具体块类型的适配

`BlockGraph` 基本通过同名方法委托实现拓扑接口，其中 `descent` 委托给固有方法 `descent_value`。`PartialBlock` 的 `cross`、`cayley` 与 `inverse_cayley` 则需要交换参数顺序，例如调用 `PartialBlock::cross(self, generator, element)`。^[block-access-modifier.md:37-39]

`PartialBlock` 还根据下降状态限制 Cayley 链：下降时，`cayley` 返回 `Some((None, None))`；非下降时，`inverse_cayley` 返回同样的结果，其余情况委托给固有方法。因此，“因下降状态而没有相应链”编码为内层无值，而不是外层 `None`，详见 [[Cayley 链的双层无效性与 PartialBlock 适配]]。^[block-access-modifier.md:39-42]

## 失败行为与证据边界

`bruhat_hasse` 所在文件没有 `Result` 错误返回；多处 `expect`（如 `"complex descent cross"`）与 `hasse[sz]` 直接索引依赖构造不变量。来源明确指出这些位置没有相应的运行时防护。^[block-access-modifier.md:34-35, block-access-modifier.md:102-103]

`block_access.rs` 没有测试模块；`BlockDescent` 的完整变体集与 `is_descent()` 定义也不在本来源包的覆盖范围内。^[block-access-modifier.md:89-89, block-access-modifier.md:100-101]

本来源属于结构性源码阅读，不构成数学或正确性验收。上游定位 `blocks.cpp:1576-1656` 转录自代码注释，未核对上游文件字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[block-access-modifier.md:9-13, block-access-modifier.md:27-27, block-access-modifier.md:114-115]

## Sources

- [只读块拓扑与块修正子（block_access.rs / block_modifier.rs）](../../sources/block-access-modifier.md)
