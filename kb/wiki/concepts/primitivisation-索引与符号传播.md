---
title: Primitivisation 索引与符号传播
summary: 对每个 descent mask 递减遍历元素，沿首个 good ascent 继承索引并传播 epsilon 符号，遇到 like-nonparity 或部分块边界则记录 DEAD_END。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T14:48:22.744Z"
updatedAt: "2026-10-09T22:29:59.637Z"
tags:
  - 扩展KLV
  - 本原化
  - 符号传播
aliases:
  - primitivisation-索引与符号传播
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Primitivisation 索引与符号传播
summary: 对每个 descent mask 递减遍历元素，沿首个 good ascent 的 cross 链接继承 primitive 索引并传播符号；遇到 like-nonparity 或 partial-block 边界则记录 DEAD_END。
sources:
  - extended-kl.md
kind: concept
tags:
  - 扩展KLV
  - 本原化
  - 符号传播
aliases:
  - primitivisation-索引与符号传播
---

# Primitivisation 索引与符号传播

Primitivisation（本原化）将扩展 KLV 表中的元素查询归约到指定 descent set 下的 primitive 位置，并记录归约产生的符号翻转。`DescentTable` 预计算这些信息，`ExtKlTable` 据此访问逐列存储的多项式池索引，并在查询多项式时恢复符号。^[extended-kl.md:19-24, extended-kl.md:46-52, extended-kl.md:72-80]

## 数据结构与判定

`DescentTable<'a>` 持有 `block: &ExtBlock`、每个元素的 `descents` 与 `good_ascents` 两个 `RankFlags`，以及 `prim_index` 和 `prim_flip`。构造时逐元素、逐生成元检查 `block.descent_type(s, x)`：若 `is_descent()` 为真，则置入下降集；否则若 `!has_double_image()`，则置入 good ascent 集，表示向上邻居至多一个。若 `rank > MAX_FOLDED_RANK`，构造返回 `StructureError::ResourceLimitExceeded`。^[extended-kl.md:39-44]

`prim_index[mask][x]` 表示 `x` 相对于 descent set `mask` 完成本原化后的位置；`prim_flip[x]` 记录哪些 descent sets 会使该本原化带上符号翻转。索引与符号分别存储。^[extended-kl.md:46-48]

相对于列元素 `y`，`x` 为 primitive 的条件是 `good_ascent_set(x) ∩ descents(y)` 为空；为 extremal 的条件是 `descent_set(x) ⊇ descents(y)`。相关预计算见 [[DescentTable 的下降集与 good ascent 预计算]]。^[extended-kl.md:59-60]

## 索引构造与符号传播

构造器针对每个 `mask`，按 `x` 递减的顺序遍历元素，取 `x` 在该 mask 内的第一个 good ascent。遇到 like-nonparity，或跨越 partial-block 边界而使 `some_scent` 返回 `None` 时，将索引记为 `DEAD_END`；否则沿 cross 链接继承索引，并根据 `epsilon` 与既有 flip 位的差异设置符号翻转位。^[extended-kl.md:48-51]

每个 mask 的处理收尾时，将索引反转为递增存储；若整行没有 primitive 元素，则保持 `DEAD_END`。因此，递减遍历描述构造顺序，递增排列描述最终存储顺序。^[extended-kl.md:48-52]

## 查询与符号恢复

`x_index`、`self_index` 和 `flips` 通过 `mask_of` 查表。辅助集合 `very_easy_set(x,y)` 等于 `good_ascents(x) ∩ descents(y)`，`easy_set(x,y)` 等于 `descents(y) ∖ descents(x)`；表还提供 `length_floor`、`col_size` 以及 `extr_back_up_mask`、`prim_back_up_mask` 回溯接口。^[extended-kl.md:54-62]

`ExtKlTable` 为每个块元素 `y` 存储一列多项式池索引，以 `x` 相对于 `descents(y)` 的 primitive 位置寻址。`kl_pol_index` 返回 `(KLIndex, bool)`；`p(x,y)` 返回 twisted KLV 多项式 \(P_{x,y}\)，在 flip 为真时通过 `scaled(-1)` 恢复符号。该机制连接了本原化与 [[扩展 KLV 多项式表的逐列存储]]。^[extended-kl.md:72-80]

多项式池复用 `KlHashTable`，条目是系数类型为 `i32` 的 `KlPol`。池索引不打包符号位；本原化符号保存在独立的 `prim_flip` bitmap 中，查询时另行读取。`raw_ext_KL` 包装层将返回对渲染为 `inx.second ? -inx.first : inx.first`，这是返回值的呈现方式。参见 [[多项式池与 primitivisation 符号分离]]。^[extended-kl.md:28-35]

## 证据范围

本页依据 `ext_kl.rs` 的结构性阅读材料；阅读快照 `2026-10-03-extended-kl.json` 记录的是 dirty 工作区字节。材料中的上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。^[extended-kl.md:9-15, extended-kl.md:108-111]

来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。扩展 KL 的正确性属于其独立 HPC 证据链，本页不扩展这一证据范围。^[extended-kl.md:10-11, extended-kl.md:114-114]

## Sources

- [extended-kl.md](../../sources/extended-kl.md) — 扩展 KLV 多项式表：primitivisation 符号与逐列存储。
