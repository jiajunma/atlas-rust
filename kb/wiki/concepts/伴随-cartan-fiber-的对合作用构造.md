---
title: 伴随 Cartan fiber 的对合作用构造
summary: 伴随 fiber 构造先核对 datum 与源对合，再按列提取 simple-root 作用并转置得到余特征作用，复用 CartanFiber 构造并验证投影下降；转置等于逆转置的理由来自对合性注释。
sources:
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:43:25.257Z"
updatedAt: "2026-10-09T14:43:25.257Z"
tags:
  - 伴随Cartan-fiber
  - 对合
  - 对偶作用
aliases:
  - 伴随-cartan-fiber-的对合作用构造
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 伴随 Cartan fiber 的对合作用构造

伴随 Cartan fiber 的构造先把根对合写成伴随根数据上的作用矩阵，再构造目标格对合与 fiber，最后验证源 fiber 到目标 fiber 的投影能够下降到子商。该流程由 `AdjointCartanFiber::build` 实现。^[cartan-fibers.md:114-136]

## 伴随根数据与投影

`AdjointBasedRootDatum` 通过 `BasedRootDatum::standard` 使用源 Cartan 矩阵的克隆构造。代码文档声明，其 character 基是源 datum 的完整 simple-root 基，cocharacter 基是对应的 fundamental-coweight 基。^[cartan-fibers.md:101-103]

`AdjointProjection` 将源余特征 `y` 映到它与源 simple roots 的配对坐标。这个限制映射可以具有中心核，因此不应视为同构；相关格结构见 [[伴随根数据与余特征格投影]]。^[cartan-fibers.md:105-112]

## 构造前置条件

构造首先检查根系与根对合的 datum 是否一致，否则返回 `DatumMismatch`；随后检查源 fiber 的对合是否与给定根对合一致，否则返回 `CartanFiberInvolutionMismatch`。下降证明覆盖的是所保存的精确源 fiber 所铸造的元素，不能仅凭另一次构造所得 fiber 的值相等就替换来源。^[cartan-fibers.md:114-121]

目标矩阵分配前还会执行预算预检。令 \(r\) 为半单秩、\(n\) 为格秩，检查包括半单秩上限、持久条目界 \(16r^2+rn\) 和投影操作界 \(2n^2r\)。超限返回相应的 `AdjointFiberResourceLimit`；详见 [[伴随 fiber 的分配前资源预算]]。^[cartan-fibers.md:122-128]

## 根作用与余特征作用

通过门控后，`root_basis_action` 逐个处理 simple root：查找根编号、取得对合像，再提取像的 simple-root 坐标，并按**列**写入 `root_action`。`id_of`、`image` 或 `simple_coordinates` 任一步缺失，都返回 `InvalidRootAutomorphism`。^[cartan-fibers.md:129-131]

余特征作用通过 `coweight_action = transpose_square(&root_action)` 构造。代码注释给出的依据是：对偶作用应为逆转置，而 `RootInvolutionData` 已验证根作用为对合，因此逆转置等于转置。若记根作用矩阵为 \(A\)，则此处使用 \(A^{-T}=A^T\)；该论证在来源包中属于注释声明。^[cartan-fibers.md:131-133, cartan-fibers.md:153-154]

随后以目标 datum、`root_action` 和 `coweight_action` 调用 `LatticeInvolution::new`，再交给 `CartanFiber::build_owned` 构造目标 fiber。这里复用 [[Cartan fiber 的先分母后分子构造]]：先计算整系数负特征格并模二约化，再计算有限域核，形成子商
\[
\ker_{\mathbf F_2}(I+\theta_Y)\big/\operatorname{red}_2\ker_{\mathbf Z}(I+\theta_Y).
\]
^[cartan-fibers.md:16-21, cartan-fibers.md:72-80, cartan-fibers.md:134-136]

## 投影下降与按需应用

目标 fiber 建成后，`source.validate_induced_map(&fiber, &projection)` 一次性验证 ambient 投影满足分子、分母两项下降条件。失败分别以 `CartanFiberMapDoesNotDescend` 的 `"numerator"` 或 `"denominator"` 关系标识报告；相关机制见 [[Ambient 映射的子商下降验证]]。^[cartan-fibers.md:90-92, cartan-fibers.md:134-136]

之后的 `FiberToAdjoint::apply` 依次取得源元素的 canonical 代表、执行模二投影、由目标 `element_from_ambient` 构造像元素。实现不保存稠密模二映射矩阵或缓存像，而是每次按需投影，详见 [[FiberToAdjoint 的按需投影]]。^[cartan-fibers.md:138-141]

## 测试锚点与证据边界

A2 twisted 测试锚定伴随 weight 与 coweight 作用矩阵分别为
\[
A=\begin{pmatrix}-1&1\\0&1\end{pmatrix},
\qquad
A^T=\begin{pmatrix}-1&0\\1&1\end{pmatrix}.
\]
该例用于检查作用矩阵的推导，而非直接复用 root 行。A1+A2 非对称例还逐坐标基向量验证交错关系
\[
\operatorname{projection}(\theta(x))
=\theta_{\mathrm{adjoint}}(\operatorname{projection}(x)).
\]
^[cartan-fibers.md:145-149]

来源包属于结构性源码阅读，不构成 fiber 层的数学验收，也未核对上游引用的原始字节。本次知识维护未运行 Atlas、Cargo、测试或 benchmark；上述测试仅作为源码中的测试锚点报告。^[cartan-fibers.md:9-12, cartan-fibers.md:167-171]

已记录的覆盖缺口包括：构造首项 `DatumMismatch` 门控没有专门测试锚点，`AdjointProjection::from_source` 未显式检查 simple-root 数量与目标 rank 是否一致，部分直接索引依赖构造不变量。^[cartan-fibers.md:159-163]

## Sources

- [cartan-fibers.md](../../sources/cartan-fibers.md)
