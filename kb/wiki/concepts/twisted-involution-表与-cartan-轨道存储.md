---
title: Twisted involution 表与 Cartan 轨道存储
summary: KGB stage b 按调用方添加 Cartan 类的顺序连续存储轨道，轨道内采用外部生成元顺序的 BFS，并分配全局连续的 InvolutionId。
sources:
  - involution-table.md
kind: concept
createdAt: "2026-10-09T14:52:18.743Z"
updatedAt: "2026-10-09T19:30:53.646Z"
tags:
  - KGB
  - 扭曲对合
  - 数据结构
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

Twisted involution 表位于 KGB 构建流水线的 stage b，按 Cartan 类连续存储 twisted involution 轨道。每条记录携带词级 Weyl 元素、矩阵级 `TwistedInvolution`（包含 $\theta$ 与根分类）、模二去重子空间、$(1+\theta)\rho$、对合长度与 Weyl 长度，以及 $(1-\theta)X^*$ 的图像基对 `lift_mat`、`M_real`。^[involution-table.md:20-24]

## 编号与存储布局

全局编号由调用方添加 Cartan 类的顺序决定，文档纪律要求按 `CartanId` 升序添加。每条轨道内部按外部生成元顺序进行广度优先搜索（BFS），`InvolutionId` 跨 Cartan 类连续递增。`orbit_slice(cartan)` 返回该类的连续轨道切片及其起始编号。^[involution-table.md:33-35]

## 初始化与轨道添加

`InvolutionTable::new` 为一个 inner class 创建空表。该 inner class 已拥有 datum、根系和 distinguished involution，因此无需额外的跨输入门控。初始化一次性导出经过验证的 simple twist、秩个简单反射元素，以及由 positivity slice 得到的 $2\rho$；BFS 遍历边时复用这些反射元素。^[involution-table.md:39-42]

`add_cartan(classification, cartan)` 将一个 Cartan 类的轨道生成为连续切片；重复添加返回已有切片。种子与期望轨道大小均来自 classification，生成结果必须恰好达到期望大小，否则报告 `InvolutionTableInvariantViolation { invariant: "orbit size" }`。种子通过 `WeylElement::from_action` 从矩阵级代表元转换一次，再按 $(W\_length+\#Cayley)/2$ 计算对合长度；分子为奇数时报 `"length parity"`。`CayleyCrossDecomposition` 按类使用，不为每条记录重建。^[involution-table.md:44-49]

容量检查在已有条目数 `records.len() == max_involutions` 时拒绝继续插入，返回 `InvolutionTableResourceLimit { resource: "involutions" }`。相关约束见 [[Cartan 轨道的幂等添加与容量约束]]。^[involution-table.md:62-63]

## BFS、去重与 cross 链接

BFS 邻居为 $s_g\cdot w\cdot s_{\mathrm{twist}(g)}$，词级通过两次 `multiply` 构造，作用级通过两次 `compose` 构造，并以前向根置换作为去重键。对尚未被去重处理的邻居，`stepped_length` 要求 Weyl 长度差恰为 $\pm2$，对应对合长度变化为 $\mp1$，否则报告 `"twisted length step"`；长度差为零意味着该边固定当前对合，已先由去重命中处理。参见 [[Twisted cross-action 的 BFS 轨道构建]]。^[involution-table.md:51-54]

每访问一个节点，构建过程便保存一条 `cross_links` 记录。构建后的 `cross(generator, id)` 因而直接查询已存储的 $s\cdot w\cdot\mathrm{twist}(s)$ 链接；来源将该访问描述为 O(1)，这是存储结构说明，不是实测性能结论。^[involution-table.md:55-57, involution-table.md:71-72]

种子插入 `index_by_permutation` 时不检查碰撞，`BTreeMap::insert` 会静默覆盖相同键。因此，不同 Cartan 轨道的键互不重叠仍依赖调用方纪律。^[involution-table.md:60-61]

## 记录字段与图像基传送

除图像基对外，所有记录字段都在入表时由 $\theta$ 典范导出。图像基对在轨道的典范 involution 处，通过 $1-\theta$ 的列阶梯归约播种，再沿 cross-action BFS 传送。该基具有路径依赖性，`y_lift` 的符号也依赖于它，因此传送基与现场重新计算的基需要区分。参见 [[实投影像基的播种与路径依赖传送]]。^[involution-table.md:26-29]

投影使用普通生成元 $s$ 的矩阵传送，而非 $\mathrm{twist}(s)$，因为 $\delta$ 已并入 $\theta$。传送后的投影先通过 `check_against` 与新记录重新推导的 $\theta$ 核对，再被采用；种子处使用 `RealProjection::build`。`push_record` 还按 $(2\rho+\theta\cdot2\rho)/2$ 计算 $(1+\theta)\rho$，若出现奇坐标则报告 `"theta rho parity"`。^[involution-table.md:55-59]

## 查询语义与调用契约

`lookup(&WeylElement)` 以前向根置换为键；stage a 已将其确定为完整相等性键，但同基数外部系统的兼容性仍属于调用方契约。表还提供 `record(id)`、`involution_count()`、`root_system()` 和 `inner_class()`；`cartan_of(id)` 通过扫描轨道切片确定所属 Cartan 类。^[involution-table.md:67-70]

`cayley(generator, id)` 通过反射乘积与置换查表计算邻居 $s\cdot w$，目标 Cartan 类尚未添加时返回 `None`。stage e 要求事先添加该实形式向上封闭的 Cartan 集合，在这一契约下，`None` 表示调用方违反不变量。`simple_root_kind(id, generator)` 则以一个访问器覆盖上游三个 `is_*_simple` 测试。参见 [[Cayley 邻居查询与向上封闭 Cartan 集合]]。^[involution-table.md:73-77]

## 测试与证据边界

来源列出的七个测试锚点涵盖 A1 轨道、幂等性与逐字段检查，B2 全表与分类对账、两次独立建表的切片一致性、记录的 $\theta$ 典范性及长度公式，Cartan 添加前后的 Cayley 查询，扭转 A2 的轨道大小，以及预算和越界守卫。扭转 A2 的轨道大小排序后为 `[1, 3]`。^[involution-table.md:79-88]

B2 投影传送测试明确锚定路径依赖：对 $\theta=[[-1,0],[2,1]]$ 的记录，传送得到的 `lift_mat` 为 `[[2],[-2]]`，现场重算则为 `[[-2],[2]]`，测试用 `assert_ne` 确认差异。^[involution-table.md:84-86]

未覆盖的分支包括四个不变量错误字面量、`AllocationFailed`，以及 `lookup` 的 `Some` 命中直接断言。来源包属于结构性阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；上游行号仅转述自源码注释，未独立重读上游。更完整的说明见 [[Involution 表的测试锚点与证据边界]]。^[involution-table.md:87-88, involution-table.md:98-103]

## Sources

- [involution-table.md](involution-table.md) — Twisted involution 表（KGB stage b）。
