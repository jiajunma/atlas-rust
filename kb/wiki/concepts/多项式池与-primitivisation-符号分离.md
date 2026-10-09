---
title: 多项式池与 primitivisation 符号分离
summary: 共享 KlHashTable 类型存储 i32 系数的 KlPol，索引不打包符号；独立 prim_flip 位图记录符号翻转，kl_pol_index 返回索引与翻转标志对。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T14:48:27.585Z"
updatedAt: "2026-10-09T14:48:27.585Z"
tags:
  - 多项式池
  - 符号处理
  - primitivisation
aliases:
  - 多项式池与-primitivisation-符号分离
  - 多P符
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 多项式池与 primitivisation 符号分离

扩展（twisted）KLV 多项式表将多项式池索引与 primitivisation 引入的符号翻转分开存储：`KLIndex` 只标识池中的多项式，独立的 `prim_flip` bitmap 记录符号信息，查询时再组合两者。Rust 与上游均不在索引中打包符号位。^[extended-kl.md:28-35]

## 多项式池与符号的职责

扩展表复用 `kl_polynomial.rs` 的 `KlHashTable`，池条目是系数为 `i32` 的 `KlPol`，对应上游 `IntPolEntry = Polynomial<int>`；该共享类型也对应上游的 `ext_KL_hash_Table`。这里分离的是 primitivisation 的符号翻转，并不意味着池中多项式的系数只能为非负数。^[extended-kl.md:28-33]

`ExtKlTable` 为每个块元素 `y` 保存一列池索引，以 `x` 相对于 `y` 的 descent set 的 primitive 位置寻址。当前构造器始终使用自有 pool；共享池指针模式与用于 partial-block 迁移的 `swallow` 被推迟到需要它们的 common-block 切片。参见 [[扩展 KLV 多项式表的逐列存储]]。^[extended-kl.md:72-74, extended-kl.md:93-95]

## primitivisation 的索引与符号传播

`DescentTable` 同时持有 `prim_index` 与 `prim_flip`。其中，`prim_index[mask][x]` 表示 `x` 相对于 descent set `mask` 做 primitivisation 后的位置；`prim_flip[x]` 则记录哪些 descent sets 会使该过程产生符号翻转。二者分别承担位置与符号的查询职责。^[extended-kl.md:39-48]

构造时，对每个 mask 按 `x` 递减遍历，选择 mask 内的第一个 good ascent。若遇到 like-nonparity，或跨越 partial-block 边界而使 `some_scent` 为 `None`，则记为 `DEAD_END`；否则沿 cross 链接继承下标，并根据 `epsilon` 与已有 flip 位的差异设置符号位。最后将索引反转为递增存储；整行没有 primitive 时保持 `DEAD_END`。参见 [[Primitivisation 索引与符号传播]]。^[extended-kl.md:46-52]

## 查询时组合索引与符号

`ExtKlTable::kl_pol_index` 返回 `(KLIndex, bool)`，显式提供池索引与翻转标志。`p(x,y)` 返回 twisted KLV 多项式 \(P_{x,y}\)，在需要翻转时对池中的多项式应用 `scaled(-1)`。这使列存储可以保留池索引，而由访问接口恢复带符号的结果。^[extended-kl.md:72-78]

`raw_ext_KL` 包装层将返回对渲染为 `inx.second ? -inx.first : inx.first`。这是输出层对索引与符号的组合方式，不是内部索引打包符号位的存储格式。^[extended-kl.md:31-35]

## 证据范围

本文依据对 `ext_kl.rs` 的结构性阅读；对应快照记录的是 dirty 工作区字节。来源包未执行构建、测试或原版运行，因此上述说明不构成数学正确性、性能或并行行为的验收结论。上游行号来自源码注释转述，未独立重读上游。^[extended-kl.md:9-15, extended-kl.md:108-114]

## Sources

- [extended-kl.md](extended-kl.md) — 扩展 KLV 多项式表：primitivisation 符号与逐列存储。
