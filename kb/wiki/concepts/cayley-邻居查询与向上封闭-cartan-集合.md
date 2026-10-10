---
title: Cayley 邻居查询与向上封闭 Cartan 集合
summary: cayley 通过 s·w 的置换查表寻找邻居，目标 Cartan 类未添加时返回 None，KGB stage e 要求预先添加对应实形式的向上封闭 Cartan 集合。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:42.234Z"
updatedAt: "2026-10-10T03:34:46.010Z"
tags:
  - Cayley变换
  - KGB
  - 调用前提
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

`InvolutionTable::cayley(generator, id)` 通过反射乘积 $s\cdot w$ 与置换查表计算 Cayley 邻居。目标 Cartan 类尚未添加时返回 `None`；KGB 构建 stage-(e) 要求预先添加对应实形式的**向上封闭 Cartan 集合**，此后返回 `None` 即表示调用方违反不变量。^[involution-table.md:79-81]

## 查询与存储基础

[[Twisted involution 表与 Cartan 轨道存储|Twisted involution 表]]属于 KGB 构建流水线的 stage b，按 Cartan 类连续存储 twisted involution 轨道。`InvolutionId` 跨 Cartan 全局连续递增，`orbit_slice(cartan)` 返回指定轨道的连续切片及其起始编号。^[involution-table.md:20-24, involution-table.md:33-35]

`add_cartan(classification, cartan)` 将指定 Cartan 类的轨道加入表，重复添加返回已有切片。种子与期望大小来自 classification，生成轨道必须恰好达到期望大小，否则报告 `"orbit size"` 不变量错误。相关约束见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:44-49]

表的 `lookup` 使用前向根置换作为键，涉及同基数外来根系元素时仍有调用方契约，详见 [[前向根置换索引及其调用方契约]]。`cayley` 经反射乘积加置换查表计算邻居；`cross(generator, id)` 则直接读取构建时存储的 $s\cdot w\cdot\mathrm{twist}(s)$ 链接，两者的作用公式与查询路径不同。^[involution-table.md:73-81]

## 向上封闭的调用契约

`None` 的含义取决于目标 Cartan 类是否已经入表。准备阶段中，它可以反映目标类尚未添加；stage-(e) 要求先添加该实形式的向上封闭 Cartan 集合，因此在满足这一前提后，调用方不能再将 `None` 解释为正常的目标类缺失。^[involution-table.md:79-81]

## 测试与证据边界

来源记录了一项 Cayley 边测试：目标 Cartan 类加入前查询返回 `None`，加入后返回 `Some`。这一锚点直接检查目标类入表前后的返回值变化；更多测试及未覆盖分支见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:85-94]

本页依据来源包的结构性阅读与测试锚点记录。来源包未执行构建、测试或原版运行，不包含数学验收、性能或并行结论；实现正确性属于其自身的 HPC 证据链。^[involution-table.md:9-16, involution-table.md:109-112]

## Sources

- [involution-table.md](../../sources/involution-table.md) — Twisted involution 表（KGB stage b）。
