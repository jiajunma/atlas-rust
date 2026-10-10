---
title: 分窗两相 BFS 构造
summary: BFS 每窗处理 64 个元素，先以 Rayon 并行计算状态和目标，再顺序去重并分配编号；该结构不构成多核加速证据。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:25.828Z"
updatedAt: "2026-10-10T00:37:42.311Z"
tags:
  - KGB
  - 图算法
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 分窗两相 BFS 构造
summary: KGB 图以 64 个元素为一窗，先并行计算状态与目标，再顺序去重并分配编号；最终编号保留 tau packet 内的 BFS 发现顺序，并行结构本身不构成多核加速证据。
sources:
  - kgb-graph-structure.md
kind: concept
tags:
  - 图算法
  - 并行计算
  - 确定性
aliases:
  - 分窗两相-bfs-构造
  - 分B构
provenanceState: extracted
---

# 分窗两相 BFS 构造

分窗两相 BFS 是 `KgbGraph::build` 构造 KGB 图所用的广度优先遍历方式：以 **64 个元素为一窗**，先并行计算状态与目标，再顺序去重并分配元素编号。每张图对应一个弱实形式，由 `RealFormSeed` 种子生成，图元素是该形式各 involution 之上的 Tits 元素。^[kgb-graph-structure.md:19-23, kgb-graph-structure.md:49-51]

## 构造前置条件

构建输入包括 `InnerClass`、Cartan 分类、强实形式分类、可变的 `InvolutionTable` 和种子。首先检查表与 inner class 是否匹配，不匹配时报 `DatumMismatch`；随后取得该形式的预期 `kgb_size` 和 Cartan 集合，任一缺失均报 `IndexOutOfRange`，再按升序幂等添加该形式的 Cartan。两处下标错误的 `upper_bound` 回退构造不同：前者经 `strong_real_data(CartanId(0))`，后者使用 `weak_real_form_count()`；来源未说明这一差异是否有意。^[kgb-graph-structure.md:38-44]

完成 Cartan 添加后，构造验证种子与表的绑定：恒等 `WeylElement` 的表内查找结果必须等于种子的 involution，且种子的 `torus_bits` 必须已经是 `mod_space` 的商代表元，否则报 `KgbInvariantViolation { invariant: "seed element" }`。`TitsCoset` 使用同一个 inner class 完成一次门控，覆盖整个 BFS；详见 [[KGB 构造的前置门控与不变量]]。^[kgb-graph-structure.md:43-47]

## 窗口内的两相处理

**第一相：并行纯计算。** 使用 Rayon 的 `into_par_iter` 处理窗口内元素，计算生成元状态、cross 目标和 Cayley 目标。这一阶段 involution 表与 coset 均保持只读。^[kgb-graph-structure.md:49-51]

状态分类先由 involution 表的 `simple_root_kind` 给出 Complex、Real 或 Imaginary；对 Imaginary 情形，再用 coset 的 `simple_grading_pregated` 区分紧致与非紧致，得到四值 `KgbStatus`。相关规则见 [[KGB 生成元状态与下降判定]]。^[kgb-graph-structure.md:27-31]

**第二相：顺序登记。** 顺序执行 `intern`，按 `TitsElement` 去重，并为新元素分配 id。目标计算与去重、编号分配由这两个阶段分别承担。^[kgb-graph-structure.md:49-51]

## 构造不变量

状态槽遵循 **write-once** 约束：同一个元素与单生成元组合 `(x, s)` 若被写入两次，即报 `KgbInvariantViolation`。非紧致 imaginary 生成元的 Cayley 目标必须存在，且目标的 involution 长度恰比源多一，对应不变量 `"Cayley length step"`。^[kgb-graph-structure.md:52-55]

BFS 结束时，元素总数必须等于强实形式分类预言的 `kgb_size`，对应不变量 `"kgb size"`。这是实现中的构造检查；来源不据此宣称 KGB 枚举的数学正确性。^[kgb-graph-structure.md:10-11, kgb-graph-structure.md:54-55]

## 发现顺序与最终编号

BFS 发现顺序随后经过编号标准化。构造先按 `(involution 长度, Weyl 长度, WeylElt::pieces 字典序)` 排列该形式的 involution，再用计数排序组织各 tau packet：packet 之间按排序后的 involution 位置排列，**packet 内保留 BFS 发现顺序**。^[kgb-graph-structure.md:60-66]

involution 排序键是严格全序，因此 `sort_unstable` 的稳定性无关紧要；承载编号语义的是计数排序的稳定性。`positions` 记录每个排序位置的 `(InvolutionId, involution 长度, CartanId)`，`first_of_tau` 是长度为 `positions.len()+1` 的累计计数，`tau_packet(position)` 返回该 packet 的首元素与大小。详见 [[tau packet 与 KGB 元素编号标准化]]。^[kgb-graph-structure.md:66-70]

## 证据边界

实现具有并行计算结构，但数学套件的计时运行强制使用 `RAYON_NUM_THREADS=1`，因此不能据此推断已有多核加速。来源属于结构性源码阅读，所读字节来自 dirty 工作区，未执行构建、测试或原版运行；KGB 枚举的数学正确性属于独立的 [[HPC 验收证据链]]。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:55-56, kgb-graph-structure.md:97-98]

模块声明其编号精确复现上游 `KGB::KGB`，但来源中的上游行号转录自源码注释，未经独立重读，可能随上游演进漂移。该声明不构成来源独立验证的兼容性结论。^[kgb-graph-structure.md:10-11, kgb-graph-structure.md:60-64, kgb-graph-structure.md:90-93]

## Sources

- [kgb-graph-structure.md](../../sources/kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）。
