---
title: twisted-involution-记录的数学不变量
summary: 记录构造检查种子长度公式的奇偶性、BFS 的 Weyl 长度差为 ±2、(2ρ+θ·2ρ)/2 的整性及传送投影与新 θ 的一致性；这些结构性检查不构成本包的数学验收。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:27.697Z"
updatedAt: "2026-10-10T02:13:56.344Z"
tags:
  - 扭曲对合
  - 构造校验
  - 数学不变量
aliases: []
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Twisted involution 记录的数学不变量

Twisted involution 记录是 KGB 构建流水线 stage b 的条目，按 Cartan 类轨道连续存储。每条记录包含词级 Weyl 元素、矩阵级 `TwistedInvolution`（对合 θ 与根分类）、模二去重子空间、$(1+\theta)\rho$、对合长度与 Weyl 长度，以及 $(1-\theta)X^*$ 的图像基对 `lift_mat`、`M_real`。存储组织见 [[Twisted involution 表与 Cartan 轨道存储]]。^[involution-table.md:20-24, involution-table.md:33-35]

## θ 与记录字段的一致性

除图像基对外，记录字段均在入表时由 θ 典范导出。B2 测试逐条核对 θ 的根作用为 `weyl.image(δ.image(root))`，即先施加 distinguished involution δ，再施加记录中的 Weyl 元素，满足 $\theta=w\delta$。^[involution-table.md:26-29, involution-table.md:85-89]

`theta_plus_one_rho` 按 $(1+\theta)\rho=(2\rho+\theta(2\rho))/2$ 计算，其中 $2\rho$ 在建表时从 positivity slice 导出。若分子出现奇坐标，`push_record` 报 `"theta rho parity"`，检查逐坐标除以 2 的整性。^[involution-table.md:39-42, involution-table.md:57-59]

## 长度公式与 BFS 步进

Cartan 轨道种子先经 `WeylElement::from_action` 从矩阵级代表元转换为词级元素，再按 $\ell_{\mathrm{inv}}=(\ell_W+\#\mathrm{Cayley})/2$ 计算对合长度。分子为奇数时报 `"length parity"`。`CayleyCrossDecomposition` 按 Cartan 类构造，不会为每条记录重建；B2 测试逐条核对该长度公式。^[involution-table.md:44-49, involution-table.md:88-89]

[[Twisted cross-action 的 BFS 轨道构建|外序 BFS]] 以 $s_g\,w\,s_{\mathrm{twist}(g)}$ 生成邻居，并以前向根置换去重。对新邻居，`stepped_length` 要求 Weyl 长度差恰为 $\pm2$，按来源记载的差值约定将对合长度相应调整为 $\mp1$；否则报 `"twisted length step"`。长度差为 0 表示该边固定对合，这种情况已由先行去重处理。^[involution-table.md:51-54]

## 图像基的路径依赖与相容性

图像基对是 θ 典范导出规则的例外：它在轨道典范 involution 处由 $1-\theta$ 的阶梯形约化播种，随后沿 cross-action BFS 传送。该基具有路径依赖性，且 `y_lift` 的符号依赖于它，相关纪律见 [[图像基的典范播种与轨道传送纪律]]。^[involution-table.md:26-29]

种子投影由 `RealProjection::build` 构造。沿边传送使用普通生成元 $s$ 的矩阵，而非 $\mathrm{twist}(s)$，因为 δ 已并入 θ。传送投影在被新记录采用前，须通过 `check_against` 与该记录新鲜推导的 θ 核对。^[involution-table.md:55-59]

B2 投影传送测试中的 arm64 oracle 锚点表明：对于 $\theta=[[-1,0],[2,1]]$，记录中的 `lift_mat` 为 `[[2],[-2]]`，现场重算则得到 `[[-2],[2]]`。测试以 `assert_ne` 固定两者的差异，说明传送所得图像基不必与现场重算结果相同。^[involution-table.md:90-92]

## 轨道完整性与置换键唯一性

`add_cartan` 从 classification 取得种子和期望轨道大小，要求生成轨道恰好填满该大小，否则报 `InvolutionTableInvariantViolation { invariant: "orbit size" }`。重复添加同一 Cartan 类返回已有切片，详见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:44-47]

`push_record` 使用没有碰撞检查的 `BTreeMap::insert` 插入置换索引，但来源的更正结论是：**静默覆盖在表自身的不变量下数学上不可达**。键是 Weyl 因子 $w$ 的完整根置换，它忠实决定 $w$，而固定 δ 后 $w$ 又唯一决定 $\theta=w\delta$。同一内类的 Cartan 轨道是两两不交的扭曲共轭类，因此不同 Cartan 添加过程产生的键集合不交；同类重复添加由幂等检查拦下，同一 BFS 内的重复则先由 lookup 消费。轨道间键不重叠因此不是额外的调用方纪律。^[involution-table.md:60-65]

保留的调用方契约涉及 `lookup` 的外来根系输入：跨根系的同形置换可能撞键，而根数匹配是唯一结构性防线。同基数外来根系元素的适用性仍由调用方负责，参见 [[前向根置换索引及其调用方契约]]。^[involution-table.md:66-67, involution-table.md:73-74]

## 测试与证据边界

A1 测试逐字段锚定 fundamental 记录的 `theta_plus_one_rho = [1]`、`mod_space` 秩 0，以及分裂记录的 `[0]`、秩 1。B2 测试核对全表与分类的一致性、两次独立建表的逐切片可复现性，并检查逐记录 θ、长度公式、ρ 公式及投影传送差异。^[involution-table.md:85-92]

来源列出的未触及分支包括四个不变量错误字面量对应的错误路径、`AllocationFailed`，以及 `lookup` 返回 `Some` 的直接命中断言。相关覆盖范围见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:93-94]

本页依据源码结构性阅读。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；上述测试描述是源码中的测试锚点，而非本次执行结果。上游 C++ 行号转述自源码注释，未独立重读上游，可能随版本变化。^[involution-table.md:9-16, involution-table.md:104-109]

## Sources

- [involution-table.md](../../sources/involution-table.md) — Twisted involution 表（KGB stage b）。
