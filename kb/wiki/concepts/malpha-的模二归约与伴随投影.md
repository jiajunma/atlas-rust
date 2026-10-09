---
title: m_alpha 的模二归约与伴随投影
summary: m_alpha 是余根在 ambient fiber 中的模二像，其伴随像通过根配对投影得到，奇性归约须保留负奇数的非零位。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:13.065Z"
updatedAt: "2026-10-09T20:54:14.320Z"
tags:
  - 余根
  - 伴随投影
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
summary: m_alpha 是余根在 ambient fiber 中的模二像，其伴随像通过根配对投影得到；构造绑定确切的 ambient fiber 来源，并保留负奇数的奇性。
sources:
  - grading.md
kind: concept
tags:
  - 模二线性代数
  - 余根
  - 伴随投影
---

# m_alpha 的模二归约与伴随投影

`CartanGradingData` 为 simple-imaginary 根保存两类纤维数据：`m_alphas` 是余根在 ambient Cartan fiber 中的模二类，`adjoint_m_alphas` 是其在伴随 fiber 中的投影像，后者使用基本余权（fundamental-coweight）坐标。这两类数据须保留各自的纤维语义。^[grading.md:29-33, grading.md:47-51]

## 模二归约与负坐标

对每个 simple-imaginary 根 \(\alpha\)，`m_alpha` 取余根 \(\alpha^\vee\) 在 ambient fiber 中的 mod-2 像。源材料以 B2 测试锚定负奇数的归约行为：余根坐标 \((2,-1)\) 归约为 \((0,1)\)，负奇数仍对应置位。^[grading.md:47-51]

同一收集流程还构造 `simple_mod_two`，以 `*coordinate % 2 != 0` 保存单根坐标的奇性位。这一区分涉及数据来源：`simple_mod_two` 来自根坐标，`m_alpha` 来自余根在纤维中的像；相关位向量表示见 [[F₂ 上的位打包向量（ModTwoVector）]]。^[grading.md:47-51]

## 伴随投影与来源绑定

伴随投影满足 \(\Pi(y)_j=\langle\alpha_j,y\rangle\)，其坐标向量正是对应的 bracket 向量。伴随 `m_alpha` 经这一投影获得，使配对计算保留在投影的单一实现中。^[grading.md:47-49]

`CartanGradingData::build` 使用 `AdjointCartanFiber::ambient_fiber` 构建 `m_alpha`，刻意不接收独立的 ambient fiber 参数。这样绑定的是伴随映射下降得到验证时使用的确切来源，因为值相等不足以表达 fiber 同一性。参见 [[CartanGradingData 与纤维来源一致性]]。^[grading.md:40-45]

构造有两道一致性检查：根系统与 involution 数据的 datum 必须一致，否则报 `DatumMismatch`；adjoint 所属 ambient fiber 的 involution 必须与 involution 数据一致，否则报 `CartanFiberInvolutionMismatch`。^[grading.md:40-45]

## 与 grading 坐标的区别

`Grading` 的 bit `i` 索引 simple-imaginary 根列表的第 `i` 项，置位表示 noncompact；ambient 与 adjoint fiber 的模二坐标则索引全 datum 的格坐标或 simple roots。在 A2 恒等对合等情形中，这些向量的维数可能相同，因此必须由类型区分其含义，不能仅依赖维数检查。参见 [[Grading 的位向量类型纪律]]。^[grading.md:17-27]

grading 的计算对 canonical ambient representative 逐根配对并取 `!dot`。quasisplit 规范化令基点 grading 全为一，因此这一计算等于基点与配对值的 XOR；`grading_shifts[i]` 则由伴随基代表与各单根奇性向量的 \(\mathbf F_2\) 配对构成。参见 [[Quasisplit 规范化与 grading 的仿射线性求值]]。^[grading.md:35-38, grading.md:47-51, grading.md:59-60]

## 测试锚点与证据边界

相关测试锚点包括 SC A1 中 `m_alpha` 非平凡而伴随像平凡、中心余权坐标下 `m_alpha` 与其伴随像的区分，以及 B2 负奇余根坐标的模二归约。这些案例分别检查纤维投影前后的差别与坐标奇性处理。^[grading.md:73-79]

来源包属于结构性阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。列出的测试缺口包括 `build` 的两处 `IndexOutOfRange`、多数溢出及分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。参见 [[紧致 grading 的测试锚点与覆盖边界]]。^[grading.md:9-13, grading.md:80-81, grading.md:91-94]

## Sources

- [grading.md](../../sources/grading.md)：紧致 grading：simple-imaginary 根的紧致性位向量。
