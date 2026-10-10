---
title: 基于 Cartan 矩阵识别的 Weyl 群阶计算
summary: weyl_order_of_cartan 检查方形后以 BFS 拆分连通分量，使用精确 Integer 累乘各分量群阶，环面因子与空系统贡献 1。
sources:
  - weyl-size-presentation.md
kind: concept
createdAt: "2026-10-09T15:18:27.616Z"
updatedAt: "2026-10-10T00:56:52.426Z"
tags:
  - Weyl群
  - 精确算术
aliases:
  - 基于-cartan-矩阵识别的-weyl-群阶计算
  - 基C矩W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 基于 Cartan 矩阵识别的 Weyl 群阶计算
summary: weyl_order_of_cartan 通过连通分量拆分与 Dynkin 分支形状识别计算 Weyl 群阶，使用精确 Integer 算术；输入检查不构成完整的 Cartan 矩阵合法性验证。
sources:
  - weyl-size-presentation.md
kind: concept
tags:
  - Weyl群
  - Cartan矩阵
  - 精确算术
aliases:
  - 基于-cartan-矩阵识别的-weyl-群阶计算
---

# 基于 Cartan 矩阵识别的 Weyl 群阶计算

`weyl_size.rs` 的 crate 内入口 `weyl_order_of_cartan` 通过拆分 Cartan 矩阵的连通分量、识别各分量的 Dynkin 分支形状，再将分量阶相乘，计算 Weyl 群阶。由于只需要群阶，B/C 的取向不影响结果：两者均使用 \(2^n n!\)。^[weyl-size-presentation.md:17-25]

模块注释将这一计算关联到按需 twisted 共轭划分的轨道大小核对，使用阶商公式 \(|W|/(|W_{\mathrm{im}}|\times|W_{\mathrm{re}}|\times|W_{\mathrm{cx}}|)\)。但来源覆盖的两个文件未显示 `CartanClassification` 内部是否调用该函数，因此这一用途说明不能视为已核实的调用关系。^[weyl-size-presentation.md:17-20, weyl-size-presentation.md:64-68]

## 分量拆分与精确算术

算法先逐行检查矩阵是否为方阵，不满足时返回 `NonSquareCartan`；随后沿非零非对角链接执行广度优先搜索（BFS），拆分连通分量，再将各分量的 `component_order` 相乘。零对角行、列作为环面因子，贡献阶 1，既不作为 BFS 种子，也不满足邻居条件。^[weyl-size-presentation.md:23-25]

群阶及分量乘积使用精确 `Integer` 算术，因为 crate 的动态秩范围内，分量乘积可能超出 `u128`。边重数 \(C_{ij}C_{ji}\) 则使用 `checked_mul` 计算，溢出时返回 `ArithmeticOverflow`。^[weyl-size-presentation.md:21-28]

## Dynkin 分支识别

`component_order` 根据最大边重数 \(m\)、分量秩 \(n\)、节点度数和排序后的分支长度分派。下表概括其识别规则；`max_multiplicity` 初值为 1，因此对于任意输入，它不一定等于所有边乘积的数学最大值。相关主题见 [[Dynkin 图分支形状与 Weyl 群阶识别]]。^[weyl-size-presentation.md:27-38]

| 最大边重数 | 条件与识别结果 | 分量阶 |
|---|---|---|
| \(m=3\) | 秩为 2，按 G₂ 处理；其他秩拒绝 | \(12\) |
| \(m=2\) | 先拒绝度数大于 2 的情形；秩为 4 且双键位于两个内节点之间，按 F₄ 处理 | \(1152\) |
| \(m=2\) | 其余通过上述检查的情形，使用 B/C 公式 | \(2^n n!\) |
| \(m=1\) | 先拒绝度数大于 3 或多个分叉的情形；无分叉时按 Aₙ 处理 | \((n+1)!\) |
| \(m=1\) | 单分叉，排序分支长度为 \([1,1,\_]\)，按 Dₙ 处理 | \(2^n n!/2\) |
| \(m=1\) | 单分叉，排序分支长度为 \([1,2,2]\)，按 E₆ 处理 | \(51840\) |
| \(m=1\) | 单分叉，排序分支长度为 \([1,2,3]\)，按 E₇ 处理 | \(2903040\) |
| \(m=1\) | 单分叉，排序分支长度为 \([1,2,4]\)，按 E₈ 处理 | \(696729600\) |

上述单分叉情形由 `branch_lengths` 从唯一的度 3 节点向外遍历，取得分支长度后按排序结果匹配；其他分支长度组合会被拒绝。^[weyl-size-presentation.md:27-34]

## 输入校验边界

这套识别不构成完整的 Cartan 矩阵合法性验证。单节点分量对任何非零对角值都返回 2；非对角项乘积为负的链接计入度数，却不会提高初值为 1 的 `max_multiplicity`，因而被当作单键情形处理；秩大于 4 的双键链不检查双键位置。这些均为源码阅读观察，详见 [[Cartan 类型识别的输入校验边界]]。^[weyl-size-presentation.md:36-38]

`branch_lengths` 没有环检测，含环输入可能导致遍历不终止，而非返回错误。因此，已有拒绝条件并不保证任意非法输入都能以错误返回结束。^[weyl-size-presentation.md:33-34]

## 测试覆盖与证据范围

来源列出的测试锚点包括经典型 \(A_4=120\)、\(B_5=C_5=3840\)、\(D_5=1920\)，例外型 \(G_2=12\)、\(F_4=1152\)、\(E_6=51840\)，以及夹有一行环面的 \(A_1\times A_1=4\) 和空系统阶为 1。^[weyl-size-presentation.md:40-41]

所有错误分支、E₇/E₈ 分支以及秩 4 的非 F₄ 分支 B₄/C₄ 均无测试。`factorial` 虽返回 `Result`，内部没有可失败操作，该签名属于预留。相关内容见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[weyl-size-presentation.md:72-74]

来源属于结构性源码阅读，不构成数学或正确性验收；阶数字面量与上游行号来自实现及其注释，上游字节未独立核对。维护者对照源码核对并改写了草案，但本次知识维护未执行 Atlas、Cargo、测试或 benchmark，上述测试锚点不代表本次执行结果。^[weyl-size-presentation.md:9-13, weyl-size-presentation.md:72-74, weyl-size-presentation.md:80-84]

## Sources

- [Weyl 群阶识别与实形展示层（weyl_size.rs / presentation.rs）](../../sources/weyl-size-presentation.md)
