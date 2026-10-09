---
title: Fiber 元素的模型来源绑定与规范代表
summary: 元素相等要求模型 Arc 指针与商坐标同时相等，规范 ambient 代表由坐标选中的 low-pivot 补基代表异或得到。
sources:
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:42:58.940Z"
updatedAt: "2026-10-09T22:26:07.005Z"
tags:
  - Rust设计
  - 来源绑定
  - 商空间
aliases:
  - fiber-元素的模型来源绑定与规范代表
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Fiber 元素的模型来源绑定与规范代表
summary: CartanFiberElement 的相等性要求模型指针与商坐标同时相等；商坐标所选基代表的 XOR 给出 low-pivot 约定下的确定性环境代表。
sources:
  - cartan-fibers.md
kind: concept
tags:
  - Rust设计
  - 来源绑定
  - 商空间
aliases:
  - fiber-元素的模型来源绑定与规范代表
---

# Fiber 元素的模型来源绑定与规范代表

`CartanFiberElement` 将商坐标与创建它的 fiber 模型绑定。相等性同时依赖模型身份和坐标；规范代表则在所属模型内提供确定性的环境（ambient）向量表示。这里的 opaque 封装用于保留来源信息，而非隐藏坐标。^[cartan-fibers.md:82-89]

## 子商与坐标约定

`CartanFiber` 是附着于 Cartan 对合的有限分量群。令 \(Y\) 为余特征格，源码文档记录其采用的子商为 \(\ker_{\mathbb F_2}(I+\theta_Y)/\operatorname{red}_2\ker_{\mathbb Z}(I+\theta_Y)\)，实现选择 low-pivot 归约基作为坐标约定。文档称它与 \(Y^\theta/(I+\theta_Y)Y\) 同构，但自然坐标不同；来源包未独立验证这一同构声明。参见 [[Cartan fiber 的有限域子商模型]] 与 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[cartan-fibers.md:16-27, cartan-fibers.md:153-154]

## 模型来源与相等性

`CartanFiber` 持有 `Arc<CartanFiberModel>`，`CartanFiberElement` 保存模型与坐标。元素的 `PartialEq` 要求 `Arc::ptr_eq(model)` 成立且坐标相等。因此，分别调用 `build` 创建的模型中，即使元素具有相同坐标，元素也不相等。来源包还记录了 `CartanFiberMismatch` 的测试锚点。^[cartan-fibers.md:33-35, cartan-fibers.md:82-85]

## 元素构造与成员条件

`element_from_ambient` 通过 `to_coordinates` 将环境模二向量转换为子商坐标。输入必须属于分子 \(\ker_{\mathbb F_2}(I+\theta_Y)\)，否则返回 `NotInModTwoSubspace`。`element_from_coweight_mod_two` 在分配前先检查 rank；它不要求输入在整数格上满足 theta-fixed 条件，成员条件由模二分子决定。^[cartan-fibers.md:84-89]

## 规范代表与基坐标

`canonical_representative` 返回 low-pivot 约定下确定性的环境代表。`coordinates` 的第 \(j\) 位选择 `basis_representatives()[j]`，所有被选中的基代表按 XOR 相加，恰好得到该元素的规范代表。商坐标与环境代表因此是同一模型内相互对应的两种表示。参见 [[F₂ 商空间的确定性代表元与陪集判定]]。^[cartan-fibers.md:87-89]

来源记录的具体测试锚点包括：恒等 rank-2 环面的 fiber 维数为 2，基代表为 \([e_0,e_1]\)；swap 对合的 fiber 平凡，且环境向量 \(e_0\) 被拒绝；rank-3 非对称例的分子为 \(\langle e_0,e_2\rangle\)，商的基代表为 \([e_2]\)。这些例子分别展示了基坐标、分子成员条件和商代表的区别。^[cartan-fibers.md:94-97]

## 来源绑定与伴随映射

`validate_induced_map` 一次性验证环境映射能沿子商的分子、分母关系下降；失败时由 `CartanFiberMapDoesNotDescend` 的 `"numerator"` 或 `"denominator"` 标明关系。伴随 fiber 的构造检查源 fiber 的对合是否匹配，`ambient_fiber()` 的接口注释另强调：只有这个精确源 fiber 铸造的元素才被下降证明覆盖，值相等不足以替代来源身份。参见 [[Ambient 映射的子商下降验证]]。^[cartan-fibers.md:90-92, cartan-fibers.md:114-121]

`FiberToAdjoint` 只能经 `fiber_map()` 获得，字段全部私有。其 `apply` 依次取得源元素的规范代表、执行 `projection.apply_mod_two`、调用目标的 `element_from_ambient`。它不保存稠密模二矩阵或缓存像，而是每次按需投影；规范代表由此连接源商元素、环境映射与目标商元素。参见 [[FiberToAdjoint 的按需投影]]。^[cartan-fibers.md:138-141]

## 证据边界

来源包属于对 `cartan_fiber.rs` 与 `adjoint_fiber.rs` 的结构性阅读，不构成 fiber 层的数学验收。上述测试内容是源码中的测试锚点记录，本次知识维护未执行 Atlas、Cargo、测试或 benchmark；底层模二向量、子空间、子商及整数负特征格算法的内部实现也不在该包覆盖范围内。^[cartan-fibers.md:9-12, cartan-fibers.md:153-158, cartan-fibers.md:167-171]

## Sources

- [cartan-fibers.md](../../sources/cartan-fibers.md) — Cartan fiber 与伴随 Cartan fiber（`cartan_fiber.rs` / `adjoint_fiber.rs`）。
