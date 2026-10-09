---
title: FiberToAdjoint 的按需投影
summary: FiberToAdjoint 仅由 fiber_map() 创建，应用时依次获取源规范代表、执行模二伴随投影并构造目标元素，不保存稠密模二矩阵或缓存像。
sources:
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:43:29.337Z"
updatedAt: "2026-10-09T14:43:29.337Z"
tags:
  - 伴随映射
  - 按需计算
  - Rust设计
aliases:
  - fibertoadjoint-的按需投影
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# FiberToAdjoint 的按需投影

`FiberToAdjoint` 将源 `CartanFiber` 的元素映射到伴随 Cartan fiber。它只由 `AdjointCartanFiber::fiber_map()` 创建，字段全部私有；每次应用时从源元素的规范代表计算投影，不保存稠密 mod-two 矩阵或缓存像。^[cartan-fibers.md:60-66, cartan-fibers.md:138-141]

## 映射依据与构造保证

底层 `AdjointProjection` 是从 ambient 余特征格到伴随格的限制映射：余特征 `y` 的目标坐标由它与源 simple roots 的配对给出。该映射可以有中心核，因此不应视为同构；其 `apply_mod_two` 按奇偶计算目标坐标，并通过 `ModTwoAmbientMap` 接口连接源与目标 fiber。参见 [[伴随根数据与余特征格投影]]。^[cartan-fibers.md:105-112]

伴随 fiber 构造时先检查根数据与源 fiber 的对合是否匹配，并在目标矩阵分配前检查资源预算；随后构造伴随对合作用与目标 fiber，最后调用 `source.validate_induced_map(&fiber, &projection)`。该验证一次性检查 ambient 映射能否沿分子与分母关系下降，失败时以 `CartanFiberMapDoesNotDescend` 区分 `"numerator"` 或 `"denominator"`。参见 [[Ambient 映射的子商下降验证]]、[[伴随 Cartan 纤维的构建与下降验证]]。^[cartan-fibers.md:90-92, cartan-fibers.md:114-136]

## 按需应用流程

`FiberToAdjoint::apply` 的顺序固定为：取得源元素的 canonical 代表，调用 `projection.apply_mod_two`，再由目标 fiber 的 `element_from_ambient` 构造目标元素。高层依赖构造时完成的下降验证，在每次调用时直接计算投影。^[cartan-fibers.md:90-92, cartan-fibers.md:138-141]

源 canonical 代表采用 low-pivot 约定，是确定性的 ambient 向量：坐标第 `j` 位选择 `basis_representatives()[j]`，所有选中代表的 XOR 正好给出该规范代表。目标 `element_from_ambient` 则将 ambient 向量转换为子商坐标；若向量不在分子空间内，返回 `NotInModTwoSubspace`。参见 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[cartan-fibers.md:84-89]

## 元素来源约束

fiber 元素绑定到创建它的模型。`CartanFiberElement` 的相等性要求 `Arc::ptr_eq(model)` 且坐标相等，分别构造的 fiber 即使坐标相同，其元素也不相等。伴随 fiber 的 `ambient_fiber()` 注释进一步强调，下降证明只覆盖由该精确源 fiber 创建的元素。参见 [[Fiber 元素的模型来源绑定与规范代表]]。^[cartan-fibers.md:82-89, cartan-fibers.md:118-121]

## 测试与证据边界

来源列出的测试锚点包括：A1 恒等投影保持生成元，且 `fiber_map.apply` 与先做余特征投影、再构造目标元素的路径一致；含中心余方向的例子中，中心方向映为 identity，root 方向映为生成元，并保持加法。A1+A2 非对称例还逐坐标基向量验证了投影与源、目标对合作用的交错关系。^[cartan-fibers.md:143-149]

这些记录来自结构性源码阅读，不能据此声称 fiber 层已完成数学验收。该来源包未核对上游字节，本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[cartan-fibers.md:9-12, cartan-fibers.md:165-171]

## Sources

- [cartan-fibers.md](../../sources/cartan-fibers.md)
