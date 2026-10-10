---
title: 扩展 KLV 递归的 easy 集与 very easy 集
summary: very_easy_set(x,y) 取 x 的 good ascents 与 y 的 descents 的交集，easy_set(x,y) 则取 y 的 descents 中不属于 x 的 descents 的生成元。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T19:28:56.787Z"
updatedAt: "2026-10-09T19:28:56.787Z"
tags:
  - 扩展KLV
  - 下降集
  - 递归算法
aliases:
  - 扩展-klv-递归的-easy-集与-very-easy-集
  - 扩K递E集VE集
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 扩展 KLV 递归的 easy 集与 very easy 集

扩展（twisted）KLV 多项式表通过 `DescentTable` 预计算下降集与 good ascent 集，并提供 `easy_set(x,y)` 和 `very_easy_set(x,y)` 查询。二者都以 `y` 的下降集为参照，分别筛选 `x` 的非下降生成元与 good ascent 生成元。^[extended-kl.md:19-24, extended-kl.md:39-43, extended-kl.md:56-57]

## 定义与预计算

记 $D(x)$ 为元素 $x$ 的下降集，$G(x)$ 为其 good ascent 集。构造 `DescentTable` 时，实现遍历每个元素 `x` 和生成元 `s`，调用 `block.descent_type(s, x)`：若 `is_descent()` 为真，则将 `s` 加入 $D(x)$；否则，若 `!has_double_image()`，则将其加入 $G(x)$。这里的 good ascent 表示至多有一个向上邻居，不要求恰好存在一个。两个集合均以 `RankFlags` 存储，参见 [[DescentTable 的下降集与 good ascent 预计算]]。^[extended-kl.md:39-44]

两个查询的定义为 $\operatorname{easy}(x,y)=D(y)\setminus D(x)$，以及 $\operatorname{very\_easy}(x,y)=G(x)\cap D(y)$。因此，easy 集要求生成元在 `y` 处下降、在 `x` 处不下降；very easy 集进一步要求该生成元在 `x` 处属于 good ascent。^[extended-kl.md:56-57]

由于 good ascent 仅在非下降分支中登记，very easy 集是 easy 集的子集。两者的区别在于：easy 集只检查下降关系，very easy 集还使用 `has_double_image()` 对向上邻居情况作筛选。^[extended-kl.md:41-43, extended-kl.md:56-57]

## 与 extremal、primitive 判定的关系

`is_extremal(x,y)` 的条件是 $D(x)\supseteq D(y)$，等价于 easy 集为空；`is_primitive(x,y)` 的条件是 $G(x)\cap D(y)=\varnothing$，即 very easy 集为空。这将两个集合查询与 extremal、primitive 判定直接对应起来，相关概念见 [[下降集、good ascent 与本原性]]。^[extended-kl.md:56-60]

## 与 primitivisation 的衔接

`DescentTable` 不仅保存集合，还预计算 `prim_index` 与独立的 `prim_flip` 位图。对每个下降集掩码 `mask`，构造过程按 `x` 递减遍历，并取 `x` 在该掩码内的第一个 good ascent；当掩码取 $D(y)$ 时，这一步使用的候选集合就是 $\operatorname{very\_easy}(x,y)$。^[extended-kl.md:39-40, extended-kl.md:46-49, extended-kl.md:56-57]

沿该步骤遇到 like-nonparity，或跨越 partial-block 边界而使 `some_scent` 为 `None` 时，索引记为 `DEAD_END`；否则沿 cross 链接继承索引，并依据 `epsilon` 与已有 flip 位的差异记录符号。收尾时索引反转为递增存储，整行没有 primitive 时保持 `DEAD_END`。参见 [[Primitivisation 索引与符号传播]]。^[extended-kl.md:49-52]

`ExtKlTable` 按块元素 `y` 逐列存储多项式池索引，并依据 `x` 相对于 $D(y)$ 的 primitive 位置寻址。查询 `kl_pol_index` 返回索引与符号翻转标志组成的二元组；`p(x,y)` 在需要翻转时通过 `scaled(-1)` 返回带符号的 twisted KLV 多项式。^[extended-kl.md:72-78]

## 证据边界

来源给出了 easy、very easy 集的定义、相关判定及 primitivisation 预计算，但没有展开完整的 extended-block 递归公式。本页据此说明集合接口及其与索引构造的关系；来源本身属于结构性阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[extended-kl.md:9-12, extended-kl.md:19-24, extended-kl.md:46-62, extended-kl.md:114-114]

## Sources

- [extended-kl.md](extended-kl.md) — 扩展 KLV 多项式表：primitivisation 符号与逐列存储。
