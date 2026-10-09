---
title: Involution 表的测试锚点与证据边界
summary: 来源记录七组测试锚点，覆盖 A1、B2、扭转 A2、投影传送及部分守卫，但本包未执行测试、核对上游字节或提供数学与性能验收。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:53:08.739Z"
updatedAt: "2026-10-09T19:31:26.273Z"
tags:
  - 回归测试
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
---

# Involution 表的测试锚点与证据边界

`InvolutionTable` 位于 KGB 构建流水线的 stage b，按 Cartan 类连续存储 twisted involution 轨道。来源记录了七个测试锚点，涉及字段、轨道构造、可复现性、Cayley 查询、投影传送及资源守卫；这些内容来自结构性源码阅读，来源包没有执行构建、测试或原版运行。^[involution-table.md:20-24, involution-table.md:79-88, involution-table.md:103-106]

## 测试锚点

### A1：轨道、幂等性与记录字段

A1 分裂案例覆盖两单元轨道、重复添加的幂等性与逐字段检查。fundamental 记录的 `theta_plus_one_rho = [1]`，`mod_space` 秩为 0；分裂记录对应 `[0]` 与秩 1。这些断言为[[Twisted involution 记录的数学不变量]]提供了具体锚点。^[involution-table.md:79-81]

### B2：全表对账与可复现性

B2 测试将全表与分类结果对账，并比较两次独立建表得到的各个轨道切片，检查结果的可复现性。编号依赖调用方的 Cartan 添加顺序，文档纪律要求按 `CartanId` 升序添加；各轨道内部按 external-order BFS 排列，`InvolutionId` 跨 Cartan 连续递增。^[involution-table.md:33-35, involution-table.md:81-82]

另一项 B2 测试逐条检查记录的 θ 典范性：根的作用像为 `weyl.image(δ.image(root))`，对合长度满足 `(W_length + #Cayley)/2`，而 `theta_plus_one_rho` 按 `(2ρ + θ·2ρ)/2` 计算。^[involution-table.md:47-49, involution-table.md:57-59, involution-table.md:82-83]

### Cayley 查询与扭转 A2

Cayley 边测试检查目标 Cartan 类加入前后，查询结果由 `None` 变为 `Some`。对应的[[Cayley 邻居查询与向上封闭 Cartan 集合|调用契约]]要求 stage-(e) 预先添加该 form 的向上封闭 Cartan 集合，此后返回 `None` 即表示调用方违反不变量。^[involution-table.md:73-75, involution-table.md:83-84]

扭转 A2 测试检查轨道大小排序后为 `[1, 3]`，为这一案例的轨道大小分布提供锚点。^[involution-table.md:84-84]

### B2：路径依赖的投影传送

image-basis 对不同于其他入表字段：它在轨道典范 involution 处由 $1-\theta$ 的阶梯归约播种，再沿 cross-action BFS 传送。该基具有路径依赖性，且 `y_lift` 的符号依赖于它，相关背景见[[图像基的典范播种与轨道传送纪律]]。^[involution-table.md:26-29]

B2 的 arm64 oracle 锚点针对 $\theta=\begin{pmatrix}-1&0\\2&1\end{pmatrix}$，检查传送所得 `lift_mat == [[2],[-2]]`；现场重算则得到 `[[-2],[2]]`，测试以 `assert_ne` 明确检查二者不同。^[involution-table.md:84-86]

### 预算与越界守卫

守卫测试覆盖上限为 1 时拒绝添加第二个 Cartan、`CartanId(9)` 越界，以及外来系统的 `lookup` 返回 `None`。实现允许条目数达到上限，但当 `records.len() == max_involutions` 时拒绝继续插入，返回资源名为 `"involutions"` 的 `InvolutionTableResourceLimit`；相关构造约束见[[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:62-63, involution-table.md:86-88]

## 未覆盖分支与调用方契约

来源明确列出的未触及分支包括四个不变量错误字面量、`AllocationFailed`，以及对 `lookup` 的 `Some` 命中的直接断言。四个不变量错误分别是轨道大小不符的 `"orbit size"`、种子长度奇偶性错误的 `"length parity"`、非法长度步进的 `"twisted length step"`，以及计算 $(1+\theta)\rho$ 时出现奇坐标的 `"theta rho parity"`。^[involution-table.md:44-59, involution-table.md:87-88]

结构性阅读还发现，种子插入 `index_by_permutation` 时没有碰撞检查：`BTreeMap::insert` 会静默覆盖同键值，因此依赖不同 Cartan 轨道键不重叠的调用纪律。`lookup` 以前向根置换为键，同基数外部系统仍受调用方契约约束。^[involution-table.md:60-61, involution-table.md:67-68]

## 证据范围

来源记录了 2026-10-03 初读与 2026-10-06 重读，两次所读字节未变、SHA-256 相同，并保留了阅读快照。两份草案经 Kimi probe 起草，再由维护者逐条对照源码核对、改写合并。^[involution-table.md:9-16]

这些阅读记录不构成测试执行、数学验收、性能或并行结论。表的正确性属于其自身的[[HPC 验收证据链]]，包括 KGB/capacity gate 等；来源包明确不重述或扩展该证据链。^[involution-table.md:11-12, involution-table.md:103-106]

上游 `involutions.h` 与 `involutions.cpp` 的行号转述自源码注释，来源包未独立重读上游，行号可能随版本演进而漂移。`ModTwoSubspace`、`RealProjection` 与 `CayleyCrossDecomposition` 的详细展开也属于后续来源包。^[involution-table.md:98-102]

## Sources

- [involution-table.md](involution-table.md) — Twisted involution 表（KGB stage b）。
