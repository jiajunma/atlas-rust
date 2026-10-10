---
title: Cartan 轨道的幂等添加与容量约束
summary: add_cartan 重复调用返回已有切片，新轨道须恰好达到分类给出的期望大小，记录数达到 max_involutions 后拒绝新增。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:23.390Z"
updatedAt: "2026-10-10T03:34:14.316Z"
tags:
  - Cartan分类
  - 构造不变量
  - 资源预算
aliases:
  - cartan-轨道的幂等添加与容量约束
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Cartan 轨道的幂等添加与容量约束

`InvolutionTable::add_cartan(classification, cartan)` 在 KGB 构建流水线的 stage b 中，将一个 Cartan 类的 twisted involution 轨道存为连续切片。重复添加同一 Cartan 类时返回已有切片；新轨道必须恰好达到分类给出的期望大小，并受全表记录数上限约束。^[involution-table.md:20-24, involution-table.md:44-49, involution-table.md:68-69]

## 添加与编号

轨道种子和期望大小均来自 classification。全表编号遵循调用方的 Cartan 添加顺序，文档纪律要求按升序 `CartanId` 添加；轨道内部按外序（external-order）BFS 排列，`InvolutionId` 跨 Cartan 全局连续递增。`orbit_slice(cartan)` 返回连续轨道切片及其起始编号，参见 [[Twisted involution 表与 Cartan 轨道存储]]。^[involution-table.md:33-35, involution-table.md:44-47]

种子通过 `WeylElement::from_action` 从矩阵级代表元转换一次，对合长度按 `(W_length + #Cayley)/2` 计算；分子为奇数时报告 `"length parity"` 不变量错误。`CayleyCrossDecomposition` 按 Cartan 类使用，不在每个条目上重新构建。^[involution-table.md:47-49]

## 轨道完整性与去重

生成的轨道必须恰好填满期望大小，否则报告 `InvolutionTableInvariantViolation { invariant: "orbit size" }`。BFS 通过邻居 `s_g · current · s_{twist(g)}` 扩展轨道，并以前向根置换为去重键，详见 [[Twisted cross-action 的 BFS 轨道构建]]。^[involution-table.md:44-54]

`push_record` 中的 `index_by_permutation.insert(key, id)` 是没有碰撞检查的 `BTreeMap::insert`，但来源核对确认：在表自身的不变量下，静默覆盖在数学上不可达。键是 Weyl 因子 $w$ 的完整根置换，忠实决定 $w$，而固定的 $\delta$ 又使 $w$ 唯一决定 $\theta=w\delta$。同一内类的不同 Cartan 轨道是两两不交的扭曲共轭类，因此不同 Cartan 添加所产生的键集合互不相交。^[involution-table.md:60-65]

同一 Cartan 类的重复添加由入口处的幂等检查拦截，同一 BFS 内的重复元素则先被 lookup 命中。因此，轨道间不发生键覆盖属于表自身的不变量，无需另加调用方纪律。仍需调用方保证的是 `lookup` 的外来根系元素契约：同基数的不同根系可能具有同形置换，而根数匹配是唯一结构性防线。参见 [[前向根置换索引及其调用方契约]]。^[involution-table.md:62-67, involution-table.md:73-74]

## 容量约束

`max_involutions` 是全表条目数的包含式上限：表中可以已有这么多条记录，但当 `records.len() == max_involutions` 时，继续插入会被拒绝，并报告 `InvolutionTableResourceLimit { resource: "involutions" }`。轨道大小不变量核对单条轨道是否符合分类规模，容量守卫则限制全表记录总量。^[involution-table.md:44-47, involution-table.md:68-69]

## 与 Cayley 查询的关系

`cayley(generator, id)` 通过反射乘积和置换查表计算邻居；目标 Cartan 类尚未添加时返回 `None`。stage-(e) 契约要求预先添加该实形式的向上封闭 Cartan 集合，此后出现 `None` 即表示调用方违反不变量。参见 [[Cayley 邻居查询与向上封闭 Cartan 集合]]。^[involution-table.md:79-81]

## 测试与证据边界

来源记录的相关测试锚点包括 A1 幂等添加、B2 全表与分类对账、两次独立建表逐切片相等，以及 Cayley 目标 Cartan 加入前后的 `None` → `Some` 变化。边界测试覆盖上限为 1 时拒绝第二个 Cartan，以及 `CartanId(9)` 越界。四个不变量错误字面量、`AllocationFailed` 和 `lookup` 的 `Some` 命中直接断言尚未覆盖，详见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:85-94]

这些说明来自结构性源码阅读。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；实现正确性属于其自身的 [[HPC 验收证据链]]。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进发生漂移。^[involution-table.md:9-16, involution-table.md:104-109]

## Sources

- [involution-table.md](../../sources/involution-table.md) — Twisted involution 表（KGB stage b）。
