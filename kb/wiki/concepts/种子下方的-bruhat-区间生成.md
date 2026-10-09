---
title: 种子下方的 Bruhat 区间生成
summary: bruhat_below 经 Bruhat_generator::block_below 生成种子下方的标准模参数列表，供 PartialBlock::build 消费。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:03:14.492Z"
updatedAt: "2026-10-09T22:41:29.595Z"
tags:
  - Bruhat序
  - 块构造
aliases:
  - 种子下方的-bruhat-区间生成
  - 种B区
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 种子下方的 Bruhat 区间生成
summary: bruhat_below 生成种子下方的标准模参数 Bruhat 区间，供 PartialBlock::build 构造部分公共块；输入按 x 排序，最终元素按 (length, x, y) 编号。
sources:
  - partial-common-block.md
kind: concept
tags:
  - Bruhat偏序
  - 部分公共块
aliases:
  - 种子下方的-bruhat-区间生成
---

# 种子下方的 Bruhat 区间生成

种子下方的 Bruhat 区间生成是部分公共块构造的输入阶段：在积分子系统上，`bruhat_below` 产出种子的 Bruhat 区间标准模参数（srm）列表，再交给 `PartialBlock::build` 消费。这一机制属于 Atlas `print_partial_block` wrapper 背后的 Rust 移植。^[partial-common-block.md:20-38]

## 种子与积分子系统

区间元素采用 `StandardReprMod` 表示，由 KGB 元素 `x` 与经 $(1-\theta)X^*$ 约化并规范化的 `gamma_lambda` 组成；其值相等对应上游哈希表的相等判定。两条构造路径分别为经 `RepContext::build_srm` 调用的 `build`，以及经 `RepContext::mod_reduce` 调用的 `mod_reduce`。打印 wrapper 使用后一条路径计算种子。^[partial-common-block.md:23-27]

`IntegralSubsystem` 的单根数据由 `integrality_simples` 构造。本模块只移植公共上下文所需的按生成元编号访问器 `parent_nr_simple`、`simple`、`to_simple` 和 `reflection`；Bruhat 生成器不需要完整的子系统根闭包。^[partial-common-block.md:28-32]

相关的 [[公共上下文的生成元操作（CommonContext）]] 将 KGB 层面的生成元作用转运到共轭的父单根上，提供 `status`、`cross`、`is_parity`、`down_cayley` 和 `up_cayley`。其中，`cross` 包含对 `gamma_lambda` 的实根平移修正；`up_cayley` 在提升后的 `gamma_lambda` 不满足奇偶条件时加入 $\alpha_s/2$ 修正。^[partial-common-block.md:44-55]

## 区间生成与最终编号

`bruhat_below` 对应上游 `Rep_table::Bruhat_below`，通过 `Bruhat_generator::block_below` 生成种子的区间 srm 列表。`PartialBlock::build` 接收按 `x` 排序的区间，并在构造结束时按 `(length, x, y)` 重新排序，使最终元素编号与 oracle 打印行号一致。因此，区间输入顺序与最终块编号采用不同的排序约定，详见 [[公共块的构造与元素编号（PartialBlock）]]。^[partial-common-block.md:35-37, partial-common-block.md:65-67]

## 区间边界与后续使用

部分块的 `lookup` 对区间外参数返回 `None`，对应上游 `UndefBlock`；`cross(s, z)` 返回 `None` 表示链离开区间或尚未设置。`cayley(s, z)` 返回 Cayley 像对，依状态表示 imaginary ascent 的前向 Cayley 目标，或 real descent 的逆 Cayley 目标。^[partial-common-block.md:71-78]

部分区间可能存在离开区间的链接，对偶化后这些链接仍保持未定义。外出的 complex ascent 会对偶化为 cross 未定义的 complex descent，`KlTable` 会拒绝此类输入。来源据此将对偶后的 KL 递归适用范围限定为链接闭合的源块（full block），详见 [[公共块对偶变换及 KL 递归的闭合限制]]。^[partial-common-block.md:92-96]

## 证据范围

本页依据结构性源码阅读。来源将 `BruhatGenerator` 内部算法的展开留待后续来源包，因此本页不补充具体遍历步骤或递归规则。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[partial-common-block.md:9-11, partial-common-block.md:103-107]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。部分公共块的正确性属于其独立的 HPC 证据链，本来源不重述或扩展该证据。^[partial-common-block.md:9-11, partial-common-block.md:108-108]

## Sources

- [partial-common-block.md](../../sources/partial-common-block.md) — 部分公共块：Bruhat 区间上的块构造。
