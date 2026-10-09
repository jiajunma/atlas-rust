---
title: CartanGradingData 与纤维来源一致性
summary: CartanGradingData 汇集一个 Cartan involution 的 grading 数据，检查 datum 与 involution 一致性，并从 adjoint descent 所用的确切 ambient fiber 构建 m_alpha。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:08.938Z"
updatedAt: "2026-10-09T14:50:08.938Z"
tags:
  - Cartan分类
  - 数据不变量
  - 纤维
aliases:
  - cartangradingdata-与纤维来源一致性
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# CartanGradingData 与纤维来源一致性

`CartanGradingData` 是一个 Cartan 对合的已验证 grading 表。其构造将根数据、对合数据和伴随纤维的来源绑定起来：`m_alpha` 必须来自伴随纤维下降验证所使用的确切 ambient fiber，不能以值相等的其他纤维替代。^[grading.md:29-45]

## 数据与坐标纪律

表中保存 `imaginary_simple_roots`、`simple_mod_two`、ambient fiber 中的 `m_alphas`、使用基本余权坐标的 `adjoint_m_alphas`，以及 `base_grading`、`grading_shifts` 和 `adjoint`。^[grading.md:31-33]

[[Grading 的位向量类型纪律|Grading]] 是 `ModTwoVector` 的 newtype，位 `i` 对应所属模型的第 `i` 个 simple-imaginary 根，置位表示非紧致。这个索引空间不同于 `CartanFiber` 和 `AdjointCartanFiber` 的格坐标或单根坐标；即使维数相同，也必须由类型区分。^[grading.md:17-27]

## 构造时的来源一致性

`build(root_system, root_involution, adjoint)` 执行两项一致性检查：根系统与对合数据必须具有一致的 datum，否则返回 `DatumMismatch`；`adjoint` 的 ambient fiber 对合必须与对合数据一致，否则返回 `CartanFiberInvolutionMismatch`。^[grading.md:40-42]

构造器刻意不接收独立的 ambient fiber 参数，因为值相等无法表达纤维同一性。它直接使用 `AdjointCartanFiber::ambient_fiber` 构建 `m_alpha`，从而沿用伴随下降已验证的确切来源。这与[[伴随 Cartan 纤维的构建与下降验证]]相衔接。^[grading.md:42-45]

逐虚根收集时，`m_alpha` 是余根在 ambient fiber 中的模二像，伴随 `m_alpha` 则由投影取得：
\[
\Pi(y)_j=\langle \alpha_j,y\rangle.
\]
该投影正是 bracket 向量，因此配对逻辑只在投影内保留一份实现。`simple_mod_two` 保存单根坐标的奇性，判定 `*coordinate % 2 != 0` 也包含负奇数。^[grading.md:47-51]

## Grading 求值与逆向恢复

采用 [[Quasisplit 规范化与 grading 的仿射线性求值|quasisplit 规范化]]后，伴随纤维零元素对应的 `base_grading` 为全一，即所有 simple-imaginary 根均为非紧致。其他元素的 grading 由其 canonical ambient representative 作仿射线性求值得到：逐根计算配对后取反，即全一基点与配对值的 XOR。^[grading.md:35-38]

`grading(element)` 先调用 `canonical_representative`，外来纤维元素会触发 `CartanFiberMismatch`，通过检查后才逐根配对取反。因此，来源约束不仅用于构造表，也用于对输入元素求值。^[grading.md:59-60]

`grading_shifts[i]` 记录伴随基代表与各单根奇性向量的 F₂ 配对。构造期通过 `ensure_faithful_shifts` 检查 shift 列的线性无关性；相关列导致 `GradingShiftsNotFaithful`。上游对应检查是断言，此处则无条件拒绝；源码注释称尚无已知公共构造路径能产生相关列，该检查属于防御措施。^[grading.md:51-55]

`element_from_grading(target)` 通过增广消元恢复具有目标 grading 的唯一伴随纤维元素。每个 shift 列附带一个标记其伴随基索引的 marker 位，消元右端为 `target XOR base`。若归约后低 `imaginary_rank` 位仍有非零余数，则返回 `StructureError::ImpossibleGrading`；否则按 marker 位对伴随基代表执行 `xor_assign`，得到 ambient 代表。解的唯一性由构造期的[[Grading shifts 的忠实性不变量]]保证。^[grading.md:61-67]

## 测试与证据边界

来源一致性的测试锚点覆盖三类外来输入拒绝：`DatumMismatch`、`CartanFiberInvolutionMismatch` 和 `CartanFiberMismatch`；另有中心余权坐标案例区分 `m_alpha` 与其伴随像，以及注入相关列的拒绝测试。^[grading.md:73-79]

源包未覆盖构造中的两处 `IndexOutOfRange`、多数溢出或分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。该包属于结构性源码阅读，未执行构建、测试或原版运行，因此这些测试锚点不构成新的数学验收或性能结论。^[grading.md:9-13, grading.md:80-81, grading.md:91-94]

## Sources

- [grading.md](grading.md)：紧致 grading：simple-imaginary 根的紧致性位向量。
