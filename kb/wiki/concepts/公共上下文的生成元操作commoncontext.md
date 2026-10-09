---
title: 公共上下文的生成元操作（CommonContext）
summary: 将 KGB 生成元作用转运到共轭父单根，实现状态判定、cross、奇偶判定及双向 Cayley 变换，并处理参数修正与奇异标志。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:03:13.909Z"
updatedAt: "2026-10-09T15:03:13.909Z"
tags:
  - 生成元作用
  - Cayley变换
  - 表示论
aliases:
  - 公共上下文的生成元操作commoncontext
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 公共上下文的生成元操作（CommonContext）

`CommonContext` 对应上游 `repr::common_context`，为标准模参数（srm）提供生成元操作，是积分子系统上的 Bruhat 区间与[[公共块的构造与元素编号（PartialBlock）|部分公共块]]构造机制的一部分。其核心作用是将 KGB 层面的生成元作用转运到共轭的父单根上。^[partial-common-block.md:20-38, partial-common-block.md:44-46]

## 参数与积分子系统

操作对象 `StandardReprMod` 由 KGB 元素 `x` 和经 $(1-\theta)X^*$ 约化、规范化的 `gamma_lambda` 组成，其值相等对应上游哈希表中的相等判定。两条构造路径分别经 `RepContext::build_srm` 与 `RepContext::mod_reduce`；后者用于打印 wrapper 的种子计算。^[partial-common-block.md:23-27]

`IntegralSubsystem` 通过 `integrality_simples` 构造单根数据，为公共上下文提供按生成元编号访问的 `parent_nr_simple`、`simple`、`to_simple` 与 `reflection`。该移植只提供公共上下文所需的访问器；Bruhat 生成器不需要完整的子系统根闭包。^[partial-common-block.md:28-32]

## 五个生成元操作

### `status`：状态与辅助标志

`status` 返回 `(KgbStatus, bool)`。布尔值的含义依赖根类型：real 情形表示 `isDoubleCayleyImage`，complex 情形表示 `isDescent`，非紧致 imaginary 情形用于区分 type-1 与 type-2 cross-move。因此，解释该标志时必须结合返回的根状态。^[partial-common-block.md:48-50]

### `cross`：交叉作用与平移修正

`cross` 先按反射词对 `x` 执行 cross，再利用 `pos_to_neg` 实根修正平移 `gamma_lambda`，最后按父根反射。这一操作同时涉及 KGB 元素与参数向量的更新。^[partial-common-block.md:51-52]

### `is_parity`、`down_cayley` 与 `up_cayley`

另外三个操作是奇偶判定 `is_parity`、向下 Cayley 操作 `down_cayley` 与向上 Cayley 操作 `up_cayley`。其中，`up_cayley` 在提升后的 `gamma_lambda` 不满足奇偶条件时，加上 $\alpha_s/2$ 进行奇偶修正；来源未进一步展开另外两个操作的算法步骤。^[partial-common-block.md:53-55]

## 奇异生成元标志

除上述五个操作外，`singular_flags` 逐一检查积分子系统生成元，判断其余根是否在 `gamma` 的分子上取零，对应上游 `common_block::singular`。在部分块的 `survives` 判定中，元素存活的条件是：不存在既为 singular、又是该元素 descent 的生成元。^[partial-common-block.md:56-57, partial-common-block.md:79-81]

## 调用契约与证据边界

这些操作依赖上游规定的调用方契约，模块不重复检查相关断言。错误处理约定区分调用前提与内部一致性：真正的内部不一致通过 [[StructureError 统一错误分类学|StructureError]] 报告，而非 panic。^[partial-common-block.md:40-42, partial-common-block.md:59-59]

本页依据结构性源码阅读材料。材料中的上游行号转述自源码注释，未独立重读上游，可能随版本变化而漂移；该材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[partial-common-block.md:9-11, partial-common-block.md:103-108]

## Sources

- [partial-common-block.md](partial-common-block.md)：部分公共块：Bruhat 区间上的块构造。
