---
title: Bruhat 偏序的 Hasse 图构造
summary: bruhat_hasse 通过首个严格良下降的 cross 或逆 Cayley 像及上升扩展生成直接下邻，无此下降时收集 RealTypeII 的第一逆 Cayley 分量；索引和 expect 依赖构造不变量。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:19.978Z"
updatedAt: "2026-10-09T14:40:19.978Z"
tags:
  - Bruhat偏序
  - 图算法
  - 结构不变量
aliases:
  - bruhat-偏序的-hasse-图构造
  - B偏H图
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Bruhat 偏序的 Hasse 图构造

`bruhat_hasse` 基于只读块拓扑构造 Bruhat 偏序的 Hasse 图：输出的第 `z` 行列出元素 `z` 的直接下邻。算法根据生成元的下降类型选择构造分支，并通过 `insert_ascents` 补充相应的上升像。^[block-access-modifier.md:27-35]

## 拓扑接口与前提

[[BlockTopology 只读块拓扑接口|BlockTopology]] 提供算法所需的块拓扑。该接口采用密封设计，禁止下游 crate 自行实现，因为消费它的 KL 算法依赖方法签名之外的结构不变量：秩不超过 32、元素按非降长度排序、格子存在、链接目标小于块大小；KL 构造会在递归前校验这些条件。^[block-access-modifier.md:17-25]

接口区分两层无效性：外层 `None` 表示无效的元素／生成元格子；`cayley` 和 `inverse_cayley` 的内层 `None` 表示相应 Cayley 链未定义。这一区分是理解构造中“必需像”与“可选像”的基础。^[block-access-modifier.md:18-20, block-access-modifier.md:29-32]

## 逐行构造规则

构造第 `z` 行时，算法首先寻找首个严格 good descent，其类型为 `ComplexDescent` 或 `RealTypeI`。找到后，按对应分支处理；没有找到时，改为遍历所有生成元并处理 `RealTypeII`。^[block-access-modifier.md:27-32]

- **`ComplexDescent`**：插入 `cross` 像，并对该像对应的行调用 `insert_ascents`。
- **`RealTypeI`**：插入 inverse-Cayley 的第一像（必需）和第二像（可选），然后仅对第一像对应的行调用 `insert_ascents`。
- **没有严格 good descent**：遍历所有生成元，对每个 `RealTypeII` 插入 inverse-Cayley 的第一分量，不取第二分量。

其中，`RealTypeI` 的上升补充仅使用第一像的行；回退分支中的 `RealTypeII` 也仅贡献第一分量。这两处对分量的选择属于明确的算法规则。^[block-access-modifier.md:29-32]

## 上升像的补充

`insert_ascents` 按上升类型选择链接：`ComplexAscent` 使用 `cross` 像，`ImaginaryTypeI` 使用 Cayley 第一分量，`ImaginaryTypeII` 使用 Cayley 两个分量。相关链接可参见 [[Cross、Cayley 与逆 Cayley 链接]]。^[block-access-modifier.md:32-34]

## 具体块类型的适配

`BlockGraph` 基本通过同名方法委托实现拓扑接口，其中 `descent` 委托给固有方法 `descent_value`。`PartialBlock` 的 `cross`、`cayley` 与 `inverse_cayley` 则需要交换元素和生成元参数的顺序。^[block-access-modifier.md:37-39]

`PartialBlock` 还根据下降状态限制 Cayley 链：下降时，`cayley` 返回 `Some((None, None))`；非下降时，`inverse_cayley` 返回相同结果。这表示格子有效但相应链未定义，而非格子无效，详见 [[Cayley 链的双层无效性与 PartialBlock 适配]]。^[block-access-modifier.md:39-42]

## 失败行为与证据边界

`bruhat_hasse` 所在文件不使用 `Result` 作为错误返回机制；多处 `expect` 与 `hasse[sz]` 直接索引依赖构造不变量，缺少相应的运行时防护。因此，这些不变量也是理解其失败行为的必要前提。^[block-access-modifier.md:34-35, block-access-modifier.md:102-103]

本来源属于结构性源码阅读，不构成数学或正确性验收。`block_access.rs` 没有测试模块；来源中的上游 `blocks.cpp:1576-1656` 定位转录自代码注释，未核对上游文件字节，本次知识维护也未执行测试或 benchmark。^[block-access-modifier.md:9-13, block-access-modifier.md:27-27, block-access-modifier.md:89-89, block-access-modifier.md:114-115]

## Sources

- [只读块拓扑与块修正子（block_access.rs / block_modifier.rs）](block-access-modifier.md)
