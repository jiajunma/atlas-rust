---
title: Cartan fiber 的先分母后分子构造
summary: 构造先计算整数负特征格并模二约化作为分母，再按余特征作用矩阵的行计算 I+θ_Y 的有限域右核作为分子，最后建立子商；整数预算与有限域分配防护分别生效。
sources:
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:42:41.552Z"
updatedAt: "2026-10-09T14:42:41.552Z"
tags:
  - Cartan-fiber
  - 构造算法
  - 资源预算
aliases:
  - cartan-fiber-的先分母后分子构造
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cartan fiber 的先分母后分子构造

`CartanFiber` 是附着于 Cartan 对合的主有限分量群。其 `build_owned` 按固定顺序构造有限域子商：先求整数负特征格并模二归约为分母，再计算模二核作为分子，最后交给 `ModTwoSubquotient::new`。这一顺序使整数格预算检查先于有限域分子的计算执行。^[cartan-fibers.md:16-27, cartan-fibers.md:70-80]

## 子商模型

设 \(Y\) 为余特征格，\(\theta_Y\) 为其上的对合作用。实现采用的模型为
\[
\frac{\ker_{\mathbf F_2}(I+\theta_Y)}
{\operatorname{red}_2\ker_{\mathbf Z}(I+\theta_Y)}.
\]
源码文档声明它与 \(Y^\theta/(I+\theta_Y)Y\) 同构，但自然坐标不同；实现采用 low-pivot 归约基下的子商坐标，参见 [[Cartan fiber 的有限域子商模型]] 与 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[cartan-fibers.md:16-27]

## 构造顺序

### 1. 先构造分母

`build_owned` 首先调用 `negative_coweight_eigenspace(coweight_matrix, budget)?`，计算整数负特征格 \(\ker_{\mathbf Z}(I+\theta_Y)\)，再通过 `reduce_basis_mod_two` 得到分母。整数格预算在这一阶段执行；测试锚定了 rank 超限时返回 `IntegerLatticeResourceLimit { resource: "rank" }` 的行为。相关数学对象见 [[余特征作用的负特征整数子格]]。^[cartan-fibers.md:72-75]

分母不能替换为 \(I+\theta_Y\) 的朴素模二像。来源中的非对称测试例明确区分了这两个空间，并指出使用后者会得到错误的秩。^[cartan-fibers.md:94-97]

### 2. 再构造分子

随后，`mod_two_i_plus_coweight_kernel` 按存储的 coweight 作用矩阵逐**行**处理，不做转置：收集每行奇数条目的列索引，再追加该行自身的索引。`from_ones` 对重复索引执行 XOR 切换，因此恰好实现模二矩阵的对角 \(+I\) 项；最后调用 `right_kernel()`，得到 \(\ker_{\mathbf F_2}(I+\theta_Y)\)。^[cartan-fibers.md:76-78]

### 3. 装配子商

分子和分母就绪后，构造器调用 `ModTwoSubquotient::new(numerator, denominator)`。有限域部分不受整数格预算控制，而依靠 `try_reserve_exact` 与 `checked_*` 防护，相关失败以 `AllocationFailed` 或 `ArithmeticOverflow` 表达。^[cartan-fibers.md:79-80]

## 构造后的坐标与使用

`dimension()` 返回子商的 \(\mathbf F_2\) 维数，不枚举全部 \(2^{\mathrm{dim}}\) 个元素。元素的第 \(j\) 个坐标位选择 `basis_representatives()[j]`，所选代表的 XOR 即 low-pivot 约定下的确定性 ambient 代表，参见 [[Fiber 元素的模型来源绑定与规范代表]]。^[cartan-fibers.md:39-40, cartan-fibers.md:82-89]

`element_from_ambient` 通过 `to_coordinates` 检查输入是否属于分子，不属于时返回 `NotInModTwoSubspace`。`element_from_coweight_mod_two` 先检查 rank 再分配；它不要求输入余权在整数意义下被 \(\theta\) 固定，成员条件由模二分子决定。^[cartan-fibers.md:84-89]

伴随纤维的构造也复用这一流程：建立目标格对合后，调用 `CartanFiber::build_owned`，随后验证源到目标的投影能够沿子商下降。相关流程见 [[伴随 Cartan 纤维的构建与下降验证]]。^[cartan-fibers.md:129-136]

## 测试锚点与证据边界

来源列出的测试包括：rank-2 恒等环面的维数为 2、基为 `[e0,e1]`；\(-I\) 的纤维维数为 0；swap 对合的纤维平凡且拒绝 `e0`。rank-3 非对称例进一步锚定分子计算使用行而不转置，其模二核为 \(\langle e_0,e_2\rangle\)，最终子商基为 `[e2]`。^[cartan-fibers.md:94-97]

这些内容来自结构性源码阅读，不构成 fiber 层的数学验收。子商与抽象商的同构是源码注释中的声明；整数负特征格、模二归约及有限域子商的内部实现不在本来源包的读取范围内，本次知识维护也未执行测试或 benchmark。^[cartan-fibers.md:10-12, cartan-fibers.md:153-158, cartan-fibers.md:167-171]

## Sources

- [cartan-fibers.md](../../sources/cartan-fibers.md) — Cartan fiber 与伴随 Cartan fiber（`cartan_fiber.rs` / `adjoint_fiber.rs`）。
