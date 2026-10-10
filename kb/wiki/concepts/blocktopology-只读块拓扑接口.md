---
title: BlockTopology 只读块拓扑接口
summary: 密封 trait 为 KL 提供最小只读块接口，支持 &T 与 Arc<T>，并依赖秩容量、长度排序和链接合法性等构造不变量。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:10.750Z"
updatedAt: "2026-10-10T00:26:54.850Z"
tags:
  - rust
  - KL
  - 块拓扑
aliases:
  - blocktopology-只读块拓扑接口
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: BlockTopology 只读块拓扑接口
summary: 密封 trait 为 KL 提供最小只读块接口，区分无效查询与未定义的 Cayley 链，并依赖递归前校验的结构不变量。
sources:
  - block-access-modifier.md
kind: concept
tags:
  - Rust设计
  - 块拓扑
  - KL算法
aliases:
  - blocktopology-只读块拓扑接口
provenanceState: extracted
---

# BlockTopology 只读块拓扑接口

`BlockTopology` 定义于 `block_access.rs`，是 KL 实现消费的最小只读块接口，由 `BlockGraph` 与 `PartialBlock` 实现。其上游对应边界是 `klsupport::KLSupport` 仅消费 `const Block_base&`。^[block-access-modifier.md:15-25, block-access-modifier.md:37-42]

## 密封边界与结构不变量

接口通过 `pub(crate) mod sealed` 与 `BlockTopology: sealed::Sealed` 实施密封：允许 crate 内的不变量测试实现该 trait，禁止下游 crate 自行实现。KL 算法依赖方法签名之外的结构不变量，包括 `rank ≤ 32`、元素按非降长度排序、格子存在，以及链接目标索引小于 `size`。KL 构造在递归前校验这些条件，相关主题见 [[KlSupport 的拓扑构造门控]]。^[block-access-modifier.md:17-25]

`&T` 与 `Arc<T>` 的 blanket impl 均带有 `?Sized` 约束，因此 `&dyn BlockTopology` 与 `Arc<dyn BlockTopology>` 也满足接口约束。^[block-access-modifier.md:24-25]

## 两层无效性语义

接口区分查询格子无效与 Cayley 链未定义：外层 `None` 表示无效的 element/generator 格子；`cayley` 与 `inverse_cayley` 返回值中的内层 `None` 表示对应的链未定义。因此，有效格子可以返回 `Some((None, None))`。详见 [[Cayley 链的双层无效性与 PartialBlock 适配]]。^[block-access-modifier.md:18-20, block-access-modifier.md:37-42]

## 具体实现与适配

`BlockGraph` 基本将接口调用委托给同名固有方法，其中 `descent` 委托给 `descent_value`。`PartialBlock` 的 `cross`、`cayley` 与 `inverse_cayley` 适配需要交换参数顺序，例如调用 `PartialBlock::cross(self, generator, element)`。^[block-access-modifier.md:37-39]

`PartialBlock` 还按下降状态控制链查询：`cayley` 在 `is_descent` 为真时返回 `Some((None, None))`，否则委托；`inverse_cayley` 在非下降时返回 `Some((None, None))`，否则委托。因下降状态而不存在的链编码为内层未定义，而非外层查询无效。^[block-access-modifier.md:39-42]

## Bruhat Hasse 图构造

`bruhat_hasse` 的第 `z` 行列出元素 `z` 的直接下邻。算法先寻找首个 strict good descent，即 `ComplexDescent` 或 `RealTypeI`，再据此构造该行；相关主题见 [[Bruhat 偏序的 Hasse 图构造]]。^[block-access-modifier.md:27-31]

遇到 `ComplexDescent` 时，算法插入 cross 像，并对该像对应的行执行 `insert_ascents`。遇到 `RealTypeI` 时，插入 inverse-Cayley 的第一像（必需）与第二像（可选），但只对第一像对应的行执行 `insert_ascents`。若没有 strict good descent，则遍历所有生成元，对每个 `RealTypeII` 仅插入 inverse-Cayley 的第一分量，不取第二分量。^[block-access-modifier.md:28-32]

`insert_ascents` 按类型加入上升像：`ComplexAscent` 加入 cross 像，`ImaginaryTypeI` 加入 cayley 第一分量，`ImaginaryTypeII` 加入 cayley 两个分量。^[block-access-modifier.md:32-34]

## 错误处理与证据边界

`block_access.rs` 不使用 `Result`。Hasse 图构造中的多处 `expect` 与 `hasse[sz]` 直接索引依赖构造不变量，没有额外的运行时防护。该文件没有测试模块；`BlockDescent` 的完整变体集与 `is_descent()` 定义也不在本来源包的覆盖范围内，可结合 [[BlockDescent 八值状态体系]] 阅读。^[block-access-modifier.md:34-35, block-access-modifier.md:89-89, block-access-modifier.md:100-103]

本页依据结构性源码阅读材料，不代表数学或正确性验收。来源中的上游行号转录自代码注释，未核对上游字节；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[block-access-modifier.md:9-13, block-access-modifier.md:99-99, block-access-modifier.md:109-115]

## Sources

- [只读块拓扑与块修正子（block_access.rs / block_modifier.rs）](../../sources/block-access-modifier.md)
