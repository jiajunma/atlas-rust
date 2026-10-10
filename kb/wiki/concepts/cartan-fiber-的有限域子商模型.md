---
title: Cartan fiber 的有限域子商模型
summary: CartanFiber 采用 ker_F₂(I+θ_Y)/red₂ ker_Z(I+θ_Y) 的 low-pivot 子商坐标；与抽象商的同构仅为源码注释声明，未作数学验收。
sources:
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:42:41.891Z"
updatedAt: "2026-10-10T00:29:00.906Z"
tags:
  - Cartan纤维
  - 有限域线性代数
aliases:
  - cartan-fiber-的有限域子商模型
  - CF的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Cartan fiber 的有限域子商模型
summary: CartanFiber 采用整数负特征格模二约化作为分母的有限域子商，以低主元归约基提供确定性坐标；与抽象商的同构仅作为源码注释声明记录。
sources:
  - cartan-fibers.md
kind: concept
tags:
  - Cartan纤维
  - 有限域线性代数
aliases:
  - cartan-fiber-的有限域子商模型
provenanceState: extracted
---

# Cartan fiber 的有限域子商模型

`CartanFiber` 是附着于一个 Cartan 对合的主有限分量群（primary finite component group）。实现采用余特征格上的有限域子商，以 low-pivot（低主元）归约基确定坐标；伴随映射、grading、弱实形式划分与强实层分别由其他类型承担。^[cartan-fibers.md:16-27]

## 数学模型

设 $Y$ 为余特征格，$\theta_Y$ 为其上的对合作用。代码文档给出的当前约定为
$$
F_\theta=
\frac{\ker_{\mathbf F_2}(I+\theta_Y)}
{\operatorname{red}_2\ker_{\mathbf Z}(I+\theta_Y)}.
$$
分子是模二作用 $I+\theta_Y$ 的核；分母先在整数格上求负特征格，再进行模二约化。^[cartan-fibers.md:16-21, cartan-fibers.md:72-79]

文档注释声明，该子商与抽象商 $Y^\theta/(I+\theta_Y)Y$ 同构，但自然坐标不同。实现选择子商坐标，相关机制见 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。这一同构只是来源转录的注释声明，未获本包独立数学核验。^[cartan-fibers.md:23-24, cartan-fibers.md:153-158]

## 构造顺序与矩阵约定

构造遵循[[Cartan fiber 的先分母后分子构造]]：先调用 `negative_coweight_eigenspace`，再以 `reduce_basis_mod_two` 得到分母；随后计算有限域分子，最后调用 `ModTwoSubquotient::new(numerator, denominator)`。整数格预算在分母阶段执行，秩超限返回 `IntegerLatticeResourceLimit { resource: "rank" }`。^[cartan-fibers.md:72-80]

计算分子时，使用存储的 coweight 矩阵的**行，不作转置**。每行收集奇数条目的列索引，再追加该行自身的索引；`from_ones` 对重复索引执行 XOR 切换，实现对角线上的 $+I$。随后调用 `right_kernel()` 得到 $\ker_{\mathbf F_2}(I+\theta_Y)$。^[cartan-fibers.md:76-79]

分母不能替换为 $I+\theta_Y$ 的朴素模二像。来源中的非对称测试专门区分整数负特征格的模二约化与这一替代构造，后者会给出错误的秩。^[cartan-fibers.md:94-97]

## 元素、坐标与规范代表

`element_from_ambient` 将环境模二向量转换为子商坐标；若向量不在分子中，返回 `NotInModTwoSubspace`。`element_from_coweight_mod_two` 先检查秩，再分配并打包坐标；它不要求输入余特征在整数格上被 $\theta_Y$ 固定，要求的是其模二像满足分子条件。^[cartan-fibers.md:84-89]

`canonical_representative` 返回低主元约定下的确定性环境代表。商坐标的第 $j$ 位选择 `basis_representatives()[j]`，所有被选中代表的 XOR 恰好等于该规范代表。^[cartan-fibers.md:87-89]

元素绑定于构造它的具体模型：`PartialEq` 同时要求 `Arc::ptr_eq(model)` 与坐标相等。因此，分别构造的两个 fiber 即使产生相同坐标，其元素也不相等。这里的 opaque 设计表达来源身份，详见 [[Fiber 元素的模型来源绑定与规范代表]]。^[cartan-fibers.md:82-84]

## 维数、资源与映射下降

`lattice_rank()` 返回子商的环境维数，`dimension()` 返回 fiber 的 $\mathbf F_2$ 维数，不枚举全部 $2^{\dim F_\theta}$ 个元素。有限域部分不使用整数格预算，而通过 `try_reserve_exact` 与受检算术防护分配失败和算术溢出，对应 `AllocationFailed` 与 `ArithmeticOverflow`。^[cartan-fibers.md:39-40, cartan-fibers.md:79-80]

内部方法 `validate_induced_map` 一次性验证环境映射能否沿分子与分母两个关系下降。失败时，`CartanFiberMapDoesNotDescend` 以 `"numerator"` 或 `"denominator"` 区分原因；两种失败均有测试锚点。高层随后按需应用映射，不缓存稠密商坐标映射。^[cartan-fibers.md:90-92]

伴随映射采用“源元素的规范代表 → 模二投影 → 目标 fiber 元素”的流程。`FiberToAdjoint::apply` 每次按需计算，不保留稠密模二矩阵或缓存像，详见 [[FiberToAdjoint 的按需投影]]。^[cartan-fibers.md:138-141]

## 测试锚点与证据边界

来源记录的测试包括：rank-2 恒等环面的 fiber 维数为 2，基为 $[e_0,e_1]$；$-I$ 对合的 fiber 平凡；swap 对合的 fiber 也平凡，且输入 $e_0$ 被拒绝。rank-3 非对称例锚定按行、不转置的约定，其分子为 $\langle e_0,e_2\rangle$，子商基代表为 $[e_2]$。^[cartan-fibers.md:94-97]

本页依据结构性源码阅读与既有测试锚点记录，不构成 fiber 层的数学验收。来源未核对上游字节，也未覆盖底层模二类型、整数负特征格及整数基模二约化算法的内部实现；来源所述知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cartan-fibers.md:9-12, cartan-fibers.md:153-158, cartan-fibers.md:167-171]

## Sources

- [cartan-fibers.md](../../sources/cartan-fibers.md) — Cartan fiber 与伴随 Cartan fiber（`cartan_fiber.rs` / `adjoint_fiber.rs`）。
