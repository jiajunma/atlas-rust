---
title: 全局 Tits 传输的上下文一致性校验
summary: 构造与交叉作用校验 datum，并比较 w·δ 的权及余权作用矩阵与存储对合，分别报告来源错误和 distinguished 对合不匹配。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:46:29.534Z"
updatedAt: "2026-10-09T22:28:21.416Z"
tags:
  - Tits交叉作用
  - 上下文校验
aliases:
  - 全局-tits-传输的上下文一致性校验
  - 全T传
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 全局 Tits 传输的上下文一致性校验
summary: validate_context 先检查根数据一致性，再核对 w·δ 在权与余权上的矩阵是否匹配存储对合；构造、单生成元交叉及 Weyl 字交叉均执行此检查。
sources:
  - error-global-tits.md
kind: concept
tags:
  - Tits交叉作用
  - 根数据
  - 构造不变量
aliases:
  - 全局-tits-传输的上下文一致性校验
  - 全T传
provenanceState: extracted
---

# 全局 Tits 传输的上下文一致性校验

全局 Tits 传输通过 `validate_context` 检查 `GlobalTitsElement` 与给定 `InnerClass` 的一致性：元素的 Weyl 作用及根对合必须使用匹配的根数据，且 Weyl 作用与内类 distinguished involution 的复合必须在权、余权两侧均等于存储对合。构造、单生成元交叉和 Weyl 字交叉都调用这一检查。^[error-global-tits.md:69-79, error-global-tits.md:92-104]

## 校验对象与顺序

`GlobalTitsElement` 是 crate 内可见的载体，私有字段为 `torus_factor: RationalCoweight` 和 `twisted_involution: TwistedInvolution`。它保留包含中心坐标的完整有理余特征，环面坐标取 `[0, 2)` 中的典范代表元；向纤维 mod-two 商的规约在后续阶段进行。表示细节见 [[全局 Tits 元素的精确有理环面表示]]。^[error-global-tits.md:60-65]

`validate_context` 首先检查 `weyl_action` 和 `root_involution` 的 datum 是否都等于 `inner_class.datum()`。任一不等便返回 `DatumMismatch`，只有这一步通过后才比较对合矩阵。^[error-global-tits.md:101-104]

令 \(w\) 为元素的 Weyl 作用，\(\delta\) 为内类的 distinguished involution。函数通过 `compose_matrices` 计算 \(w\cdot\delta\) 的 weight 与 coweight 矩阵，并分别与存储对合的对应矩阵比较；任一不等便返回 `DistinguishedInvolutionMismatch`。检查同时涉及权与余权两侧，可结合 [[WeylAction 的对偶全格作用]] 阅读。^[error-global-tits.md:101-104]

## 各入口的检查时机

构造器 `new` 先比较 `torus_factor.dimension()` 与 `inner_class.datum().lattice_rank()`，不等时返回 `RankMismatch`；随后执行上下文校验，最后复制环面坐标、逐坐标模 2 规范化，并通过 `RationalCoweight::from_coordinates` 存储。传入的 twisted involution 原样保存，因此构造中的秩错误先于上下文错误报告。^[error-global-tits.md:69-73]

`crossed_generator` 每次调用都重新校验上下文，再检查生成元索引。若 `generator >= semisimple_rank`，返回 `IndexOutOfRange`；若无法取得对应单根的根类型，返回 `InvalidRootAutomorphism`。方法采用 `&self -> Result<Self>`，成功时返回新元素，原值不变。^[error-global-tits.md:75-90]

`crossed_word` 先在入口校验一次上下文，再按切片顺序逐个折叠 `crossed_generator`，因此每一步也会重新校验。非法生成元不在入口统一预检，而是在执行到相应位置时返回 `IndexOutOfRange`。这一顺序约定见 [[Weyl 字的前向交叉作用顺序]]。^[error-global-tits.md:77-79, error-global-tits.md:92-97]

## 与其他合法性检查的边界

上下文校验之外，交叉作用还检查环面因子和生成元像。虚根分支要求根与环面因子的配对为整数，否则返回 `InvalidStrongTorusFactor`；`distinguished_generator_image` 在生成元像缺失或像不属于单根集合时返回 `InvalidBasedAutomorphism`。^[error-global-tits.md:81-85, error-global-tits.md:105-106]

容量检查、矩阵复合、`WeylAction` 操作及 `TwistedInvolution::new` 的错误通过 `?` 传播；来源未枚举这些间接错误的具体变体。统一错误模型见 [[StructureError 统一错误分类学]]。^[error-global-tits.md:112-116]

## 测试与证据范围

测试 `rejects_rank_generator_datum_and_distinguished_mismatches` 精确断言 `RankMismatch`、`IndexOutOfRange`、`DatumMismatch` 和 `DistinguishedInvolutionMismatch` 四种错误；其中 distinguished mismatch 使用 A2 交换对合构造。^[error-global-tits.md:136-138]

来源列出的未覆盖分支包括 `InvalidRootAutomorphism`、`InvalidBasedAutomorphism`、`rational_pair`／`add_scaled_coroot` 内部的 `RankMismatch`、容量检查失败，以及非空 Weyl 字的错误传播。两个有理运算辅助函数的调用点已保证同秩。相关覆盖说明见 [[全局 Tits 传输的测试锚点与证据边界]]。^[error-global-tits.md:140-142]

本页依据结构性源码阅读，不构成错误覆盖面或 Tits 传输的数学验收。来源记录了维护者对照源码逐条核对改写的过程，但本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[error-global-tits.md:9-14, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](../../sources/error-global-tits.md) — StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）。
