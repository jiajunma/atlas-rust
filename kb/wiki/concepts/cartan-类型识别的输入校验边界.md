---
title: Cartan 类型识别的输入校验边界
summary: 识别器接受任意非零单节点对角值，可能将异号边按单键处理，且不检查高秩双键位置，分支遍历缺少环检测。
sources:
  - weyl-size-presentation.md
kind: concept
createdAt: "2026-10-09T15:18:28.816Z"
updatedAt: "2026-10-09T22:54:37.941Z"
tags:
  - 输入校验
  - Cartan矩阵
  - 算法边界
aliases:
  - cartan-类型识别的输入校验边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cartan 类型识别的输入校验边界
summary: Weyl 群阶识别器检查矩阵形状和部分分支条件，但未完整校验对角值、边符号及高秩双键位置，分支遍历也缺少环检测。
sources:
  - weyl-size-presentation.md
kind: concept
tags:
  - 输入校验
  - Cartan矩阵
  - 算法边界
aliases:
  - cartan-类型识别的输入校验边界
---

# Cartan 类型识别的输入校验边界

`weyl_order_of_cartan` 是 crate 内部的 Weyl 群阶计算入口，通过连通分量拆分与分支形状分析求阶，服务于 twisted 共轭轨道大小的阶商核对。由于只需要群阶，它不区分阶同为 \(2^n n!\) 的 B/C 取向。来源记录了该识别器的若干输入校验缺口；算法背景见 [[基于 Cartan 矩阵识别的 Weyl 群阶计算]]。^[weyl-size-presentation.md:17-38]

## 入口检查与算术边界

入口逐行检查矩阵是否为方形，不满足时返回 `NonSquareCartan`；随后按非零非对角链接进行 BFS，拆分连通分量，并将各分量的阶相乘。零对角行、列作为环面因子，贡献阶 1，既不作为搜索种子，也不满足邻居条件。^[weyl-size-presentation.md:23-25]

群阶使用精确 `Integer` 算术，因为分量乘积在 crate 的动态秩范围内可超出 `u128`。`component_order` 使用 `checked_mul` 计算边重数 \(C_{ij}C_{ji}\)，溢出返回 `ArithmeticOverflow`；这是该函数中唯一的受检算术操作。^[weyl-size-presentation.md:21-28]

## 已实现的形状检查

`component_order` 按最大边重数分派。最大重数为 3 时，仅接受秩 2，并返回 G2 的阶 12。最大重数为 2 时，拒绝度数大于 2 的节点；秩 4 且双键位于两个内节点之间时返回 F4 的阶 1152，其余按 B/C 链的公式 \(2^n n!\) 计算。^[weyl-size-presentation.md:27-30]

最大重数为 1 时，拒绝度数大于 3 或存在多个分叉的情况。无分叉时按 A 型返回 \((n+1)!\)；单叉时按排序后的分支长度匹配：\([1,1,\_]\) 对应 D 型，\([1,2,2]\)、\([1,2,3]\)、\([1,2,4]\) 分别对应 E6、E7、E8，其余形状拒绝。详细规则见 [[Dynkin 图分支形状与 Weyl 群阶识别]]。^[weyl-size-presentation.md:30-34]

## 已知校验缺口

**单节点对角值。** 单节点分量对任何非零对角值都返回 2，没有将接受范围限制为某个特定非零对角值。^[weyl-size-presentation.md:36-38]

**非对角元素的符号。** 符号不一致的非对角元素对若乘积为负，仍计入节点度数，但不会提升初值为 1、只取最大值的 `max_multiplicity`，因而被当作单键连接处理。^[weyl-size-presentation.md:36-38]

**高秩双键位置。** 对于秩大于 4 的双键链，实现不检查双键位置；秩 4 的 F4 位置判定不能理解为对所有秩都实施了双键位置校验。^[weyl-size-presentation.md:27-30, weyl-size-presentation.md:38-38]

**含环输入的终止性。** `branch_lengths` 从唯一的度 3 节点向外行走，没有环检测。来源将“含环输入可能不终止，而非报错”明确标为源码阅读观察。^[weyl-size-presentation.md:33-34]

据这些缺口可推知，成功返回群阶不能单独证明输入通过了完整的 Cartan 合法性验证，也不能假定所有非法输入都会以错误返回结束。

## 测试与证据边界

已有测试锚点包括 A4=120、B5=C5=3840、D5=1920、G2=12、F4=1152、E6=51840，以及夹有一行环面的 A1×A1=4 和空系统=1。所有错误分支、E7/E8 分支及秩 4 的非 F4 分支 B4/C4 均无测试，详见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[weyl-size-presentation.md:40-41, weyl-size-presentation.md:72-74]

本页依据结构性源码阅读，不代表数学或正确性验收。来源中的上游引用仅转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。上述校验缺口的证据性质是阅读发现，而非本次运行验证的失败案例。^[weyl-size-presentation.md:9-13, weyl-size-presentation.md:80-84]

## Sources

- [weyl-size-presentation.md](../../sources/weyl-size-presentation.md) — Weyl 群阶识别与实形展示层（weyl_size.rs / presentation.rs）。
