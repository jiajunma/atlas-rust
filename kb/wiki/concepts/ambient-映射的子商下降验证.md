---
title: Ambient 映射的子商下降验证
summary: validate_induced_map 一次性检查源分子和分母分别映入目标对应子空间，失败时区分两类关系，使后续映射可按需计算。
sources:
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:43:01.802Z"
updatedAt: "2026-10-09T22:26:17.430Z"
tags:
  - 子商映射
  - 构造校验
aliases:
  - ambient-映射的子商下降验证
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Ambient 映射的子商下降验证
summary: validate_induced_map 一次性验证环境映射保持子商的分子与分母关系，使伴随 fiber 映射能够在绑定的模型上按需投影，而不缓存稠密商坐标矩阵。
sources:
  - cartan-fibers.md
kind: concept
tags:
  - 诱导映射
  - 子商
  - 不变量验证
aliases:
  - ambient-映射的子商下降验证
provenanceState: extracted
---

# Ambient 映射的子商下降验证

Ambient 映射的子商下降验证用于检查环境空间上的映射能否诱导 Cartan fiber 之间的映射。`CartanFiber::validate_induced_map` 是 crate 内部接口，一次性验证分子与分母两层关系；高层随后按需应用映射，不缓存稠密商坐标映射。^[cartan-fibers.md:90-92]

## 子商模型与下降条件

[[Cartan fiber 的有限域子商模型|Cartan fiber]] 采用子商 \(N/D\)，其中 \(Y\) 为余特征格，\(N=\ker_{\mathbf F_2}(I+\theta_Y)\)，\(D=\operatorname{red}_2\ker_{\mathbf Z}(I+\theta_Y)\)。实现使用 low-pivot 归约基；源码注释声明它与 \(Y^\theta/(I+\theta_Y)Y\) 同构，但自然坐标不同。^[cartan-fibers.md:16-24]

对于源子商 \(N_s/D_s\)、目标子商 \(N_t/D_t\) 和 ambient 映射 \(f\)，两层下降关系对应 \(f(N_s)\subseteq N_t\) 与 \(f(D_s)\subseteq D_t\)。验证失败分别返回 `CartanFiberMapDoesNotDescend { relation: "numerator" }` 或 `CartanFiberMapDoesNotDescend { relation: "denominator" }`，两类错误均有测试锚点。^[cartan-fibers.md:90-92]

`validate_induced_map` 接收目标 `CartanFiber` 与实现 `ModTwoAmbientMap` 的映射。伴随投影 `AdjointProjection` 实现该 trait，连接 Cartan fiber 与伴随 fiber 两层实现。^[cartan-fibers.md:50-50, cartan-fibers.md:105-112]

## 构造顺序与模型绑定

在[[伴随 Cartan 纤维的构建与下降验证]]中，构造器依次检查根数据一致性、源 fiber 的对合一致性及资源预算，再构造伴随投影、目标对合与目标 fiber，最后调用 `source.validate_induced_map(&fiber, &projection)`。目标 fiber 复用[[Cartan fiber 的先分母后分子构造]]流程。^[cartan-fibers.md:114-136]

下降证明覆盖构造时使用的精确源 fiber 所铸造的元素。元素相等性要求模型满足 `Arc::ptr_eq` 且坐标相等，因此不同 `build` 产生的同坐标元素并不相等。该[[Fiber 元素的模型来源绑定与规范代表|模型来源绑定]]限定了验证结果的适用范围。^[cartan-fibers.md:82-89, cartan-fibers.md:118-121]

## 验证后的按需应用

[[FiberToAdjoint 的按需投影|FiberToAdjoint]] 只能由 `fiber_map()` 创建，字段均为私有。其 `apply` 依次取得源元素的 canonical ambient 代表、调用 `projection.apply_mod_two`，再通过目标的 `element_from_ambient` 构造结果；实现不保留稠密 mod-two 矩阵，也不缓存像。^[cartan-fibers.md:138-141]

canonical 代表采用 low-pivot 约定：商坐标第 \(j\) 位选择 `basis_representatives()[j]`，所选代表的 XOR 即为 canonical 代表。目标端的 `element_from_ambient` 通过 `to_coordinates` 转换坐标；输入不在分子中时返回 `NotInModTwoSubspace`。^[cartan-fibers.md:84-89]

伴随投影以源余特征与源 simple roots 的配对作为目标坐标，源码注释明确允许中心核。因此，通过下降验证不意味着投影是同构。测试包含中心方向映为 identity、根方向映为生成元且保持加法的情形。^[cartan-fibers.md:105-106, cartan-fibers.md:143-145]

## 测试与证据边界

除分子、分母下降失败的测试锚点外，来源还记录了 A1 上 fiber 映射与整余特征投影路径的一致性，以及 A1+A2 非对称例逐坐标验证的交错关系 \(\operatorname{projection}(\theta(x))=\theta_{\mathrm{adjoint}}(\operatorname{projection}(x))\)。这些是特定测试的源码记录。^[cartan-fibers.md:90-92, cartan-fibers.md:143-149]

来源属于结构性阅读，不构成 fiber 层的数学验收；上游引用转录自代码注释，未核对上游字节。`ModTwoSubquotient`、`ModTwoSubspace` 与 `ModTwoVector` 的内部实现不在本包范围内，本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[cartan-fibers.md:9-12, cartan-fibers.md:151-158, cartan-fibers.md:167-171]

## Sources

- [cartan-fibers.md](../../sources/cartan-fibers.md) — Cartan fiber 与伴随 Cartan fiber（`cartan_fiber.rs` / `adjoint_fiber.rs`）。
