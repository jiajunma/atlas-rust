---
title: m_alpha 的模二归约与伴随投影
summary: m_alpha 是余根在环境纤维中的模二像，伴随像经根配对投影得到，奇性判断保留负奇数的非零位。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:13.065Z"
updatedAt: "2026-10-10T00:34:25.336Z"
tags:
  - Cartan纤维
  - 模二线性代数
aliases:
  - malpha-的模二归约与伴随投影
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: m_alpha 的模二归约与伴随投影
summary: m_alpha 是余根在 ambient fiber 中的模二像，其伴随像由根配对投影得到；构造绑定确切的纤维来源，奇性归约保留负奇数的非零位。
sources:
  - grading.md
kind: concept
tags:
  - Cartan纤维
  - 模二线性代数
  - 伴随投影
---

# m_alpha 的模二归约与伴随投影

`CartanGradingData` 为 simple-imaginary 根保存两类纤维数据：`m_alphas` 是余根在环境纤维（ambient fiber）中的模二类，`adjoint_m_alphas` 是其伴随纤维中的投影类，后者使用基本余权（fundamental-coweight）坐标。环境类非平凡并不意味着伴随像非平凡：SC A1 测试即包含前者非平凡、后者平凡的情形。^[grading.md:31-33, grading.md:47-51, grading.md:73-77]

## 模二归约与负坐标

对每个 simple-imaginary 根 $\alpha$，`m_alpha` 取余根 $\alpha^\vee$ 在环境纤维中的模二像。负奇数必须保留为非零位；B2 测试以余根坐标 $(2,-1)$ 归约为 $(0,1)$ 为锚点。^[grading.md:47-51]

同一收集流程还生成 `simple_mod_two`，用 `*coordinate % 2 != 0` 提取单根坐标的奇性位，该判断包含负奇数。两类数据的来源应明确区分：`simple_mod_two` 来自根坐标，`m_alpha` 来自余根在纤维中的像。相关位向量表示见 [[F₂ 上的位打包向量（ModTwoVector）]]。^[grading.md:47-51]

## 伴随投影与来源绑定

伴随投影满足 $\Pi(y)_j=\langle\alpha_j,y\rangle$，所得坐标向量正是 bracket 向量。伴随 `m_alpha` 通过该投影获得，使配对计算保留在投影的单一实现中。^[grading.md:47-49]

`CartanGradingData::build` 刻意不接收独立的环境纤维参数，而是使用 `AdjointCartanFiber::ambient_fiber` 构建 `m_alpha`。值相等不足以表达纤维同一性，因此必须使用伴随映射下降得到验证时的确切来源。参见 [[CartanGradingData 与纤维来源一致性]]。^[grading.md:40-45]

构造有两道一致性检查：根系统与 involution 数据的 datum 不一致时，报 `DatumMismatch`；adjoint 所属环境纤维的 involution 与 involution 数据不一致时，报 `CartanFiberInvolutionMismatch`。^[grading.md:40-45]

## 与 grading 坐标的关系

`Grading` 的第 `i` 位对应所属模型的 simple-imaginary 根列表第 `i` 项，置位表示 noncompact。环境纤维与伴随纤维的模二坐标则索引全 datum 的格坐标或 simple roots。在 A2 恒等对合等情形中，这些向量维数相同，维数检查无法区分其语义，必须依靠类型。参见 [[Grading 的位向量类型纪律]]。^[grading.md:17-27]

quasisplit 规范化令零伴随纤维元素的 `base_grading` 全为一；其余元素的 grading 对 canonical ambient representative 逐根配对并取 `!dot`，即全一基点与配对值的 XOR。`grading_shifts[i]` 则由伴随基代表与各单根奇性向量的 $\mathbf F_2$ 配对构成。参见 [[Quasisplit 规范化与 grading 的仿射线性求值]]。^[grading.md:35-38, grading.md:47-51]

## 测试锚点与证据边界

直接相关的测试锚点包括 SC A1 中非平凡 `m_alpha` 的平凡伴随像、中心余权坐标下环境类与伴随像的区分，以及 B2 负奇余根坐标的归约。来源还列出外来输入触发 `DatumMismatch`、`CartanFiberInvolutionMismatch` 和 `CartanFiberMismatch` 的拒绝测试。^[grading.md:73-79]

这些描述来自结构性阅读，来源包未执行构建、测试或原版运行，不构成数学验收、性能或并行结论。已知测试覆盖缺口包括 `build` 的两处 `IndexOutOfRange`、多数溢出与分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。^[grading.md:9-13, grading.md:80-81, grading.md:91-94]

## Sources

- [grading.md](../../sources/grading.md)：紧致 grading：simple-imaginary 根的紧致性位向量。
