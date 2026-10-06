---
title: Twisted involution 表（KGB stage b）
source: atlas-rust/involution-table
ingestedAt: 2026-10-03T10:32:42Z
---

# Twisted involution 表（KGB stage b）

编辑状态：**结构性阅读（两次：2026-10-03 初读、2026-10-06 重读，字节未变，
SHA-256 相同）；两份草案均经 Kimi probe 起草、维护者逐条对照源码核对后改写合并**。
本包解释 `involution_table.rs`（840 行）的记录格式、播种/传送与访问语义；其正确性属于
它自己的 HPC 证据链（KGB/capacity gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-involution-table.json`](snapshots/2026-10-03-involution-table.json)
（初读）与
[`snapshots/2026-10-06-weak-real-form-involution-table.json`](snapshots/2026-10-06-weak-real-form-involution-table.json)
（重读，同一 SHA-256 `aab8c87b…`，与 weak_real_form.rs 同包进行）。

## 定位

KGB 构建流水线的 stage b：按 Cartan 类连续存储 twisted involution 的轨道。
每条记录携带：word-level 元素、matrix-level `TwistedInvolution`（theta 加
root 分类）、mod-2 dedup 子空间、$(1+\theta)\rho$、两个长度
（involution length 与 Weyl length），以及 $(1-\theta)X^*$ 的 image-basis 对
（`lift_mat`、`M_real`）。

**播种与传送**：除 image-basis 对外，所有记录字段在入表时由 theta 典范导出；
image-basis 对是例外——在轨道典范 involution 处由 $1-\theta$ 的 echelon
reduction 播种（involutions.cpp:196-208），随后沿 cross-action BFS 传送
（involutions.cpp:242-243），因为该基是路径依赖的且 `y_lift` 的符号依赖于它。

## 编号与存储

编号是调用方的 Cartan 添加顺序（文档纪律：升序 `CartanId`），每条轨道内部按
external-order BFS 排列。`InvolutionId` 跨 Cartan 全局连续递增。
`orbit_slice(cartan)` 返回连续轨道切片及其起始编号。

## 构建

`InvolutionTable::new` 为一个 inner class 建空表（inner class 同时拥有
datum、root system 与 distinguished involution，无需 cross-input gate），并
一次性导出验证过的 simple twist、rank 个 simple-reflection 元素（BFS 边
**不**每次调用重建反射）和来自 positivity slice 的 $2\rho$。

`add_cartan(classification, cartan)` 把一个 Cartan 类的轨道生成为连续切片；
**幂等**（重复添加返回已有切片）；种子与期望大小均来自 classification，生成
的轨道必须恰好填满期望大小（`InvolutionTableInvariantViolation { invariant:
"orbit size" }`）。种子经 `WeylElement::from_action` 从矩阵级代表元转换一次，
再应用 (W_length + #Cayley)/2 公式（奇数报 `"length parity"`）；
`CayleyCrossDecomposition` 是 per-class 工具——绝不在每个条目上重建。

外序 BFS（2026-10-06 重读补充）：邻居 = `s_g · current · s_{twist(g)}`（词级
两次 `multiply`，作用级两次 `compose`）；去重键 = 前向根置换；新长度
`stepped_length`——Weyl 长度差恰 ±2 则对合长度 ∓1，否则
`"twisted length step"`（差 0 意味着边固定该对合，去重命中已先行消费）；
投影用**普通生成元 s 而非 twist(s)** 的矩阵传送（δ 已并入 θ，
involutions.cpp:242-243）；每访问一个节点推入一条 `cross_links`（此后
`cross` 是存储直查）。`push_record` 里，`(1+θ)ρ` 以 `(2ρ + θ·2ρ)/2` 计算
（奇坐标报 `"theta rho parity"`）；传送的投影先 `check_against` 本记录新鲜
推导的 θ 再采用（边数学对账），种子处则 `RealProjection::build`。
**种子插入 `index_by_permutation` 无碰撞检查**——`BTreeMap::insert` 同键
静默覆盖，依赖不同 Cartan 轨道键不重叠的调用纪律（阅读观察）。
条目上限是包含式（`records.len() == max_involutions` 即拒，
`InvolutionTableResourceLimit { resource: "involutions" }`）。

## 访问器

- `lookup(&WeylElement)`：以 forward root permutation 为键（stage (a) 已把
  它固定为完整相等性键；同基数的外部系统仍是调用方契约）。
- `record(id)`、`involution_count()`、`root_system()`、`inner_class()`、
  `cartan_of(id)`（轨道切片扫描）。
- `cross(generator, id)`：存储的 cross-action 链接 $s \cdot w \cdot
  \mathrm{twist}(s)$，构建后 O(1)（此处仅转述文档，不作性能结论）。
- `cayley(generator, id)`：Cayley 邻居 $s \cdot w$，经反射乘积加置换查表
  计算；其 Cartan 类尚未添加时返回 `None`——stage-(e) 契约要求先添加该
  form 的 upward-closed Cartan 集合，此后 `None` 即调用方违反不变量。
- `simple_root_kind(id, generator)`：一个访问器覆盖上游的三个
  `is_*_simple` 测试。

测试锚点（2026-10-06 重读补充，7 个）：A1 分裂两单元轨道 + 幂等 + 逐字段
锚定（fundamental 的 `theta_plus_one_rho = [1]`、`mod_space` 秩 0；分裂的
`[0]`、秩 1）；B2 全表与分类对账且两次独立建表逐切片相等（可复现性）；
B2 每条记录的 θ 典范性（像 = `weyl.image(δ.image(root))`、(W+Cayley)/2
公式、`(2ρ+θ·2ρ)/2`）；Cayley 边在其 Cartan 加入前后（`None` → `Some`）；
扭转 A2 轨道大小排序后 `[1, 3]`；**B2 投影传送锚点**（arm64 oracle：
θ=`[[-1,0],[2,1]]` 的记录其 `lift_mat == [[2],[-2]]`，而现场重算得
`[[-2],[2]]`，`assert_ne`）；预算/越界守卫（上限 1 时第二 Cartan 拒、
`CartanId(9)` 越界、外来系统 `lookup` 为 `None`）。未触分支：四个不变量
错误字面量、`AllocationFailed`、`lookup` 的 `Some` 命中直断。

## 来源与限制

- 源码：[involution_table.rs](../../../crates/atlas-real-group/src/involution_table.rs)；
  阅读快照
  [`2026-10-03-involution-table.json`](snapshots/2026-10-03-involution-table.json)
  （初读）与
  [`2026-10-06-weak-real-form-involution-table.json`](snapshots/2026-10-06-weak-real-form-involution-table.json)
  （重读，同一 SHA-256 `aab8c87b…`）。
- 上游行号均转述自源码注释（involutions.h/involutions.cpp），未独立重读
  上游，随版本演进可能漂移。
- 关联：[KGB 图结构](kgb-graph-structure.md)、[Weyl 群层](weyl-layer.md)、
  [Cartan 分类](cartan-classification.md)；`ModTwoSubspace`、
  `RealProjection`、`CayleyCrossDecomposition` 的展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 初读起草经由本地 Kimi probe（exit 0，115.5s）；重读同样经 Kimi probe
  （1600s 期限，exit 0，658.1s），其 BFS 细节、push_record 不变量与 7 个测试
  锚点均精确，已并入正文。调用记录见两份快照的 `kimi_assist`。
