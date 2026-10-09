---
title: Cayley 链的双层无效性与 PartialBlock 适配
summary: 外层 None 表示无效格子，内层 None 表示未定义链；PartialBlock 适配交换生成元与元素参数，并按下降状态将不可用的 Cayley 链编码为 Some((None, None))。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:06.618Z"
updatedAt: "2026-10-09T14:40:06.618Z"
tags:
  - Rust设计
  - Cayley变换
  - 块拓扑
aliases:
  - cayley-链的双层无效性与-partialblock-适配
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cayley 链的双层无效性与 PartialBlock 适配

[[BlockTopology 只读块拓扑接口|BlockTopology]] 是 KL 实现消费的最小只读块接口。它通过两层 `Option` 区分“无效的元素／生成元格子”与“有效格子上未定义的 Cayley 链”，使调用方能够分别处理索引无效和链不存在的情形。^[block-access-modifier.md:17-25]

## 双层无效性

对于 `cayley` 和 `inverse_cayley`，外层 `None` 表示 element／generator 格子无效；外层有效时，返回值中各分量的内层 `None` 表示对应 Cayley 链未定义。因此，`Some((None, None))` 表示有效格子没有可用的 Cayley 链，并不等同于外层 `None`。^[block-access-modifier.md:17-20, block-access-modifier.md:37-42]

## PartialBlock 的适配规则

[[公共块的构造与元素编号（PartialBlock）|PartialBlock]] 实现接口时，需要调整参数顺序：`cross`、`cayley` 和 `inverse_cayley` 委托到固有方法时交换 element 与 generator，例如调用 `PartialBlock::cross(self, generator, element)`。相比之下，`BlockGraph` 基本采用同名委托，其中接口的 `descent` 对应固有方法 `descent_value`。^[block-access-modifier.md:37-39]

适配层还依据 `is_descent` 对 Cayley 查询进行门控。查询 `cayley` 时，下降状态返回 `Some((None, None))`，非下降状态才委托；查询 `inverse_cayley` 时恰好相反，非下降状态返回 `Some((None, None))`，下降状态才委托。因此，由下降状态排除的链始终编码在内层，而不是被当作无效格子。^[block-access-modifier.md:39-42]

## 结构不变量与消费方

`BlockTopology` 采用密封 trait 模式，允许 crate 内的不变量测试实现接口，但禁止下游 crate 实现。原因是 KL 算法依赖方法签名之外的结构条件：rank 不超过 32、元素按非降长度排序、格子存在，以及链接目标小于块大小；KL 构造会在递归前检查这些条件。^[block-access-modifier.md:20-25]

双层无效性并不意味着消费方会对所有缺失链接进行可恢复处理。例如，[[Bruhat 偏序的 Hasse 图构造|bruhat_hasse]] 在 `RealTypeI` 分支要求逆 Cayley 的第一像存在，而第二像可选；在 `RealTypeII` 分支只取逆 Cayley 的第一分量。该实现使用多处 `expect` 和直接索引，依赖构造不变量，没有相应的运行时防护。^[block-access-modifier.md:27-35, block-access-modifier.md:102-103]

## 证据边界

本说明依据对 `block_access.rs` 的结构性阅读，不构成数学或正确性验收。该文件没有测试模块；`BlockDescent` 的完整变体集及 `is_descent()` 定义也不在本来源包的覆盖范围内，相关状态语义可结合 [[BlockDescent 八值状态体系]] 阅读。^[block-access-modifier.md:9-13, block-access-modifier.md:89-89, block-access-modifier.md:99-101]

## Sources

- [block-access-modifier.md](block-access-modifier.md) — 只读块拓扑与块修正子（block_access.rs / block_modifier.rs）
