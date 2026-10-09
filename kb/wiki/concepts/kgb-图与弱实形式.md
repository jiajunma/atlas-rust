---
title: KGB 图与弱实形式
summary: KGB 集合表示 K 在旗簇 G/B 上的轨道；本实现每个弱实形式对应一张由 RealFormSeed 生成、以 Tits 元素为顶点的 KgbGraph。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:53:51.245Z"
updatedAt: "2026-10-09T20:57:00.428Z"
tags:
  - KGB
  - 实形式
  - 数据结构
aliases:
  - kgb-图与弱实形式
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KGB 图与弱实形式
summary: 每张 KgbGraph 对应一个弱实形式，由 RealFormSeed 生成，通过分窗两相 BFS 枚举 Tits 元素并按 tau packet 标准化编号。
sources:
  - kgb-graph-structure.md
kind: concept
tags:
  - KGB
  - 实形式
aliases:
  - kgb-图与弱实形式
provenanceState: extracted
---

# KGB 图与弱实形式

KGB 集合是 $K$ 在旗簇 $G/B$ 上的轨道集合。本实现中，一张 `KgbGraph` 对应一个弱实形式，由 stage-(d) 的 `RealFormSeed` 生成；模块文档称这一阶段为 KGB stage e。图中元素是该弱实形式各对合（involution）之上的 Tits 元素，每个元素在每个单生成元处携带状态、cross 链接，以及可能存在的 Cayley 与 inverse-Cayley 链接。^[kgb-graph-structure.md:19-23]

## 构造与弱实形式的绑定

`KgbGraph::build` 接收 `InnerClass`、Cartan 分类、强实形式分类、可变的 `InvolutionTable` 和种子。该弱实形式的 Cartan 集合决定构造时需要添加的 Cartan 类；强实形式分类提供的 `kgb_size` 则作为 BFS 完成后元素总数的校验目标。相关机制见 [[KGB 构造的前置门控与不变量]]。^[kgb-graph-structure.md:38-55]

前置检查首先验证表与 inner class 是否匹配，不匹配报 `DatumMismatch`；随后检查该形式的 `kgb_size` 和 Cartan 集合是否存在，缺失报 `IndexOutOfRange`。通过这些检查后，构造按升序幂等添加该形式的 Cartan 类，再验证种子与表的绑定，因此种子检查发生在这一步表修改之后。^[kgb-graph-structure.md:38-46]

种子必须满足两项条件：恒等 `WeylElement` 的表内 lookup 等于种子的 involution，且 `torus_bits` 已是 `mod_space` 的商代表元，否则报 `KgbInvariantViolation { invariant: "seed element" }`。`TitsCoset` 使用同一个 inner class 完成覆盖整个 BFS 的一次门控。^[kgb-graph-structure.md:44-47]

## 生成元状态与链接

`KgbStatus` 有四种取值：`Complex`、`ImaginaryCompact`、`Real` 和 `ImaginaryNoncompact`。分类先由 involution 表的 `simple_root_kind` 确定 complex、real 或 imaginary，再对 imaginary 情形使用 coset 的 `simple_grading_pregated` 区分 compact 与 noncompact。下降判定中，real 恒为下降，imaginary 恒非下降；complex 则在 cross 目标的 involution 长度更短时为下降。详见 [[KGB 生成元状态与下降判定]]。^[kgb-graph-structure.md:27-34]

Cayley 链接在非紧致 imaginary 状态下存在，其目标的 involution 长度必须恰比源增加一。inverse-Cayley 链接在 real 状态下存在：单个前像对应 II 型，两个前像对应 I 型，后一种情形满足 `first < second`。逆 Cayley 链接在元素编号标准化后通过升序后处理安装，相关接口见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:52-55, kgb-graph-structure.md:75-80]

## 枚举与编号

图采用 [[分窗两相 BFS 构造]]，每窗处理 64 个元素。第一相使用 Rayon 的 `into_par_iter` 并行计算状态及 cross、Cayley 目标，期间表与 coset 只读；第二相顺序执行 `intern`，按 `TitsElement` 去重并分配新编号。每个 `(x, s)` 状态槽只能写入一次，重复写入报 `KgbInvariantViolation`；最终元素总数必须等于预期的 `kgb_size`。^[kgb-graph-structure.md:49-55]

BFS 发现顺序随后经过 [[tau packet 与 KGB 元素编号标准化]]。实现先按 involution 长度、Weyl 长度、`WeylElt::pieces` 字典序排列 involution，再通过计数排序归整各 tau packet：packet 之间遵循 involution 排序，packet 内保持 BFS 发现顺序。前一排序键是严格全序，因此 `sort_unstable` 的稳定性不影响结果；承载编号语义的稳定性来自计数排序。模块声明这一编号复现上游 `KGB::KGB`。^[kgb-graph-structure.md:60-67]

`positions` 保存每个排序位置的 `(InvolutionId, involution 长度, CartanId)`，`first_of_tau` 是长度为 `positions.len()+1` 的累计计数。`tau_packet(position)` 返回对应 packet 的首元素及大小。^[kgb-graph-structure.md:67-70]

## 存储与访问

`statuses`、`cross`、`cayley` 和 `inverse_cayley` 均按 `x * rank + s` 平铺。`cross(x, s)` 返回 `Option<KgbId>`，越界时为 `None`；`cayley(x, s)` 返回 `Result<Option<KgbId>, _>`，其中 `Ok(None)` 表示该生成元不是非紧致 imaginary，错误只来自下标检查。`inverse_cayley(x, s)` 的 `Ok(None)` 则表示该生成元不是 real。^[kgb-graph-structure.md:74-80]

图采用 [[KGB 图的混合自包含存储]]：每个 involution 位置的数据和 cocharacter 都复制进图内。除 `torus_factor` 外，所有访问器均不需要 involution 表；`torus_factor` 仍依赖逐 involution 的 theta 来计算精确有理环面因子。`base_grading` 对应上游 `KGB_base::base_grading`，用于 `var_print_KGB` 的 `Base grading: [...]` 输出头部。^[kgb-graph-structure.md:81-86]

## 证据边界

本页依据的来源属于结构性源码阅读，解释数据布局与构造算法，不构成 KGB 枚举数学正确性、性能或兼容性的验收结论。所读 `kgb_graph.rs` 字节来自 dirty 工作区，并由阅读快照记录；来源未执行构建、测试或原版运行。数学正确性证据属于独立的 HPC gate 链，来源中的上游引用行号则来自源码注释，未经独立重读。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:90-98]

实现具有并行计算结构，但尚不能据此声称多核加速；数学套件的计时运行强制使用 `RAYON_NUM_THREADS=1`。^[kgb-graph-structure.md:49-56]

## Sources

- [kgb-graph-structure.md](../../sources/kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）
