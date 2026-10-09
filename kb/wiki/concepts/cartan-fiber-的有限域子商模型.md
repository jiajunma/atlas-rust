---
title: Cartan fiber 的有限域子商模型
summary: CartanFiber 采用 ker_F2(I+θ_Y)/red_2 ker_Z(I+θ_Y) 的子商坐标与 low-pivot 归约基；其与 Y^θ/(I+θ_Y)Y 的同构仅作为代码注释声明记录，未获本包数学验收。
sources:
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:42:41.891Z"
updatedAt: "2026-10-09T14:42:41.891Z"
tags:
  - Cartan-fiber
  - 数学模型
  - 有限域
aliases:
  - cartan-fiber-的有限域子商模型
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cartan fiber 的有限域子商模型

`CartanFiber` 是附着于一个 Cartan 对合的主有限分量群（primary finite component group）。实现采用余特征格上的有限域子商坐标，并使用 low-pivot 归约基；伴随映射、grading、弱实形式划分与强实层由后续类型分别承担。^[cartan-fibers.md:16-27]

## 数学模型

设 \(Y\) 为余特征格，\(\theta_Y\) 为其上的对合作用。代码文档给出的当前约定为
\[
F_\theta=
\frac{\ker_{\mathbf F_2}(I+\theta_Y)}
{\operatorname{red}_2\ker_{\mathbf Z}(I+\theta_Y)}.
\]
分子是模二作用 \(I+\theta_Y\) 的核，分母是整数负特征格 \(\ker_{\mathbf Z}(I+\theta_Y)\) 的模二约化。^[cartan-fibers.md:16-21, cartan-fibers.md:72-79]

文档注释声明，这个子商与抽象商 \(Y^\theta/(I+\theta_Y)Y\) 同构，但二者的自然坐标不同。实现选择前者的子商坐标，相关坐标机制可参见 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。该同构在本来源中属于注释声明，并非独立核验的数学结果。^[cartan-fibers.md:23-24, cartan-fibers.md:153-158]

## 构造顺序与矩阵约定

构造严格遵循[[Cartan fiber 的先分母后分子构造]]：先调用 `negative_coweight_eigenspace` 求整数负特征格，再通过 `reduce_basis_mod_two` 得到分母；随后计算有限域分子，最后调用 `ModTwoSubquotient::new(numerator, denominator)`。整数格预算在分母阶段执行，因此秩超限会先返回 `IntegerLatticeResourceLimit { resource: "rank" }`。^[cartan-fibers.md:72-80]

分子计算使用存储的 coweight 矩阵的**行，不作转置**。每行收集奇数条目的列索引，再追加该行自身的索引；`from_ones` 对重复索引执行 XOR 切换，恰好实现对角线上的 \(+I\)。随后通过 `right_kernel()` 得到 \(\ker_{\mathbf F_2}(I+\theta_Y)\)。^[cartan-fibers.md:76-79]

分母必须由整数核约化得到，不能替换为 \(I+\theta_Y\) 的朴素模二像。来源中的非对称测试专门区分这两种构造，后者会给出错误的秩。^[cartan-fibers.md:94-97]

## 元素、坐标与规范代表

`element_from_ambient` 将 ambient 模二向量转换为子商坐标；向量若不在分子中，返回 `NotInModTwoSubspace`。`element_from_coweight_mod_two` 先检查秩，再分配并打包坐标；它不要求输入余特征在整数格上被 \(\theta\) 固定，所需条件是其模二像属于分子。^[cartan-fibers.md:84-89]

`canonical_representative` 返回 low-pivot 约定下确定的 ambient 代表。商坐标的第 \(j\) 位对应 `basis_representatives()[j]`，所有被选中代表的 XOR 正好等于该规范代表。这将商坐标与 [[F₂ 商空间的确定性代表元与陪集判定]] 联系起来。^[cartan-fibers.md:87-89]

元素还绑定于铸造它的具体模型：`PartialEq` 同时要求 `Arc::ptr_eq(model)` 和坐标相等，因此分别构造的两个 fiber 即使给出相同坐标，其元素也不相等。此处的 opaque 设计用于保留来源身份，详见 [[Fiber 元素的模型来源绑定与规范代表]]。^[cartan-fibers.md:82-84]

## 维数、资源与映射下降

`lattice_rank()` 表示子商的 ambient 维数，`dimension()` 表示 fiber 的 \(\mathbf F_2\) 维数，计算维数不枚举全部 \(2^{\dim F_\theta}\) 个元素。有限域部分不使用整数格预算，而依靠 `try_reserve_exact` 与受检算术处理分配失败和算术溢出。^[cartan-fibers.md:39-40, cartan-fibers.md:79-80]

ambient 映射要诱导 fiber 之间的映射，必须分别满足分子与分母的下降条件。内部方法 `validate_induced_map` 一次性验证这两个条件，失败时以 `CartanFiberMapDoesNotDescend` 的 `"numerator"` 或 `"denominator"` 区分原因。高层随后按需应用映射，不缓存稠密商坐标矩阵；相关机制见 [[Ambient 映射的子商下降验证]] 与 [[FiberToAdjoint 的按需投影]]。^[cartan-fibers.md:90-92, cartan-fibers.md:138-141]

## 测试锚点与证据边界

来源记录的测试包括：rank-2 恒等环面的 fiber 维数为 2，基为 \([e_0,e_1]\)；\(-I\) 对合的 fiber 平凡；swap 对合的 fiber 也平凡，且输入 \(e_0\) 被拒绝。另一个 rank-3 非对称例验证按行、不转置的约定，其分子为 \(\langle e_0,e_2\rangle\)，子商基代表为 \([e_2]\)。^[cartan-fibers.md:94-97]

这些信息来自结构性源码阅读与既有测试锚点记录，不构成 fiber 层的数学验收。本来源未核对上游字节，也未覆盖底层模二类型及整数负特征格算法的内部实现；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cartan-fibers.md:9-12, cartan-fibers.md:153-158, cartan-fibers.md:167-171]

## Sources

- [cartan-fibers.md](../../sources/cartan-fibers.md) — Cartan fiber 与伴随 Cartan fiber（`cartan_fiber.rs` / `adjoint_fiber.rs`）。
