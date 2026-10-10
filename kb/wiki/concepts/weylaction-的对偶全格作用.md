---
title: WeylAction 的对偶全格作用
summary: 权格与余权格反射矩阵采用受检 i128 运算和 i32 收窄，而矩阵复合以 i64 累加后未经检查截断为 i32。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:25.853Z"
updatedAt: "2026-10-10T00:56:15.139Z"
tags:
  - Weyl群
  - 整数算术
aliases:
  - weylaction-的对偶全格作用
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: WeylAction 的对偶全格作用
summary: WeylAction 携带根数据并保存 character 与 cocharacter 全格上的矩阵作用；反射构造使用受检算术，矩阵复合则存在未经检查的 i32 收窄。
sources:
  - weyl-layer.md
kind: concept
tags:
  - Weyl群
  - 对偶格
  - 整数算术
---

# WeylAction 的对偶全格作用

`WeylAction` 是 Weyl 群的矩阵级作用表示，同时作用于 character 与 cocharacter 两个全格，并携带所属根数据。它与以枚举根置换表示元素的词级组合层共同构成 [[Weyl 群的矩阵作用与词级元素双层结构]]；构造矩阵作用无需枚举整个 Weyl 群。^[weyl-layer.md:19-26]

## 数据与反射构造

`WeylAction` 保存 `datum: Arc<BasedRootDatum>`、`weight_matrix` 和 `coweight_matrix`，分别承载根数据及两个对偶格上的作用矩阵。`identity` 构造 `lattice_rank` 阶单位矩阵，覆盖完整格，而不局限于半单部分。根数据的背景见 [[BasedRootDatum：带基根数据与构造不变量]]。^[weyl-layer.md:19-21, weyl-layer.md:30-32]

`simple_reflection` 根据根与余根的对偶配对构造反射矩阵，条目为 $M_{ij}=\delta_{ij}-\mathrm{reflected}_i\,\mathrm{pairing}_j$。构造采用受检的 `i128` 算术，再收窄为 `i32`。`root_reflection` 支持任意枚举根上的反射，结果与根的符号无关，用于 Cayley/cross 分解重放中的强正交根反射。^[weyl-layer.md:31-36]

## 作用、复合与层间转换

`act` 与 `act_on_coweight` 分别执行两个格上的作用。`compose(right)` 的顺序是先作用 `right`，再作用 `self`。访问器包括 `datum`、`datum_arc`、`rank`、`matrix` 和 `coweight_matrix`；其中 `datum_arc` 会增加 `Arc` 引用计数。^[weyl-layer.md:37-39]

矩阵层与词级层通过 `RootSystem::action_permutation` 互查，`WeylElement::from_action` 将矩阵作用桥接为词级元素。词级层提供长度、下降、乘法、逆、扭曲共轭与按需约化词等操作，详见 [[WeylElement 的置换表示与长度下降不变量]]。^[weyl-layer.md:22-26]

全部作用的枚举与构造刻意分离：`WeylGroup::enumerate_actions(budget)` 接受显式基数预算。来源将结果描述为按 character-lattice 作用矩阵的字典序排列；重读记录指出，该顺序依赖 `CompactWeyl` 输出序及 rayon 保序收集，没有测试断言。参见 [[带基数预算的 Weyl 群作用枚举]]。^[weyl-layer.md:39-41, weyl-layer.md:52-55]

## 等值语义

“相等即矩阵作用相等”需要限定在 datum 值相同的情形。实现采用派生的逐字段比较，包含 datum 值；即使矩阵相同，datum 值不同的两个作用仍不相等。`Arc` 的 `PartialEq` 委派给内部值，因此这里不是按指针身份判等。详见 [[WeylAction 的等值与 datum 身份语义]]。^[weyl-layer.md:45-47]

## 算术与错误边界

反射构造与矩阵复合的算术保护程度不同。`compose_matrices` 使用 `i64` 累加，随后以 `sum as i32` 无检查截断；源码注释以 Weyl 矩阵条目的 Cartan 界解释其合理性，但该论证未形式化。供枚举热循环使用的 `compose_fast` 为 `pub(crate)`，完全无检查，违反前置条件存在 panic 风险。^[weyl-layer.md:48-51]

`simple_reflection` 对根使用 `.get`，越界返回 `IndexOutOfRange { upper_bound: semisimple_rank }`；余根则直接下标访问，长度不一致时存在潜在 panic 风险。`apply_matrix` 对不规则矩阵行复用 `InvalidRootAutomorphism` 错误变体。^[weyl-layer.md:33-35, weyl-layer.md:51-51]

## 测试锚点与证据范围

来源列出七个测试锚点：格秩为 2、半单秩为 1 的 A1 全格作用；A2 编织关系值相等；非对称 Cartan 下双作用保持配对；空半单部分的越界错误；A2 枚举预算 6 成功、预算 5 返回 `ResourceLimitExceeded`；同秩异 datum 返回 `DatumMismatch`；以及 `i32::MAX` 根坐标触发 `ArithmeticOverflow`。^[weyl-layer.md:59-62]

本页依据结构性源码阅读。来源未执行构建、测试或原版运行，上述测试锚点不代表本次执行结果，也不构成数学验收、性能或并行结论。Weyl 层的正确性属于其自身的 [[HPC 验收证据链]]，本来源不重述或扩展该证据范围。^[weyl-layer.md:9-15, weyl-layer.md:121-121]

## Sources

- [weyl-layer.md](../../sources/weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构。
