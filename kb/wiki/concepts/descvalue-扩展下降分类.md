---
title: DescValue 扩展下降分类
summary: DescValue 以 One、Two、Three 三族组织 32 种下降类型，通过奇偶枚举值、类型谓词、生成元长度与链接数描述局部结构。
sources:
  - extended-block.md
kind: concept
createdAt: "2026-10-09T14:47:26.925Z"
updatedAt: "2026-10-09T14:47:26.925Z"
tags:
  - 扩展块
  - 下降分类
  - 局部结构
aliases:
  - descvalue-扩展下降分类
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# DescValue 扩展下降分类

`DescValue` 是扩展块中的 32 值下降分类，按 `One*`、`Two*`、`Three*` 三族排列。扩展块取普通块的 $\delta$-不动部分，并将生成元折叠进 $\delta$-轨道；`DescValue` 为这一结构提供下降、像的数量及链接性质等分类查询。^[extended-block.md:19-33]

## 分类与谓词

`is_descent` 以枚举值的奇偶性判定下降：奇数枚举值对应下降。分类还提供十二个谓词，包括 `is_complex`、`is_unique_image`、`has_double_image` 与 `is_like_noncompact`，分别用于查询复类型、唯一像、双像及类似非紧类型的性质。^[extended-block.md:26-30]

`generator_length` 表示折叠生成元的长度，按 `One*`、`Two*`、`Three*` 三族分别返回 1、2、3。`link_count` 给出链接数；零链接类型包括 `OneRealNonparity` 和 `OneImaginaryCompact`，这些类型不记录 cross action。^[extended-block.md:29-31]

## 偶数长度差链接

`has_october_surprise` 使用如下判定关系：当 `has_defect` 为真时，检查 `generator_length == 3`；否则检查 `generator_length == 2`。源码注释将其解释为自 2016 年 10 月起单独标出的、具有偶数长度差的链接。^[extended-block.md:31-33]

## 在扩展块中的使用

父块上的 `extended_type` 执行纯组合的局部类型识别；构造完成后，可通过 `descent_type(s, n)` 查询生成元 `s` 与扩展元素 `n` 对应的下降类型。相关构造见 [[局部扩展类型识别与全父块构造]]，整体结构见 [[扩展块与 δ-不动部分]]。^[extended-block.md:44-50, extended-block.md:71-77]

折叠生成元由 `ExtGen { kind: ExtGenKind, s0, s1 }` 表示，`ExtGenKind::One/Two/Three` 的 `length()` 同样返回 1、2、3。这里的生成元长度应与扩展块元素的 `length(n)` 区分：后者等于父块中的 `parent.length(z(n))`，其中 `z(n)` 是扩展元素的父索引，参见 [[扩展块与父块的索引映射]]。^[extended-block.md:35-42, extended-block.md:73-77]

## 证据范围

本页依据 `ext_block.rs` 的结构性阅读材料。来源未逐项列出全部 32 个变体或十二个谓词的完整定义；其中上游行号转述自源码注释，未独立重读上游。该材料未执行构建、测试或原版运行，因此不构成数学验收、性能或并行行为的证据。^[extended-block.md:9-15, extended-block.md:24-33, extended-block.md:84-90]

## Sources

- [extended-block.md](extended-block.md)
