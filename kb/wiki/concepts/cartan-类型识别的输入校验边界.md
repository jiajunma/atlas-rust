---
title: Cartan 类型识别的输入校验边界
summary: 源码阅读指出识别器未完整验证 Cartan 合法性：接受任意非零单节点对角值、可能将异号边视为单键、不检查高秩双键位置，且分支遍历缺少环检测。
sources:
  - weyl-size-presentation.md
kind: concept
createdAt: "2026-10-09T15:18:28.816Z"
updatedAt: "2026-10-09T15:18:28.816Z"
tags:
  - 输入校验
  - 算法边界
  - 源码阅读
aliases:
  - cartan-类型识别的输入校验边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cartan 类型识别的输入校验边界

`weyl_order_of_cartan` 通过连通分量与 Dynkin 分支形状识别 Cartan 类型，计算 Weyl 群阶，供 twisted 共轭轨道的阶商核对使用。它只需要群阶，因此不区分阶同为 \(2^n n!\) 的 B/C 取向；其现有检查不能视为完整的 Cartan 矩阵合法性验证。^[weyl-size-presentation.md:17-38]

## 入口检查与分量划分

入口逐行检查矩阵是否为方形，不满足时返回 `NonSquareCartan`。随后按非零非对角链接进行 BFS，拆分连通分量，再将各分量的阶相乘。零对角行、列作为环面因子，贡献阶 1，不作为搜索种子，也不满足邻居条件。^[weyl-size-presentation.md:23-25]

群阶及分量乘积使用精确 `Integer` 算术，因为在 crate 的动态秩范围内，分量乘积可能超出 `u128`。边重数 \(m=C_{ij}C_{ji}\) 则通过 `checked_mul` 计算，溢出返回 `ArithmeticOverflow`；这是分量识别中唯一的受检算术操作。^[weyl-size-presentation.md:21-28]

## 分支形状检查

`component_order` 按最大边重数分派。最大重数为 3 时，仅接受秩 2，返回 G2 的阶 12；最大重数为 2 时，拒绝度数大于 2 的节点，秩 4 且双键位于两个内节点之间时返回 F4 的阶 1152，其余按 B/C 链的公式计算。^[weyl-size-presentation.md:27-30]

最大重数为 1 时，拒绝度数大于 3 或存在多个分叉的情况。无分叉时按 A 型计算；只有一个分叉时，按排序后的分支长度识别 D 型及 E6、E7、E8，其余形状拒绝。具体识别规则可参见 [[Dynkin 图分支形状与 Weyl 群阶识别]]。^[weyl-size-presentation.md:30-34]

## 未覆盖的合法性条件

单节点分量对任何非零对角值都返回 2，因此该路径没有验证对角值必须为 2。符号不一致的非对角元素对若乘积为负，仍计入节点度数，但不会提升初值为 1 的 `max_multiplicity`，因而被当作单键连接处理。^[weyl-size-presentation.md:36-38]

对于秩大于 4 的双键链，实现不检查双键位置。这意味着仅凭该函数返回 B/C 阶公式，不能确认输入的双键位置符合对应有限型的形状要求。^[weyl-size-presentation.md:38-38]

`branch_lengths` 从唯一的度 3 节点向外行走，没有环检测。来源将“含环输入可能不终止，而非返回错误”列为源码阅读观察，因此不能假定所有非法输入都会以可恢复错误结束。^[weyl-size-presentation.md:33-34]

## 测试与证据边界

现有测试锚点覆盖 A4、B5、C5、D5、G2、F4、E6，以及夹有一行环面的 A1×A1 和空系统。所有错误分支、E7/E8 分支、秩 4 的非 F4 分支 B4/C4 均无测试；因此这些正常输入测试不能证明输入校验完整。相关背景见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[weyl-size-presentation.md:40-41, weyl-size-presentation.md:72-74]

本页依据结构性源码阅读材料，不代表数学或正确性验收。来源未核对所引上游代码的字节，也未在本次知识维护中执行 Atlas、Cargo、测试或 benchmark；上述缺口应理解为阅读发现，而非运行验证结果。^[weyl-size-presentation.md:9-13, weyl-size-presentation.md:72-84]

## Sources

- [weyl-size-presentation.md](../../sources/weyl-size-presentation.md)
