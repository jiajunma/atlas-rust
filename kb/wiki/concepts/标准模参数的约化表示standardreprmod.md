---
title: 标准模参数的约化表示（StandardReprMod）
summary: 以 KGB 元素 x 和经 (1−θ)X* 约化、规范化的 gamma_lambda 表示标准模参数，提供 build 与 mod_reduce 两条构造路径。
sources:
  - partial-common-block.md
kind: concept
createdAt: "2026-10-09T15:02:47.139Z"
updatedAt: "2026-10-09T15:02:47.139Z"
tags:
  - 标准模参数
  - 表示论
  - Rust设计
aliases:
  - 标准模参数的约化表示standardreprmod
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 标准模参数的约化表示（StandardReprMod）

`StandardReprMod` 是部分公共块模块中的标准模参数表示，对应上游 `repr::StandardReprMod`。它由 KGB 元素 `x` 与经过 $(1-\theta)X^*$ 约化并规范化的 `gamma_lambda` 组成；该值的相等判定对应上游哈希表使用的相等判定。^[partial-common-block.md:23-27]

## 构造路径

`StandardReprMod` 有两条构造路径：`build` 经由 `RepContext::build_srm` 构造；`mod_reduce` 经由 `RepContext::mod_reduce` 构造，后者用于 `print_partial_block` wrapper 的种子计算。来源将两者分别对应到上游 `repr.cpp:61-67` 与 `repr.cpp:52-58`。^[partial-common-block.md:23-27]

## 生成元操作

[[公共上下文的生成元操作（CommonContext）|CommonContext]] 在 srm 层面提供 `status`、`cross`、`is_parity`、`down_cayley` 与 `up_cayley` 五个生成元操作，将 KGB 层面的生成元作用转运到共轭的父单根上。其中，`cross` 先按反射词对 `x` 做 cross，再以 `pos_to_neg` 实根修正平移 `gamma_lambda`，随后按父根反射；`up_cayley` 在提升后的 `gamma_lambda` 不满足奇偶条件时，加入 $\alpha_s/2$ 的修正。^[partial-common-block.md:33-34, partial-common-block.md:44-55]

## 在部分公共块中的作用

`bruhat_below` 产生种子的 Bruhat 区间 srm 列表，供 [[公共块的构造与元素编号（PartialBlock）|PartialBlock]] 的 `build` 消费。输入区间按 `x` 排序，构造结束后块元素再按 `(length, x, y)` 排序，使元素编号与 oracle 打印行号一致。`lookup` 将 srm 映射到块元素，区间外的参数返回 `None`，对应上游的 `UndefBlock`。^[partial-common-block.md:35-37, partial-common-block.md:65-73]

## 对偶与证据边界

公共块的 `dual()` 返回 `BareBlock`：对偶块的行不是原块 context 中的标准模参数，因此原有参数池与查找表没有对偶对应物。解释这一行为时，应区分块图的数据变换与 `StandardReprMod` 所属上下文中的参数表示，参见 [[公共块对偶变换及 KL 递归的闭合限制]]。^[partial-common-block.md:83-96]

本来源只概述 `StandardReprMod` 的组成、构造入口及块构造用途；`RepContext::mod_reduce` 与 `build_srm` 的展开被列为后续来源包内容。来源未独立重读上游，所列上游行号来自源码注释；也未执行构建、测试或原版运行，不构成数学验收或性能证据。^[partial-common-block.md:103-108]

## Sources

- [partial-common-block.md](partial-common-block.md) — 部分公共块：Bruhat 区间上的块构造。
