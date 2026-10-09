---
title: Involution 表的测试锚点与证据边界
summary: 源码测试锚点覆盖 A1、B2、扭转 A2、投影传送及部分守卫，但若干错误分支未覆盖，本文的结构性阅读不构成测试执行、数学验收或性能证据。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:53:08.739Z"
updatedAt: "2026-10-09T14:53:08.739Z"
tags:
  - 测试覆盖
  - 证据边界
  - 源码阅读
aliases:
  - involution-表的测试锚点与证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Involution 表的测试锚点与证据边界

`InvolutionTable` 位于 KGB 构建流水线的 stage b，按 Cartan 类连续存储 twisted involution 轨道。测试锚点涉及记录字段、轨道构造、可复现性、Cayley 查询、投影传送及资源守卫；来源包提供的是源码结构性阅读，不构成新的数学验收或运行验证。^[involution-table.md:9-24, involution-table.md:79-88, involution-table.md:103-106]

## 测试锚点

### A1：轨道、幂等性与字段

A1 分裂案例覆盖两单元轨道、重复添加的幂等性与逐字段检查。fundamental 记录的 `theta_plus_one_rho = [1]`，`mod_space` 秩为 0；分裂记录对应 `[0]` 与秩 1。这些断言为[[Twisted involution 记录的数学不变量]]提供具体的小秩锚点。^[involution-table.md:79-81]

### B2：全表对账与可复现性

B2 测试将全表与分类结果对账，并比较两次独立建表得到的各个轨道切片。该比较覆盖切片结果的可复现性；表的编号遵循调用方的 Cartan 添加顺序，文档纪律要求按 `CartanId` 升序添加，各轨道内部则采用 external-order BFS。^[involution-table.md:33-35, involution-table.md:81-82]

B2 还逐条检查 θ 的典范性：根的作用像为 `weyl.image(δ.image(root))`，对合长度满足 `(W_length + #Cayley)/2`，而 `theta_plus_one_rho` 按 `(2ρ + θ·2ρ)/2` 计算。这些检查对应记录中的作用与数值字段。^[involution-table.md:47-49, involution-table.md:57-59, involution-table.md:82-83]

### Cayley 邻居与扭转 A2 轨道

Cayley 边测试检查目标 Cartan 类加入前后的返回值由 `None` 变为 `Some`。这一行为对应[[Cayley 邻居查询与向上封闭 Cartan 集合]]的调用契约：stage-(e) 要求预先添加该 form 的向上封闭 Cartan 集合，此后缺失邻居意味着调用方违反不变量。^[involution-table.md:73-75, involution-table.md:83-84]

扭转 A2 案例检查轨道大小排序后为 `[1, 3]`。该断言锚定轨道大小分布，来源并未将它描述为全部记录字段或全部作用边的逐项验证。^[involution-table.md:84-84]

### B2：路径依赖的投影传送

image-basis 对是记录字段中的例外：它在典范 involution 处由 $1-\theta$ 的 echelon reduction 播种，再沿 cross-action BFS 传送。该基具有路径依赖性，且 `y_lift` 的符号依赖于它，因此相关行为应结合[[图像基的典范播种与轨道传送纪律]]理解。^[involution-table.md:26-29]

B2 的 arm64 oracle 锚点针对
$\theta=\begin{pmatrix}-1&0\\2&1\end{pmatrix}$
检查传送所得 `lift_mat == [[2],[-2]]`；现场重算则得到 `[[-2],[2]]`，测试以 `assert_ne` 明确保留二者的差异。这是投影基不能简单替换为逐记录重新计算结果的具体案例。^[involution-table.md:84-86]

### 预算与越界守卫

守卫测试覆盖三种情况：上限为 1 时拒绝添加第二个 Cartan、`CartanId(9)` 越界，以及外来系统的 `lookup` 返回 `None`。实现中，当 `records.len() == max_involutions` 时拒绝继续插入，并返回资源名为 `"involutions"` 的 `InvolutionTableResourceLimit`。^[involution-table.md:62-63, involution-table.md:86-88]

## 未覆盖分支与调用方责任

来源明确列出的未触及分支包括四个不变量错误字面量、`AllocationFailed`，以及对 `lookup` 的 `Some` 命中的直接断言。四个不变量错误分别为轨道大小不符的 `"orbit size"`、种子长度奇偶性错误的 `"length parity"`、非法长度步进的 `"twisted length step"`，以及计算 $(1+\theta)\rho$ 时出现奇坐标的 `"theta rho parity"`。^[involution-table.md:44-59, involution-table.md:87-88]

另一个结构性阅读观察是：种子插入 `index_by_permutation` 时没有碰撞检查，`BTreeMap::insert` 会静默覆盖同键值，因此依赖不同 Cartan 轨道键不重叠的调用纪律。`lookup` 使用前向根置换作为键，对同基数外部系统的处理仍受调用方契约约束；已有外来系统返回 `None` 的测试不能替代这项契约。^[involution-table.md:60-61, involution-table.md:67-68, involution-table.md:86-88]

## 证据范围

来源包记录了两次结构性阅读：2026-10-03 初读与 2026-10-06 重读，所读文件字节未变、SHA-256 相同，并保留了两份阅读快照。草案经维护者逐条对照源码核对后改写合并；这些记录说明材料的阅读与编辑过程。^[involution-table.md:9-16]

来源包没有执行构建、测试或原版运行，也不包含数学验收、性能或并行结论。表的正确性属于其自身的[[HPC 验收证据链]]，包括 KGB/capacity gate 等；本页所述测试锚点不能扩展该证据链的适用范围。^[involution-table.md:11-12, involution-table.md:103-106]

上游 `involutions.h` 与 `involutions.cpp` 的行号转述自 Rust 源码注释，来源包未独立重读上游，行号可能随版本漂移。`ModTwoSubspace`、`RealProjection` 与 `CayleyCrossDecomposition` 的完整展开也不在本包范围内。^[involution-table.md:98-102]

## Sources

- [involution-table.md](involution-table.md) — Twisted involution 表（KGB stage b）。
