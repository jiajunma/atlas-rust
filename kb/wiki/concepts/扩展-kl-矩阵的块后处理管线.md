---
title: 扩展 KL 矩阵的块后处理管线
summary: ext_kl_matrix 在已有 ExtBlock 上依次执行填充、展开、condense、奇偶符号翻转及池索引矩阵生成；本切片不负责从 StandardRepr 构建扩展块，亦未提供数学验收。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T19:29:03.406Z"
updatedAt: "2026-10-09T19:29:03.406Z"
tags:
  - 扩展KLV
  - 矩阵构造
  - 移植边界
aliases:
  - 扩展-kl-矩阵的块后处理管线
  - 扩K矩
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 扩展 KL 矩阵的块后处理管线

扩展 KL 矩阵的块后处理管线是 `ext_kl_matrix` 在已有 `ExtBlock` 之后执行的处理部分，依次涵盖填充（fill）、展开（expand）、`condense`、parity 符号翻转，以及多项式池索引矩阵的形成。当前移植以 `ExtBlock` 为边界，不包含从 `StandardRepr` 构建扩展块的前置过程。^[extended-kl.md:101-104]

## 输入边界与模块分工

从 `StandardRepr` 构建扩展块需要 `StandardReprMod` 与 `common_block`，来源将这部分留给后续切片。因此，本管线的输入前提是扩展块已经存在；相关参数背景可参见 [[StandardRepr 标准表示参数]] 与 [[标准模参数的约化表示（StandardReprMod）]]。^[extended-kl.md:101-104]

扩展 KLV 模块分为三个层次：`DescentTable` 预计算下降集、good ascent 和带符号翻转信息的 primitive 索引；`ExtKlTable` 按块元素 `y` 存储多项式池索引列；模块尾部提供 `condense`、`ext_kl_matrix`、`contributions` 等自由函数。矩阵后处理建立在前两层的数据之上。^[extended-kl.md:19-24]

## 列填充与展开基础

`ExtKlTable` 为每个块元素 `y` 保存一列池索引，按 `x` 相对于 `y` 的下降集所对应的 primitive 位置寻址。`kl_pol_index` 返回 `(KLIndex, bool)`，而 `p(x,y)` 根据翻转标志应用 `scaled(-1)`，得到 twisted KLV 多项式 \(P_{x,y}\)。这一访问方式与 [[Primitivisation 索引与符号传播]] 相关。^[extended-kl.md:72-80]

`fill_columns(limit)` 计算所有 `y < limit` 的列，`limit == 0` 时填满整块。填充失败会清空出错列并传播错误，维持“每列要么为空、要么完整”的不变式；这是相对于上游吞掉异常行为的有意偏离，参见 [[列填充的完整性不变式与错误传播]]。^[extended-kl.md:84-87]

## 池索引与符号处理

多项式池使用 `KlHashTable`，条目是系数为 `i32` 的 `KlPol`。池索引本身不打包符号位；primitivisation 的符号信息存放在独立的 `prim_flip` 位图中，查询时另行读取。`raw_ext_KL` 包装层会把返回的索引与翻转标志渲染为带正负号的索引。^[extended-kl.md:28-35]

矩阵管线在 fill、expand 和 `condense` 之后还包含 parity 符号翻转，最后形成 pool-index 矩阵。来源分别记载了这一后处理步骤与 primitive 查询中的符号翻转，但没有展开 `expand`、`condense` 的内部算法或 parity 判定公式。^[extended-kl.md:32-35, extended-kl.md:101-104]

## 实现范围与证据边界

当前 `ExtKlTable` 始终拥有自己的多项式池；共享池指针模式，以及用于 partial-block 迁移的 `swallow`，被推迟到需要它们的 common-block 切片。另一个未移植的接口是 `check_polys`，因为它依赖 `ExtBlock` 未保留的 untwisted block。^[extended-kl.md:93-100]

本说明依据结构性源码阅读。来源没有执行构建、测试或原版运行，不提供数学验收、性能或并行结论；扩展 KL 的正确性属于独立的 [[HPC 验收证据链]]。所列上游行号转述自源码注释，未独立重读上游，可能随版本变化。^[extended-kl.md:9-15, extended-kl.md:110-114]

## Sources

- [extended-kl.md](extended-kl.md) — 扩展 KLV 多项式表：primitivisation 符号与逐列存储。
