---
title: CartanGradingData 与纤维来源一致性
summary: 构造 grading 表时验证 datum 和对合一致性，并从伴随下降所使用的确切 ambient fiber 构造 m_alpha，以保证来源身份。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:08.938Z"
updatedAt: "2026-10-09T20:54:01.123Z"
tags:
  - 来源一致性
  - Cartan纤维
  - 构造校验
aliases:
  - cartangradingdata-与纤维来源一致性
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: CartanGradingData 与纤维来源一致性
summary: CartanGradingData 校验 datum 与对合一致性，并从伴随下降所用的确切 ambient fiber 构造 m_alpha，避免以值相等替代来源同一性。
sources:
  - grading.md
kind: concept
tags:
  - grading
  - Cartan纤维
  - 来源校验
---

# CartanGradingData 与纤维来源一致性

`CartanGradingData` 是一个 Cartan 对合的已验证 grading 表。其核心约束是将 grading 数据绑定到伴随纤维的确切来源：`m_alpha` 必须从伴随下降验证所用的 ambient fiber 构建，纤维的值相等不足以表达来源同一性。^[grading.md:29-45]

## 数据与坐标纪律

表中保存 `imaginary_simple_roots`、`simple_mod_two`、ambient fiber 类 `m_alphas`、使用基本余权坐标的伴随纤维类 `adjoint_m_alphas`，以及 `base_grading`、`grading_shifts` 和 `adjoint`。^[grading.md:31-33]

[[Grading 的位向量类型纪律|Grading]] 是 `ModTwoVector` 的 newtype，位 `i` 对应所属模型的第 `i` 个 simple-imaginary 根，置位表示非紧致。根列表来自 `RootInvolutionData::imaginary_simple_roots`，遵循本 crate 的确定性根序。grading 的位置索引与纤维使用的全 datum 格坐标或单根坐标不同；即使维数相同，也必须通过类型区分。^[grading.md:17-27]

## 构造时的来源一致性

`build(root_system, root_involution, adjoint)` 设置两道一致性检查：根系统与对合数据的 datum 必须一致，否则返回 `DatumMismatch`；伴随纤维所持 ambient fiber 的对合必须与对合数据一致，否则返回 `CartanFiberInvolutionMismatch`。^[grading.md:40-42]

构造器刻意不接收独立的 ambient fiber 参数，而是直接使用 `AdjointCartanFiber::ambient_fiber` 生成 `m_alpha`。这样，构造所用纤维就是伴随下降得到证明时的确切来源，不需要以值比较代替纤维同一性。^[grading.md:42-45]

逐虚根收集时，`m_alpha` 是余根在 ambient fiber 中的模二像，伴随 `m_alpha` 经投影取得。投影公式 \(\Pi(y)_j=\langle\alpha_j,y\rangle\) 正是其 bracket 向量，因此配对逻辑只在投影内保留一份实现，详见 [[m_alpha 的模二归约与伴随投影]]。`simple_mod_two` 保存单根坐标的奇性，判定式 `*coordinate % 2 != 0` 也包含负奇数。^[grading.md:47-51]

## Grading 求值与逆向恢复

在 [[Quasisplit 规范化与 grading 的仿射线性求值|quasisplit 规范化]]下，伴随纤维零元素对应的 `base_grading` 为全一，即所有 simple-imaginary 根均为非紧致。其他元素的 grading 通过其典范 ambient 代表作仿射线性求值：逐根计算配对后取反，相当于全一基点与配对值的 XOR。^[grading.md:35-38]

来源约束也适用于求值输入。`grading(element)` 先调用 `canonical_representative`；外来纤维元素触发 `CartanFiberMismatch`，通过检查后才逐根配对取反。^[grading.md:59-60]

`grading_shifts[i]` 记录伴随基代表与各单根奇性向量的 F₂ 配对。构造期的 `ensure_faithful_shifts` 检查 shift 列的线性无关性，相关列触发 `GradingShiftsNotFaithful`。上游对应检查是断言，此处改为无条件拒绝；源码注释称尚无已知公共构造路径能产生相关列，因此这是防御性检查，参见 [[Grading shifts 的忠实性不变量]]。^[grading.md:51-55]

`element_from_grading(target)` 使用[[通过增广消元反求 grading 对应元素|增广消元]]恢复具有目标 grading 的唯一伴随纤维元素。每列在 grading 位之外附加 marker 位，其位置为 `imaginary_rank + adjoint_basis_index`；右端为 `target XOR base`，因此标记目标中的紧致位置。归约时同时累计求解组合，若余数的低 `imaginary_rank` 位仍有置位，则返回 `StructureError::ImpossibleGrading`；否则按 marker 位对伴随基代表执行 `xor_assign`，汇总为 ambient 代表。唯一性由构造期的忠实性不变量保证。^[grading.md:61-67]

`grading_shift(adjoint_basis_index)` 提供 shift 表访问，其索引上界由 `adjoint_fiber().dimension()` 决定。^[grading.md:68-69]

## 测试与证据边界

来源一致性的测试锚点覆盖三类外来输入拒绝：`DatumMismatch`、`CartanFiberInvolutionMismatch` 和 `CartanFiberMismatch`。另有中心余权坐标案例区分 `m_alpha` 与其伴随像，以及重复列、零列注入测试验证忠实性检查。^[grading.md:52-55, grading.md:73-79]

求值与恢复的测试锚点包括 SC A1 的 quasisplit 规范化及双向往返、A2 恒等对合的四元素双射，以及 A2 扭转对全紧 grading 的 `ImpossibleGrading` 拒绝。未覆盖的分支包括构造中的两处 `IndexOutOfRange`、多数溢出或分配分支，以及 `element_from_grading` 入口的 `RankMismatch`，参见 [[紧致 grading 的测试锚点与覆盖边界]]。^[grading.md:73-81]

来源属于结构性源码阅读，未执行构建、测试或原版运行。上述测试锚点描述源码中的覆盖，不构成新的数学验收、性能或并行结论；正确性仍属于该实现自身的 HPC 证据链。^[grading.md:9-13, grading.md:91-94]

## Sources

- [grading.md](../../sources/grading.md)：紧致 grading：simple-imaginary 根的紧致性位向量。
