---
title: m_alpha 的模二归约与伴随投影
summary: m_alpha 是余根在环境纤维中的模二像，其伴随像由根配对投影得到，坐标奇性判断保留负奇数的非零位。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:13.065Z"
updatedAt: "2026-10-09T22:31:15.312Z"
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: m_alpha 的模二归约与伴随投影
summary: m_alpha 是余根在 ambient fiber 中的模二像，其伴随像由根配对投影得到；构造绑定确切的纤维来源，奇性归约保留负奇数的非零位。
sources:
  - grading.md
kind: concept
tags:
  - 余根
  - 模二线性代数
  - 伴随投影
---

# m_alpha 的模二归约与伴随投影

`CartanGradingData` 为 simple-imaginary 根保存两类纤维数据：`m_alphas` 是余根在 ambient fiber 中的模二类，`adjoint_m_alphas` 是其伴随投影像，后者使用基本余权（fundamental-coweight）坐标。两者具有不同的纤维语义，投影后的类可能变为平凡。^[grading.md:29-33, grading.md:47-51, grading.md:73-77]

## 模二归约与负坐标

对每个 simple-imaginary 根 \(\alpha\)，`m_alpha` 取余根 \(\alpha^\vee\) 在 ambient fiber 中的模二像。归约必须保留负奇数的奇性：B2 测试以余根坐标 \((2,-1)\) 归约为 \((0,1)\) 为锚点，负奇数对应非零位。^[grading.md:47-51]

同一收集流程还构造 `simple_mod_two`，通过 `*coordinate % 2 != 0` 提取单根坐标的奇性位。应区分两种数据来源：`simple_mod_two` 来自根坐标，而 `m_alpha` 来自余根在纤维中的像；位向量的相关表示见 [[F₂ 上的位打包向量（ModTwoVector）]]。^[grading.md:47-51]

## 伴随投影与来源绑定

伴随投影满足 \(\Pi(y)_j=\langle\alpha_j,y\rangle\)，其坐标向量正是对应的 bracket 向量。伴随 `m_alpha` 通过该投影获得，使根与余特征的配对保留在投影的单一实现中。^[grading.md:47-49]

`CartanGradingData::build` 使用 `AdjointCartanFiber::ambient_fiber` 构建 `m_alpha`，刻意不接收独立的 ambient fiber 参数。原因是值相等不足以表达 fiber 同一性：构造必须绑定伴随映射下降得到验证时所用的确切来源。参见 [[CartanGradingData 与纤维来源一致性]]。^[grading.md:40-45]

构造执行两道一致性检查：根系统与 involution 数据的 datum 不一致时，报 `DatumMismatch`；adjoint 所属 ambient fiber 的 involution 与 involution 数据不一致时，报 `CartanFiberInvolutionMismatch`。^[grading.md:40-45]

## 与 grading 坐标的区别

`Grading` 的 bit `i` 对应模型中 simple-imaginary 根列表的第 `i` 项，置位表示 noncompact。相比之下，ambient 与 adjoint fiber 的模二坐标索引全 datum 的格坐标或 simple roots。在 A2 恒等对合等情形中，这些向量维数相同，不能仅靠维数检查区分，必须由类型保留其含义。参见 [[Grading 的位向量类型纪律]]。^[grading.md:17-27]

quasisplit 规范化令零伴随纤维元素的 `base_grading` 全为一；其他元素的 grading 对 canonical ambient representative 逐根配对并取 `!dot`，即基点与配对值的 XOR。`grading_shifts[i]` 则由伴随基代表与各单根奇性向量的 \(\mathbf F_2\) 配对构成。参见 [[Quasisplit 规范化与 grading 的仿射线性求值]]。^[grading.md:35-38, grading.md:47-51, grading.md:59-60]

## 测试锚点与证据边界

与本概念直接相关的测试锚点包括：SC A1 中 `m_alpha` 非平凡而伴随像平凡；中心余权坐标下 `m_alpha` 与其伴随像的区分；B2 负奇余根坐标的模二归约。来源还列出外来输入触发 `DatumMismatch`、`CartanFiberInvolutionMismatch` 和 `CartanFiberMismatch` 的拒绝测试。^[grading.md:73-79]

来源包属于结构性阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。已列出的覆盖缺口包括 `build` 的两处 `IndexOutOfRange`、多数溢出及分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。^[grading.md:9-13, grading.md:80-81, grading.md:91-94]

## Sources

- [grading.md](../../sources/grading.md)：紧致 grading：simple-imaginary 根的紧致性位向量。
