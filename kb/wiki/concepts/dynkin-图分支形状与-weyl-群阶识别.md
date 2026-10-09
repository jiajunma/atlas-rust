---
title: Dynkin 图分支形状与 Weyl 群阶识别
summary: component_order 按边重数、顶点度数和分支长度识别经典型及例外型群阶，B/C 型同阶使识别无需区分取向。
sources:
  - weyl-size-presentation.md
kind: concept
createdAt: "2026-10-09T15:18:23.673Z"
updatedAt: "2026-10-09T22:54:33.916Z"
tags:
  - Dynkin图
  - Weyl群
  - 类型识别
aliases:
  - dynkin-图分支形状与-weyl-群阶识别
  - D图W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Dynkin 图分支形状与 Weyl 群阶识别
summary: component_order 根据边重数、节点度数和分支长度识别 Weyl 群阶；B/C 同阶，无需区分取向，但实现不提供完整的 Cartan 输入校验。
sources:
  - weyl-size-presentation.md
kind: concept
tags:
  - Dynkin图
  - Weyl群
  - 类型识别
aliases:
  - dynkin-图分支形状与-weyl-群阶识别
---

# Dynkin 图分支形状与 Weyl 群阶识别

`weyl_size.rs` 通过 Cartan 矩阵的连通分量、边重数与分支形状计算 Weyl 群阶。其唯一入口 `weyl_order_of_cartan` 为 `pub(crate)`；来源说明，其用途是支持 twisted 共轭轨道大小与阶商公式 \(|W|/(|W_{\mathrm{im}}|\times|W_{\mathrm{re}}|\times|W_{\mathrm{cx}}|)\) 的核对。由于只需要群阶，B/C 型无需区分取向：两者的阶均为 \(2^n n!\)。^[weyl-size-presentation.md:17-21]

## 分量拆分与精确算术

入口先逐行检查矩阵是否方形，否则返回 `NonSquareCartan`；随后沿非零非对角链接进行 BFS，拆分连通分量，并将各分量的 `component_order` 相乘。零对角的环面行、列不作为搜索种子，也不满足邻居条件，贡献的阶为 1。群阶使用精确 `Integer` 算术，因为分量乘积在 crate 的动态秩范围内可以超出 `u128`。完整流程参见 [[基于 Cartan 矩阵识别的 Weyl 群阶计算]]。^[weyl-size-presentation.md:21-25]

## 边重数与分支形状

`component_order` 按最大边重数 \(m=\max(C_{ij}C_{ji})\) 分派，实际累积变量 `max_multiplicity` 的初值为 1。条目乘积使用 `checked_mul`，溢出返回 `ArithmeticOverflow`。下表中的 \(n\) 表示分量秩，概括来源描述的分派规则。^[weyl-size-presentation.md:27-38]

| 最大边重数 | 形状与检查条件 | 返回的群阶 |
|---|---|---|
| \(m=3\) | 秩为 2，按 G₂ 处理；否则拒绝 | \(12\) |
| \(m=2\) | 度大于 2 时拒绝；秩为 4 且双键连接两个内节点时，按 F₄ 处理 | \(1152\) |
| \(m=2\) | 其余通过检查的情形，采用 B/C 型阶公式 | \(2^n n!\) |
| \(m=1\) | 度大于 3 或存在多个分叉时拒绝；无分叉时按 Aₙ 处理 | \((n+1)!\) |
| \(m=1\) | 单分叉，排序后的分支长度为 \([1,1,\_]\)，按 Dₙ 处理 | \(2^n n!/2\) |
| \(m=1\) | 单分叉，排序后的分支长度为 \([1,2,2]\)，按 E₆ 处理 | \(51840\) |
| \(m=1\) | 单分叉，排序后的分支长度为 \([1,2,3]\)，按 E₇ 处理 | \(2903040\) |
| \(m=1\) | 单分叉，排序后的分支长度为 \([1,2,4]\)，按 E₈ 处理 | \(696729600\) |

上述单分叉规则由 `branch_lengths` 支持：它从唯一的度为 3 的节点沿各分支向外行走，再按排序后的长度匹配模式；其他分支长度组合被拒绝。相关主题见 [[基于图结构的 Dynkin 单分量分类]]。^[weyl-size-presentation.md:27-34]

## 输入校验边界

来源的结构性阅读指出，实现没有环检测，含环输入可能导致遍历不终止，而非返回错误。单节点分量对任何非零对角值都返回 2；符号不一致、乘积为负的非对角项仍计入度数，却不会提高初始为 1 的 `max_multiplicity`，因此按单连通情形处理。此外，秩大于 4 的双键链不检查双键位置。这些限制见 [[Cartan 类型识别的输入校验边界]]。^[weyl-size-presentation.md:33-38]

## 测试与证据范围

来源列出的测试锚点包括 A₄ 的阶 120、B₅/C₅ 的阶 3840、D₅ 的阶 1920，以及 G₂ 的阶 12、F₄ 的阶 1152、E₆ 的阶 51840；另有夹一行环面的 A₁×A₁，其阶为 4，以及阶为 1 的空系统。^[weyl-size-presentation.md:40-41]

所有错误分支、E₇/E₈ 分支以及秩为 4 的非 F₄ 分支（B₄/C₄）均无测试覆盖。`factorial` 虽返回 `Result`，内部没有可失败操作，该签名属于预留。参见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[weyl-size-presentation.md:72-74]

本页依据结构性源码阅读材料，不构成数学或正确性验收。来源中的上游行号仅转录自代码注释，未核对上游字节；此次知识维护也未执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点不能视为本次运行通过的结果。^[weyl-size-presentation.md:9-13, weyl-size-presentation.md:72-74, weyl-size-presentation.md:80-84]

## Sources

- [weyl-size-presentation.md](../../sources/weyl-size-presentation.md) — Weyl 群阶识别与实形展示层（weyl_size.rs / presentation.rs）。
