---
title: m_alpha 的模二归约与伴随投影
summary: m_alpha 是余根在 ambient fiber 中的模二像，其伴随像由根配对投影得到；坐标奇性归约必须正确处理负奇数。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:13.065Z"
updatedAt: "2026-10-09T14:50:13.065Z"
tags:
  - 模二线性代数
  - 余根
  - 伴随投影
aliases:
  - malpha-的模二归约与伴随投影
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# m_alpha 的模二归约与伴随投影

`CartanGradingData` 为 simple-imaginary 根保存两类数据：`m_alphas` 是余根在 ambient Cartan fiber 中的模二像，`adjoint_m_alphas` 是其在伴随 fiber 中的投影像，后者使用 fundamental-coweight 坐标。两者分别属于不同的 fiber 模型，不能混同。^[grading.md:29-33, grading.md:47-51]

## 模二归约

对每个 simple-imaginary 根 \(\alpha\)，`m_alpha` 由余根 \(\alpha^\vee\) 在 ambient fiber 中的 mod-2 像得到。坐标奇性判断采用 `*coordinate % 2 != 0`，因此负奇数也归约为置位；B2 测试以余根坐标 \((2,-1)\) 归约为 \((0,1)\) 锚定这一行为。^[grading.md:47-51]

同一收集流程还构造 `simple_mod_two`，保存单根坐标的奇性位。它与 `m_alpha` 的来源不同：前者来自根坐标，后者来自余根在 fiber 中的像。相关底层表示可参见 [[F₂ 上的位打包向量（ModTwoVector）]]。^[grading.md:47-51]

## 伴随投影与来源绑定

伴随投影按
\[
\Pi(y)_j=\langle\alpha_j,y\rangle
\]
计算；这一坐标向量正是对应的 bracket 向量。实现通过投影取得伴随 `m_alpha`，将配对计算保留在投影的单一实现中。相关背景见 [[伴随根数据与余特征格投影]]。^[grading.md:47-49]

`CartanGradingData::build` 刻意不接收独立的 ambient fiber 参数，而是使用 `AdjointCartanFiber::ambient_fiber` 构建 `m_alpha`。这样使用的是伴随映射下降验证所依赖的确切来源；仅凭值相等不足以表达 fiber 同一性。构造还检查 datum 一致性与 involution 一致性，分别以 `DatumMismatch` 和 `CartanFiberInvolutionMismatch` 拒绝不匹配输入。参见 [[CartanGradingData 与纤维来源一致性]]。^[grading.md:40-45]

## 与 grading 坐标的区别

`Grading` 的 bit `i` 索引 simple-imaginary 根列表的第 `i` 项，置位表示 noncompact；ambient 与 adjoint fiber 的模二坐标则索引全 datum 的格坐标或 simple roots。即使这些向量维数相同，也必须通过类型区分其含义，不能以维数检查替代坐标语义检查。参见 [[Grading 的位向量类型纪律]]。^[grading.md:17-27]

grading 的计算使用 canonical ambient representative 与单根奇性向量的配对。quasisplit 规范化令基点 grading 全为一，其余 grading 逐根取 `!dot`，即全一基点与配对值的 XOR；`grading_shifts[i]` 则由伴随基代表与各单根奇性向量的 \(\mathbf F_2\) 配对组成。参见 [[Quasisplit 规范化与 grading 的仿射线性求值]]。^[grading.md:35-38, grading.md:47-51]

## 测试锚点与证据边界

源码测试锚点包括：SC A1 中 `m_alpha` 非平凡而伴随像平凡、中心余权坐标下 `m_alpha` 与伴随像的区分，以及 B2 负奇余根坐标的模二归约。这些案例明确保留了 ambient 类与伴随像之间的差别。^[grading.md:73-79]

本来源包只记录结构性阅读与测试锚点，未执行构建、测试或原版运行，不构成数学验收或性能结论；其列出的未覆盖分支包括 `build` 的两处 `IndexOutOfRange` 和多数溢出、分配分支。^[grading.md:9-13, grading.md:80-81, grading.md:91-94]

## Sources

- [grading.md](grading.md)：紧致 grading：simple-imaginary 根的紧致性位向量。
