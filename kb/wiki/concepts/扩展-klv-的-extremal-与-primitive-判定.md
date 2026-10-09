---
title: 扩展 KLV 的 extremal 与 primitive 判定
summary: Extremal 要求 descents(x) 包含 descents(y)，primitive 要求 good_ascents(x) 与 descents(y) 不交；相关集合查询及回溯接口由 DescentTable 提供。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T14:48:30.245Z"
updatedAt: "2026-10-09T14:48:30.245Z"
tags:
  - 扩展KLV
  - 集合判定
  - 回溯
aliases:
  - 扩展-klv-的-extremal-与-primitive-判定
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 扩展 KLV 的 extremal 与 primitive 判定

扩展（twisted）KLV 多项式表通过 `DescentTable` 预计算每个块元素的下降集与 good ascent 集，并据此判定元素对 \((x,y)\) 的 extremal 与 primitive 性质。primitive 位置还用于[[扩展 KLV 多项式表的逐列存储]]中的列寻址。^[extended-kl.md:19-24, extended-kl.md:39-43, extended-kl.md:72-78]

## 集合定义与判定条件

记 \(D(x)\) 为元素 \(x\) 的下降集，\(G(x)\) 为其 good ascent 集。构造时，对每个生成元 \(s\) 查询 `block.descent_type(s, x)`：若 `is_descent()` 成立，将 \(s\) 加入 \(D(x)\)；否则，若 `!has_double_image()`，将其加入 \(G(x)\)。因此 good ascent 对应非下降且至多有一个向上邻居的情形。相关状态由 [[DescValue 扩展下降分类]]描述，两个集合以 `RankFlags` 存储。^[extended-kl.md:39-44, extended-kl.md:75-75]

`is_extremal(x,y)` 要求 \(x\) 的下降集包含 \(y\) 的全部下降方向，即
\[
D(x)\supseteq D(y).
\]
等价地，`easy_set(x,y) = D(y)\setminus D(x)` 为空。^[extended-kl.md:56-60]

`is_primitive(x,y)` 要求 \(x\) 的 good ascent 集与 \(y\) 的下降集不相交，即
\[
G(x)\cap D(y)=\varnothing.
\]
这恰好表示 `very_easy_set(x,y) = G(x)\cap D(y)` 为空。两个判定都以 \(y\) 的下降集为参照，但分别检查 \(x\) 的下降集与 good ascent 集。^[extended-kl.md:56-60]

由于构造时 \(G(x)\) 只包含非下降生成元，\(D(x)\) 与 \(G(x)\) 不相交。因此，extremal 条件蕴含 primitive 条件；反向推导不能仅依赖这些集合定义，因为不属于 \(D(x)\) 的生成元未必属于 \(G(x)\)。

## primitive 位置与符号

`prim_index[mask][x]` 记录 \(x\) 相对于下降集 `mask` 进行 primitivisation 后的位置。构造对每个 `mask` 按 \(x\) 递减遍历，选择 `mask` 内的第一个 good ascent：若遇到 like-nonparity，或 `some_scent` 为 `None`、跨越 partial-block 边界，则记为 `DEAD_END`；否则沿 cross 链接继承索引，并根据 `epsilon` 与已有 flip 位的差异设置符号翻转。最终将索引反转为递增存储；整行没有 primitive 元素时保留 `DEAD_END`。^[extended-kl.md:46-52]

查询接口 `x_index`、`self_index` 与 `flips` 经 `mask_of` 查表。`ExtKlTable` 的第 \(y\) 列按 \(x\) 相对于 \(D(y)\) 的 primitive 位置寻址，因此 primitive 判定与预计算索引共同支撑列访问。^[extended-kl.md:58-58, extended-kl.md:72-78]

primitivisation 的符号独立存放在 `prim_flip` bitmap 中，不打包进多项式池索引。`kl_pol_index` 返回 `(KLIndex, bool)`，而 `p(x,y)` 在 flip 为真时通过 `scaled(-1)` 得到带符号的多项式。这一设计参见[[多项式池与 primitivisation 符号分离]]。^[extended-kl.md:28-35, extended-kl.md:76-78]

## 实现与证据边界

`DescentTable` 在 `rank > MAX_FOLDED_RANK` 时返回 `StructureError::ResourceLimitExceeded`。本页依据的是源码结构性阅读；源材料未执行构建、测试或原版运行，不提供数学验收或性能结论。上游行号来自 Rust 源码注释的转述，未独立重读上游。^[extended-kl.md:39-44, extended-kl.md:108-114]

## Sources

- [extended-kl.md](extended-kl.md)：扩展 KLV 多项式表：primitivisation 符号与逐列存储。
