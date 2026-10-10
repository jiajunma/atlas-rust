---
title: Involution 表的测试锚点与证据边界
summary: 来源记录七组测试锚点及不变量、分配失败等覆盖缺口，但未执行测试或独立核对上游字节，不构成数学或性能验收。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:53:08.739Z"
updatedAt: "2026-10-10T00:36:50.247Z"
tags:
  - 测试覆盖
  - 证据边界
  - 对合
aliases:
  - involution-表的测试锚点与证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Involution 表的测试锚点与证据边界
summary: 七组测试锚点覆盖轨道、记录字段、可复现性、投影传送与部分守卫；来源属于结构性阅读，不提供测试执行通过报告或数学、性能验收。
sources:
  - involution-table.md
kind: concept
tags:
  - 测试覆盖
  - 证据边界
aliases:
  - involution-表的测试锚点与证据边界
provenanceState: extracted
---

# Involution 表的测试锚点与证据边界

`InvolutionTable` 位于 KGB 构建流水线的 stage b，按 Cartan 类连续存储 twisted involution 轨道。来源记录七组测试锚点，覆盖轨道构造、记录字段、可复现性、Cayley 查询、投影传送和资源守卫。这些记录来自结构性源码阅读，来源包没有执行测试。^[involution-table.md:20-24, involution-table.md:79-88, involution-table.md:103-106]

## 七组测试锚点

### 1. A1：轨道、幂等性与字段

A1 分裂案例检查两单元轨道、重复添加的幂等性及逐字段值。fundamental 记录的 `theta_plus_one_rho = [1]`、`mod_space` 秩为 0；分裂记录对应 `[0]`、秩为 1。这些检查对应 [[Twisted involution 表与 Cartan 轨道存储]] 的构造与记录语义。^[involution-table.md:44-45, involution-table.md:79-81]

### 2. B2：全表对账与可复现性

B2 测试将全表与分类结果对账，并比较两次独立建表所得的各个轨道切片。编号遵循调用方的 Cartan 添加顺序，文档纪律要求升序 `CartanId`；轨道内部按 external-order BFS 排列，`InvolutionId` 跨 Cartan 全局连续递增。^[involution-table.md:33-35, involution-table.md:81-82]

### 3. B2：逐条记录的不变量

B2 逐条记录检查 θ 的作用像为 `weyl.image(δ.image(root))`，对合长度满足 `(W_length + #Cayley)/2`，并核对 `theta_plus_one_rho` 的计算式 `(2ρ + θ·2ρ)/2`。这些断言对应 [[Twisted involution 记录的数学不变量]]。^[involution-table.md:47-49, involution-table.md:57-59, involution-table.md:82-83]

### 4. Cayley：目标 Cartan 加入前后

Cayley 边测试检查目标 Cartan 类加入前后，查询结果由 `None` 变为 `Some`。stage-(e) 的调用契约要求先添加该 form 的向上封闭 Cartan 集合；满足这一前提后仍返回 `None`，即表示调用方违反不变量，参见 [[Cayley 邻居查询与向上封闭 Cartan 集合]]。^[involution-table.md:73-75, involution-table.md:83-84]

### 5. 扭转 A2：轨道大小

扭转 A2 测试检查轨道大小排序后为 `[1, 3]`，锚定该案例的轨道大小分布。^[involution-table.md:84-84]

### 6. B2：路径依赖的投影传送

image-basis 对是记录字段中的例外：它在轨道典范 involution 处由 $1-\theta$ 的阶梯归约播种，再沿 cross-action BFS 传送。该基具有路径依赖性，且 `y_lift` 的符号依赖于它，详见 [[图像基的典范播种与轨道传送纪律]]。^[involution-table.md:26-29]

来源标为 arm64 oracle 的 B2 锚点针对 $\theta=\begin{pmatrix}-1&0\\2&1\end{pmatrix}$：记录中的 `lift_mat` 为 $\begin{pmatrix}2\\-2\end{pmatrix}$，现场重算则得到 $\begin{pmatrix}-2\\2\end{pmatrix}$。测试使用 `assert_ne` 检查二者不同，保留传送与重算之间的符号差异。^[involution-table.md:84-86]

### 7. 预算、越界与外来系统

守卫测试覆盖上限为 1 时拒绝添加第二个 Cartan、`CartanId(9)` 越界，以及外来系统的 `lookup` 返回 `None`。条目容量允许达到上限，但在 `records.len() == max_involutions` 时继续插入会被拒绝，返回资源名为 `"involutions"` 的 `InvolutionTableResourceLimit`，参见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:62-63, involution-table.md:86-88]

## 未覆盖分支与调用方契约

来源明确列出的未触及分支包括四个不变量错误字面量、`AllocationFailed`，以及对 `lookup` 的 `Some` 命中的直接断言。四个不变量错误分别为轨道大小不符的 `"orbit size"`、种子长度奇偶性错误的 `"length parity"`、非法长度步进的 `"twisted length step"`，以及计算 $(1+\theta)\rho$ 时出现奇坐标的 `"theta rho parity"`。^[involution-table.md:44-59, involution-table.md:87-88]

结构性阅读还发现，种子插入 `index_by_permutation` 时没有碰撞检查：`BTreeMap::insert` 会静默覆盖同键值，依赖不同 Cartan 轨道键不重叠的调用纪律。`lookup` 以前向根置换为键，同基数外部系统仍受调用方契约约束。^[involution-table.md:60-61, involution-table.md:67-68]

## 证据范围

来源记录了 2026-10-03 初读与 2026-10-06 重读，两次所读字节未变、SHA-256 相同，并保留各自的阅读快照。两份草案经 Kimi probe 起草，再由维护者逐条对照源码核对、改写合并；这是文档的结构性审阅过程。^[involution-table.md:9-16]

来源包未执行任何构建、测试或原版运行，不包含数学验收、性能或并行结论。表的正确性属于其自身的 [[HPC 验收证据链]]，包括 KGB/capacity gate 等；本包不重述或扩展该证据链，七组测试锚点也不构成执行通过报告。^[involution-table.md:11-12, involution-table.md:79-88, involution-table.md:103-106]

上游 `involutions.h`、`involutions.cpp` 的行号均转述自源码注释，来源包未独立重读上游，行号可能随版本演进而漂移。`ModTwoSubspace`、`RealProjection` 与 `CayleyCrossDecomposition` 的详细展开属于后续来源包。^[involution-table.md:98-102]

## Sources

- [involution-table.md](../../sources/involution-table.md) — Twisted involution 表（KGB stage b）。
