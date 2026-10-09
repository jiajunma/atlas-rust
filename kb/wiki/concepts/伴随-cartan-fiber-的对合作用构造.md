---
title: 伴随 Cartan fiber 的对合作用构造
summary: 构造时检查 datum 与源对合一致性，以根像的单根坐标建立作用矩阵，并利用对合性将其转置为余权作用后构造目标纤维。
sources:
  - adjoint-fiber.md
kind: concept
createdAt: "2026-10-09T14:43:25.257Z"
updatedAt: "2026-10-09T20:28:11.963Z"
tags:
  - Cartan纤维
  - 对合作用
aliases:
  - 伴随-cartan-fiber-的对合作用构造
  - 伴CF的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 伴随 Cartan fiber 的对合作用构造
summary: AdjointCartanFiber::build 按列构造单根基上的对合作用，利用对合性以转置得到余权作用，再构建目标 fiber 并验证投影下降。
sources:
  - adjoint-fiber.md
kind: concept
tags:
  - 伴随Cartan-fiber
  - 对合
  - 对偶作用
aliases:
  - 伴随-cartan-fiber-的对合作用构造
provenanceState: extracted
---

# 伴随 Cartan fiber 的对合作用构造

`AdjointCartanFiber::build` 接收已验证的 root-datum 对合与已构建的 ambient `CartanFiber`，在伴随半单商上构造有限 \(\mathbf F_2\) Cartan fiber。核心步骤是构造单根基上的根作用矩阵、转置得到余权作用、构建目标格对合与 fiber，最后验证源到目标的投影能够下降。^[adjoint-fiber.md:10-13, adjoint-fiber.md:25-37]

## 伴随根数据与投影

`AdjointBasedRootDatum` 仅使用源 datum 的 Cartan 矩阵调用 `BasedRootDatum::standard`。文档注释将其 character 基描述为源单根全基，cocharacter 基描述为对应的基本余权基。整数投影 \(Y\to P^\vee\) 按源单根顺序计算 `pair(root, y)`，以这些配对值作为目标坐标；该映射可以有中心核，不是一般意义上的同构。参见 [[伴随根数据与余特征格投影]]。^[adjoint-fiber.md:17-23, adjoint-fiber.md:39-44]

`AmbientCoweight` 与 `AdjointCoweight` 将裸 `Coweight` 绑定到 `Arc<AdjointProjectionModel>`。相等比较同时要求模型指针相同与坐标相同，以阻止跨投影复用；两次独立构建的投影会拒绝彼此的绑定坐标并返回 `DatumMismatch`。相关类型纪律见 [[余权坐标的投影出处绑定]]。^[adjoint-fiber.md:19-23]

## 构造前置检查

构造依次检查 datum 一致性、源 fiber 的对合一致性以及构建预算。前两项失败分别返回 `DatumMismatch` 与 `CartanFiberInvolutionMismatch`；检查通过后才构造投影，克隆源单根并建立伴随 datum。^[adjoint-fiber.md:25-30]

令 \(r\) 为半单秩、\(n\) 为格秩，预算检查包括半单秩上限、持久条目需求 \(16r^2+rn\) 与构建期投影操作需求 \(2n^2r\)。预算乘加使用受检算术，溢出返回 `ArithmeticOverflow`；整数格预算还会在后续 fiber 构建中生效。详见 [[伴随 fiber 的分配前资源预算]]。^[adjoint-fiber.md:33-34, adjoint-fiber.md:50-62]

## 根作用与余权作用

`root_basis_action` 将根的对合像写成单根坐标，以这些坐标为**列**构造半单秩方阵。该步骤中的三处可选值若缺失，均转为 `InvalidRootAutomorphism`。^[adjoint-fiber.md:29-31]

余权作用取根作用矩阵的转置。代码注释给出的依据是：对偶作用通常为逆转置，而 `RootInvolutionData` 已验证根作用为对合，因此若根作用矩阵为 \(A\)，此处有 \(A^{-T}=A^T\)。这一转置规则依赖已验证的对合性。^[adjoint-fiber.md:31-32]

得到作用矩阵后，构造链调用 `LatticeInvolution::new`，再通过 `CartanFiber::build_owned` 建立目标 fiber；整数格预算在这一阶段生效。^[adjoint-fiber.md:33-34]

## 投影下降与按需应用

目标 fiber 建成后，`source.validate_induced_map(&fiber, &projection)` 验证投影下降，投影以 `ModTwoAmbientMap` 实现者身份传入。`AdjointCartanFiber` 保存源 fiber 的克隆、投影和目标 fiber；相关机制见 [[Ambient 映射的子商下降验证]]。^[adjoint-fiber.md:34-37]

`fiber_map()` 每次调用都克隆这三项来构造 `FiberToAdjoint`，不缓存稠密模二矩阵。应用映射时，`FiberToAdjoint::apply` 依次取得源元素的 `canonical_representative`、执行模二投影，再调用目标的 `element_from_ambient`。模二投影按根系数的奇性逐坐标翻转奇偶位，详见 [[FiberToAdjoint 的按需投影]]。^[adjoint-fiber.md:35-37, adjoint-fiber.md:45-48]

## 测试锚点与证据边界

A2 twisted 测试锚定伴随 weight 与 coweight 作用矩阵分别为 \(A=\begin{pmatrix}-1&1\\0&1\end{pmatrix}\) 与 \(A^T=\begin{pmatrix}-1&0\\1&1\end{pmatrix}\)。A1+A2 非对称作用测试还逐坐标基验证交织关系 \(\operatorname{map}(\theta y)=\theta_{\mathrm{adjoint}}\operatorname{map}(y)\)。^[adjoint-fiber.md:66-71]

其他测试覆盖 A1 恒等作用的生成元投影、中心余权入核及可加性、异源 fiber 与跨投影坐标拒绝、rank-33 动态秩，以及持久条目和投影操作两条预算拒绝路径。未覆盖的错误分支包括 `AllocationFailed`、`ArithmeticOverflow`、`InvalidRootAutomorphism`、`InvalidInvolution` 和构造入口的 `DatumMismatch`；部分 fiber 查询接口也缺少直接调用测试。^[adjoint-fiber.md:66-75]

来源属于结构性源码阅读，不声称数学验收；上游引用仅转录自代码注释。本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述内容描述的是源码中的测试锚点，而非本次运行结果。^[adjoint-fiber.md:9-13, adjoint-fiber.md:79-85]

## Sources

- [adjoint-fiber.md](../../sources/adjoint-fiber.md)：伴随 Cartan 纤维：构建、投影与 mod-2 商（adjoint_fiber.rs）。
