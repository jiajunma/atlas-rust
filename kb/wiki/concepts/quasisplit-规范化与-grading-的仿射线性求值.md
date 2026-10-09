---
title: Quasisplit 规范化与 grading 的仿射线性求值
summary: Quasisplit 规范化令零 adjoint fiber 元素的 grading 为全一，其余元素通过 canonical ambient representative 与单根奇性向量配对后取反求值。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:11.521Z"
updatedAt: "2026-10-09T14:50:11.521Z"
tags:
  - 规范化
  - 仿射线性映射
  - 紧致性
aliases:
  - quasisplit-规范化与-grading-的仿射线性求值
  - Q规G的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Quasisplit 规范化与 grading 的仿射线性求值

Quasisplit 规范化把 adjoint fiber 的零元素选为基点，并将其每个 simple-imaginary 根标记为 noncompact。因此，`base_grading` 是全一位向量；其他元素的 grading 由其 canonical ambient representative 的配对值与基点 grading 做 XOR 得到。^[grading.md:35-38]

## 坐标与位的含义

`Grading` 是 `ModTwoVector` 的 newtype，第 `i` 位对应模型中 `imaginary_simple_roots` 列表的第 `i` 个根，置位表示 noncompact。该列表采用 crate 的确定性根序。grading 的位置索引与 ambient fiber 的格坐标、adjoint fiber 的 simple-root 坐标具有不同含义，即使维数相同也必须通过类型区分，详见 [[Grading 的位向量类型纪律]]。^[grading.md:17-27]

## 仿射线性求值

设 simple-imaginary 根为 \(\alpha_0,\ldots,\alpha_{r-1}\)，adjoint fiber 元素 \(x\) 的 canonical ambient representative 为 \(\widetilde{x}\)。在 \(\mathbf F_2\) 上，求值可写为 \(g_i(x)=1\oplus\langle\alpha_i,\widetilde{x}\rangle\)，即 \(g(x)=\mathbf1\oplus L(\widetilde{x})\)，其中 \(L\) 收集逐根的模二配对。实现逐根计算 `!dot`：配对为零时结果为 noncompact，配对为一时结果为 compact。全一基点提供了仿射公式中的常量项。^[grading.md:35-38, grading.md:59-60]

`simple_mod_two` 保存各单根坐标的奇性，使用 `*coordinate % 2 != 0`，因此负奇数也会被识别为置位。`grading_shifts[j]` 则记录第 `j` 个伴随基代表与各单根奇性向量的 \(\mathbf F_2\) 配对；它给出沿该基方向改变元素时 grading 的变化量。^[grading.md:47-51]

## 来源一致性与唯一性

`CartanGradingData::build` 要求根系统与 involution 数据属于同一 datum，并要求 adjoint 所属 ambient fiber 的 involution 与输入 involution 数据一致；不满足时分别返回 `DatumMismatch` 或 `CartanFiberInvolutionMismatch`。构造器使用 `AdjointCartanFiber::ambient_fiber` 这一确切来源构建 `m_alpha`，相关约束见 [[CartanGradingData 与纤维来源一致性]]。实际调用 `grading(element)` 时，取得 canonical representative 的步骤还会以 `CartanFiberMismatch` 拒绝外来纤维元素。^[grading.md:40-45, grading.md:59-60]

构造期通过 `ensure_faithful_shifts` 检查 shift 列的线性独立性；相关列会触发 `GradingShiftsNotFaithful`。这一 [[Grading shifts 的忠实性不变量]] 保证可实现 grading 所对应的 adjoint fiber 元素唯一，但并不保证任意 grading 都可实现。^[grading.md:52-55, grading.md:61-67]

## 从 grading 恢复元素

`element_from_grading(target)` 求解的右端为 `target XOR base`。由于 `base` 全一，该右端恰好标记目标 grading 中的 compact 位置。实现使用增广消元：每个 shift 列附带位于 `imaginary_rank + adjoint_basis_index` 的 marker 位，在归约右端的同时记录所需的基组合。^[grading.md:61-65]

若归约余数的低 `imaginary_rank` 位仍有置位，目标不在可实现范围内，返回 `StructureError::ImpossibleGrading`；否则按 marker 位对伴随基代表执行 `xor_assign`，汇总得到 ambient 代表。解的唯一性由构造期的 faithful 检查保证。^[grading.md:65-67]

## 测试锚点与证据边界

源码测试锚点包括 SC A1 的 quasisplit 规范化与双向往返、A2 恒等对合下的四元素双射，以及 A2 扭转情形对全紧 grading 的拒绝。其中 A2 恒等情形的根序 index 0 为 \(\alpha_2\)，shift 为置换矩阵；A2 扭转情形的 adjoint fiber 维数为零，`grading_shift(0)` 返回 `None`。^[grading.md:73-76]

来源材料记录的是结构性阅读及源码中的测试锚点，并未执行构建、测试或原版运行，不能据此声称数学验收或性能结论。明确未覆盖的分支包括 `element_from_grading` 入口的 `RankMismatch`、构造中的两处 `IndexOutOfRange`，以及多数溢出和分配分支。^[grading.md:9-13, grading.md:80-81, grading.md:91-94]

## Sources

- [grading.md](grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量。
