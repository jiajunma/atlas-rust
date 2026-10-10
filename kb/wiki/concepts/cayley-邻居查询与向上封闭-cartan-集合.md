---
title: Cayley 邻居查询与向上封闭 Cartan 集合
summary: cayley 通过 s·w 的置换查找邻居，目标 Cartan 类未加入时返回 None；KGB stage e 要求预先添加对应实形式的向上封闭 Cartan 集合。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:42.234Z"
updatedAt: "2026-10-10T00:36:36.587Z"
tags:
  - KGB
  - Cayley变换
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Cayley 邻居查询与向上封闭 Cartan 集合
summary: cayley 通过反射乘积 s·w 与前向根置换查表寻找邻居；目标 Cartan 类尚未添加时返回 None，KGB stage e 要求预先添加对应实形式的向上封闭 Cartan 集合。
sources:
  - involution-table.md
kind: concept
tags:
  - Cayley变换
  - KGB
  - 调用契约
aliases:
  - cayley-邻居查询与向上封闭-cartan-集合
  - C邻C集
provenanceState: extracted
---

# Cayley 邻居查询与向上封闭 Cartan 集合

`InvolutionTable::cayley(generator, id)` 通过反射乘积与置换查表，查询对合记录的 Cayley 邻居 \(s\cdot w\)。目标 Cartan 类尚未加入表时返回 `None`；KGB 构建 stage-(e) 要求预先添加该实形式的**向上封闭 Cartan 集合**，因此必须结合这一前提解释查询结果。^[involution-table.md:73-75]

## 查询与存储基础

[[Twisted involution 表与 Cartan 轨道存储|Twisted involution 表]]属于 KGB 构建流水线的 stage b，按 Cartan 类连续存储 twisted involution 轨道。`InvolutionId` 跨 Cartan 全局连续递增，`orbit_slice(cartan)` 返回连续轨道切片及其起始编号。^[involution-table.md:20-24, involution-table.md:33-35]

`add_cartan(classification, cartan)` 将指定 Cartan 类的轨道加入表，重复添加则返回已有切片。种子与期望轨道大小来自 classification，生成的轨道必须恰好填满期望大小，否则报告 `"orbit size"` 不变量错误。相关构造约束见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:44-49]

表的 `lookup` 使用前向根置换作为查找键。`cayley` 通过反射乘积 \(s\cdot w\) 加置换查表计算邻居；`cross(generator, id)` 则对应 \(s\cdot w\cdot\mathrm{twist}(s)\)，直接读取构建时存储的 cross-action 链接。两者的作用公式与查询路径不同。^[involution-table.md:67-75]

## 向上封闭的调用契约

stage-(e) 的准备工作要求先添加对应实形式的向上封闭 Cartan 集合。在准备完成前，`None` 可以反映目标 Cartan 类尚未入表；满足该阶段的准备要求后，来源将 `None` 视为调用方违反不变量。目标类是否已添加，是解释返回值的关键条件。^[involution-table.md:73-75]

## 测试与证据边界

来源记录的 Cayley 边测试检查了目标 Cartan 类加入前后的返回值变化：加入前为 `None`，加入后为 `Some`。该测试锚点覆盖了目标类是否入表对查询结果的影响；其他测试及未覆盖分支见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:79-88]

本页依据来源包的结构性源码阅读及其记录的测试锚点。来源包未执行构建、测试或原版运行，不包含数学验收、性能或并行结论；实现正确性属于其自身的 HPC 证据链，本页不扩展这一证据范围。^[involution-table.md:9-16, involution-table.md:103-106]

## Sources

- [involution-table.md](../../sources/involution-table.md) — Twisted involution 表（KGB stage b）。
