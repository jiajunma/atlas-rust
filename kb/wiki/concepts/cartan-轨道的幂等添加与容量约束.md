---
title: Cartan 轨道的幂等添加与容量约束
summary: add_cartan 重复添加返回已有切片，新轨道须恰好达到分类给出的期望大小，记录数达到 max_involutions 时拒绝继续插入。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:23.390Z"
updatedAt: "2026-10-10T00:36:23.832Z"
tags:
  - Cartan
  - 资源预算
  - 幂等性
aliases:
  - cartan-轨道的幂等添加与容量约束
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Cartan 轨道的幂等添加与容量约束
summary: add_cartan 重复添加返回已有切片，新轨道须恰好达到分类给出的期望大小；全表记录数达到 max_involutions 后拒绝继续插入。
sources:
  - involution-table.md
kind: concept
tags:
  - Cartan分类
  - 资源预算
  - 幂等性
aliases:
  - cartan-轨道的幂等添加与容量约束
---

# Cartan 轨道的幂等添加与容量约束

`InvolutionTable::add_cartan(classification, cartan)` 在 KGB 构建流水线的 stage b 中，将一个 Cartan 类的 twisted involution 轨道存为连续切片。接口具有幂等性：重复添加同一 Cartan 类时返回已有切片；新轨道则必须符合分类给出的期望大小，并受全表记录数上限约束。^[involution-table.md:20-24, involution-table.md:44-49, involution-table.md:62-63]

## 添加与编号

轨道种子和期望大小均来自 classification。全表编号遵循调用方的 Cartan 添加顺序，文档纪律要求按升序 `CartanId` 添加；轨道内部采用 external-order BFS，`InvolutionId` 跨 Cartan 全局连续递增。`orbit_slice(cartan)` 返回连续轨道切片及其起始编号，参见 [[Twisted involution 表与 Cartan 轨道存储]]。^[involution-table.md:33-35, involution-table.md:44-47]

种子通过 `WeylElement::from_action` 从矩阵级代表元转换一次，对合长度按 `(W_length + #Cayley)/2` 计算；分子为奇数时报告 `"length parity"` 不变量错误。`CayleyCrossDecomposition` 按 Cartan 类使用，不在每个条目上重新构建。^[involution-table.md:47-49]

## 轨道完整性与去重

生成的轨道必须恰好填满期望大小，否则报告 `InvolutionTableInvariantViolation { invariant: "orbit size" }`。BFS 通过邻居 `s_g · current · s_{twist(g)}` 扩展轨道，并以前向根置换为去重键，相关过程见 [[Twisted cross-action 的 BFS 轨道构建]]。^[involution-table.md:44-54]

种子插入 `index_by_permutation` 时没有碰撞检查：`BTreeMap::insert` 遇到同键会静默覆盖。因此，实现依赖不同 Cartan 轨道键互不重叠的调用纪律。置换键的访问契约另见 [[前向根置换索引及其调用方契约]]。^[involution-table.md:60-61, involution-table.md:67-68]

## 容量约束

`max_involutions` 是全表条目数的包含式上限：允许表中已有这么多条记录，但在 `records.len() == max_involutions` 时继续插入会被拒绝，并报告 `InvolutionTableResourceLimit { resource: "involutions" }`。轨道大小不变量核对单条轨道是否符合分类规模，容量守卫则限制全表记录总量。^[involution-table.md:44-47, involution-table.md:62-63]

## 与 Cayley 查询的关系

`cayley(generator, id)` 通过反射乘积和置换查表计算邻居；目标 Cartan 类尚未添加时返回 `None`。stage-(e) 契约要求预先添加该实形式的向上封闭 Cartan 集合，此后出现 `None` 即表示调用方违反不变量。参见 [[Cayley 邻居查询与向上封闭 Cartan 集合]]。^[involution-table.md:73-75]

## 测试与证据边界

来源记录的相关测试锚点包括 A1 幂等添加、B2 全表与分类对账、两次独立建表逐切片相等，以及 Cayley 目标 Cartan 加入前后的 `None` → `Some` 变化。边界测试覆盖上限为 1 时拒绝第二个 Cartan，以及 `CartanId(9)` 越界。四个不变量错误字面量、`AllocationFailed` 和 `lookup` 的 `Some` 命中直接断言尚未覆盖，详见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:79-88]

这些说明来自结构性源码阅读。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；实现正确性属于其自身的 HPC 证据链，本页不扩展其接受范围。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进发生漂移。^[involution-table.md:9-16, involution-table.md:98-103]

## Sources

- [involution-table.md](../../sources/involution-table.md) — Twisted involution 表（KGB stage b）。
