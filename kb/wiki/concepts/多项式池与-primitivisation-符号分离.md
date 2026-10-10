---
title: 多项式池与 primitivisation 符号分离
summary: KlHashTable 保存 i32 系数多项式，池索引不打包符号；prim_flip 独立保存翻转信息，kl_pol_index 返回索引与符号标志对。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T14:48:27.585Z"
updatedAt: "2026-10-10T00:32:48.512Z"
tags:
  - 扩展KLV
  - 多项式
  - 存储设计
aliases:
  - 多项式池与-primitivisation-符号分离
  - 多P符
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 多项式池与 primitivisation 符号分离
summary: 扩展 KLV 表以 KlHashTable 保存有符号整数系数多项式，池索引不打包符号；prim_flip 独立记录 primitivisation 翻转，查询时组合索引与翻转标志。
sources:
  - extended-kl.md
kind: concept
tags:
  - 扩展KLV
  - 多项式存储
  - 符号传播
aliases:
  - 多项式池与-primitivisation-符号分离
  - 多P符
provenanceState: extracted
---

# 多项式池与 primitivisation 符号分离

扩展（twisted）KLV 多项式表将池索引与 primitivisation 引入的符号翻转分别存储：`KLIndex` 标识多项式池条目，独立的 `prim_flip` 位图记录翻转信息，查询时再组合两者。Rust 实现与上游均不在池索引中打包符号位。^[extended-kl.md:28-35]

## 多项式池与所有权

扩展表复用 `kl_polynomial.rs` 的 `KlHashTable` 类型，条目为 `i32` 系数的 `KlPol`，对应上游 `IntPolEntry = Polynomial<int>`；该池类型对应上游 `ext_KL_hash_Table`。独立存储的符号专指 primitivisation 翻转，多项式系数本身仍是有符号整数。^[extended-kl.md:28-35]

`ExtKlTable::new` 始终创建自有池。共享池指针模式，以及用于 partial-block 迁移的 `swallow`，推迟到需要它们的 common-block 切片。因此，复用 `KlHashTable` 类型并不表示不同扩展表共享池实例，相关边界见 [[扩展 KLV 的 Rust 移植边界与有意偏离]]。^[extended-kl.md:72-74, extended-kl.md:93-95]

## primitive 索引与翻转传播

`DescentTable` 保存 `prim_index` 与 `prim_flip`。`prim_index[mask][x]` 表示元素 `x` 相对于 descent set `mask` 做 primitivisation 后的位置；`prim_flip[x]` 记录哪些 descent sets 会使该过程产生符号翻转。^[extended-kl.md:39-48]

构造时，对每个 mask 按 `x` 递减遍历，选择 `x` 在 mask 内的第一个 good ascent。遇到 like-nonparity，或跨越 partial-block 边界而使 `some_scent` 为 `None`，则记为 `DEAD_END`；否则沿 cross 链接继承下标，并按 `epsilon` 与已有 flip 位的差异设置翻转位。最后将索引反转为递增存储；整行没有 primitive 时保持 `DEAD_END`。详见 [[Primitivisation 索引与符号传播]]。^[extended-kl.md:46-52]

## 查询时恢复符号

`ExtKlTable` 为每个块元素 `y` 保存一列池索引，按 `x` 相对于 `y` 的 descent set 的 primitive 位置寻址；翻转信息在查询时单独读取。列布局参见 [[扩展 KLV 多项式表的逐列存储]]。^[extended-kl.md:31-33, extended-kl.md:72-78]

`ExtKlTable::kl_pol_index` 返回 `(KLIndex, bool)`，分别给出池索引与翻转标志。`p(x,y)` 返回 twisted KLV 多项式 \(P_{x,y}\)，需要翻转时通过 `scaled(-1)` 将池中的多项式取负。^[extended-kl.md:76-78]

`raw_ext_KL` 包装层将返回对渲染为 `inx.second ? -inx.first : inx.first`。这一带符号索引是包装层的呈现方式，内部仍保留池索引与翻转标志的分离表示。^[extended-kl.md:31-35]

## 证据范围

本页依据 `ext_kl.rs` 的结构性阅读，所读字节来自 dirty 工作区，由 `2026-10-03-extended-kl.json` 快照记录。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；扩展 KL 的正确性属于独立的 HPC 证据链。上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[extended-kl.md:9-15, extended-kl.md:108-114]

## Sources

- [extended-kl.md](../../sources/extended-kl.md) — 扩展 KLV 多项式表：primitivisation 符号与逐列存储。
