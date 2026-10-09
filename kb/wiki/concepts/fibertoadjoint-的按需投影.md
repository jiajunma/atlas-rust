---
title: FiberToAdjoint 的按需投影
summary: apply 依次取得源规范代表、按根系数奇性执行模二投影并构造目标纤维元素，不缓存稠密模二矩阵。
sources:
  - adjoint-fiber.md
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:43:29.337Z"
updatedAt: "2026-10-09T22:10:45.170Z"
tags:
  - 有限域
  - 纤维映射
  - 按需计算
aliases:
  - fibertoadjoint-的按需投影
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: FiberToAdjoint 的按需投影
summary: FiberToAdjoint 由 fiber_map() 创建，在构造期验证子商下降，每次应用时从源规范代表计算模二伴随投影，不保存稠密矩阵或缓存像。
sources:
  - adjoint-fiber.md
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:43:29.337Z"
tags:
  - 伴随映射
  - 有限域线性代数
  - 按需计算
aliases:
  - fibertoadjoint-的按需投影
---

# FiberToAdjoint 的按需投影

`FiberToAdjoint` 将源 `CartanFiber` 的元素映射到伴随 Cartan 纤维。它仅由 `AdjointCartanFiber::fiber_map()` 创建，字段全部私有；每次应用都从源元素的规范代表重新计算投影，不保存稠密模二矩阵或缓存像。^[cartan-fibers.md:138-141]

## 构造与下降验证

`AdjointCartanFiber` 持有源纤维的克隆、投影和目标纤维。目标纤维构造完成后，构建流程调用 `source.validate_induced_map(&fiber, &projection)`，验证投影能下降为纤维之间的映射；此后每次调用 `fiber_map()`，都克隆这三者来构造 `FiberToAdjoint`。参见 [[伴随 Cartan 纤维的构建与下降验证]]。^[adjoint-fiber.md:33-37]

下降验证一次性检查环境映射是否保持子商的分子与分母关系。失败时返回 `CartanFiberMapDoesNotDescend`，以 `"numerator"` 或 `"denominator"` 标明违反的关系。验证完成后，高层按需应用映射，无须缓存稠密商坐标映射。参见 [[Ambient 映射的子商下降验证]]。^[cartan-fibers.md:90-92]

底层 `AdjointProjection` 是余特征格到伴随格的限制映射：目标坐标依次由余特征与源单根的配对给出。该映射可以有中心核，并非同构；其模二版本按根系数的奇性计算各目标坐标，并通过 `ModTwoAmbientMap` 接口参与下降验证。参见 [[伴随根数据与余特征格投影]]。^[adjoint-fiber.md:41-46, cartan-fibers.md:105-112]

## 按需应用流程

`FiberToAdjoint::apply` 固定执行三步：调用源纤维的 `canonical_representative` 取得环境向量，经 `projection.apply_mod_two` 执行模二投影，再通过目标纤维的 `element_from_ambient` 构造目标元素。每次调用都会重新计算这一投影。^[cartan-fibers.md:138-141]

源规范代表采用 low-pivot 约定：子商坐标第 `j` 位选择 `basis_representatives()[j]`，所选代表的 XOR 恰好给出确定性的环境代表。目标 `element_from_ambient` 将环境向量转换为子商坐标；若向量不在目标分子空间内，则返回 `NotInModTwoSubspace`。参见 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[cartan-fibers.md:84-89]

## 来源绑定与资源约束

源元素须属于下降验证覆盖的纤维模型。`CartanFiberElement` 的相等性要求 `Arc::ptr_eq(model)` 且坐标相等，分别构造的纤维即使坐标相同，其元素也不相等；跨模型使用有 `CartanFiberMismatch` 测试锚点。`ambient_fiber()` 的注释强调，只有确切源纤维创建的元素才受下降证明覆盖。参见 [[Fiber 元素的模型来源绑定与规范代表]]。^[cartan-fibers.md:82-89, cartan-fibers.md:118-121]

伴随构建在目标矩阵分配前检查预算。设环境格秩为 \(n\)、半单秩为 \(r\)，投影操作预检量为 \(2n^2r\)，持久存储条目需求量为 \(16r^2+rn\)。运行期 `check_projection_work` 按“源秩 × 目标秩 × 向量数”检查，每次调用独立计量，不跨调用累计；预算乘加溢出返回 `ArithmeticOverflow`。参见 [[伴随 fiber 的分配前资源预算]]。^[cartan-fibers.md:122-128, adjoint-fiber.md:52-58]

## 测试与证据边界

来源列出的测试锚点包括：A1 恒等投影保持生成元，且 `fiber_map.apply` 与先执行 `map_coweight`、再构造目标元素的路径一致；含中心余方向的例子中，中心方向映为单位元，根方向映为生成元，并保持加法。A1+A2 非对称例还逐坐标基向量验证投影与源、目标对合作用的交织关系。^[cartan-fibers.md:143-149]

这些材料属于结构性源码阅读，不构成纤维层的数学验收。来源记录的知识维护未执行 Atlas、Cargo、测试或 benchmark；上游引用仅转录自代码注释，未独立核对上游字节。参见 [[伴随纤维映射的测试证据与覆盖边界]]。^[cartan-fibers.md:9-12, cartan-fibers.md:165-171]

## Sources

- [adjoint-fiber.md](../../sources/adjoint-fiber.md) — 伴随 Cartan 纤维：构建、投影与 mod-2 商。
- [cartan-fibers.md](../../sources/cartan-fibers.md) — Cartan fiber 与伴随 Cartan fiber。
