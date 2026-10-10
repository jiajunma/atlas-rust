---
title: DescValue 扩展下降分类
summary: DescValue 以 One、Two、Three 三族组织 32 种状态，用奇数枚举值标记下降，并提供生成元长度、链接数量及局部性质判定。
sources:
  - extended-block.md
kind: concept
createdAt: "2026-10-09T14:47:26.925Z"
updatedAt: "2026-10-10T00:32:14.428Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: DescValue 扩展下降分类
summary: DescValue 以 One、Two、Three 三族共 32 个变体描述扩展下降类型，提供下降判定、像的性质、折叠生成元长度和链接数查询。
sources:
  - extended-block.md
kind: concept
tags:
  - 扩展块
  - 下降分类
aliases:
  - descvalue-扩展下降分类
---

# DescValue 扩展下降分类

`DescValue` 是[[扩展块与 δ-不动部分|扩展块]]中的 32 值扩展下降分类，按 `One*`、`Two*`、`Three*` 三族排列。扩展块取普通块的 $\delta$-不动部分，并将生成元折叠进 $\delta$-轨道；`DescValue` 提供相应的下降判定与局部结构查询。^[extended-block.md:19-33]

## 分类与查询规则

`is_descent` 以枚举值的奇偶性判定下降：奇数枚举值对应下降。分类还提供十二个谓词，包括 `is_complex`、`is_unique_image`、`has_double_image` 和 `is_like_noncompact`；来源未逐项展开全部变体或全部谓词的定义。^[extended-block.md:24-33]

`generator_length` 按 `One*`、`Two*`、`Three*` 三族分别返回折叠生成元长度 1、2、3。`link_count` 给出链接数；`OneRealNonparity` 和 `OneImaginaryCompact` 是零链接类型，不记录 cross action。^[extended-block.md:29-31]

## 偶数长度差链接

`has_october_surprise` 的定义式为 `generator_length == if has_defect { 3 } else { 2 }`：有 defect 时检查生成元长度是否为 3，否则检查是否为 2。源码注释将其解释为自 2016 年 10 月起单独标出的、具有偶数长度差的链接。^[extended-block.md:31-33]

## 局部识别与访问接口

`extended_type` 在父块上执行纯组合的局部类型识别，相关构造见[[局部扩展类型识别与全父块构造]]。扩展块通过 `descent_type(s, n)` 查询下降类型。^[extended-block.md:44-50, extended-block.md:73-77]

[[折叠生成元与 fold_orbits|折叠生成元]]由 `ExtGen { kind: ExtGenKind, s0, s1 }` 表示，其 `length()` 按 `One/Two/Three` 返回 1、2、3。生成元长度与扩展块元素长度是不同的查询：元素的 `length(n)` 等于 `parent.length(z(n))`，其中 `z(n)` 是扩展元素的父块索引，参见[[扩展块与父块的索引映射]]。^[extended-block.md:37-42, extended-block.md:73-77]

## 证据范围

本页依据 `ext_block.rs` 的结构性阅读材料，阅读快照记录的是 dirty 工作区字节。材料中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[extended-block.md:9-15, extended-block.md:81-85]

来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。扩展块正确性属于其自身的 HPC 证据链，包括 ext-KL/unitarity gate；本材料不重述或扩展该证据。^[extended-block.md:10-11, extended-block.md:90-90]

## Sources

- [extended-block.md](../../sources/extended-block.md) — 扩展块：delta-不动部分与折叠生成元。
