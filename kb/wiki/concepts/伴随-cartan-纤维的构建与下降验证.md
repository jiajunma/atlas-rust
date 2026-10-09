---
title: 伴随 Cartan 纤维的构建与下降验证
summary: AdjointCartanFiber 构造目标纤维后调用 validate_induced_map 验证投影可下降到子商；该来源仅提供结构性阅读证据。
sources:
  - adjoint-fiber.md
kind: concept
createdAt: "2026-10-09T14:23:47.105Z"
updatedAt: "2026-10-09T20:28:15.345Z"
tags:
  - Cartan纤维
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
summary: AdjointCartanFiber 按 datum、对合与预算的顺序检查输入，构造伴随半单商上的有限 F₂ 纤维，并验证源到目标投影的下降条件；证据限于结构性源码阅读。
sources:
  - adjoint-fiber.md
kind: concept
tags:
  - Cartan纤维
  - 伴随半单商
  - 构建验证
aliases:
  - 伴随-cartan-纤维的构建与下降验证
  - 伴C纤
provenanceState: extracted
---

# 伴随 Cartan 纤维的构建与下降验证

`AdjointCartanFiber::build` 给定已验证的 root-datum 对合与已构建的 ambient `CartanFiber`，在伴随半单商上构造有限 \(\mathbb F_2\) Cartan 纤维，并验证源纤维到目标纤维的投影能够下降。来源属于结构性源码阅读，不代表数学验收；其中上游引用仅转录自代码注释。^[adjoint-fiber.md:9-13, adjoint-fiber.md:25-37]

## 伴随根数据与出处绑定

`AdjointBasedRootDatum` 仅使用源 datum 的 Cartan 矩阵调用 `BasedRootDatum::standard`。其 character 基对应源单根全基，cocharacter 基对应基本余权基。整数投影 \(Y\to P^\vee\) 将源余特征与各源单根配对，按单根顺序生成目标坐标；它可以有中心核，不能视为同构。参见 [[伴随根数据与余特征格投影]]。^[adjoint-fiber.md:17-19, adjoint-fiber.md:41-44]

`AmbientCoweight` 与 `AdjointCoweight` 均持有 `Arc<AdjointProjectionModel>` 和裸 `Coweight`。等值判定同时要求模型指针相同和坐标相等；这种绑定记录调用方断言，并阻止跨投影复用。两次独立构建得到的投影会以 `DatumMismatch` 拒绝对方的绑定坐标。参见 [[余权坐标的投影出处绑定]]。^[adjoint-fiber.md:19-23]

## 构建顺序与对合作用

构建首先检查 datum 一致性，再检查源纤维的对合一致性，分别可能返回 `DatumMismatch` 和 `CartanFiberInvolutionMismatch`。随后执行 `validate_adjoint_build_budget`，通过后才克隆源单根并构造伴随 datum 与投影。该顺序将一致性检查和预算预检置于投影构造之前。^[adjoint-fiber.md:27-29]

`root_basis_action` 以根像的单根坐标作为矩阵列，构造半单秩大小的方阵；其中三处 `Option::None` 均转换为 `InvalidRootAutomorphism`。伴随余权作用取该矩阵的转置：对偶作用通常是逆转置，但 `RootInvolutionData` 已验证根作用为对合，所以逆转置等于转置。参见 [[伴随 Cartan fiber 的对合作用构造]]。^[adjoint-fiber.md:29-32]

得到余权作用后，构建链调用 `LatticeInvolution::new` 与 `CartanFiber::build_owned`，整数格预算在此生效。目标纤维构造完成后，还需调用下降验证钩子，才能完成 `AdjointCartanFiber` 的构建。^[adjoint-fiber.md:33-35]

## 下降验证与映射应用

下降验证入口是 `source.validate_induced_map(&fiber, &projection)`，投影以 `ModTwoAmbientMap` 实现者身份传入。来源将这一步明确标为下降证明钩子；它是构建链的一部分，不能仅凭目标纤维已构造就省略。相关背景见 [[Ambient 映射的子商下降验证]]。^[adjoint-fiber.md:33-35]

应用映射时，`FiberToAdjoint::apply` 先获取源元素的 `canonical_representative`，再执行 mod-2 投影，最后调用目标纤维的 `element_from_ambient`。mod-2 投影按根系数的奇性逐坐标翻转奇偶位，其中 `bit(i) == None` 的分支按非 1 处理。^[adjoint-fiber.md:45-48]

`AdjointCartanFiber` 保存源纤维克隆、投影与目标纤维。每次调用 `fiber_map()` 都重新克隆三者，构造 `FiberToAdjoint`；实现不缓存稠密 mod-2 矩阵，而是在每次 `apply` 时计算投影。参见 [[FiberToAdjoint 的按需投影]]。^[adjoint-fiber.md:35-37]

## 资源预算与失败边界

`AdjointFiberBudget` 透传整数格预算，并在半单秩超过 `max_rank` 时以 `"semisimple rank"` 拒绝。设半单秩为 \(ss\)、源格秩为 \(lr\)，持久存储需求估算为 \(16ss^2+ss\,lr\)，涵盖 16 份方阵和克隆源单根；构建期投影操作预检为 \(2lr^2ss\)。运行期 `check_projection_work` 按 `lr·target·vector_count` 检查，每次调用独立计费，没有跨调用累计。参见 [[伴随 fiber 的分配前资源预算]]。^[adjoint-fiber.md:52-57]

预算乘加经 `checked_product` 与 `checked_sum` 检查，溢出返回 `ArithmeticOverflow`；分配使用 `try_reserve_exact`，失败返回 `AllocationFailed`。错误接口还包括 `RankMismatch`、`AdjointFiberResourceLimit`、`InvalidRootAutomorphism` 和 `InvalidInvolution`，后者包括转置输入非方阵的情形。非测试代码没有 panic 或 assert。^[adjoint-fiber.md:58-62]

## 测试证据与覆盖限制

来源列出九个测试，覆盖 A1 恒等对合的生成元投影与典范代表元、中心余权入核及可加性、A2 twisted 的作用矩阵、源纤维对合不匹配和跨投影坐标拒绝。另有 A1+A2 非对称作用的矩阵检查及逐坐标基交织关系 `map(θ·y) == θ_adjoint·map(y)`、rank-33 动态秩，以及持久存储和投影操作两条预算拒绝路径。^[adjoint-fiber.md:64-72]

未覆盖的错误分支包括 `AllocationFailed`、`ArithmeticOverflow`、`InvalidRootAutomorphism`、`InvalidInvolution` 与构建入口的 `DatumMismatch`；`coordinates`、`same_class`、`basis_representatives` 和 `identity` 的直接调用也未覆盖。参见 [[伴随纤维映射的测试证据与覆盖边界]]。^[adjoint-fiber.md:72-75]

`AdjointCartanFiber` 没有实现 `Eq`，因为值相等不能表达出处；来源要求使用 `ambient_fiber()` 的确切实例。`dimension()` 不枚举纤维元素。^[adjoint-fiber.md:76-77]

精确读取身份记录于来源所链接的 `2026-10-06-adjoint-fiber.json`，绑定 Git base、文件字节 SHA-256 与草案调用记录。维护者已对照源码逐条核对改写，但本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点不能表述为本次运行验证结果。^[adjoint-fiber.md:81-85]

## Sources

- [adjoint-fiber.md](../../sources/adjoint-fiber.md)：伴随 Cartan 纤维：构建、投影与 mod-2 商（adjoint_fiber.rs）。
