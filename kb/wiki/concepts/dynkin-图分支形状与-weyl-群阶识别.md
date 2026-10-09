---
title: Dynkin 图分支形状与 Weyl 群阶识别
summary: component_order 按边重数、节点度数和分支长度识别 A、B/C、D、E、F、G 型的群阶，其中 B/C 同阶而无需区分取向。
sources:
  - weyl-size-presentation.md
kind: concept
createdAt: "2026-10-09T15:18:23.673Z"
updatedAt: "2026-10-09T15:18:23.673Z"
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Dynkin 图分支形状与 Weyl 群阶识别

`weyl_size.rs` 通过 Cartan 矩阵的连通分量、边重数与分支形状计算 Weyl 群阶。入口 `weyl_order_of_cartan` 为 `pub(crate)`；其用途是支持 twisted 共轭轨道大小与阶商公式 `|W| / (|W_im| × |W_re| × |W_cx|)` 的核对。由于只需要群阶，B/C 取向无需区分：两者的阶均为 \(2^n n!\)。^[weyl-size-presentation.md:17-21]

## 分量拆分与精确算术

算法先逐行检查矩阵是否方形，否则返回 `NonSquareCartan`；随后沿非零非对角链接进行 BFS，拆出连通分量，并将各分量的 `component_order` 相乘。零对角的环面行、列不作为种子，也不满足邻居条件，其贡献为 1。群阶使用精确 `Integer` 算术，因为在 crate 的动态秩范围内，分量乘积可能超出 `u128`。相关流程见 [[基于 Cartan 矩阵识别的 Weyl 群阶计算]]。^[weyl-size-presentation.md:21-25]

## 按边重数和分支形状识别

`component_order` 按最大边重数 \(m=\max(C_{ij}C_{ji})\) 分派；乘积使用 `checked_mul`，溢出返回 `ArithmeticOverflow`。下表概括各识别分支，其中 \(n\) 为分量秩。^[weyl-size-presentation.md:27-33]

| 最大边重数 | 分量形状或条件 | 返回的群阶 |
|---|---|---|
| \(m=3\) | 秩为 2，识别为 G₂；否则拒绝 | \(12\) |
| \(m=2\) | 度大于 2 时拒绝；秩为 4 且双键连接两个内节点时识别为 F₄ | \(1152\) |
| \(m=2\) | 其余通过检查的情形，按 B/C 同阶处理 | \(2^n n!\) |
| \(m=1\) | 度大于 3 或存在多个分叉时拒绝；无分叉时按 Aₙ 处理 | \((n+1)!\) |
| \(m=1\) | 单分叉，排序后的分支长度为 \([1,1,\_]\)，识别为 Dₙ | \(2^n n!/2\) |
| \(m=1\) | 分支长度为 \([1,2,2]\)，识别为 E₆ | \(51840\) |
| \(m=1\) | 分支长度为 \([1,2,3]\)，识别为 E₇ | \(2903040\) |
| \(m=1\) | 分支长度为 \([1,2,4]\)，识别为 E₈ | \(696729600\) |

单分叉情况下，`branch_lengths` 从唯一的度为 3 的节点沿各分支向外行走，再按排序后的长度匹配上述模式；其他分支长度组合被拒绝。这一识别过程以图形状为核心，相关主题见 [[基于图结构的 Dynkin 单分量分类]]。^[weyl-size-presentation.md:30-34]

## 输入校验边界

该实现没有环检测；来源中的阅读观察指出，含环输入可能使遍历不终止，而非返回错误。此外，单节点分量对任何非零对角值都返回 2；符号不一致、乘积为负的非对角项仍计入度数，却不会提高初始为 1 的 `max_multiplicity`，因而被按单连通处理；秩大于 4 的双键链不检查双键位置。理解这些行为时应同时参照 [[Cartan 类型识别的输入校验边界]]。^[weyl-size-presentation.md:33-38]

## 测试与证据范围

已列出的测试锚点包括 A₄ 的阶 120、B₅/C₅ 的阶 3840、D₅ 的阶 1920，以及 G₂、F₄、E₆；另有夹一行环面的 A₁×A₁，其阶为 4，以及阶为 1 的空系统。所有错误分支、E₇/E₈ 分支与秩为 4 的非 F₄ 分支（B₄/C₄）均未被这些测试覆盖，参见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[weyl-size-presentation.md:40-41, weyl-size-presentation.md:72-74]

本页依据结构性源码阅读材料，不构成数学或正确性验收。来源未核对所引上游文件的字节；此次知识维护也未执行 Atlas、Cargo、测试或 benchmark，因此不能将测试锚点理解为本次运行通过的结果。^[weyl-size-presentation.md:9-13, weyl-size-presentation.md:72-74, weyl-size-presentation.md:80-84]

## Sources

- [weyl-size-presentation.md](weyl-size-presentation.md) — Weyl 群阶识别与实形展示层（weyl_size.rs / presentation.rs）
