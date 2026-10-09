---
title: Cartan 轨道的幂等添加与容量约束
summary: add_cartan 重复调用返回已有切片，新轨道须恰好达到分类给出的期望大小，且记录数量达到 max_involutions 时拒绝继续插入。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:23.390Z"
updatedAt: "2026-10-09T14:52:23.390Z"
tags:
  - Cartan分类
  - 幂等性
  - 资源限制
aliases:
  - cartan-轨道的幂等添加与容量约束
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cartan 轨道的幂等添加与容量约束

`InvolutionTable::add_cartan(classification, cartan)` 在 KGB 构建流水线的 stage b 中，将一个 Cartan 类的 twisted involution 轨道加入表中。每个轨道占据连续切片；重复添加同一 Cartan 类时返回已有切片，形成幂等的添加接口。^[involution-table.md:20-24, involution-table.md:44-49]

## 添加与编号

轨道的种子和期望大小均来自 Cartan classification。编号遵循调用方的 Cartan 添加顺序，文档要求按升序 `CartanId` 添加；轨道内部采用 external-order BFS，`InvolutionId` 则跨 Cartan 全局连续递增。`orbit_slice(cartan)` 返回对应的连续切片及起始编号，参见 [[Twisted involution 表与 Cartan 轨道存储]]。^[involution-table.md:33-35, involution-table.md:44-47]

种子通过 `WeylElement::from_action` 从矩阵级代表元转换一次，其对合长度由 `(W_length + #Cayley)/2` 计算；若分子为奇数，则报告 `"length parity"` 不变量错误。`CayleyCrossDecomposition` 按 Cartan 类使用，不在每个条目上重新构建。^[involution-table.md:47-49]

## 轨道完整性与去重

生成的轨道必须恰好达到 classification 给出的期望大小，否则报告 `InvolutionTableInvariantViolation { invariant: "orbit size" }`。BFS 通过邻居 `s_g · current · s_{twist(g)}` 扩展轨道，并以前向根置换作为去重键。^[involution-table.md:44-54]

种子插入 `index_by_permutation` 时没有碰撞检查：`BTreeMap::insert` 遇到同键会静默覆盖，因此实现依赖不同 Cartan 轨道的键互不重叠这一调用纪律。幂等接口并未替代该跨轨道约束；相关键语义见 [[前向根置换索引及其调用方契约]]。^[involution-table.md:60-61, involution-table.md:67-68]

## 容量约束

`max_involutions` 是表条目数的包含式上限：当 `records.len() == max_involutions` 时，继续插入条目会被拒绝，并报告 `InvolutionTableResourceLimit { resource: "involutions" }`。因此，轨道大小校验与容量守卫承担不同职责：前者核对分类给出的轨道规模，后者限制表中记录的总量。^[involution-table.md:44-47, involution-table.md:62-63]

## 与 Cayley 查询的关系

Cartan 类是否已添加会影响 `cayley(generator, id)` 的结果。该访问器通过反射乘积和置换查表寻找邻居；若目标 Cartan 类尚未加入，返回 `None`。stage-(e) 契约要求预先添加相应实形式的向上封闭 Cartan 集合，此后出现 `None` 即表示调用方违反不变量，参见 [[Cayley 邻居查询与向上封闭 Cartan 集合]]。^[involution-table.md:73-75]

## 测试与证据边界

来源记录的测试锚点包括 A1 的幂等添加、B2 全表与分类对账及两次独立建表的逐切片相等、Cayley 目标 Cartan 加入前后的 `None` → `Some` 变化，以及容量上限为 1 时拒绝第二个 Cartan、`CartanId(9)` 越界等守卫。四个不变量错误字面量和 `AllocationFailed` 等分支尚未被这些测试触及，参见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:79-88]

这些内容来自结构性源码阅读；来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。其正确性仍属于独立的 HPC 证据链，不能由本页的接口说明或测试锚点推定。^[involution-table.md:9-16, involution-table.md:103-106]

## Sources

- [involution-table.md](involution-table.md)
