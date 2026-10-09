---
title: 积分子系统（IntegralSubsystem）
summary: 由 integrality_simples 构造积分子系统的单根数据，提供按生成元编号的访问器，满足公共上下文需求而无需完整根闭包。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:02:45.545Z"
updatedAt: "2026-10-09T15:02:45.545Z"
tags:
  - 积分子系统
  - 根系
  - 数据结构
aliases:
  - 积分子系统integralsubsystem
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 积分子系统（IntegralSubsystem）

`IntegralSubsystem` 是部分公共块模块中的积分子系统单根数据，对应上游 `subsystem::SubSystem`。它由 `integrality_simples` 构造，为积分子系统上的 Bruhat 区间与公共块构造提供所需的单根访问接口。^[partial-common-block.md:20-32]

## 实现范围

该实现只移植 [[公共上下文的生成元操作（CommonContext）|CommonContext]] 所需的按生成元编号访问器：`parent_nr_simple`、`simple`、`to_simple` 和 `reflection`。Bruhat 生成器不需要完整的子系统根闭包，因此这里的移植范围限于所需的单根数据与接口。^[partial-common-block.md:28-32]

## 与公共块构造的关系

`CommonContext` 利用子系统的生成元数据，将 KGB 层面的生成元作用转运到共轭的父单根上。相关操作包括 `status`、`cross`、`is_parity`、`down_cayley` 和 `up_cayley`；其中 `cross` 涉及按反射词对 `x` 作 cross、利用实根修正平移 `gamma_lambda`，再按父根反射。^[partial-common-block.md:44-55]

奇异性判定 `singular_flags` 也按子系统生成元逐一进行：检查相应余根是否在参数 `gamma` 的分子上取零。^[partial-common-block.md:56-57]

`bruhat_below` 生成种子的 Bruhat 区间标准模参数列表，交由 [[公共块的构造与元素编号（PartialBlock）|PartialBlock]] 的 `build` 消费。完整公共块构造器 `build_full` 使用同一套 packet 构造处理 ambient-full 与真积分子系统两种情形；积分子系统为空时，所得块只有一个元素。^[partial-common-block.md:35-38, partial-common-block.md:61-67]

## 证据边界

本页依据对 `partial_block.rs` 的结构性阅读，所记录的源码字节来自 dirty 工作区快照。来源中的上游位置转述自源码注释，未独立重读上游，行号可能随版本变化；来源未执行构建、测试或原版运行，不能据此宣称数学验收或性能结论。^[partial-common-block.md:9-16, partial-common-block.md:100-108]

## Sources

- [partial-common-block.md](partial-common-block.md) — 部分公共块：Bruhat 区间上的块构造。
