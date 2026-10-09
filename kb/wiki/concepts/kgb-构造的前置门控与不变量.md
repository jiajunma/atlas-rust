---
title: KGB 构造的前置门控与不变量
summary: 构造检查 inner class、形式索引及种子绑定，并要求状态槽仅写一次、Cayley 目标的 involution 长度增加一、最终元素数等于分类预言的 kgb_size；这些检查不构成数学正确性验收。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:53:59.298Z"
updatedAt: "2026-10-09T14:53:59.298Z"
tags:
  - KGB
  - 不变量
  - 错误处理
aliases:
  - kgb-构造的前置门控与不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# KGB 构造的前置门控与不变量

`KgbGraph::build` 从 `RealFormSeed` 生成对应一个弱实形式的 KGB 图。构造过程先检查上下文、分类数据与种子的有效性，再通过分窗两相 BFS 枚举元素，并检查状态写入、Cayley 长度变化和最终元素总数等不变量。^[kgb-graph-structure.md:19-23, kgb-graph-structure.md:38-56]

## 前置门控与执行顺序

构造输入包括 `InnerClass`、Cartan 分类、强实形式分类、可变的 `InvolutionTable` 和种子。首先检查 involution 表与 inner class 是否匹配；不匹配时返回 `DatumMismatch`。随后取得 `strong.kgb_size(form)` 与 `classification.cartan_set(form)`，任一缺失均返回 `IndexOutOfRange`。^[kgb-graph-structure.md:38-41]

两处 `IndexOutOfRange` 的 `upper_bound` 回退构造不同：一处经由 `strong_real_data(CartanId(0))`，另一处使用 `weak_real_form_count()`。来源记录了这一实现差异，但未发现注释说明其是否有意。^[kgb-graph-structure.md:40-43]

分类数据检查之后，构造器按升序幂等地添加该形式的 Cartan 类，进入修改阶段，然后才验证种子与表的绑定。因此，种子检查发生在 Cartan 添加之后。相关机制见 [[Cartan 轨道的幂等添加与容量约束]]。^[kgb-graph-structure.md:43-46]

## 种子绑定与 coset 门控

种子必须同时满足两个条件：恒等 `WeylElement` 在表内的 lookup 结果等于种子的 involution；种子的 `torus_bits` 已经是 `mod_space` 的商代表元。任一条件不满足，均返回 `KgbInvariantViolation { invariant: "seed element" }`。这一检查将种子绑定到当前表中的 involution 及其商代表元约定。^[kgb-graph-structure.md:44-46]

`TitsCoset` 使用同一个 inner class 完成一次门控，该门控覆盖整个 BFS。相关背景见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[kgb-graph-structure.md:46-47]

## BFS 构造不变量

BFS 采用 [[分窗两相 BFS 构造]]：每窗包含 64 个元素，第一相通过 Rayon `into_par_iter` 计算状态分类、cross 和 Cayley 目标，此时表与 coset 保持只读；第二相顺序执行 `intern`，按 `TitsElement` 去重并分配新 id。^[kgb-graph-structure.md:49-51]

状态槽遵守 write-once 约束：同一元素与生成元对 `(x, s)` 不得写入两次，否则返回 `KgbInvariantViolation`。状态分类本身可参见 [[KGB 生成元状态与下降判定]]。^[kgb-graph-structure.md:27-31, kgb-graph-structure.md:52-53]

对于非紧致 imaginary 状态，Cayley 目标必须存在，且目标 involution 的长度必须恰好比源 involution 多一；相应检查以 `"Cayley length step"` 标识。该约束涉及 involution 长度。链接语义见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:53-55]

BFS 结束时，枚举得到的元素总数必须等于强实形式分类预言的 `kgb_size`；相应检查以 `"kgb size"` 标识。它把实际枚举规模与构造前取得的分类数据联系起来。^[kgb-graph-structure.md:38-40, kgb-graph-structure.md:54-55]

## 证据边界

上述内容来自对 `kgb_graph.rs` 的结构性阅读，所读字节属于 dirty 工作区快照。这些检查说明实现如何约束构造过程，并不构成 KGB 枚举数学正确性的验收结论；来源未执行构建、测试或原版运行，正确性需另据 [[HPC 验收证据链]] 判断。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:97-98]

实现包含并行计算结构，但数学套件的计时运行强制 `RAYON_NUM_THREADS=1`，因此不能据此声称已有多核加速证据。^[kgb-graph-structure.md:49-56]

## Sources

- [kgb-graph-structure.md](kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）。
