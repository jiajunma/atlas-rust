---
title: DescentTable 的下降集与 good ascent 预计算
summary: DescentTable 按元素和生成元预计算下降集及至多具有一个向上邻居的 good ascents，并检查折叠秩容量。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T14:48:19.776Z"
updatedAt: "2026-10-09T20:52:57.916Z"
tags:
  - 扩展KLV
  - 下降集
  - 资源限制
aliases:
  - descenttable-的下降集与-good-ascent-预计算
  - D的GA预
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: DescentTable 的下降集与 good ascent 预计算
summary: DescentTable 预计算扩展块元素的下降集、good ascent 集，以及各下降集掩码下的 primitive 索引和符号翻转，为扩展 KLV 多项式列提供寻址数据。
sources:
  - extended-kl.md
kind: concept
tags:
  - 预计算
  - 下降集
  - 扩展块
aliases:
  - descenttable-的下降集与-good-ascent-预计算
provenanceState: extracted
---

# DescentTable 的下降集与 good ascent 预计算

`DescentTable<'a>` 是扩展（twisted）KLV 多项式表的预计算层，记录每个块元素的下降集、good ascent 集，以及 primitivisation 所需的索引和符号翻转。[[扩展 KLV 多项式表的逐列存储|ExtKlTable]] 使用这些数据按 primitive 位置访问多项式列。^[extended-kl.md:19-24, extended-kl.md:72-77]

## 数据结构与生成元分类

`DescentTable` 持有扩展块引用 `block: &ExtBlock`、每个元素的两个 [[RankFlags：简单生成元位集|RankFlags]] 位集 `descents` 和 `good_ascents`，以及 `prim_index` 与 `prim_flip`。^[extended-kl.md:39-41]

构造时，对每个块元素 `x` 和每个生成元 `s` 调用 `block.descent_type(s, x)`。若类型满足 `is_descent()`，则将 `s` 加入 `descents(x)`；否则，仅在 `!has_double_image()` 时将其加入 `good_ascents(x)`。这里的 good ascent 表示“至多一个向上邻居”，并不保证存在向上邻居。若 `rank > MAX_FOLDED_RANK`，则返回 `StructureError::ResourceLimitExceeded`。^[extended-kl.md:41-44]

## Primitive 索引与符号预计算

`prim_index[mask][x]` 表示 `x` 相对于下降集掩码 `mask` 做 primitivisation 后的位置；`prim_flip[x]` 记录哪些下降集会使该过程产生符号翻转。索引与符号独立存储，查询时再组合使用，相关机制见 [[Primitivisation 索引与符号传播]]。^[extended-kl.md:28-35, extended-kl.md:46-48]

构造对每个 `mask` 按 `x` 递减遍历，选择 `x` 在该掩码内的第一个 good ascent。遇到 like-nonparity，或跨越 partial-block 边界使 `some_scent` 返回 `None` 时，索引记为 `DEAD_END`；否则沿 cross 链接继承索引，并按 `epsilon` 与已有 flip 位的差异设置符号位。收尾时将索引反转为递增存储；整行没有 primitive 时保持 `DEAD_END`。^[extended-kl.md:48-52]

## 查询集合与判定条件

对于元素对 `(x, y)`，两个查询集合定义为
\[
\begin{aligned}
\operatorname{very\_easy\_set}(x,y)
&=\operatorname{good\_ascents}(x)\cap\operatorname{descents}(y),\\
\operatorname{easy\_set}(x,y)
&=\operatorname{descents}(y)\setminus\operatorname{descents}(x).
\end{aligned}
\]
前者取 good ascent 集与目标下降集的交集，后者取目标下降集中不属于源下降集的生成元。^[extended-kl.md:56-57]

`is_extremal` 的条件是 `descent_set(x) ⊇ descents(y)`；`is_primitive` 的条件是 `good_ascent_set(x) ∩ descents(y)` 为空。两者分别检查下降集的包含关系与 good ascent 交集是否为空。^[extended-kl.md:59-60]

`x_index`、`self_index` 和 `flips` 经 `mask_of` 查表。此外，接口还提供 `length_floor`、`col_size`，以及回溯操作 `extr_back_up_mask` 和 `prim_back_up_mask`。^[extended-kl.md:58-62]

## 与多项式列访问的衔接

`ExtKlTable` 为每个块元素 `y` 保存一列多项式池索引，并以 `x` 相对于 `descents(y)` 的 primitive 位置寻址。`kl_pol_index` 返回 `(KLIndex, bool)`；`p(x,y)` 返回 twisted KLV 多项式 \(P_{x,y}\)，在 flip 为真时通过 `scaled(-1)` 应用符号。池索引本身不打包符号位，参见 [[多项式池与 primitivisation 符号分离]]。^[extended-kl.md:28-35, extended-kl.md:72-78]

## 证据边界

本页依据 `ext_kl.rs` 的结构性阅读材料，其阅读快照对应 dirty 工作区字节。材料中的上游行号转述自源码注释，未独立重读上游；材料也未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。扩展 KL 的正确性属于独立的 HPC 证据链。^[extended-kl.md:9-15, extended-kl.md:108-114]

## Sources

- [extended-kl.md](../../sources/extended-kl.md) — 扩展 KLV 多项式表：primitivisation 符号与逐列存储。
