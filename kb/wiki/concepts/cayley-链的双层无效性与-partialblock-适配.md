---
title: Cayley 链的双层无效性与 PartialBlock 适配
summary: 外层 None 表示无效格子，内层 None 表示未定义链；PartialBlock 适配交换参数顺序，并按下降状态门控直接与逆 Cayley 链。
sources:
  - block-access-modifier.md
kind: concept
createdAt: "2026-10-09T14:40:06.618Z"
updatedAt: "2026-10-10T00:26:59.037Z"
tags:
  - 块拓扑
  - Cayley
  - rust
aliases:
  - cayley-链的双层无效性与-partialblock-适配
  - C链P适
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Cayley 链的双层无效性与 PartialBlock 适配
summary: 外层 None 表示无效的元素／生成元格子，内层 None 表示未定义的 Cayley 链；PartialBlock 通过参数交换和下降状态门控适配只读块拓扑接口。
sources:
  - block-access-modifier.md
kind: concept
tags:
  - 块拓扑
  - Cayley变换
  - 接口契约
aliases:
  - cayley-链的双层无效性与-partialblock-适配
provenanceState: extracted
---

# Cayley 链的双层无效性与 PartialBlock 适配

[[BlockTopology 只读块拓扑接口|BlockTopology]] 是 KL 实现消费的最小只读块接口。它区分“元素／生成元格子无效”与“格子有效但 Cayley 链未定义”。[[公共块的构造与元素编号（PartialBlock）|PartialBlock]] 通过交换参数顺序和按下降状态门控查询，适配这一接口契约。^[block-access-modifier.md:17-25, block-access-modifier.md:37-42]

## 双层无效性

`cayley` 和 `inverse_cayley` 的外层 `None` 表示无效的 element／generator 格子，内层 `None` 表示对应 Cayley 链未定义。`Some((None, None))` 因而表示格子有效、两个链分量均未定义，不能与外层 `None` 混同。^[block-access-modifier.md:17-20, block-access-modifier.md:39-42]

## PartialBlock 的适配规则

`PartialBlock` 的 `cross`、`cayley` 和 `inverse_cayley` 在委托到固有方法时交换元素与生成元的参数顺序，例如 `PartialBlock::cross(self, generator, element)`。另一实现者 `BlockGraph` 基本采用同名委托，其中接口方法 `descent` 对应固有方法 `descent_value`。^[block-access-modifier.md:37-39]

下降状态决定 Cayley 查询是否继续委托：`cayley` 在 `is_descent` 为真时返回 `Some((None, None))`，否则委托；`inverse_cayley` 恰好相反，在非下降状态返回 `Some((None, None))`，下降时委托。因此，因下降状态而不存在的链被编码为内层缺失，而不是外层格子无效。^[block-access-modifier.md:39-42]

## 结构不变量与消费方要求

`BlockTopology` 采用密封 trait 模式，允许 crate 内的不变量测试实现接口，禁止下游 crate 实现。KL 算法依赖方法签名之外的结构不变量：rank 不超过 32、元素按非降长度排序、格子存在，以及链接目标小于块大小。KL 构造会在递归前校验这些条件。^[block-access-modifier.md:20-25]

双层可缺失表示不意味着消费方允许任意链缺失。[[Bruhat 偏序的 Hasse 图构造|bruhat_hasse]] 在 `RealTypeI` 分支要求逆 Cayley 的第一像存在，第二像可选，并仅对第一像的行执行 `insert_ascents`；没有 strict good descent 时，对 `RealTypeII` 只取逆 Cayley 的第一分量。该实现不返回 `Result`，多处 `expect` 和直接索引依赖构造不变量，没有相应的运行时防护。^[block-access-modifier.md:27-35, block-access-modifier.md:102-103]

## 证据边界

来源对 `block_access.rs` 的说明属于结构性阅读，不构成数学或正确性验收。该文件没有测试模块；[[BlockDescent 八值状态体系|BlockDescent]] 的完整变体集与 `is_descent()` 定义位于本来源包覆盖范围之外。^[block-access-modifier.md:9-13, block-access-modifier.md:89-89, block-access-modifier.md:99-103]

来源中的上游引用仅转录自代码注释，未核对上游字节；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。上述接口与分支说明应视为源码阅读记录，而非本次运行验证结果。^[block-access-modifier.md:11-13, block-access-modifier.md:109-115]

## Sources

- [block-access-modifier.md](../../sources/block-access-modifier.md) — 只读块拓扑与块修正子（block_access.rs / block_modifier.rs）。
