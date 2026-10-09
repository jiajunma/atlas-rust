---
title: Twisted involution 的规范约化表达式
summary: canonical_involution_expr 按外部生成元编号选择字典序最小的约化 twisted-involution expression，以 s 编码 cross、!s 编码 twisted conjugation，其终止性依赖输入确为当前 inner class 的 twisted involution 的 Weyl part。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:51:08.217Z"
updatedAt: "2026-10-09T14:51:08.217Z"
tags:
  - 扭曲对合
  - 约化表达式
  - 生成元编码
aliases:
  - twisted-involution-的规范约化表达式
  - TI的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Twisted involution 的规范约化表达式

`canonical_involution_expr` 为 twisted involution 的 Weyl 部分生成约化的 twisted-involution 表达式，并在 **外部生成元编号（EXTERNAL generator numbering）** 下选择字典序最小的表达式。该方法移植自上游 `TwistedWeylGroup::canonical_involution_expr`。^[inner-class.md:74-85]

## 输入与前置条件

在 inner class 的分解 $\theta=w\cdot\delta$ 中，$\delta$ 是 distinguished involution，$w$ 是 Weyl 元素。`canonical_involution_expr` 要求输入 `weyl` 确实是本 inner class 某个 twisted involution 的 Weyl 部分；这是调用方必须保证的契约，循环终止性依赖这一前提。相关成员判定见 [[InnerClass 成员判定与 twisted 分解]]。^[inner-class.md:53-59, inner-class.md:84-85]

## 逐步下降与规范选举

算法每一步按生成元编号递增的顺序选取第一个 descent，即上游的 **external-least** 规则；这里使用外部编号，而非内部重编号。随后通过 `hasTwistedCommutation` 区分该步采用 cross 还是 twisted conjugation。在输入满足前置条件时，每一步都会降低 twisted length，从而保证循环终止。^[inner-class.md:81-85]

## 带符号条目编码

表达式的每一步以一个带符号条目记录：普通条目 `s` 表示 cross，即左乘 `s`；按位取反的条目 `!s` 表示由 `s` 进行 twisted conjugation。上游将两种操作打包到同一个 `int` 中，打印代码也按这一约定解码；因此 `!s` 应理解为按位取反编码。相关展示约定见 [[对合表达式的打印约定]]。^[inner-class.md:76-81]

## 与对合规范化的区别

`canonical_involution_expr` 选择的是约化表达式。另一个方法 `canonicalize` 则使用三阶段算法，将输入搬运到 canonical representative：先使正实根与正虚根之和均为 dominant，再限制到与两个和都正交的简单生成元，最后使实际 involution 在残余复根子系统中保持正性。其返回生成元按执行顺序用于变换 $\sigma\mapsto s\cdot\sigma\cdot\delta(s)$；详见 [[Twisted involution 的三阶段规范化]]。^[inner-class.md:61-85]

## 证据范围

本页依据的来源包属于源码结构性阅读，未执行构建、测试或原版运行，不提供数学验收或性能结论。上游文件及行号来自 Rust 源码注释的转述，未独立重读上游，可能随版本变化而漂移。^[inner-class.md:9-15, inner-class.md:103-114]

## Sources

- [inner-class.md](inner-class.md)
