---
title: tau packet 与 KGB 元素编号标准化
summary: 先按对合长度、Weyl 长度及 pieces 字典序排列对合，再以稳定计数排序组织 tau packet 并保留包内 BFS 发现顺序；上游编号一致性仅为来源声明。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:25.989Z"
updatedAt: "2026-10-09T22:34:30.624Z"
tags:
  - KGB
  - 排序算法
aliases:
  - tau-packet-与-kgb-元素编号标准化
  - TP与K元
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: tau packet 与 KGB 元素编号标准化
summary: 按对合长度、Weyl 长度及 pieces 字典序排列对合，再通过稳定计数排序组织 tau packet，保留包内 BFS 发现顺序；上游编号一致性属于源码声明。
sources:
  - kgb-graph-structure.md
kind: concept
tags:
  - KGB
  - 排序
  - 编号
aliases:
  - tau-packet-与-kgb-元素编号标准化
---

# tau packet 与 KGB 元素编号标准化

`KgbGraph` 的一张图对应一个弱实形式，元素是该形式各 involution 之上的 Tits 元素。编号标准化将这些元素按 involution 组织为 tau packet：packet 之间按 involution 的排序位置排列，同一 packet 内保留 BFS 发现顺序。背景见 [[KGB 图与弱实形式]]。^[kgb-graph-structure.md:19-23, kgb-graph-structure.md:60-70]

## involution 的排序键

标准化首先排列该形式的 involution，依次比较 involution 长度、Weyl 长度以及 `WeylElt::pieces` 的字典序。源码将此规则对应到上游 `Cartan_orbits::comparer`，并指出上游部分 “internal number” 注释已经过时，实际比较的是 `TwistedInvolution` 的值。相关表示见 [[Weyl 群的紧凑 Transducer 表示]]。^[kgb-graph-structure.md:60-64]

该排序键构成严格全序，因此这里使用 `sort_unstable` 不影响排序语义。编号所需的稳定性由后续计数排序承担：involution 排序确定 packet 之间的顺序，计数排序保留 packet 内的 BFS 发现顺序。^[kgb-graph-structure.md:64-67]

## 从 BFS 发现顺序到最终编号

元素构造采用 [[分窗两相 BFS 构造]]，每窗包含 64 个元素。第一相用 Rayon 并行计算状态、cross 和 Cayley 目标，期间 involution 表与 coset 只读；第二相顺序执行 `intern`，按 `TitsElement` 去重并分配新 id。随后，计数排序依据已排序的 involution 位置归集元素，形成最终编号。^[kgb-graph-structure.md:49-51, kgb-graph-structure.md:64-67]

## packet 的索引与查询

`positions` 在每个排序位置保存 `(InvolutionId, involution 长度, CartanId)`。`first_of_tau` 保存累计计数，其长度为 `positions.len() + 1`。`tau_packet(position)` 以排序位置为参数，返回对应 packet 的 `(首元素, 大小)`。^[kgb-graph-structure.md:67-70]

编号标准化完成后，inverse-Cayley 链接通过升序后处理安装。生成元不是 real 时，`inverse_cayley(x, s)` 返回 `Ok(None)`；II 型只有一个前像，记录为 `(first, None)`；I 型有两个前像，记录为 `Some((first, Some(second)))`，并保证 `first < second`。参见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:78-80]

## 兼容性与证据边界

模块声明上述编号精确复现上游 `KGB::KGB`。这一对应关系属于源码声明；来源文档是结构性阅读，不据此声称兼容性或 KGB 枚举的数学正确性已经得到验证。所读 `kgb_graph.rs` 字节来自 dirty 工作区，由阅读快照记录。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:60-64]

上游引用行号来自源码注释，未经独立重读，可能随上游演进而漂移。来源文档未执行构建、测试或原版运行；KGB 枚举的正确性属于独立的 [[HPC 验收证据链]]，本页不扩展其验收范围。^[kgb-graph-structure.md:90-98]

## Sources

- [kgb-graph-structure.md](../../sources/kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）。
