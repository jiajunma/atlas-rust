---
title: Cayley 邻居查询与向上封闭 Cartan 集合
summary: cayley 以 s·w 的置换查表寻找邻居，目标 Cartan 未添加时返回 None；stage e 要求预先添加该 form 的向上封闭 Cartan 集合。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:42.234Z"
updatedAt: "2026-10-09T14:52:42.234Z"
tags:
  - Cayley变换
  - Cartan分类
  - 接口契约
aliases:
  - cayley-邻居查询与向上封闭-cartan-集合
  - C邻C集
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cayley 邻居查询与向上封闭 Cartan 集合

`InvolutionTable::cayley(generator, id)` 查询对合记录的 Cayley 邻居，其词级表达式为 \(s\cdot w\)。查询通过反射乘积与根置换查表完成；目标 Cartan 类尚未加入表时，返回 `None`。因此，返回值的解释依赖于调用方是否已准备好所需的 Cartan 集合。^[involution-table.md:73-75]

## 查询与存储基础

[[Twisted involution 表与 Cartan 轨道存储|Twisted involution 表]]按 Cartan 类连续存储对合轨道，`InvolutionId` 跨 Cartan 全局连续递增。`add_cartan(classification, cartan)` 将指定类的轨道加入表，重复添加则返回已有切片；种子与期望轨道大小来自 classification，生成结果必须恰好符合期望大小。^[involution-table.md:20-24, involution-table.md:33-35, involution-table.md:44-49]

表的查找键是前向根置换。Cayley 查询计算 \(s\cdot w\) 后通过置换查表定位目标；相较之下，`cross(generator, id)` 查询的是 \(s\cdot w\cdot\mathrm{twist}(s)\)，使用构建时存储的链接。两者的作用公式与查询路径不同。^[involution-table.md:67-75]

## 向上封闭的调用契约

KGB 构建的 stage-(e) 契约要求：调用 Cayley 邻居查询前，先添加该实形式的向上封闭 Cartan 集合。该准备工作完成后，`None` 即表示调用方违反不变量；不能将它解释为符合该契约的正常查询结果。此要求将邻居查询的有效性与表中已加入的 Cartan 类范围联系起来。^[involution-table.md:73-75]

## 测试与证据边界

来源记录了一项针对 Cayley 边的测试锚点：目标 Cartan 类加入前，查询返回 `None`；加入后，同一条边的查询返回 `Some`。这一锚点直接覆盖了 Cartan 类是否入表对查询结果的影响。^[involution-table.md:79-84]

本页依据的是源码结构性阅读及所记录的测试锚点。来源包未执行构建、测试或原版运行，也不提供数学验收、性能或并行结论；相关正确性仍属于独立的 [[HPC 验收证据链]]。^[involution-table.md:9-16, involution-table.md:103-106]

## Sources

- [involution-table.md](involution-table.md) — Twisted involution 表（KGB stage b）
