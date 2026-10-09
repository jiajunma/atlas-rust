---
title: 积分子系统（IntegralSubsystem）
summary: 由 integrality_simples 构造按生成元编号的单根访问数据，满足公共上下文与 Bruhat 生成需求，无需完整子系统根闭包。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:02:45.545Z"
updatedAt: "2026-10-09T19:33:43.900Z"
tags:
  - 积分子系统
  - 根系
aliases:
  - 积分子系统integralsubsystem
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 积分子系统（IntegralSubsystem）

`IntegralSubsystem` 是部分公共块模块中的积分子系统单根数据，对应上游 `subsystem::SubSystem`，由 `integrality_simples` 构造。它为积分子系统上的 Bruhat 区间与公共块构造提供所需的单根接口。^[partial-common-block.md:20-32]

## 实现范围

该实现只移植 [[公共上下文的生成元操作（CommonContext）|CommonContext]] 所需的按生成元编号访问器：`parent_nr_simple`、`simple`、`to_simple` 和 `reflection`。Bruhat 生成器不需要完整的子系统根闭包，因此此处的实现范围限于所需的单根数据与访问接口。^[partial-common-block.md:28-32]

## 公共上下文中的作用

`CommonContext` 将 KGB 层面的生成元作用转运到共轭的父单根上，提供 `status`、`cross`、`is_parity`、`down_cayley` 和 `up_cayley` 五个操作。其中，`cross` 先按反射词对 `x` 作 cross，利用 `pos_to_neg` 实根修正平移 `gamma_lambda`，再按父根反射。^[partial-common-block.md:44-55]

奇异性判定 `singular_flags` 同样逐一遍历子系统生成元，检查相应余根是否在参数 `gamma` 的分子上取零。^[partial-common-block.md:56-57]

## 与公共块构造的关系

`bruhat_below` 生成种子下方的 Bruhat 区间标准模参数列表，交由 [[公共块的构造与元素编号（PartialBlock）|PartialBlock]] 的 `build` 消费。输入区间按 `x` 排序，构造结束后再按 `(length, x, y)` 排序，使元素编号与 oracle 打印的行号一致。^[partial-common-block.md:35-38, partial-common-block.md:65-67]

完整公共块构造器 `build_full` 使用同一套 packet 构造处理 ambient-full 与真积分子系统两种情形；积分子系统为空时，所得块只有一个元素。^[partial-common-block.md:63-64]

## 证据边界

本页依据对 `partial_block.rs` 的结构性阅读，所读源码来自 dirty 工作区快照。来源中的上游位置转述自源码注释，未独立重读上游，行号可能随版本变化；来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[partial-common-block.md:9-16, partial-common-block.md:100-108]

## Sources

- [partial-common-block.md](partial-common-block.md) — 部分公共块：Bruhat 区间上的块构造。
