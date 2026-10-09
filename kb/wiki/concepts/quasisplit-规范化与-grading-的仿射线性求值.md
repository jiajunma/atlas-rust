---
title: Quasisplit 规范化与 grading 的仿射线性求值
summary: 伴随纤维零元的基 grading 为全一，其余元素先取规范环境代表，再逐虚单根计算模二配对并取反。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:11.521Z"
updatedAt: "2026-10-09T22:31:17.983Z"
tags:
  - 紧致分级
  - 仿射映射
aliases:
  - quasisplit-规范化与-grading-的仿射线性求值
  - Q规G的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Quasisplit 规范化与 grading 的仿射线性求值
summary: Quasisplit 规范化将零伴随纤维元素的 grading 设为全一；其他元素通过规范环境代表的模二配对取反求值，可实现 grading 的唯一逆像由 shift 列的忠实性保证。
sources:
  - grading.md
kind: concept
tags:
  - grading
  - 拟分裂规范化
  - 模二线性代数
---

# Quasisplit 规范化与 grading 的仿射线性求值

Quasisplit（拟分裂）规范化以 adjoint fiber（伴随纤维）的零元素为基点，将每个 simple-imaginary 根标记为 noncompact（非紧），因此 `base_grading` 是全一位向量。其他元素的 grading 由其 canonical ambient representative（规范环境代表）逐根配对并取反得到，即实现中的 `!dot`，等价于将配对向量与全一基点做 XOR。^[grading.md:35-38]

## 位向量与坐标约定

`Grading` 是 `ModTwoVector` 的 newtype，第 \(i\) 位对应所属模型的 `imaginary_simple_roots` 列表中的第 \(i\) 个根，置位表示 noncompact。该列表是 `RootInvolutionData::imaginary_simple_roots` 的副本，采用 crate 的确定性根序。^[grading.md:17-20]

grading 的索引表示 simple-imaginary 根的位置，而 ambient fiber 与 adjoint fiber 的模二坐标索引全 datum 的格坐标或单根。即使这些空间维数相同，也必须通过类型区分，不能仅依赖维数检查；参见 [[Grading 的位向量类型纪律]]。^[grading.md:22-27]

## 仿射线性求值

设 simple-imaginary 根列表为 \(\alpha_0,\ldots,\alpha_{r-1}\)，伴随纤维元素 \(x\) 的规范环境代表为 \(\widetilde{x}\)。在 \(\mathbf F_2\) 上，求值公式为 \(g_i(x)=1\oplus\langle\alpha_i,\widetilde{x}\rangle\)，其中配对值按模二解释。配对为零时对应 noncompact，配对为一时对应 compact（紧）；全一向量是这一仿射线性映射的常量项。^[grading.md:35-38, grading.md:59-60]

`simple_mod_two` 保存单根坐标的奇性，使用 `*coordinate % 2 != 0` 判定，包含负奇数。`grading_shifts[j]` 保存第 \(j\) 个伴随基代表与各单根奇性向量的 \(\mathbf F_2\) 配对，表示该基方向引起的 grading 变化量。^[grading.md:47-51]

`grading(element)` 先调用 `canonical_representative`，再逐根配对取反。外来纤维元素会触发 `CartanFiberMismatch`，因此求值要求元素来自相应纤维。^[grading.md:59-60]

## 构造约束与忠实性

`CartanGradingData::build` 检查根系统与 involution 数据的 datum 一致性，以及 adjoint 所属 ambient fiber 的 involution 与输入的一致性；失败时分别返回 `DatumMismatch` 和 `CartanFiberInvolutionMismatch`。构造器不接收独立的 ambient fiber 参数，而使用 `AdjointCartanFiber::ambient_fiber` 构建 `m_alpha`，以保留伴随下降验证所用的确切来源；参见 [[CartanGradingData 与纤维来源一致性]]。^[grading.md:40-45]

构造期通过 `ensure_faithful_shifts` 检查 shift 列的线性独立性，相关列会导致 `GradingShiftsNotFaithful`。这一 [[Grading shifts 的忠实性不变量]] 保证每个可实现 grading 对应唯一的伴随纤维元素，但不保证所有 grading 都可实现。上游对应检查是断言，Rust 实现改为无条件拒绝；来源将其描述为防御性检查，没有已知公共构造路径能产生相关列。^[grading.md:52-55, grading.md:61-67]

## 从 grading 恢复元素

`element_from_grading(target)` 使用增广消元求解，右端为 `target XOR base`。由于 `base` 全一，右端恰好标记目标 grading 的 compact 位置。每个 shift 列附带一个位于 `imaginary_rank + adjoint_basis_index` 的 marker 位，归约右端时同步累计所需的基组合。^[grading.md:61-65]

若归约余数的低 `imaginary_rank` 位仍有置位，则目标不可实现，返回 `StructureError::ImpossibleGrading`；否则按 marker 位选择伴随基代表，以 `xor_assign` 汇总得到 ambient 代表。解的唯一性来自构造期的 faithful 检查。^[grading.md:65-67]

## 测试锚点与证据边界

源码测试锚点包括 SC A1 的 quasisplit 规范化与双向往返、A2 恒等对合下的四元素双射，以及 A2 扭转情形对全紧 grading 的拒绝。A2 恒等情形中，根序 index 0 为 \(\alpha_2\)，shift 为置换矩阵；A2 扭转情形中，伴随纤维维数为零，`grading_shift(0)` 返回 `None`。另有注入重复列与零列的测试直接检查忠实性拒绝行为。^[grading.md:52-55, grading.md:73-78]

来源属于结构性阅读，未执行构建、测试或原版运行，不构成数学验收、性能或并行结论。明确未覆盖的分支包括 `build` 中的两处 `IndexOutOfRange`、多数溢出与分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。^[grading.md:9-13, grading.md:80-81, grading.md:91-91]

## Sources

- [grading.md](../../sources/grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量。
