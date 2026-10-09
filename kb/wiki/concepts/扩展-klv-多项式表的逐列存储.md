---
title: 扩展 KLV 多项式表的逐列存储
summary: ExtKlTable 为每个 y 存储按 primitive 位置寻址的池索引列，提供带符号多项式、递减非零列及指定次数系数查询。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T14:48:15.069Z"
updatedAt: "2026-10-09T20:53:02.342Z"
tags:
  - 扩展KLV
  - 数据结构
  - 多项式查询
aliases:
  - 扩展-klv-多项式表的逐列存储
  - 扩K多
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 扩展 KLV 多项式表的逐列存储
summary: ExtKlTable 为每个块元素 y 保存一列多项式池索引，按 primitive 位置寻址，独立记录符号翻转，并在填充失败时保持列为空或完整的不变式。
sources:
  - extended-kl.md
kind: concept
tags:
  - 扩展KLV
  - 数据结构
  - 列式存储
aliases:
  - 扩展-klv-多项式表的逐列存储
  - 扩K多
---

# 扩展 KLV 多项式表的逐列存储

`ExtKlTable<'a>` 是扩展（twisted）KLV 多项式表，为每个块元素 `y` 保存一列多项式池索引。其结构镜像普通 KLV 表，但递归采用 extended-block 版本；列内条目通过 `DescentTable` 预计算的 primitive 索引定位。^[extended-kl.md:19-24, extended-kl.md:72-74]

## 列布局与寻址

每列对应一个块元素 `y`，列内按 `x` 相对于 `y` 的下降集进行 primitivisation 后的 primitive 位置寻址。`DescentTable` 保存每个元素的 `descents`、`good_ascents`、`prim_index` 和 `prim_flip`，其中 `prim_index[mask][x]` 表示 `x` 对下降集 `mask` 进行 primitivisation 后的位置。相关预计算见 [[DescentTable 的下降集与 good ascent 预计算]]。^[extended-kl.md:39-48, extended-kl.md:72-74]

索引构造针对每个 `mask` 按 `x` 递减遍历，选择 `mask` 内的第一个 good ascent。遇到 like-nonparity 或跨越 partial-block 边界（`some_scent` 为 `None`）时记为 `DEAD_END`；否则沿 cross 链接继承索引，并按 `epsilon` 与已有 flip 位的差异更新符号。最后将索引反转为递增存储；整行没有 primitive 元素时保持 `DEAD_END`。^[extended-kl.md:46-52]

## 多项式池与符号分离

表复用 `kl_polynomial.rs` 的 `KlHashTable` 类型，池条目为系数属于 `i32` 的 `KlPol`，但 `ExtKlTable::new` 始终使用自有池。池索引不打包符号位；primitivisation 的符号保存在独立的 `prim_flip` bitmap 中，查询时另行读取。详见 [[多项式池与 primitivisation 符号分离]]。^[extended-kl.md:28-35, extended-kl.md:72-74]

`kl_pol_index(x,y)` 返回 `(KLIndex, bool)`，分别表示池索引与符号翻转标志；`p(x,y)` 返回 twisted KLV 多项式 \(P_{x,y}\)，需要翻转时通过 `scaled(-1)` 应用符号。`raw_ext_KL` 包装层将索引与标志渲染为 `inx.second ? -inx.first : inx.first`。^[extended-kl.md:33-35, extended-kl.md:76-77]

## 查询接口

表提供 `rank`、`size`、`descent_set`、`descent_type`、`l` 和 `polys` 等访问器，其中 `descent_type` 返回 `DescValue`。`nonzero_column(y)` 从 `y` 开始递减列出非零条目对应的 `x`；`mu(i,x,y)` 提取 \(q^{(\ell(y/x)-i)/2}\) 的系数，其中 \(i=1,2,3\)。^[extended-kl.md:74-80]

`is_extremal` 判断 `descent_set(x)` 是否包含 `descents(y)`；`is_primitive` 判断 `good_ascent_set(x)` 与 `descents(y)` 的交集是否为空。这些判定与 primitive 寻址所用的下降集及 good ascent 数据一致。^[extended-kl.md:46-48, extended-kl.md:59-60]

## 列填充与失败语义

`fill_columns(limit)` 计算所有 `y < limit` 的列；`limit == 0` 表示填满整块。填充失败时，Rust 实现清空出错列并传播错误，维持 `column[y]`“要么为空、要么完整”的不变式。此行为有意偏离上游通过 `catch(...)` 吞掉错误的策略，详见 [[列填充的完整性不变式与错误传播]]。^[extended-kl.md:84-87]

## 实现与证据边界

当前实现始终拥有自己的多项式池。共享池指针模式与用于 partial-block 迁移的 `swallow` 被推迟到需要它们的 common-block 切片，不能据当前表结构认定已经支持这些能力。相关范围见 [[扩展 KLV 的 Rust 移植边界与有意偏离]]。^[extended-kl.md:93-95]

本页依据对 `ext_kl.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。材料中的上游行号转述自源码注释，未独立重读上游；材料也未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。扩展 KL 的正确性属于独立的 HPC 证据链，本材料不重述或扩展其结论。^[extended-kl.md:9-15, extended-kl.md:108-114]

## Sources

- [extended-kl.md](../../sources/extended-kl.md)：扩展 KLV 多项式表：primitivisation 符号与逐列存储。
