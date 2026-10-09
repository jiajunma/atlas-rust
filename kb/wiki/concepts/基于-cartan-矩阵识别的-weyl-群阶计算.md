---
title: 基于 Cartan 矩阵识别的 Weyl 群阶计算
summary: weyl_order_of_cartan 通过方形检查、连通分量 BFS 拆分及分量阶乘积计算群阶，环面因子贡献 1，并使用精确 Integer 算术避免固定宽度整数溢出。
sources:
  - weyl-size-presentation.md
kind: concept
createdAt: "2026-10-09T15:18:27.616Z"
updatedAt: "2026-10-09T15:18:27.616Z"
tags:
  - Weyl群
  - Cartan矩阵
  - 精确算术
aliases:
  - 基于-cartan-矩阵识别的-weyl-群阶计算
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 基于 Cartan 矩阵识别的 Weyl 群阶计算

`weyl_size.rs` 通过识别 Cartan 矩阵的连通分量及其 Dynkin 分支形状计算 Weyl 群阶，唯一入口为 crate 内可见的 `weyl_order_of_cartan`。模块注释将其用途关联到按需 twisted 共轭划分中的轨道大小校验，所用阶商公式为 \(|W|/(|W_{\mathrm{im}}|\times|W_{\mathrm{re}}|\times|W_{\mathrm{cx}}|)\)。此处只需群阶，因此不必区分同阶的 B/C 取向。^[weyl-size-presentation.md:17-21]

## 分量拆分与精确算术

算法先逐行检查矩阵是否为方阵，不满足时返回 `NonSquareCartan`；随后沿非零非对角链接进行 BFS，拆出连通分量，并将各分量的 `component_order` 相乘。零对角行、列作为环面因子，贡献阶 1，既不作为 BFS 种子，也不满足邻居条件。阶及分量乘积使用精确 `Integer` 算术，因为 crate 的动态秩范围内可能超出 `u128`。^[weyl-size-presentation.md:21-25]

## Dynkin 类型与群阶

对每个分量，`component_order` 依据边重数 \(C_{ij}C_{ji}\) 的最大值分派，再结合秩、节点度数和分支长度识别类型。边重数乘法使用 `checked_mul`，溢出时返回 `ArithmeticOverflow`。具体分派规则如下。^[weyl-size-presentation.md:27-34]

| 最大边重数 | 分量形状或条件 | 群阶 |
|---|---|---|
| 3 | 秩 2，按 G₂ 处理；其他秩拒绝 | \(12\) |
| 2 | 节点度数大于 2 时拒绝；秩 4 且双键位于两个内节点之间，按 F₄ 处理 | \(1152\) |
| 2 | 其余通过上述检查的情形，按 B/C 同阶公式处理 | \(2^n n!\) |
| 1 | 度数大于 3 或存在多个分叉时拒绝；无分叉时按 Aₙ 处理 | \((n+1)!\) |
| 1 | 单分叉，排序分支长度为 \([1,1,\_]\)，按 Dₙ 处理 | \(2^n n!/2\) |
| 1 | 单分叉，排序分支长度为 \([1,2,2]\)，按 E₆ 处理 | \(51840\) |
| 1 | 单分叉，排序分支长度为 \([1,2,3]\)，按 E₇ 处理 | \(2903040\) |
| 1 | 单分叉，排序分支长度为 \([1,2,4]\)，按 E₈ 处理 | \(696729600\) |

其中 \(n\) 为分量的秩；单分叉的其他分支长度组合会被拒绝。`branch_lengths` 从唯一的度 3 节点向外遍历，得到用于匹配的分支长度。相关形状识别可参见 [[Dynkin 图分支形状与 Weyl 群阶识别]]。^[weyl-size-presentation.md:29-34]

## 输入校验边界

这套识别不能视为完整的 Cartan 矩阵合法性验证：单节点分量对任何非零对角值均返回 2；非对角项乘积为负时，该链接计入度数，却不会提高初值为 1 的 `max_multiplicity`，因而按单键情形处理；秩大于 4 的双键链不检查双键位置。这些都是来源中的源码阅读观察，相关主题为 [[Cartan 类型识别的输入校验边界]]。^[weyl-size-presentation.md:36-38]

`branch_lengths` 没有环检测，含环输入可能导致遍历不终止，而不是返回错误。因此，已列出的拒绝条件并不意味着任意非法输入都会得到可恢复错误。^[weyl-size-presentation.md:33-34]

## 测试覆盖与证据限制

来源列出的测试锚点包括经典型 \(A_4=120\)、\(B_5=C_5=3840\)、\(D_5=1920\)，例外型 \(G_2=12\)、\(F_4=1152\)、\(E_6=51840\)，以及夹有一行环面的 \(A_1\times A_1=4\) 和空系统阶为 1。^[weyl-size-presentation.md:40-41]

所有错误分支、E₇/E₈ 分支和秩 4 的非 F₄ 分支 B₄/C₄ 均无测试；`factorial` 虽返回 `Result`，内部没有可失败操作，该签名属于预留。可结合 [[Dynkin 分类器的测试覆盖与证据边界]] 理解这些覆盖限制。^[weyl-size-presentation.md:72-74]

该来源属于结构性源码阅读，不构成数学或正确性验收；上游引用仅从代码注释转录，未核对上游字节，本次知识维护也未运行 Atlas、Cargo、测试或 benchmark。此外，仅凭所覆盖的两个文件，无法确认 `CartanClassification` 内部是否实际调用 `weyl_order_of_cartan`。^[weyl-size-presentation.md:10-13, weyl-size-presentation.md:64-68, weyl-size-presentation.md:80-84]

## Sources

- [Weyl 群阶识别与实形展示层（weyl_size.rs / presentation.rs）](weyl-size-presentation.md)
