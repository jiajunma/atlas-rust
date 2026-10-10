---
title: 全局 Tits 传输的上下文一致性校验
summary: 构造及交叉作用检查 datum，并验证 w·δ 的权与余权作用矩阵均匹配存储对合，分别报告来源不一致或 distinguished 对合不匹配。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:46:29.534Z"
updatedAt: "2026-10-10T00:31:18.641Z"
tags:
  - Tits群
  - 不变量
aliases:
  - 全局-tits-传输的上下文一致性校验
  - 全T传
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 全局 Tits 传输的上下文一致性校验
summary: validate_context 先检查根数据一致性，再比较 w·δ 在权与余权上的矩阵是否匹配存储对合；构造、单生成元交叉及 Weyl 字交叉均执行此检查。
sources:
  - error-global-tits.md
kind: concept
tags:
  - Tits交叉作用
  - 上下文校验
  - 构造不变量
aliases:
  - 全局-tits-传输的上下文一致性校验
  - 全T传
provenanceState: extracted
---

# 全局 Tits 传输的上下文一致性校验

全局 Tits 传输通过 `validate_context` 检查 `GlobalTitsElement` 与给定 `InnerClass` 的一致性：元素的 Weyl 作用和根对合必须具有匹配的根数据，且 Weyl 作用与内类 distinguished involution 的复合必须在权、余权两侧均匹配存储对合。构造、单生成元交叉和 Weyl 字交叉都会执行此检查。^[error-global-tits.md:69-79, error-global-tits.md:92-104]

## 校验对象与顺序

`GlobalTitsElement` 是 crate 内可见的载体，私有字段为 `torus_factor: RationalCoweight` 和 `twisted_involution: TwistedInvolution`。它保留包含中心坐标的完整有理余特征，环面坐标取 `[0, 2)` 中的典范代表元；向纤维 mod-two 商的规约发生在后续阶段。表示细节见 [[全局 Tits 元素的精确有理环面表示]]。^[error-global-tits.md:60-65]

`validate_context` 首先检查 `weyl_action` 和 `root_involution` 的 datum 是否都等于 `inner_class.datum()`。任一不等便返回 `DatumMismatch`；只有根数据检查通过后，才比较对合矩阵。^[error-global-tits.md:101-104]

令 \(w\) 为元素的 Weyl 作用，\(\delta\) 为内类的 distinguished involution。函数通过 `compose_matrices` 计算 \(w\cdot\delta\) 的 weight 与 coweight 矩阵，并分别与存储对合的对应矩阵比较；任一不等便返回 `DistinguishedInvolutionMismatch`。相关作用背景见 [[WeylAction 的对偶全格作用]]。^[error-global-tits.md:101-104]

## 各入口的检查时机

构造器 `new` 先比较 `torus_factor.dimension()` 与 `inner_class.datum().lattice_rank()`，不等时返回 `RankMismatch`；随后执行上下文校验，最后复制环面坐标、逐坐标模 2 规范化，并通过 `RationalCoweight::from_coordinates` 构造存储值。传入的 twisted involution 原样保存，因此构造中的秩检查先于上下文检查。^[error-global-tits.md:69-73]

`crossed_generator` 每次调用都重新校验上下文，再检查生成元索引。若 `generator >= semisimple_rank`，返回 `IndexOutOfRange`；若 `root_involution().kind(simple_root)` 返回 `None`，则报 `InvalidRootAutomorphism`。交叉作用采用 `&self -> Result<Self>`，返回新元素，原值不变。^[error-global-tits.md:75-90]

`crossed_word` 先在入口校验一次上下文，再按切片顺序逐个折叠 `crossed_generator`，因此每一步也会重新校验。非法生成元不在入口统一预检，而是在执行到相应位置时返回 `IndexOutOfRange`。顺序约定见 [[Weyl 字的前向交叉作用顺序]]。^[error-global-tits.md:77-79, error-global-tits.md:92-97]

## 与其他合法性检查的边界

上下文一致性检查之外，虚根交叉分支要求根与环面因子的配对为整数，否则返回 `InvalidStrongTorusFactor`；`distinguished_generator_image` 在生成元像缺失或像不属于单根集合时返回 `InvalidBasedAutomorphism`。这些检查分别约束环面因子与生成元像。^[error-global-tits.md:81-85, error-global-tits.md:105-106]

容量检查 `try_capacity`、矩阵复合 `compose_matrices`、`WeylAction` 操作及 `TwistedInvolution::new` 的错误通过 `?` 传播，来源未枚举这些间接错误的具体变体。统一错误模型见 [[StructureError 统一错误分类学]]。^[error-global-tits.md:112-116]

## 测试与证据范围

测试 `rejects_rank_generator_datum_and_distinguished_mismatches` 精确断言 `RankMismatch`、`IndexOutOfRange`、`DatumMismatch` 和 `DistinguishedInvolutionMismatch` 四种错误；其中 distinguished mismatch 使用 A2 交换对合构造。^[error-global-tits.md:136-138]

未覆盖分支包括 `InvalidRootAutomorphism`、`InvalidBasedAutomorphism`、`rational_pair`／`add_scaled_coroot` 内部的 `RankMismatch`、`try_capacity` 失败路径，以及非空 Weyl 字的错误传播。两个有理运算辅助函数的调用点已保证同秩。相关说明见 [[全局 Tits 传输的测试锚点与证据边界]]。^[error-global-tits.md:140-142]

直接索引 `datum.simple_roots()[generator]` 等位置的越界保护依赖 `generator < semisimple_rank`；来源未核实 `semisimple_rank ≤ simple_roots().len()` 不变量，因此仍将其列为潜在 panic 面的阅读观察。^[error-global-tits.md:146-148]

本页依据结构性源码阅读，不构成错误覆盖面或 Tits 传输的数学验收。来源记录了维护者对照源码逐条核对改写的过程，但该次知识维护未执行 Atlas、Cargo、测试或 benchmark；上述测试描述是源码中的测试锚点，不是本次运行结果。^[error-global-tits.md:9-14, error-global-tits.md:118-120, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](../../sources/error-global-tits.md) — StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）。
