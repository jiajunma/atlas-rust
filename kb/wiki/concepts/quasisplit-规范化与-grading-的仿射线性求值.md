---
title: Quasisplit 规范化与 grading 的仿射线性求值
summary: 零伴随纤维元素的基 grading 为全一，其余元素取规范环境代表后逐根计算配对并取反，得到仿射线性的非紧致位向量。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:11.521Z"
updatedAt: "2026-10-09T19:29:21.751Z"
tags:
  - grading
  - 拟分裂规范化
  - 有限域
aliases:
  - quasisplit-规范化与-grading-的仿射线性求值
  - Q规G的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Quasisplit 规范化与 grading 的仿射线性求值

Quasisplit 规范化将 adjoint fiber 的零元素作为基点，并把每个 simple-imaginary 根标记为 noncompact，因此 `base_grading` 是全一位向量。其他元素的 grading 通过其 canonical ambient representative 与根的模二配对求得：将配对结果与全一基点做 XOR，即逐根计算 `!dot`。^[grading.md:35-38]

## 坐标与位的含义

`Grading` 是 `ModTwoVector` 的 newtype，第 `i` 位对应模型中 `imaginary_simple_roots` 列表的第 `i` 个根，置位表示 noncompact。该列表采用 crate 的确定性根序。grading 索引的是 simple-imaginary 根的位置，与 fiber 的环境格坐标或 simple-root 坐标不同；即使维数相同，也必须通过类型区分，参见 [[Grading 的位向量类型纪律]]。^[grading.md:17-27]

## 仿射线性公式

设 simple-imaginary 根为 \(\alpha_0,\ldots,\alpha_{r-1}\)，adjoint fiber 元素 \(x\) 的 canonical ambient representative 为 \(\widetilde{x}\)。在 \(\mathbf F_2\) 上，求值公式为 \(g_i(x)=1\oplus\langle\alpha_i,\widetilde{x}\rangle\)，其中配对值按模二解释。配对为零时该根为 noncompact，配对为一时为 compact；全一基点是这一仿射线性映射的常量项。^[grading.md:35-38, grading.md:59-60]

`simple_mod_two` 保存单根坐标的奇性，使用 `*coordinate % 2 != 0`，因此也正确识别负奇数。`grading_shifts[j]` 保存第 `j` 个伴随基代表与各单根奇性向量的 \(\mathbf F_2\) 配对，描述该基方向对应的 grading 变化量。^[grading.md:47-51]

## 来源一致性与忠实性

`CartanGradingData::build` 检查根系统与 involution 数据的 datum 是否一致，以及 adjoint 所属 ambient fiber 的 involution 是否与输入一致；不满足时分别返回 `DatumMismatch` 和 `CartanFiberInvolutionMismatch`。构造器使用 `AdjointCartanFiber::ambient_fiber` 这一确切来源构建 `m_alpha`，详见 [[CartanGradingData 与纤维来源一致性]]。求值时，`grading(element)` 先取得 canonical representative，并以 `CartanFiberMismatch` 拒绝外来纤维元素。^[grading.md:40-45, grading.md:59-60]

构造期的 `ensure_faithful_shifts` 要求 shift 列线性独立，否则返回 `GradingShiftsNotFaithful`。这一 [[Grading shifts 的忠实性不变量]] 保证每个可实现 grading 对应唯一的 adjoint fiber 元素，但不保证所有 grading 都可实现。^[grading.md:52-55, grading.md:61-67]

## 从 grading 恢复元素

`element_from_grading(target)` 使用增广消元求解，右端为 `target XOR base`。因为基点全一，右端恰好标记目标 grading 的 compact 位置。每个 shift 列附带位于 `imaginary_rank + adjoint_basis_index` 的 marker 位，在归约右端时同步记录所需的基组合，参见 [[通过增广消元反求 grading 对应元素]]。^[grading.md:61-65]

若余数的低 `imaginary_rank` 位仍有置位，则目标不可实现，返回 `StructureError::ImpossibleGrading`；否则按 marker 位选取伴随基代表，通过 `xor_assign` 汇总得到 ambient 代表。解的唯一性由构造期的 faithful 检查保证。^[grading.md:65-67]

## 测试锚点与证据边界

源码测试锚点包括 SC A1 的 quasisplit 规范化与双向往返、A2 恒等对合下的四元素双射，以及 A2 扭转情形对全紧 grading 的拒绝。A2 恒等情形中，根序 index 0 为 \(\alpha_2\)，shift 为置换矩阵；A2 扭转情形中，adjoint fiber 维数为零，`grading_shift(0)` 返回 `None`。^[grading.md:73-76]

来源属于结构性阅读，未执行构建、测试或原版运行，不构成数学验收、性能或并行结论。明确未覆盖的分支包括构造中的两处 `IndexOutOfRange`、多数溢出与分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。^[grading.md:9-13, grading.md:80-81, grading.md:91-94]

## Sources

- [grading.md](grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量。
