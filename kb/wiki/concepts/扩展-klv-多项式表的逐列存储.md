---
title: 扩展 KLV 多项式表的逐列存储
summary: ExtKlTable 为每个块元素 y 保存一列多项式池索引，按 x 相对于 y 的 descent set 的 primitive 位置寻址，并提供带符号多项式及系数查询。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T14:48:15.069Z"
updatedAt: "2026-10-09T14:48:15.069Z"
tags:
  - 扩展KLV
  - 数据结构
  - 列式存储
aliases:
  - 扩展-klv-多项式表的逐列存储
  - 扩K多
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 扩展 KLV 多项式表的逐列存储

`ExtKlTable<'a>` 是扩展（twisted）KLV 多项式表，结构上采用逐列存储，递归则使用 extended-block 版本。它为每个块元素 `y` 保存一列多项式池索引，并通过 `DescentTable` 预计算的数据定位条目。^[extended-kl.md:19-24]

## 列布局与寻址

每列对应一个块元素 `y`，列内按 `x` 相对于 `y` 的下降集进行 primitivisation 后的 primitive 位置寻址。`DescentTable` 保存每个元素的 `descents`、`good_ascents`、`prim_index` 和 `prim_flip`；其中 `prim_index[mask][x]` 给出 `x` 对下降集 `mask` 进行 primitivisation 后的位置。这使列访问依赖 primitive 索引，而非直接使用块元素编号 `x`。^[extended-kl.md:39-48, extended-kl.md:72-74]

索引构造针对每个下降集 `mask` 按 `x` 递减遍历，选择该集合内的第一个 good ascent。遇到 like-nonparity 或跨越 partial-block 边界时记为 `DEAD_END`；否则沿 cross 链接继承索引，并根据 `epsilon` 与已有 flip 位更新符号。构造结束后将索引反转为递增存储；整行没有 primitive 元素时保持 `DEAD_END`。相关判定见 [[扩展 KLV 的 extremal 与 primitive 判定]]。^[extended-kl.md:48-52]

## 多项式池与符号

表复用 `kl_polynomial.rs` 的 `KlHashTable` 类型，池条目为系数属于 `i32` 的 `KlPol`；`ExtKlTable::new` 始终创建自有池。列中存储的是池索引，索引本身不打包符号位。primitivisation 引入的符号单独存储在 `prim_flip` bitmap 中，查询时再读取，参见 [[多项式池与 primitivisation 符号分离]]。^[extended-kl.md:28-35, extended-kl.md:72-74]

`kl_pol_index(x,y)` 返回 `(KLIndex, bool)`，分别表示池索引与符号翻转标志；`p(x,y)` 返回 twisted KLV 多项式 \(P_{x,y}\)，需要翻转时通过 `scaled(-1)` 应用符号。`raw_ext_KL` 包装层则将这对值渲染为 `inx.second ? -inx.first : inx.first`。^[extended-kl.md:33-35, extended-kl.md:76-77]

## 查询与列填充

除多项式访问外，表提供 `rank`、`size`、`descent_set`、`descent_type`、`l` 和 `polys` 等访问器，以及 `is_extremal`、`is_primitive` 判定。`nonzero_column(y)` 从 `y` 开始递减列出非零条目对应的 `x`；`mu(i,x,y)` 提取 \(q^{(\ell(y/x)-i)/2}\) 的系数，其中 \(i=1,2,3\)。^[extended-kl.md:74-80]

`fill_columns(limit)` 计算所有 `y < limit` 的列；`limit == 0` 表示填满整块。填充失败时，Rust 实现清空出错列并传播错误，维持 `column[y]`“要么为空、要么完整”的不变式。这是相对于上游通过 `catch(...)` 吞掉错误的有意偏离，相关约束见 [[列填充的完整性不变式与错误传播]]。^[extended-kl.md:84-87]

## 实现与证据边界

当前表始终拥有自己的多项式池；共享池指针模式和用于 partial-block 迁移的 `swallow` 尚未移植，被推迟到需要这些能力的 common-block 切片。该边界属于 [[扩展 KLV 的 Rust 移植边界与有意偏离]]，不能从当前逐列存储结构推断已有跨表共享池或迁移支持。^[extended-kl.md:93-95]

本页依据的材料是对 `ext_kl.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。材料中的上游行号转述自源码注释，未独立重读上游；该材料也未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论。扩展 KL 的正确性依赖其独立的 [[HPC 验收证据链]]。^[extended-kl.md:9-15, extended-kl.md:108-114]

## Sources

- [extended-kl.md](extended-kl.md)：扩展 KLV 多项式表：primitivisation 符号与逐列存储。
