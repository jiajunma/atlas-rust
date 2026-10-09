---
title: 伴随 Cartan 纤维的构建与下降验证
summary: 构建依次检查 datum、源对合和预算，以根作用转置构造余权对合，并验证投影可下降到纤维子商；本包不构成数学验收。
sources:
  - adjoint-fiber.md
kind: concept
createdAt: "2026-10-09T14:23:47.105Z"
updatedAt: "2026-10-09T22:10:47.959Z"
tags:
  - Cartan纤维
  - 对合
  - 子商映射
aliases:
  - 伴随-cartan-纤维的构建与下降验证
  - 伴C纤
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 伴随 Cartan 纤维的构建与下降验证
summary: AdjointCartanFiber 依次校验 datum、对合与资源预算，在伴随半单商上构造有限 F₂ Cartan 纤维，并验证源投影能够下降到目标子商；证据限于结构性源码阅读。
sources:
  - adjoint-fiber.md
kind: concept
tags:
  - Cartan纤维
  - 伴随半单商
  - 子商映射
aliases:
  - 伴随-cartan-纤维的构建与下降验证
  - 伴C纤
provenanceState: extracted
---

# 伴随 Cartan 纤维的构建与下降验证

`AdjointCartanFiber::build` 接收已验证的 root-datum 对合与已构建的 ambient `CartanFiber`，在伴随半单商上构造有限 \(\mathbb F_2\) Cartan 纤维，并通过下降验证钩子检查源到目标的投影。来源属于结构性源码阅读，不构成数学验收。^[adjoint-fiber.md:9-13, adjoint-fiber.md:25-37]

## 伴随根数据与投影出处

`AdjointBasedRootDatum` 仅使用源 datum 的 Cartan 矩阵调用 `BasedRootDatum::standard`。按文档注释，其 character 基是源单根全基，cocharacter 基是对应的基本余权基。整数投影 \(Y\to P^\vee\) 通过与源单根逐一配对、按单根顺序生成目标坐标；它可能有中心核，不能视为同构。参见 [[伴随根数据与余特征格投影]]。^[adjoint-fiber.md:17-19, adjoint-fiber.md:41-44]

`AmbientCoweight` 与 `AdjointCoweight` 均将裸 `Coweight` 绑定到 `Arc<AdjointProjectionModel>`。等值判定要求模型指针相同且坐标相等；这种绑定记录调用方的出处断言，并阻止跨投影复用。两次独立构建的投影会以 `DatumMismatch` 拒绝对方的绑定坐标，详见 [[余权坐标的投影出处绑定]]。^[adjoint-fiber.md:19-23]

## 构建顺序与对合作用

构建首先检查 datum 一致性，再检查源纤维对合一致性，对应错误分别为 `DatumMismatch` 与 `CartanFiberInvolutionMismatch`。随后调用 `validate_adjoint_build_budget`，通过预算预检后才克隆源单根、构造伴随 datum 与投影。^[adjoint-fiber.md:27-29]

`root_basis_action` 以根像的单根坐标为矩阵列，构造大小为半单秩的方阵；其中三处 `Option::None` 转换为 `InvalidRootAutomorphism`。余权作用取该矩阵的转置：对偶作用原本是逆转置，但 `RootInvolutionData` 已验证根作用为对合，因此逆转置等于转置。^[adjoint-fiber.md:29-32]

得到余权作用后，构建链调用 `LatticeInvolution::new` 和 `CartanFiber::build_owned`，整数格预算在此生效。目标纤维构造完成后，仍须执行 `source.validate_induced_map(&fiber, &projection)`，由实现 `ModTwoAmbientMap` 的投影参与下降验证。^[adjoint-fiber.md:33-35]

## 下降验证与映射应用

`validate_induced_map` 是来源明确标出的下降证明钩子，位于完整构建链的末尾。它连接环境格上的投影与纤维子商上的映射；相关概念见 [[Ambient 映射的子商下降验证]]。来源未展开该钩子的内部验证算法。^[adjoint-fiber.md:25-35]

`FiberToAdjoint::apply` 按照“源元素的 `canonical_representative` → mod-2 投影 → 目标纤维的 `element_from_ambient`”执行。私有的 `apply_mod_two` 根据根系数的奇性逐坐标翻转奇偶位，`bit(i) == None` 的分支按非 1 处理。^[adjoint-fiber.md:45-48]

`AdjointCartanFiber` 持有源纤维克隆、投影与目标纤维。每次调用 `fiber_map()` 都克隆三者并构造 `FiberToAdjoint`；实现不缓存稠密 mod-2 矩阵，而是在每次 `apply` 时计算投影。参见 [[FiberToAdjoint 的按需投影]]。^[adjoint-fiber.md:35-37]

## 资源预算与错误边界

`AdjointFiberBudget` 透传整数格预算，并在半单秩超过 `max_rank` 时以 `"semisimple rank"` 拒绝。设半单秩为 \(ss\)、源格秩为 \(lr\)，持久存储需求估算为 \(16ss^2+ss\,lr\)，包括 16 份方阵与克隆的源单根；构建期投影操作预检为 \(2lr^2ss\)。运行期 `check_projection_work` 按 `lr·target·vector_count` 检查，每次调用独立计费，没有跨调用累计。参见 [[伴随 fiber 的分配前资源预算]]。^[adjoint-fiber.md:52-57]

预算乘加使用 `checked_product` 与 `checked_sum`，溢出返回 `ArithmeticOverflow`；分配一律使用 `try_reserve_exact`，失败返回 `AllocationFailed`。错误面还包括 `RankMismatch`、`AdjointFiberResourceLimit`、`InvalidRootAutomorphism` 与 `InvalidInvolution`，其中转置输入非方阵对应 `InvalidInvolution`。非测试代码无 panic 或 assert。^[adjoint-fiber.md:58-62]

## 测试锚点与证据限制

来源列出九个测试，覆盖 A1 恒等对合的生成元投影与典范代表元、中心余权入核及可加性、A2 twisted 的作用矩阵、源纤维对合不匹配，以及跨投影坐标拒绝。另有 A1+A2 非对称作用的矩阵检查和逐坐标基交织关系 `map(θ·y) == θ_adjoint·map(y)`、rank-33 动态秩，以及持久存储和投影操作两条预算拒绝路径。^[adjoint-fiber.md:64-72]

未覆盖的错误分支包括 `AllocationFailed`、`ArithmeticOverflow`、`InvalidRootAutomorphism`、`InvalidInvolution` 和构建入口的 `DatumMismatch`；`coordinates`、`same_class`、`basis_representatives` 与 `identity` 的直接调用也未覆盖。参见 [[伴随纤维映射的测试证据与覆盖边界]]。^[adjoint-fiber.md:72-75]

`AdjointCartanFiber` 没有实现 `Eq`，因为值相等不能表达出处；来源强调使用 `ambient_fiber()` 的确切实例。`dimension()` 不枚举纤维元素。^[adjoint-fiber.md:76-77]

来源通过 `2026-10-06-adjoint-fiber.json` 记录 Git base、文件字节 SHA-256 与草案调用记录，并说明维护者已对照源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此测试锚点不应表述为本次运行验证结果。^[adjoint-fiber.md:81-85]

## Sources

- [adjoint-fiber.md](../../sources/adjoint-fiber.md)：伴随 Cartan 纤维：构建、投影与 mod-2 商（adjoint_fiber.rs）。
