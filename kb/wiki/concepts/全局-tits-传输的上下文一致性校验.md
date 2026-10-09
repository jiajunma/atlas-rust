---
title: 全局 Tits 传输的上下文一致性校验
summary: 构造及交叉作用通过 validate_context 检查根数据一致性，并核对 w·δ 的权与余权矩阵是否匹配存储对合，分别以 DatumMismatch 和 DistinguishedInvolutionMismatch 表达失败。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:46:29.534Z"
updatedAt: "2026-10-09T14:46:29.534Z"
tags:
  - 不变量
  - 根数据
  - Tits交叉作用
aliases:
  - 全局-tits-传输的上下文一致性校验
  - 全T传
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 全局 Tits 传输的上下文一致性校验

全局 Tits 传输通过 `validate_context` 检查 `GlobalTitsElement` 与给定 `InnerClass` 的一致性：既要求根数据匹配，也要求元素的 Weyl 作用与该内类的 distinguished involution 复合后，得到元素存储的根对合。该检查用于构造、单生成元交叉和 Weyl 字交叉。^[error-global-tits.md:69-79, error-global-tits.md:92-104]

## 校验对象与判定顺序

`GlobalTitsElement` 是 crate 内可见的载体，私有字段为 `torus_factor: RationalCoweight` 与 `twisted_involution: TwistedInvolution`。它保留包含中心坐标的完整有理余特征，环面坐标规范化到 `[0, 2)`；向纤维 mod-two 商的规约发生在后续阶段。相关表示见 [[全局 Tits 元素的精确有理环面表示]]。^[error-global-tits.md:60-65]

`validate_context` 首先检查 `weyl_action` 和 `root_involution` 的 datum 是否均与 `inner_class.datum()` 相等；任一不等，立即返回 `DatumMismatch`。根数据检查通过后，才进行 distinguished involution 的兼容性检查。^[error-global-tits.md:101-104]

令 \(w\) 为元素的 Weyl 作用，\(\delta\) 为内类的 distinguished involution。校验通过 `compose_matrices` 计算 \(w\cdot\delta\) 在 weight 与 coweight 上的矩阵，并分别与存储的对合矩阵比较；任一不等，返回 `DistinguishedInvolutionMismatch`。因此，匹配条件同时涉及权与余权两侧的作用，相关概念见 [[WeylAction 的对偶全格作用]]。^[error-global-tits.md:101-104]

## 各入口的检查时机

构造器 `new` 首先比较环面因子的维数与根数据的 `lattice_rank()`，不符时返回 `RankMismatch`；随后调用 `validate_context`，最后才复制并逐坐标模 2 规范化环面因子。传入的 twisted involution 原样存储。这一顺序使秩错误先于上下文错误报告。^[error-global-tits.md:69-73]

`crossed_generator` 每次调用都会重新校验上下文，随后检查生成元是否小于 `semisimple_rank`，越界返回 `IndexOutOfRange`；若无法取得对应单根的根类型，则返回 `InvalidRootAutomorphism`。方法采用 `&self -> Result<Self>`，原元素保持不变。^[error-global-tits.md:75-90]

`crossed_word` 在入口先校验一次上下文，再按切片顺序逐个调用 `crossed_generator`，因此每一步也会重新校验。非法生成元不会在入口统一预检，而是在折叠执行到该位置时返回 `IndexOutOfRange`。该前向执行约定见 [[Weyl 字的前向交叉作用顺序]]。^[error-global-tits.md:77-79, error-global-tits.md:92-97]

## 相关错误边界

上下文一致性检查之外，交叉传输还有独立的合法性门槛：虚根分支要求根与环面因子的配对为整数，否则返回 `InvalidStrongTorusFactor`；计算 distinguished involution 对生成元的像时，若像缺失或不在单根集合中，则返回 `InvalidBasedAutomorphism`。这些检查分别约束环面因子与生成元像。^[error-global-tits.md:81-85, error-global-tits.md:105-106]

矩阵复合、`WeylAction` 操作、`TwistedInvolution::new` 以及容量检查产生的错误通过 `?` 传播，来源包未枚举这些间接错误的具体变体。统一错误模型见 [[StructureError 统一错误分类学]]。^[error-global-tits.md:112-116]

## 测试与证据范围

测试 `rejects_rank_generator_datum_and_distinguished_mismatches` 对 `RankMismatch`、`IndexOutOfRange`、`DatumMismatch` 和 `DistinguishedInvolutionMismatch` 作出精确错误断言，其中 distinguished mismatch 使用 A2 交换对合构造。^[error-global-tits.md:136-138]

来源列出的未覆盖路径包括 `InvalidRootAutomorphism`、`InvalidBasedAutomorphism`、内部有理运算辅助函数的秩错误、容量检查失败以及非空 Weyl 字的错误传播。该材料属于结构性源码阅读，不声称 Tits 传输已获数学验收；本次知识维护也未执行测试或 benchmark。进一步的覆盖边界见 [[全局 Tits 传输的测试锚点与证据边界]]。^[error-global-tits.md:10-14, error-global-tits.md:140-142, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](../../sources/error-global-tits.md) — `StructureError` 错误分类学与全局 Tits 交叉作用传输层。
