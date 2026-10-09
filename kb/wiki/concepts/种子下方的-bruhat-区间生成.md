---
title: 种子下方的 Bruhat 区间生成
summary: bruhat_below 经 Bruhat_generator::block_below 生成种子下方的标准模参数区间，作为部分公共块构造的输入。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:03:14.492Z"
updatedAt: "2026-10-09T15:03:14.492Z"
tags:
  - Bruhat序
  - 区间生成
  - 算法
aliases:
  - 种子下方的-bruhat-区间生成
  - 种B区
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 种子下方的 Bruhat 区间生成

种子下方的 Bruhat 区间生成是部分公共块构造的输入阶段：在积分子系统上，`bruhat_below` 生成种子下方 Bruhat 区间的标准模参数（srm）列表，再由 `PartialBlock::build` 消费。该机制属于 Atlas `print_partial_block` wrapper 背后的 Rust 移植。^[partial-common-block.md:20-38]

## 种子与积分子系统

区间中的参数用 `StandardReprMod` 表示，由 KGB 元素 `x` 与经 $(1-\theta)X^*$ 约化并规范化的 `gamma_lambda` 组成；其值相等对应上游哈希表的相等判定。构造路径包括经 `RepContext::build_srm` 调用的 `build`，以及经 `RepContext::mod_reduce` 调用的 `mod_reduce`；打印 wrapper 使用后一条路径计算种子。^[partial-common-block.md:23-27]

`IntegralSubsystem` 的单根数据由 `integrality_simples` 构造。此处只移植公共上下文需要的生成元编号访问器：`parent_nr_simple`、`simple`、`to_simple` 和 `reflection`；Bruhat 生成器不需要完整的子系统根闭包。^[partial-common-block.md:28-32]

## 生成与块构造

`bruhat_below` 对应上游 `Rep_table::Bruhat_below`，经 `Bruhat_generator::block_below` 产出区间的 srm 列表。`PartialBlock::build` 接收按 `x` 排序的列表，并在块构造结束时按 `(length, x, y)` 重新排序，使最终元素编号与 oracle 打印行号一致。输入列表的顺序与最终块编号因此需要分别理解，详见 [[公共块的构造与元素编号（PartialBlock）]]。^[partial-common-block.md:35-37, partial-common-block.md:65-67]

相关的 [[公共上下文的生成元操作（CommonContext）]] 将 KGB 层面的生成元作用转运到共轭的父单根上，提供 `status`、`cross`、`is_parity`、`down_cayley` 和 `up_cayley`。这些操作承载状态判定、交叉作用与 Cayley 变换；其中 `up_cayley` 在提升后的 `gamma_lambda` 不满足奇偶条件时加入 $\alpha_s/2$ 修正。^[partial-common-block.md:44-55]

## 区间边界与后续使用

部分块的 `lookup` 对区间外参数返回 `None`，对应上游 `UndefBlock`；`cross(s, z)` 返回 `None` 则表示链离开区间或尚未设置。`cayley(s, z)` 的像对依状态表示 imaginary ascent 的前向 Cayley 目标，或 real descent 的逆 Cayley 目标。使用区间结果时必须保留这些未定义链接的语义。^[partial-common-block.md:71-78]

部分区间不保证链接闭合，这会限制对偶块的 KL 递归。外出的 complex ascent 经对偶化可成为 cross 未定义的 complex descent，导致 `KlTable` 拒绝；来源说明，对偶后的 KL 递归只对链接闭合的源块（full block）有效。相关限制见 [[公共块对偶变换及 KL 递归的闭合限制]]。^[partial-common-block.md:92-96]

## 证据范围

本来源包提供结构性说明，`BruhatGenerator` 内部算法的展开属于后续来源包，因此不能据此补全具体遍历步骤或递归规则。它未执行构建、测试或原版运行，也不提供数学验收、性能或并行结论；partial block 的正确性由独立 HPC 证据链覆盖，本包不重述或扩展该证据。^[partial-common-block.md:9-11, partial-common-block.md:103-108]

## Sources

- [partial-common-block.md](partial-common-block.md) — 部分公共块：Bruhat 区间上的块构造。
