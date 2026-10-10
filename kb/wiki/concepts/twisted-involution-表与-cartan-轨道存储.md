---
title: Twisted involution 表与 Cartan 轨道存储
summary: KGB stage b 按调用方添加 Cartan 类的顺序连续存储轨道，类内采用外部生成元顺序 BFS，InvolutionId 跨类连续编号。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:18.743Z"
updatedAt: "2026-10-10T03:34:30.487Z"
tags:
  - 扭曲对合
  - KGB
  - 轨道存储
aliases:
  - twisted-involution-表与-cartan-轨道存储
  - TI表C轨
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Twisted involution 表与 Cartan 轨道存储

Twisted involution 表位于 KGB 构建流水线的 stage b，按 Cartan 类连续存储扭曲对合轨道。每条记录包含词级 Weyl 元素、矩阵级 `TwistedInvolution`（$\theta$ 与根分类）、模二去重子空间、$(1+\theta)\rho$、对合长度与 Weyl 长度，以及 $(1-\theta)X^*$ 的图像基对 `lift_mat`、`M_real`。^[involution-table.md:20-24]

## 编号与存储布局

轨道排列由调用方添加 Cartan 类的顺序决定，文档纪律要求按 `CartanId` 升序添加。每条轨道内部按外部生成元顺序执行广度优先搜索（BFS），`InvolutionId` 跨 Cartan 类全局连续递增。`orbit_slice(cartan)` 返回对应的连续轨道切片及其起始编号。^[involution-table.md:33-35]

## 初始化与轨道添加

`InvolutionTable::new` 为一个 inner class 创建空表。该 inner class 同时拥有 datum、根系与 distinguished involution，因此无需跨输入门控。初始化一次性导出经过验证的 simple twist、数量等于秩的简单反射元素，以及来自 positivity slice 的 $2\rho$；BFS 遍历边时复用这些反射元素。^[involution-table.md:39-42]

`add_cartan(classification, cartan)` 将一个 Cartan 类的轨道生成为连续切片，重复添加则返回已有切片。种子与期望大小均来自 classification，生成轨道必须恰好达到期望大小，否则报告 `"orbit size"` 不变量错误。种子通过 `WeylElement::from_action` 从矩阵级代表元转换一次，再按 $(W_{\mathrm{length}}+\#\mathrm{Cayley})/2$ 计算对合长度；分子为奇数时报 `"length parity"`。`CayleyCrossDecomposition` 按类使用，不为每条记录重建。^[involution-table.md:44-49]

记录总数允许达到 `max_involutions`，达到上限后继续插入会被拒绝，返回 `InvolutionTableResourceLimit { resource: "involutions" }`。相关约束见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:68-69]

## BFS、去重与置换索引

BFS 邻居为 $s_g\cdot w\cdot s_{\mathrm{twist}(g)}$，词级通过两次 `multiply` 构造，作用级通过两次 `compose` 构造，并以前向根置换作为去重键。对未被去重处理的新邻居，`stepped_length` 要求 Weyl 长度差恰为 $\pm2$，对应对合长度变化为 $\mp1$，否则报告 `"twisted length step"`。长度差为零意味着该边固定当前对合，已先由去重命中处理。详见 [[Twisted cross-action 的 BFS 轨道构建]]。^[involution-table.md:51-54]

每访问一个节点，构建过程便保存一条 `cross_links` 记录。构建后的 `cross(generator, id)` 直接查询已存储的 cross-action 链接；来源将其描述为 O(1) 访问，但未提供实测性能结论。^[involution-table.md:55-57, involution-table.md:77-78]

`push_record` 使用不检查碰撞的 `BTreeMap::insert` 写入 `index_by_permutation`，但来源核对确认：**静默覆盖在表自身的不变量下数学上不可达**。键是 Weyl 因子 $w$ 的完整根置换，忠实决定 $w$；固定的 $\delta$ 又使 $w$ 唯一决定 $\theta=w\delta$。同一内类的不同 Cartan 轨道是互不相交的扭曲共轭类，因此不同轨道的键集合不交；同类重复添加由幂等检查拦截，同一 BFS 内的重复则先由 lookup 消费。^[involution-table.md:60-65]

仍需调用方保证的是 `lookup` 对同基数外来根系元素的使用契约：跨根系的同形置换可能撞键，而根数匹配是唯一结构性防线。这与表内不同 Cartan 轨道的键不相交是不同问题，参见 [[前向根置换索引及其调用方契约]]。^[involution-table.md:66-67, involution-table.md:73-74]

## 记录字段与图像基传送

除图像基对外，所有记录字段均在入表时由 $\theta$ 典范导出。图像基对在轨道典范 involution 处通过 $1-\theta$ 的阶梯归约播种，随后沿 cross-action BFS 传送。该基具有路径依赖性，`y_lift` 的符号也依赖于它，因此传送纪律是记录语义的一部分。详见 [[图像基的典范播种与轨道传送纪律]]。^[involution-table.md:26-29]

投影使用普通生成元 $s$ 的矩阵传送，而非 $\mathrm{twist}(s)$，因为 $\delta$ 已并入 $\theta$。传送后的投影先通过 `check_against` 与新记录重新推导的 $\theta$ 核对，再被采用；种子处使用 `RealProjection::build`。`push_record` 按 $(2\rho+\theta\cdot2\rho)/2$ 计算 $(1+\theta)\rho$，若出现奇坐标则报告 `"theta rho parity"`。^[involution-table.md:55-59]

## 查询语义

`lookup(&WeylElement)` 以前向根置换为键。表还提供 `record(id)`、`involution_count()`、`root_system()` 和 `inner_class()`；`cartan_of(id)` 通过扫描轨道切片确定所属 Cartan 类。^[involution-table.md:73-76]

`cayley(generator, id)` 通过反射乘积与置换查表计算邻居 $s\cdot w$，目标 Cartan 类尚未添加时返回 `None`。stage e 要求事先添加该实形式向上封闭的 Cartan 集合，此后 `None` 表示调用方违反不变量。`simple_root_kind(id, generator)` 则以一个访问器覆盖上游三个 `is_*_simple` 测试。参见 [[Cayley 邻居查询与向上封闭 Cartan 集合]]。^[involution-table.md:79-83]

## 测试锚点与证据边界

来源列出的七个测试锚点涵盖 A1 轨道、幂等性与逐字段检查，B2 全表与分类对账及独立建表的切片一致性，B2 记录的 $\theta$ 典范性与公式检查，Cartan 添加前后的 Cayley 查询，扭转 A2 的轨道大小，B2 投影传送，以及预算和越界守卫。其中，扭转 A2 的轨道大小排序后为 `[1, 3]`；Cayley 查询在目标 Cartan 加入前后由 `None` 变为 `Some`。^[involution-table.md:85-94]

B2 投影传送测试以 arm64 oracle 为锚点：当记录的 $\theta=\begin{pmatrix}-1&0\\2&1\end{pmatrix}$ 时，传送得到的 `lift_mat` 为 $\begin{pmatrix}2\\-2\end{pmatrix}$，现场重算则为 $\begin{pmatrix}-2\\2\end{pmatrix}$，测试用 `assert_ne` 确认两者不同。这一案例体现了传送与重新播种之间可观察的符号差异。^[involution-table.md:90-92]

未覆盖的分支包括四个不变量错误字面量、`AllocationFailed`，以及 `lookup` 的 `Some` 命中直接断言。详见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:93-94]

来源属于结构性阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。表的正确性属于其自身的 HPC 证据链，来源不重述或扩展；上游行号仅转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[involution-table.md:9-16, involution-table.md:104-109]

## Sources

- [involution-table.md](../../sources/involution-table.md) — Twisted involution 表（KGB stage b）。
