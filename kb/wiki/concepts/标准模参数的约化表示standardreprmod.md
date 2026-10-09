---
title: 标准模参数的约化表示（StandardReprMod）
summary: 以 KGB 元素 x 与经 (1−θ)X* 约化并规范化的 gamma_lambda 表示参数，提供 build 与打印种子使用的 mod_reduce 两条构造路径。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:02:47.139Z"
updatedAt: "2026-10-09T22:41:08.116Z"
tags:
  - 表示论
  - 参数表示
aliases:
  - 标准模参数的约化表示standardreprmod
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 标准模参数的约化表示（StandardReprMod）
summary: 以 KGB 元素 x 和经 (1−θ)X* 约化、规范化的 gamma_lambda 表示参数，连接种子构造、生成元操作与部分公共块查找。
sources:
  - partial-common-block.md
kind: concept
tags:
  - 表示参数
  - 商格
  - Rust设计
aliases:
  - 标准模参数的约化表示standardreprmod
---

# 标准模参数的约化表示（StandardReprMod）

`StandardReprMod` 是部分公共块模块中的标准模参数表示，对应上游 `repr::StandardReprMod`。它由 KGB 元素 `x` 与经 $(1-\theta)X^*$ 约化并规范化的 `gamma_lambda` 组成；其值相等判定对应上游哈希表使用的相等判定。^[partial-common-block.md:23-27]

## 构造路径

`StandardReprMod` 有两条构造路径：`build` 经由 `RepContext::build_srm`，对应上游 `repr.cpp:61-67`；`mod_reduce` 经由 `RepContext::mod_reduce`，对应上游 `repr.cpp:52-58`。后者用于 `print_partial_block` wrapper 的种子计算。来源仅说明这些入口及用途，未展开内部约化算法。^[partial-common-block.md:23-27, partial-common-block.md:103-107]

## 生成元操作

[[公共上下文的生成元操作（CommonContext）|CommonContext]] 在 srm 层面提供 `status`、`cross`、`is_parity`、`down_cayley` 与 `up_cayley` 五个生成元操作，将 KGB 层面的生成元作用转运到共轭的父单根上。^[partial-common-block.md:33-34, partial-common-block.md:44-55]

其中，`cross` 先按反射词对 `x` 做 cross，并以 `pos_to_neg` 实根修正平移 `gamma_lambda`，再按父根反射。`up_cayley` 在提升后的 `gamma_lambda` 不满足奇偶条件时，加上 $\alpha_s/2$ 进行奇偶修正。^[partial-common-block.md:51-55]

## 在部分公共块中的作用

`bruhat_below` 生成种子的 Bruhat 区间 srm 列表，交给 [[公共块的构造与元素编号（PartialBlock）|PartialBlock::build]] 消费。输入区间按 `x` 排序；构造结束后，块元素再按 `(length, x, y)` 排序，使元素编号与 oracle 打印行号一致。^[partial-common-block.md:35-37, partial-common-block.md:65-67]

`PartialBlock::lookup` 将 srm 映射到对应的块元素，区间外返回 `None`，对应上游的 `UndefBlock`。块还提供 `element`、`x`、`y`、`length` 与 `gamma_lambda` 等返回 `Option` 的访问器。^[partial-common-block.md:69-73]

## 对偶与上下文边界

公共块的 `dual()` 返回 `BareBlock`。对偶块的行不是原块 context 中的标准模参数，因此参数池与查找表没有对偶对应物，不能将这一纯数据变换理解为原上下文内的参数重建。相关限制见 [[公共块对偶变换及 KL 递归的闭合限制]]。^[partial-common-block.md:83-96]

部分块中离开区间的未定义链接在对偶化后仍保持未定义。KL 递归只对链接闭合的源块（完整块）的对偶有效；部分区间的外出 complex ascent 可能变成 cross 未定义的 complex descent，因而被 `KlTable` 拒绝。^[partial-common-block.md:92-96]

## 证据范围

来源属于对 `partial_block.rs` 的结构性阅读，所读字节来自 dirty 工作区并记录于快照。它概述了 `StandardReprMod` 的组成、构造入口和块构造用途；`RepContext::mod_reduce` 与 `build_srm` 的详细展开属于后续来源包。^[partial-common-block.md:9-16, partial-common-block.md:100-107]

所列上游行号转述自源码注释，未独立重读上游，可能随版本变化而漂移。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；部分公共块的正确性属于其独立的 [[HPC 验收证据链]]。^[partial-common-block.md:10-11, partial-common-block.md:103-108]

## Sources

- [partial-common-block.md](../../sources/partial-common-block.md) — 部分公共块：Bruhat 区间上的块构造。
