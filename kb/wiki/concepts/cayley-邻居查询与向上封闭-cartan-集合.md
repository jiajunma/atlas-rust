---
title: Cayley 邻居查询与向上封闭 Cartan 集合
summary: cayley 通过 s·w 的置换查表寻找邻居，目标 Cartan 类尚未添加时返回 None；stage e 要求预先添加对应实形式的向上封闭 Cartan 集合。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:42.234Z"
updatedAt: "2026-10-09T19:31:15.048Z"
tags:
  - Cayley变换
  - Cartan分类
  - KGB
aliases:
  - cayley-邻居查询与向上封闭-cartan-集合
  - C邻C集
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Cayley 邻居查询与向上封闭 Cartan 集合

`InvolutionTable::cayley(generator, id)` 查询对合记录的 Cayley 邻居 \(s\cdot w\)，通过反射乘积与根置换查表定位目标。目标 Cartan 类尚未加入表时，查询返回 `None`；KGB 构建 stage-(e) 要求预先添加该实形式的向上封闭 Cartan 集合，因此返回值必须结合这一调用前提解释。^[involution-table.md:73-75]

## 查询与存储基础

[[Twisted involution 表与 Cartan 轨道存储|Twisted involution 表]]属于 KGB 构建的 stage b，按 Cartan 类连续存储对合轨道。`InvolutionId` 跨 Cartan 全局连续递增，`orbit_slice(cartan)` 返回对应的连续轨道切片及其起始编号。^[involution-table.md:20-24, involution-table.md:33-35]

`add_cartan(classification, cartan)` 将指定 Cartan 类的轨道加入表；重复添加返回已有切片。轨道种子与期望大小来自 classification，生成结果必须恰好填满期望大小，否则报告 `"orbit size"` 不变量错误。相关构造约束见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:44-49]

表以**前向根置换**作为查找键。`cayley` 计算反射乘积 \(s\cdot w\) 后查表；`cross(generator, id)` 则查询 \(s\cdot w\cdot\mathrm{twist}(s)\)，直接读取构建时存储的链接。两者的作用公式与查询路径不同。^[involution-table.md:67-75]

## 向上封闭的调用契约

stage-(e) 的准备要求是先添加该实形式的**向上封闭 Cartan 集合**。在此之前，`None` 可以反映目标 Cartan 类尚未入表；满足这一准备契约后，来源将 `None` 认定为调用方违反不变量，而非正常的邻居查询结果。^[involution-table.md:73-75]

## 测试与证据边界

来源记录的 Cayley 边测试检查了目标 Cartan 类加入前后的变化：加入前返回 `None`，加入后返回 `Some`。这一锚点覆盖了目标 Cartan 是否入表对查询结果的影响，可结合 [[Involution 表的测试锚点与证据边界]] 阅读。^[involution-table.md:79-88]

本页依据源码结构性阅读及来源记录的测试锚点。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；正确性属于该实现自身的 [[HPC 验收证据链]]。^[involution-table.md:9-16, involution-table.md:103-106]

## Sources

- [involution-table.md](involution-table.md) — Twisted involution 表（KGB stage b）
