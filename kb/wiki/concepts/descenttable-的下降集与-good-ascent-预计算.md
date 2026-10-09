---
title: DescentTable 的下降集与 good ascent 预计算
summary: DescentTable 按元素和生成元预计算 descents 与至多具有一个向上邻居的 good ascents，并在秩超过 MAX_FOLDED_RANK 时返回资源限制错误。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T14:48:19.776Z"
updatedAt: "2026-10-09T14:48:19.776Z"
tags:
  - 预计算
  - 下降集
  - 扩展块
aliases:
  - descenttable-的下降集与-good-ascent-预计算
  - D的GA预
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# DescentTable 的下降集与 good ascent 预计算

`DescentTable<'a>` 是扩展（twisted）KLV 多项式表的预计算层：它记录每个块元素的下降集与 good ascent 集，并针对下降集掩码预计算 primitive 索引及 primitivisation 的符号翻转。[[扩展 KLV 多项式表的逐列存储|ExtKlTable]] 使用这些数据按 primitive 位置访问多项式列。^[extended-kl.md:19-24, extended-kl.md:72-77]

## 数据结构与生成元分类

`DescentTable` 持有扩展块引用 `block: &ExtBlock`，以及每个元素的两个 `RankFlags` 位集 `descents`、`good_ascents`，另有 `prim_index` 与 `prim_flip`。位集表示可关联 [[RankFlags：简单生成元位集]]。^[extended-kl.md:39-41]

构造时，对每个块元素 `x` 和每个生成元 `s` 调用 `block.descent_type(s, x)`：若类型满足 `is_descent()`，便将 `s` 加入 `descents(x)`；否则，仅当 `!has_double_image()` 时加入 `good_ascents(x)`。因此，good ascent 对应“至多一个向上邻居”，并不保证存在向上邻居。若 `rank > MAX_FOLDED_RANK`，构造返回 `StructureError::ResourceLimitExceeded`。^[extended-kl.md:41-44]

## Primitive 索引与符号预计算

`prim_index[mask][x]` 表示元素 `x` 相对于下降集 `mask` 做 primitivisation 后的位置；`prim_flip[x]` 则记录哪些下降集会使这一过程拾取符号翻转。索引与符号分别存储，后续查询再组合使用，参见 [[Primitivisation 索引与符号传播]]。^[extended-kl.md:46-48, extended-kl.md:28-35]

构造对每个 `mask` 按 `x` 递减遍历，选择 `x` 在该掩码内的第一个 good ascent。若遇到 like-nonparity，或跨越 partial-block 边界而使 `some_scent` 返回 `None`，便记为 `DEAD_END`；否则沿 cross 链接继承索引，并根据 `epsilon` 与已有 flip 位的差异设置符号位。最后将索引反转为递增存储；整行没有 primitive 时保持 `DEAD_END`。^[extended-kl.md:48-52]

## 查询集合与判定条件

对于元素对 `(x, y)`，预计算位集给出两个查询集合：
\[
\operatorname{very\_easy\_set}(x,y)
=
\operatorname{good\_ascents}(x)\cap\operatorname{descents}(y),
\qquad
\operatorname{easy\_set}(x,y)
=
\operatorname{descents}(y)\setminus\operatorname{descents}(x).
\]
前者使用 good ascent 与目标下降集的交集，后者使用两个下降集的差集。^[extended-kl.md:56-57]

`is_extremal` 要求 `descent_set(x) ⊇ descents(y)`；`is_primitive` 则要求 `good_ascent_set(x) ∩ descents(y)` 为空。这两个条件分别约束下降集的包含关系与 good ascent 的交集，详见 [[扩展 KLV 的 extremal 与 primitive 判定]]。^[extended-kl.md:59-60]

`x_index`、`self_index` 和 `flips` 经 `mask_of` 查表；此外还提供 `length_floor`、`col_size`，以及回溯接口 `extr_back_up_mask`、`prim_back_up_mask`。^[extended-kl.md:58-62]

## 与多项式访问的衔接

`ExtKlTable` 为每个块元素 `y` 保存一列多项式池索引，并以 `x` 相对于 `descents(y)` 的 primitive 位置寻址。`kl_pol_index` 返回 `(KLIndex, bool)`，而 `p(x,y)` 在 flip 为真时通过 `scaled(-1)` 应用符号。因此，`prim_flip` 的符号信息独立于池索引，索引本身不打包符号位，参见 [[多项式池与 primitivisation 符号分离]]。^[extended-kl.md:28-35, extended-kl.md:72-78]

## 证据边界

本页依据对 `ext_kl.rs` 的结构性阅读材料；其快照对应 dirty 工作区字节。材料中的上游行号来自源码注释，未独立重读上游；该材料也未执行构建、测试或原版运行，不能据此声称数学验收、性能或并行结论。^[extended-kl.md:9-15, extended-kl.md:108-114]

## Sources

- [extended-kl.md](extended-kl.md)
