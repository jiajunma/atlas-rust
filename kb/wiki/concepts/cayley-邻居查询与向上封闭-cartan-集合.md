---
title: Cayley 邻居查询与向上封闭 Cartan 集合
summary: cayley 通过 s·w 的置换查表寻找邻居，目标 Cartan 类未添加时返回 None；KGB stage e 要求预先添加对应实形式的向上封闭 Cartan 集合。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:42.234Z"
updatedAt: "2026-10-09T20:56:07.765Z"
tags:
  - Cayley变换
  - KGB
  - 调用契约
aliases:
  - cayley-邻居查询与向上封闭-cartan-集合
  - C邻C集
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cayley 邻居查询与向上封闭 Cartan 集合
summary: cayley 通过 s·w 的反射乘积与前向根置换查表寻找邻居；目标 Cartan 类尚未添加时返回 None，stage e 要求预先添加对应实形式的向上封闭 Cartan 集合。
sources:
  - involution-table.md
kind: concept
tags:
  - Cayley变换
  - Cartan分类
  - KGB
aliases:
  - cayley-邻居查询与向上封闭-cartan-集合
  - C邻C集
provenanceState: extracted
---

# Cayley 邻居查询与向上封闭 Cartan 集合

`InvolutionTable::cayley(generator, id)` 查询对合记录的 Cayley 邻居 \(s\cdot w\)，通过反射乘积与置换查表定位目标。目标 Cartan 类尚未加入表时，查询返回 `None`；KGB 构建 stage-(e) 要求预先添加该实形式的向上封闭 Cartan 集合，因此必须结合调用前提解释这一返回值。^[involution-table.md:73-75]

## 查询与存储基础

[[Twisted involution 表与 Cartan 轨道存储|Twisted involution 表]]属于 KGB 构建的 stage b，按 Cartan 类连续存储对合轨道。`InvolutionId` 跨 Cartan 全局连续递增，`orbit_slice(cartan)` 返回对应的连续轨道切片及其起始编号。^[involution-table.md:20-24, involution-table.md:33-35]

`add_cartan(classification, cartan)` 将指定 Cartan 类的轨道加入表；重复添加返回已有切片。轨道种子与期望大小来自 classification，生成结果必须恰好填满期望大小，否则报告 `"orbit size"` 不变量错误。相关约束见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:44-49]

表以**前向根置换**作为查找键。`cayley` 计算反射乘积 \(s\cdot w\) 后查表；`cross(generator, id)` 查询的则是 \(s\cdot w\cdot\mathrm{twist}(s)\)，直接读取构建时存储的 cross-action 链接。两者具有不同的作用公式与查询路径。^[involution-table.md:67-75]

## 向上封闭的调用契约

stage-(e) 要求先添加该实形式的**向上封闭 Cartan 集合**。在完成准备之前，`None` 可以反映目标 Cartan 类尚未入表；满足这一准备契约后，来源将 `None` 认定为调用方违反不变量。因此，目标是否已被纳入表中，是理解 Cayley 查询结果的必要条件。^[involution-table.md:73-75]

## 测试与证据边界

来源记录的 Cayley 边测试检查目标 Cartan 类加入前后的变化：加入前返回 `None`，加入后返回 `Some`。这一锚点覆盖了目标 Cartan 是否入表对查询结果的影响，相关测试范围见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:79-88]

本页依据来源包的结构性源码阅读及其记录的测试锚点。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；实现的正确性属于其自身的 HPC 证据链，本页不扩展该证据范围。^[involution-table.md:9-16, involution-table.md:103-106]

## Sources

- [involution-table.md](../../sources/involution-table.md) — Twisted involution 表（KGB stage b）
