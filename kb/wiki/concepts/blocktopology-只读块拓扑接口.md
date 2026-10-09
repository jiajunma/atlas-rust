---
title: BlockTopology 只读块拓扑接口
summary: 以密封 trait 提供 KL 所需的最小只读块表面，并依赖秩、长度排序、格子存在性和链接范围等结构不变量；支持引用及 Arc 包装的动态 trait 对象。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:10.750Z"
updatedAt: "2026-10-09T14:40:10.750Z"
tags:
  - Rust设计
  - 块拓扑
  - KL算法
aliases:
  - blocktopology-只读块拓扑接口
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# BlockTopology 只读块拓扑接口

`BlockTopology` 是 KL 实现消费的最小只读块接口，对应上游 `klsupport::KLSupport` 仅消费 `const Block_base&` 的边界。它在 `block_access.rs` 中定义，由 `BlockGraph` 与 `PartialBlock` 实现。^[block-access-modifier.md:15-25, block-access-modifier.md:37-42]

## 密封边界与结构不变量

该接口通过 `pub(crate) mod sealed` 与 `BlockTopology: sealed::Sealed` 实施密封：crate 内的不变量测试可以实现该 trait，下游 crate 则不能自行实现。原因是 KL 算法依赖方法签名之外的结构不变量：`rank ≤ 32`、元素按非降长度排序、格子存在，以及链接目标索引小于 `size`。KL 构造会在递归前校验这些条件，相关主题见 [[KlSupport 的拓扑构造门控]]。^[block-access-modifier.md:17-25]

`&T` 与 `Arc<T>` 的 blanket impl 均使用 `?Sized` 约束，因此引用和共享指针形式的 trait object（`&dyn`、`Arc<dyn>`）也满足接口约束。^[block-access-modifier.md:24-25]

## 两层无效性语义

接口区分“无效的查询格子”与“有效格子上未定义的链”：外层 `None` 表示无效的 element/generator 格子；`cayley` 与 `inverse_cayley` 返回值中的内层 `None` 表示对应的 Cayley 链未定义。两者不可混同，详见 [[Cayley 链的双层无效性与 PartialBlock 适配]]。^[block-access-modifier.md:18-20, block-access-modifier.md:37-42]

## 具体实现与适配

`BlockGraph` 基本将接口调用委托给同名固有方法，其中 `descent` 对应固有方法 `descent_value`。`PartialBlock` 的 `cross`、`cayley` 与 `inverse_cayley` 适配需要交换参数顺序，例如调用 `PartialBlock::cross(self, generator, element)`。^[block-access-modifier.md:37-39]

`PartialBlock` 还按下降状态限制链查询：`cayley` 在 `is_descent` 为真时返回 `Some((None, None))`，否则委托；`inverse_cayley` 则在非下降时返回 `Some((None, None))`，否则委托。因此，因下降状态而不存在的链属于内层无定义，不编码为外层 `None`。^[block-access-modifier.md:39-42]

## Bruhat Hasse 图构造

`bruhat_hasse` 的第 `z` 行列出元素 `z` 的直接下邻。算法首先寻找首个严格 good descent，其类型为 `ComplexDescent` 或 `RealTypeI`，并据此构造该行；相关概念见 [[Bruhat 偏序的 Hasse 图构造]]。^[block-access-modifier.md:27-31]

遇到 `ComplexDescent` 时，算法插入 cross 像，并对该像对应的行执行 `insert_ascents`。遇到 `RealTypeI` 时，插入 inverse-Cayley 的第一像（必需）与第二像（可选），但只对第一像对应的行执行 `insert_ascents`。若没有严格 good descent，则遍历所有生成元，对每个 `RealTypeII` 仅插入 inverse-Cayley 的第一分量。^[block-access-modifier.md:28-32]

`insert_ascents` 按类型加入上升像：`ComplexAscent` 加入 cross 像，`ImaginaryTypeI` 加入 cayley 第一分量，`ImaginaryTypeII` 加入 cayley 两个分量。^[block-access-modifier.md:32-34]

## 错误处理与证据边界

`block_access.rs` 不使用 `Result`；Hasse 图构造中的多处 `expect` 和 `hasse[sz]` 直接索引依赖构造不变量，未提供运行时防护。该文件没有测试模块；`BlockDescent` 的完整变体集与 `is_descent()` 定义也不在本来源包的覆盖范围内，可参阅 [[BlockDescent 八值状态体系]]。^[block-access-modifier.md:34-35, block-access-modifier.md:89-89, block-access-modifier.md:100-103]

本页依据结构性阅读材料，不代表数学或正确性验收。来源中的上游行号转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[block-access-modifier.md:9-13, block-access-modifier.md:99-99, block-access-modifier.md:109-115]

## Sources

- [只读块拓扑与块修正子（block_access.rs / block_modifier.rs）](block-access-modifier.md)
