---
title: tau packet 与 KGB 元素编号标准化
summary: 先按对合长度、Weyl 长度及 pieces 字典序排列对合，再以稳定计数排序组织 tau packet，保留包内 BFS 发现顺序。
sources:
  - kgb-graph-structure.md
kind: concept
createdAt: "2026-10-09T14:54:25.989Z"
updatedAt: "2026-10-09T19:31:46.324Z"
tags:
  - KGB
  - 排序
aliases:
  - tau-packet-与-kgb-元素编号标准化
  - TP与K元
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# tau packet 与 KGB 元素编号标准化

在 `KgbGraph` 中，一张图对应一个弱实形式，元素是该形式各 involution 之上的 Tits 元素。编号标准化将 BFS 发现的元素组织为 tau packet：不同 packet 按 involution 的排序位置排列，同一 packet 内保留 BFS 发现顺序。相关背景见 [[KGB 图与弱实形式]]。^[kgb-graph-structure.md:19-23, kgb-graph-structure.md:60-70]

## involution 的排序键

标准化首先对该弱实形式的 involution 排序，依次比较 involution 长度、Weyl 长度和 `WeylElt::pieces` 的字典序。源码将这一规则对应到上游的 `Cartan_orbits::comparer`，并指出上游部分注释中的 “internal number” 已过时，实际比较的是 `TwistedInvolution` 的值。紧凑表示的背景见 [[Weyl 群的紧凑 Transducer 表示]]。^[kgb-graph-structure.md:60-64]

该排序键构成严格全序，因此 `sort_unstable` 的稳定性不影响这里的排序语义；编号所需的稳定性由后续计数排序承担。^[kgb-graph-structure.md:64-67]

## 从 BFS 发现顺序到最终编号

BFS 采用[[分窗两相 BFS 构造]]，每窗包含 64 个元素。第一相通过 Rayon 并行计算状态、cross 和 Cayley 目标，此时表与 coset 只读；第二相顺序执行 `intern`，按 `TitsElement` 去重并分配新 id。这个阶段产生标准化时需要保留的 packet 内发现顺序。^[kgb-graph-structure.md:49-51, kgb-graph-structure.md:64-66]

随后，计数排序按排序后的 involution 位置归集元素，使 packet 之间遵循 involution 顺序，packet 内保持原有 BFS 顺序。包内顺序因此也是最终编号语义的一部分。^[kgb-graph-structure.md:64-67]

## packet 的索引与查询

`positions` 在每个排序位置记录三元组 `(InvolutionId, involution 长度, CartanId)`。`first_of_tau` 保存累计计数，长度为 `positions.len() + 1`。访问器 `tau_packet(position)` 按排序位置返回对应 packet 的 `(首元素, 大小)`。^[kgb-graph-structure.md:67-70]

标准化后的编号还用于 inverse-Cayley 链接的升序后处理。II 型记录一个前像，表示为 `(first, None)`；I 型记录两个前像，表示为 `Some((first, Some(second)))`，并保证 `first < second`。生成元不是 real 时，访问器返回 `Ok(None)`。参见 [[Cross、Cayley 与逆 Cayley 链接]]。^[kgb-graph-structure.md:78-80]

## 兼容性与证据边界

模块声明该编号方式精确复现上游 `KGB::KGB`，但来源文档仅提供结构性阅读，不据此声称兼容性或 KGB 枚举的数学正确性已获验证。上游引用行号来自源码注释，未经独立重读；来源文档也未执行构建、测试或原版运行。数学正确性的验收属于独立的 [[HPC 验收证据链]]。^[kgb-graph-structure.md:9-15, kgb-graph-structure.md:60-64, kgb-graph-structure.md:90-98]

## Sources

- [kgb-graph-structure.md](../../sources/kgb-graph-structure.md) — KGB 图的结构与构造（每个弱实形式一张图）。
