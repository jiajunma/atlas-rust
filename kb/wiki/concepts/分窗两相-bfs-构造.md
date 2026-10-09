---
title: 分窗两相 BFS 构造
summary: KGB 枚举以 64 个元素为一窗，先用 Rayon 在只读表与 coset 上计算状态及目标，再顺序 intern 去重并分配编号；并行结构本身不代表已有多核加速证据。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:25.828Z"
updatedAt: "2026-10-09T14:54:25.828Z"
tags:
  - 图算法
  - BFS
  - 并行计算
aliases:
  - 分窗两相-bfs-构造
  - 分B构
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 分窗两相 BFS 构造

分窗两相 BFS 是 `KgbGraph::build` 构造 KGB 图时采用的遍历方式：每次处理一个包含 64 个元素的窗口，先并行计算目标，再顺序去重并分配元素编号。一张图对应一个弱实形式，由 `RealFormSeed` 种子生成；图元素是各 involution 之上的 Tits 元素。^[kgb-graph-structure.md:19-23, kgb-graph-structure.md:49-51]

## 构造前置条件

构建输入包括 `InnerClass`、Cartan 分类、强实形式分类、可变的 `InvolutionTable` 和种子。构造先检查表与 inner class 的一致性，再取得该弱实形式的预期 `kgb_size` 与 Cartan 集合；前者不匹配报 `DatumMismatch`，后两项缺失报 `IndexOutOfRange`。随后按升序幂等添加该形式的 Cartan。^[kgb-graph-structure.md:38-44]

种子必须与表绑定一致：恒等 `WeylElement` 在表内的查找结果必须等于种子的 involution，且 `torus_bits` 必须已经是 `mod_space` 的商代表元，否则报 `KgbInvariantViolation { invariant: "seed element" }`。`TitsCoset` 使用同一个 inner class 完成一次门控，并覆盖整个 BFS；相关细节见 [[KGB 构造的前置门控与不变量]]。^[kgb-graph-structure.md:44-47]

## 窗口内的两相处理

第一相使用 Rayon 的 `into_par_iter`，并行处理窗口内的元素，计算生成元状态、cross 目标及 Cayley 目标。这一阶段是纯计算，involution 表与 coset 均保持只读。^[kgb-graph-structure.md:49-51]

第二相顺序执行 `intern`，按 `TitsElement` 去重并为新元素分配 id。窗口内的并行计算与顺序登记由此分为两个明确阶段。^[kgb-graph-structure.md:49-51]

状态分类先通过 involution 表的 `simple_root_kind` 区分 Complex、Real 与 Imaginary，再对 Imaginary 使用 coset 的 `simple_grading_pregated` 区分紧致与非紧致，得到四值 `KgbStatus`；参见 [[KGB 生成元状态与下降判定]]。^[kgb-graph-structure.md:27-31]

## 构造不变量

状态槽遵循 write-once 约束：同一个元素与单生成元组合 `(x, s)` 的状态若被写入两次，即报 `KgbInvariantViolation`。非紧致 imaginary 生成元的 Cayley 目标必须存在，而且目标的 involution 长度必须恰比源多一，对应不变量 `"Cayley length step"`。^[kgb-graph-structure.md:52-55]

BFS 结束时，图的元素总数必须等于强实形式分类给出的 `kgb_size`，对应不变量 `"kgb size"`。这是构造过程中的一致性检查；源材料并未据此宣称 KGB 枚举的数学正确性。^[kgb-graph-structure.md:10-11, kgb-graph-structure.md:54-55]

## BFS 发现顺序与最终编号

BFS 发现顺序还需经过编号标准化。构造先按 involution 长度、Weyl 长度和 `WeylElt::pieces` 字典序排列 involution，再用计数排序组织各 tau packet：packet 之间遵循排序后的 involution 位置，packet 内保留 BFS 发现顺序。^[kgb-graph-structure.md:60-66]

这里的 involution 排序键是严格全序，因此 `sort_unstable` 的稳定性不影响结果；真正承载编号语义的是计数排序对 packet 内发现顺序的保留。详见 [[tau packet 与 KGB 元素编号标准化]]。^[kgb-graph-structure.md:66-70]

## 证据边界

该实现具有并行计算结构，但数学套件的计时运行强制使用 `RAYON_NUM_THREADS=1`，因此不能将其视为已有多核加速证据。源材料属于结构性源码阅读，未执行构建、测试或原版运行；枚举正确性属于独立的 [[HPC 验收证据链]]，本页不扩展其结论。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:55-56, kgb-graph-structure.md:97-98]

## Sources

- [kgb-graph-structure.md](../../sources/kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）
