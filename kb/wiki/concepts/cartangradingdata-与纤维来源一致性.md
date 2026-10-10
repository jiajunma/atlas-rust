---
title: CartanGradingData 与纤维来源一致性
summary: 构造 grading 表时校验 datum 与对合一致性，并使用伴随下降验证所绑定的确切 ambient fiber 构建 m_alpha。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:08.938Z"
updatedAt: "2026-10-10T00:34:28.870Z"
tags:
  - grading
  - Cartan纤维
aliases:
  - cartangradingdata-与纤维来源一致性
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: CartanGradingData 与纤维来源一致性
summary: CartanGradingData 校验 datum 与对合一致性，并从伴随下降所用的确切 ambient fiber 构造 m_alpha，避免以值相等替代来源同一性。
sources:
  - grading.md
kind: concept
tags:
  - 紧致分级
  - Cartan纤维
  - 来源一致性
---

# CartanGradingData 与纤维来源一致性

`CartanGradingData` 是一个 Cartan 对合的已验证 grading 表。其核心来源约束是：`m_alpha` 必须从伴随下降验证所使用的确切 ambient fiber 构建；纤维的值相等不足以表达来源同一性。^[grading.md:29-45]

## 数据与坐标纪律

表中保存 `imaginary_simple_roots`、`simple_mod_two`、ambient fiber 类 `m_alphas`、使用基本余权坐标的伴随纤维类 `adjoint_m_alphas`，以及 `base_grading`、`grading_shifts` 和 `adjoint`。^[grading.md:31-33]

[[Grading 的位向量类型纪律|Grading]] 是 `ModTwoVector` 的 newtype。位 `i` 对应所属模型的第 `i` 个 simple-imaginary 根，置位表示非紧致；根列表复制自 `RootInvolutionData::imaginary_simple_roots`，遵循本 crate 的确定性根序。grading 的位置索引不同于纤维使用的全 datum 格坐标或单根坐标，即使维数相同，也必须通过类型区分。^[grading.md:17-27]

## 构造时的来源校验

`build(root_system, root_involution, adjoint)` 按顺序设置两道一致性检查：根系统与对合数据的 datum 必须一致，否则返回 `DatumMismatch`；随后，伴随纤维所持 ambient fiber 的对合必须与对合数据一致，否则返回 `CartanFiberInvolutionMismatch`。^[grading.md:40-42]

构造器刻意不接收独立的 ambient fiber 参数，而是直接针对 `AdjointCartanFiber::ambient_fiber` 构建 `m_alpha`。这使构造所用纤维保持为伴随下降得到证明时的确切来源，而不依赖纤维值相等来表达同一性。^[grading.md:42-45]

逐虚根收集时，`m_alpha` 是余根在 ambient fiber 中的模二像，伴随 `m_alpha` 则经投影取得。投影公式 $\Pi(y)_j=\langle\alpha_j,y\rangle$ 正是其 bracket 向量，因此配对逻辑只在投影内保留一份实现，详见 [[m_alpha 的模二归约与伴随投影]]。`simple_mod_two` 保存单根坐标的奇性，判定式 `*coordinate % 2 != 0` 包含负奇数。^[grading.md:47-51]

## 求值输入与逆向恢复

在 [[Quasisplit 规范化与 grading 的仿射线性求值|quasisplit 规范化]]下，伴随纤维零元素对应的 `base_grading` 为全一，即所有 simple-imaginary 根均为非紧致。其他元素的 grading 通过其典范 ambient 代表作仿射线性求值：逐根取 `!dot`，相当于全一基点与配对值的 XOR。^[grading.md:35-38]

来源约束同样适用于求值输入。`grading(element)` 先调用 `canonical_representative`，外来纤维元素触发 `CartanFiberMismatch`；通过检查后才逐根配对取反。^[grading.md:59-60]

`grading_shifts[i]` 记录伴随基代表与各单根奇性向量的 F₂ 配对。构造期的 `ensure_faithful_shifts` 要求 shift 列线性无关，否则返回 `GradingShiftsNotFaithful`。上游对应检查是断言，Rust 实现改为无条件拒绝；源码注释称没有已知公共构造路径能产生相关列，因此这是防御性检查，参见 [[Grading shifts 的忠实性不变量]]。^[grading.md:51-55]

`element_from_grading(target)` 通过增广消元恢复具有目标 grading 的唯一伴随纤维元素。每列附加位于 `imaginary_rank + adjoint_basis_index` 的 marker 位，以记录求解组合；右端为 `target XOR base`，标记目标中的紧致位置。若归约后低 `imaginary_rank` 位仍有置位，则返回 `StructureError::ImpossibleGrading`；否则按 marker 位对伴随基代表执行 `xor_assign`，汇总为 ambient 代表。可实现目标的解唯一性依赖构造期检查的忠实性不变量。^[grading.md:61-67]

`grading_shift(adjoint_basis_index)` 提供 shift 表访问，其索引上界由 `adjoint_fiber().dimension()` 决定。^[grading.md:68-69]

## 测试与证据边界

来源一致性的测试锚点覆盖三类外来输入拒绝：`DatumMismatch`、`CartanFiberInvolutionMismatch` 和 `CartanFiberMismatch`。中心余权坐标案例区分 `m_alpha` 与其伴随像；重复列和零列注入测试直接验证忠实性检查。^[grading.md:52-55, grading.md:73-79]

求值与恢复的测试锚点包括 SC A1 的 quasisplit 规范化及双向往返、A2 恒等对合的四元素双射，以及 A2 扭转对全紧 grading 的 `ImpossibleGrading` 拒绝。尚未覆盖构造中的两处 `IndexOutOfRange`、多数溢出或分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。^[grading.md:73-81]

来源属于结构性源码阅读，未执行构建、测试或原版运行。上述锚点描述源码中的测试覆盖，不构成新的数学验收、性能或并行结论；实现正确性仍属于其自身的 [[HPC 验收证据链]]。^[grading.md:9-13, grading.md:91-94]

## Sources

- [grading.md](../../sources/grading.md)：紧致 grading：simple-imaginary 根的紧致性位向量。
