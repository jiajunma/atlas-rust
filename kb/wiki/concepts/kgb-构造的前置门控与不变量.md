---
title: KGB 构造的前置门控与不变量
summary: 构造依次检查上下文和形式索引、添加 Cartan 类并验证种子绑定，随后约束状态槽只写一次、Cayley 长度增加一及最终元素数符合分类预言。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:53:59.298Z"
updatedAt: "2026-10-09T20:57:12.461Z"
tags:
  - KGB
  - 构造校验
  - 不变量
aliases:
  - kgb-构造的前置门控与不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KGB 构造的前置门控与不变量
summary: KGB 构造依次检查上下文、分类数据与种子绑定，并约束状态槽写入、Cayley 长度变化和最终元素总数；这些实现检查不构成数学验收。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:53:59.298Z"
updatedAt: "2026-10-10"
tags:
  - KGB
  - 构造校验
aliases:
  - kgb-构造的前置门控与不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# KGB 构造的前置门控与不变量

`KgbGraph::build` 从 `RealFormSeed` 构造对应一个弱实形式的 [[KGB 图与弱实形式|KGB 图]]。构造过程依次检查上下文与分类数据、添加 Cartan 类、验证种子绑定，再通过 BFS 枚举元素，并检查状态写入、Cayley 长度变化及最终元素总数。^[kgb-graph-structure.md:19-23, kgb-graph-structure.md:38-56]

## 前置检查与修改顺序

构造输入包括 `InnerClass`、Cartan 分类、强实形式分类、可变的 `InvolutionTable` 和种子。首先检查 involution 表与 inner class 是否匹配，不匹配时报 `DatumMismatch`；随后获取 `strong.kgb_size(form)` 与 `classification.cartan_set(form)`，任一缺失均报 `IndexOutOfRange`。^[kgb-graph-structure.md:38-41]

两处 `IndexOutOfRange` 的 `upper_bound` 回退构造不同：一处经由 `strong_real_data(CartanId(0))`，另一处使用 `weak_real_form_count()`。来源记录了这一差异，但未见注释说明其是否有意。^[kgb-graph-structure.md:40-43]

分类数据检查完成后，构造器按升序幂等地添加该形式的 Cartan 类，进入修改阶段，随后才验证种子与表的绑定。因此，种子校验并不先于所有修改；相关机制见 [[Cartan 轨道的幂等添加与容量约束]]。^[kgb-graph-structure.md:43-46]

## 种子绑定与 coset 门控

种子必须同时满足两个条件：恒等 `WeylElement` 在表内的 lookup 结果等于种子的 involution；种子的 `torus_bits` 已经是 `mod_space` 的商代表元。任一条件不满足，均报 `KgbInvariantViolation { invariant: "seed element" }`。^[kgb-graph-structure.md:44-46]

`TitsCoset` 使用同一个 inner class 完成一次门控，覆盖整个 BFS。相关背景见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[kgb-graph-structure.md:46-47]

## BFS 构造不变量

枚举采用 [[分窗两相 BFS 构造]]，以 64 个元素为一窗。第一相通过 Rayon `into_par_iter` 纯计算状态分类、cross 与 Cayley 目标，期间表与 coset 只读；第二相顺序执行 `intern`，按 `TitsElement` 去重并分配新 id。^[kgb-graph-structure.md:49-51]

**状态槽仅写一次。** 同一元素与单生成元对 `(x, s)` 的状态槽若被写入两次，报 `KgbInvariantViolation`。状态本身区分 complex、real、紧致 imaginary 与非紧致 imaginary，参见 [[KGB 生成元状态与下降判定]]。^[kgb-graph-structure.md:27-31, kgb-graph-structure.md:52-53]

**Cayley 目标存在且长度增加一。** 对非紧致 imaginary 状态，Cayley 目标必须存在，其 involution 长度必须恰比源 involution 多一；对应不变量标识为 `"Cayley length step"`。链接语义见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:53-55]

**最终元素数符合分类预言。** BFS 结束时，元素总数必须等于强实形式分类给出的 `kgb_size`；对应不变量标识为 `"kgb size"`。^[kgb-graph-structure.md:54-55]

## 证据边界

这些门控和不变量来自对 `kgb_graph.rs` 的结构性阅读，所读字节属于 dirty 工作区，记录于 `snapshots/2026-10-03-kgb-graph.json`。它们描述实现对构造过程的约束，不构成 KGB 枚举数学正确性的验收；来源未执行构建、测试或原版运行，正确性证据属于独立的 HPC gate 链。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:97-98]

实现具有并行计算结构，但数学套件的计时运行强制 `RAYON_NUM_THREADS=1`，因此这一结构本身不提供多核加速证据。^[kgb-graph-structure.md:49-56]

## Sources

- [kgb-graph-structure.md](../../sources/kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）。
