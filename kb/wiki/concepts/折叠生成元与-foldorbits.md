---
title: 折叠生成元与 fold_orbits
summary: ExtGen 表示折叠生成元，fold_orbits 根据 Cartan 矩阵和 δ 诱导的简单根置换生成有序轨道，并拒绝非对合 twist。
sources:
  - extended-block.md
kind: concept
createdAt: "2026-10-09T14:47:43.718Z"
updatedAt: "2026-10-09T14:47:43.718Z"
tags:
  - 生成元折叠
  - 根系
  - 算法
aliases:
  - 折叠生成元与-foldorbits
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 折叠生成元与 fold_orbits

折叠生成元用于描述[[扩展块与 δ-不动部分]]中的生成元结构：扩展块取普通块的 $\delta$-不动部分，简单生成元则折叠到 $\delta$-轨道中。`fold_orbits` 根据 Cartan 矩阵和 $\delta$ 诱导的简单根置换生成这些轨道。^[extended-block.md:19-22, extended-block.md:35-42]

## 数据表示与输入约定

折叠生成元表示为 `ExtGen { kind: ExtGenKind, s0, s1 }`，其中 `ExtGenKind` 分为 `One`、`Two`、`Three`，`length()` 分别返回 1、2、3。在 `One` 情形下，`s1` 使用 `usize::MAX` 作为哨兵值，对应上游的 `~0`。^[extended-block.md:37-38]

`fold_orbits(cartan, twist)` 特化于父块的简单生成元。参数 `twist` 是 $\delta$ 诱导的简单根置换；Cartan 矩阵遵循 crate 的配对约定
$\mathrm{cartan}[i][j]=\langle\alpha_i,\alpha_j^\vee\rangle$。输出轨道按 `s0` 递增排列；非对合的 `twist` 对应上游错误 “Not a distinguished involution”。^[extended-block.md:39-42]

## 全父块与部分公共块中的使用

`ExtBlock::build` 在全父块上构造扩展块，使用平凡的 block modifier。此时，不动点测试中的 `transformed_twisted` 退化为对两个 KGB 坐标分别应用 `kgb.twisted`，构造随后经 `complete_construction` 与诱导置换 `induced` 完成。^[extended-block.md:46-50]

在[[公共块的构造与元素编号（PartialBlock）]]上，`ExtBlock::build_partial` 使用真积分子系统的 Cartan 矩阵和生成元 twist 执行 `fold_orbits`。这一路径的不动点测试采用 `transformed_twisted` 的 `x + gamma_lambda` 形式：部分块的 `y` 是合成的子系统计数，并非对偶 KGB 元素，因此不能直接使用全块路径的对偶 `kgb.twisted` 测试。^[extended-block.md:52-57]

## cofold 与生成元姿态

部分块路径在 `complete_construction` 之后执行生成元姿态的 cofold：diagram、轨道和链接表通过 `induced(orbits, bm.simple_pi)` 置换，各轨道成员的编号再由 `bm.simple_pi` 重写。这将折叠结构与[[BlockModifier 块修正子]]所指定的生成元姿态联系起来。^[extended-block.md:58-60]

当前移植仅支持恒等生成元姿态。恒等姿态对应恒等置换，cofold 在既有路径上不产生变化；非恒等的 `bm.simple_pi` 会显式失败。^[extended-block.md:60-61]

## 查询接口与下降分类

扩展块通过 `rank` 返回轨道数，通过 `orbit(s)` 和 `folded_generators` 访问轨道及折叠生成元，通过 `folded_cartan` 返回上游 `folded_diagram` 的整数矩阵形式。^[extended-block.md:71-77]

[[DescValue 扩展下降分类]]按 `One*`、`Two*`、`Three*` 三族组织，其 `generator_length` 按族返回 1、2、3，用于表示折叠生成元长度。下降类型还决定链接数量；例如 `OneRealNonparity` 和 `OneImaginaryCompact` 这类零链接类型不记录 cross action。^[extended-block.md:24-33]

## 证据范围

本页依据扩展块结构切片的源码阅读材料。材料中的上游行号来自源码注释，未独立重读上游；该来源未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论。^[extended-block.md:9-15, extended-block.md:79-90]

## Sources

- [extended-block.md](extended-block.md) — 扩展块：delta-不动部分与折叠生成元。
