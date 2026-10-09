---
title: FiberToAdjoint 的按需投影
summary: FiberToAdjoint::apply 依次取得源规范代表、按根系数奇性执行模二投影并构造目标元素，不缓存稠密模二矩阵。
sources:
  - adjoint-fiber.md
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:43:29.337Z"
updatedAt: "2026-10-09T20:28:08.252Z"
tags:
  - 有限域线性代数
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
summary: FiberToAdjoint 由 fiber_map() 创建，每次应用依次取得源规范代表、执行模二伴随投影并构造目标元素；构造期验证子商下降，不保存稠密模二矩阵或缓存像。
sources:
  - cartan-fibers.md
  - adjoint-fiber.md
kind: concept
createdAt: "2026-10-09T14:43:29.337Z"
tags:
  - 伴随映射
  - 按需计算
  - Rust设计
aliases:
  - fibertoadjoint-的按需投影
provenanceState: extracted
---

# FiberToAdjoint 的按需投影

`FiberToAdjoint` 将源 `CartanFiber` 的元素映射到伴随 Cartan 纤维。它仅由 `AdjointCartanFiber::fiber_map()` 创建，字段全部私有；每次应用都从源元素的规范代表计算投影，不保存稠密模二矩阵或缓存像。^[cartan-fibers.md:138-141]

## 构造与下降保证

`AdjointCartanFiber` 持有源纤维的克隆、投影和目标纤维；每次调用 `fiber_map()`，会再次克隆这三者来构造 `FiberToAdjoint`。目标纤维构造完成后，构建流程调用 `source.validate_induced_map(&fiber, &projection)`，在映射对象可用之前完成下降验证。^[adjoint-fiber.md:33-37]

该验证一次性检查 ambient 映射是否保持子商的分子与分母关系；失败时返回 `CartanFiberMapDoesNotDescend`，以 `"numerator"` 或 `"denominator"` 区分违反的关系。高层随后按需应用映射，无须缓存稠密商坐标映射。参见 [[Ambient 映射的子商下降验证]]、[[伴随 Cartan 纤维的构建与下降验证]]。^[cartan-fibers.md:90-92]

底层 `AdjointProjection` 是从环境余特征格到伴随格的限制映射：余特征 `y` 的目标坐标由它与源单根的配对按单根顺序给出。该映射可以有中心核，不保证是同构；模二版本按根系数的奇性逐坐标计算，并通过 `ModTwoAmbientMap` 接口参与下降验证。参见 [[伴随根数据与余特征格投影]]。^[adjoint-fiber.md:41-46, cartan-fibers.md:105-112]

## 按需应用流程

`FiberToAdjoint::apply` 固定执行三个步骤：先调用源纤维的 `canonical_representative`，再调用 `projection.apply_mod_two`，最后通过目标纤维的 `element_from_ambient` 构造目标元素。每次调用都会重新计算模二投影。^[cartan-fibers.md:138-141]

源规范代表采用 low-pivot 约定，是确定性的环境向量：子商坐标第 `j` 位选择 `basis_representatives()[j]`，所有选中代表的 XOR 正好给出规范代表。目标 `element_from_ambient` 将环境向量转换为子商坐标；若向量不在目标分子空间内，则返回 `NotInModTwoSubspace`。参见 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[cartan-fibers.md:84-89]

## 来源绑定与资源约束

源元素必须属于下降验证所覆盖的纤维模型。`CartanFiberElement` 的相等性要求 `Arc::ptr_eq(model)` 且坐标相等，分别构造的纤维即使坐标相同，其元素也不相等；`ambient_fiber()` 的注释强调，只有该精确源纤维创建的元素才受下降证明覆盖。参见 [[Fiber 元素的模型来源绑定与规范代表]]。^[cartan-fibers.md:82-89, cartan-fibers.md:118-121]

伴随构建在目标矩阵分配前预检资源预算。设环境格秩为 \(n\)、半单秩为 \(r\)，投影操作预检量为 \(2n^2r\)；运行期的 `check_projection_work` 按“源秩 × 目标秩 × 向量数”计费，每次调用独立检查，不跨调用累计。预算乘加使用检查算术，溢出返回 `ArithmeticOverflow`。参见 [[伴随 fiber 的分配前资源预算]]。^[cartan-fibers.md:122-128, adjoint-fiber.md:52-58]

## 测试与证据边界

来源列出的测试锚点包括：A1 恒等投影保持生成元，且 `fiber_map.apply` 与先调用 `map_coweight`、再构造目标元素的路径一致；含中心余方向的例子中，中心方向映为单位元，根方向映为生成元，并保持加法。A1+A2 非对称例还逐坐标基向量验证了投影与源、目标对合作用的交织关系。^[cartan-fibers.md:143-149]

这些材料属于结构性源码阅读，不构成纤维层的数学验收。来源记录的维护工作未执行 Atlas、Cargo、测试或 benchmark；`cartan-fibers.md` 中的上游引用也仅转录自代码注释，未核对上游字节。参见 [[伴随纤维映射的测试证据与覆盖边界]]。^[cartan-fibers.md:9-12, cartan-fibers.md:165-171, adjoint-fiber.md:79-85]

## Sources

- [cartan-fibers.md](../../sources/cartan-fibers.md)
- [adjoint-fiber.md](../../sources/adjoint-fiber.md)
