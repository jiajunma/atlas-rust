---
title: KGB 图与弱实形式
summary: KGB 集合描述 K 在旗簇 G/B 上的轨道；本实现从 RealFormSeed 构造每个弱实形式对应的 KgbGraph，元素为各 involution 之上的 Tits 元素。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:53:51.245Z"
updatedAt: "2026-10-09T14:53:51.245Z"
tags:
  - KGB
  - 弱实形式
  - 数学对象
aliases:
  - kgb-图与弱实形式
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# KGB 图与弱实形式

KGB 集合是 $K$ 在旗簇 $G/B$ 上的轨道集合。在本实现中，一张 `KgbGraph` 对应一个弱实形式，由 stage-(d) 的 `RealFormSeed` 种子生成，模块文档将这一阶段称为 KGB stage e。图中的元素是该弱实形式各 involution 之上的 Tits 元素；每个元素在每个单生成元处记录状态、cross 链接，以及可能存在的 Cayley 与 inverse-Cayley 链接。^[kgb-graph-structure.md:19-23]

## 弱实形式如何限定图的构造

`KgbGraph::build` 接收 `InnerClass`、Cartan 分类、强实形式分类、可变的 `InvolutionTable` 和种子。构造依赖该弱实形式的 Cartan 集合与强实形式分类给出的 `kgb_size`：前者限定需要添加的 Cartan 类，后者作为 BFS 结束时元素总数的校验目标。相关机制见 [[KGB 构造的前置门控与不变量]]。^[kgb-graph-structure.md:38-55]

构造首先检查 involution 表与 inner class 是否匹配，以及所需的大小和 Cartan 集合是否存在；随后按升序幂等添加该形式的 Cartan 类。种子还必须满足两项绑定条件：恒等 `WeylElement` 在表中的 lookup 等于种子的 involution，且 `torus_bits` 已是 `mod_space` 的商代表元，否则返回 `KgbInvariantViolation { invariant: "seed element" }`。`TitsCoset` 使用同一 inner class 完成覆盖整个 BFS 的一次门控。^[kgb-graph-structure.md:38-47]

## 元素状态与链接

`KgbStatus` 包含 `Complex`、`ImaginaryCompact`、`Real` 和 `ImaginaryNoncompact` 四种状态。分类先由 involution 表的 `simple_root_kind` 确定根是 complex、real 还是 imaginary，再由 coset 的 `simple_grading_pregated` 将 imaginary 分为 compact 与 noncompact。下降判定中，real 恒为下降，imaginary 恒非下降；complex 则在 cross 目标的 involution 长度更短时为下降。详见 [[KGB 生成元状态与下降判定]]。^[kgb-graph-structure.md:27-34]

Cayley 链接仅在非紧致 imaginary 状态下存在；inverse-Cayley 链接仅在 real 状态下存在。逆 Cayley 的单个前像表示 II 型，两个前像表示 I 型，两个前像按元素编号递增排列，并在编号标准化后通过升序后处理安装。相关接口见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:75-80]

## 枚举与编号

图采用 [[分窗两相 BFS 构造]]：每窗处理 64 个元素，第一相使用 Rayon 并行计算状态及 cross、Cayley 目标，期间表与 coset 只读；第二相顺序执行 `intern`，按 `TitsElement` 去重并分配新编号。状态槽只能写入一次；非紧致 imaginary 的 Cayley 目标必须存在，且其 involution 长度恰比源增加一；最终元素数必须等于预期的 `kgb_size`。^[kgb-graph-structure.md:49-55]

BFS 发现顺序还需经过 [[tau packet 与 KGB 元素编号标准化]]。实现先按 involution 长度、Weyl 长度和 `WeylElt::pieces` 字典序排列 involution，再通过计数排序将元素归入各 tau packet：packet 之间遵循 involution 排序，packet 内保持 BFS 发现顺序。这里承载语义的稳定性来自计数排序。^[kgb-graph-structure.md:60-70]

## 存储与证据边界

状态及三类链接均按 `x * rank + s` 平铺存储。图采用 [[KGB 图的混合自包含存储]]，将每个 involution 位置的数据和 cocharacter 复制到图内；除需要逐 involution 的 theta 来计算精确有理环面因子的 `torus_factor` 外，其他访问器均不再依赖 involution 表。^[kgb-graph-structure.md:74-84]

本页依据的是结构性源码阅读材料，仅解释数据布局与构造算法，不构成 KGB 枚举数学正确性、性能或兼容性的验收结论。来源未执行构建、测试或原版运行；实现包含并行结构也不代表已有多核加速证据，数学套件的计时运行强制使用 `RAYON_NUM_THREADS=1`。数学验收应另行查阅 [[HPC 验收证据链]]。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:55-56, kgb-graph-structure.md:97-98]

## Sources

- [kgb-graph-structure.md](kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）
