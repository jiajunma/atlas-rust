---
title: DescValue 扩展下降分类
summary: DescValue 用 One、Two、Three 三族共 32 个变体描述局部下降类型，奇数枚举值表示下降，并提供生成元长度及链接数等判定。
sources:
  - extended-block.md
kind: concept
createdAt: "2026-10-09T14:47:26.925Z"
updatedAt: "2026-10-09T19:28:20.195Z"
tags:
  - 扩展块
  - 下降分类
aliases:
  - descvalue-扩展下降分类
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# DescValue 扩展下降分类

`DescValue` 是[[扩展块与 δ-不动部分|扩展块]]中的 32 值下降分类，按 `One*`、`Two*`、`Three*` 三族排列。扩展块取普通块的 $\delta$-不动部分，并将生成元折叠进 $\delta$-轨道；该分类提供下降判定、像的性质、生成元长度及链接数等局部结构查询。^[extended-block.md:19-33]

## 分类与谓词

`is_descent` 按枚举值的奇偶性判定下降：奇数枚举值对应下降。分类提供十二个谓词，包括 `is_complex`、`is_unique_image`、`has_double_image` 和 `is_like_noncompact`，用于查询复类型、唯一像、双像及类似非紧类型等性质。^[extended-block.md:26-30]

`generator_length` 返回折叠生成元的长度，按 `One*`、`Two*`、`Three*` 三族分别取 1、2、3。`link_count` 给出链接数；`OneRealNonparity` 和 `OneImaginaryCompact` 属于零链接类型，不记录 cross action。^[extended-block.md:29-31]

## 偶数长度差链接

`has_october_surprise` 的判定式为 `generator_length == if has_defect { 3 } else { 2 }`：有 defect 时检查生成元长度是否为 3，否则检查是否为 2。源码注释将其解释为自 2016 年 10 月起单独标出的、具有偶数长度差的链接。^[extended-block.md:31-33]

## 局部识别与查询

父块上的 `extended_type` 执行纯组合的局部类型识别，相关构造见[[局部扩展类型识别与全父块构造]]。扩展块提供 `descent_type(s, n)`，用于查询生成元 `s` 与扩展元素 `n` 对应的下降类型。^[extended-block.md:44-50, extended-block.md:71-77]

[[折叠生成元与 fold_orbits|折叠生成元]]由 `ExtGen { kind: ExtGenKind, s0, s1 }` 表示，其 `ExtGenKind::One/Two/Three` 的 `length()` 同样返回 1、2、3。生成元长度与扩展块元素的 `length(n)` 含义不同：后者等于 `parent.length(z(n))`，其中 `z(n)` 是扩展元素的父块索引，参见[[扩展块与父块的索引映射]]。^[extended-block.md:37-42, extended-block.md:73-77]

## 证据范围

来源是对 `ext_block.rs` 的结构性阅读，未逐项展开全部 32 个变体及十二个谓词的完整定义。材料中的上游行号转述自源码注释，未独立重读上游；本包也未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论。扩展块正确性属于其自身的 [[HPC 验收证据链]]。^[extended-block.md:9-15, extended-block.md:24-33, extended-block.md:84-90]

## Sources

- [extended-block.md](extended-block.md) — 扩展块：delta-不动部分与折叠生成元。
