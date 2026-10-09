---
title: 折叠生成元与 fold_orbits
summary: fold_orbits 根据 Cartan 矩阵与 δ 诱导的简单生成元对合置换构造 ExtGen，按 s0 递增输出，并拒绝非对合 twist。
sources:
  - extended-block.md
kind: concept
createdAt: "2026-10-09T14:47:43.718Z"
updatedAt: "2026-10-09T20:52:20.032Z"
tags:
  - 生成元折叠
  - 根系
aliases:
  - 折叠生成元与-foldorbits
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 折叠生成元与 fold_orbits
summary: fold_orbits 根据 Cartan 矩阵及 δ 诱导的简单根对合置换构造折叠生成元，按 s0 递增输出轨道；部分公共块使用子系统数据，当前仅支持恒等生成元姿态。
sources:
  - extended-block.md
kind: concept
tags:
  - 根系
  - 生成元折叠
aliases:
  - 折叠生成元与-foldorbits
---

# 折叠生成元与 fold_orbits

折叠生成元描述[[扩展块与 δ-不动部分]]中的生成元结构：扩展块取普通块的 $\delta$-不动部分，生成元折叠进 $\delta$-轨道。`fold_orbits` 根据 Cartan 矩阵和 $\delta$ 诱导的简单根置换构造有序轨道。^[extended-block.md:19-22, extended-block.md:35-42]

## 数据表示与输入约定

折叠生成元表示为 `ExtGen { kind: ExtGenKind, s0, s1 }`。`ExtGenKind` 包含 `One`、`Two`、`Three`，`length()` 分别返回 1、2、3；在 `One` 情形下，`s1` 使用 `usize::MAX` 作为哨兵值，对应上游的 `~0`。^[extended-block.md:37-38]

`fold_orbits(cartan, twist)` 移植自上游 `rootdata::fold_orbits(rd, delta)`，并特化到父块的简单生成元。`twist` 是 $\delta$ 诱导的简单根置换，Cartan 矩阵采用配对约定 $\mathrm{cartan}[i][j]=\langle\alpha_i,\alpha_j^\vee\rangle$。输出轨道按 `s0` 递增排列；非对合的 `twist` 对应上游错误 “Not a distinguished involution”。^[extended-block.md:39-42]

## 全父块与部分公共块

`ExtBlock::build` 在全父块上构造扩展块，使用平凡的块修正子。此时 `transformed_twisted` 退化为对两个 KGB 坐标分别应用 `kgb.twisted`，构造经 `complete_construction` 与诱导置换 `induced` 完成。^[extended-block.md:46-50]

`ExtBlock::build_partial` 在[[公共块的构造与元素编号（PartialBlock）]]上构造扩展块，`fold_orbits` 使用真积分子系统的 Cartan 矩阵与子系统生成元 twist。不动点测试采用 `transformed_twisted` 的 `x + gamma_lambda` 形式：部分块的 `y` 是合成的子系统 y-计数，并非对偶 KGB 元素，因此不能沿用全块路径的对偶 `kgb.twisted` 测试。^[extended-block.md:52-57]

## cofold 与生成元姿态

部分块路径在 `complete_construction` 之后执行生成元姿态的 cofold：diagram、轨道和链接表通过 `induced(orbits, bm.simple_pi)` 置换，各轨道成员编号再经 `bm.simple_pi` 重写。相关流程见[[部分公共块上的扩展块构造与 cofold]]。^[extended-block.md:58-60]

当前移植仅支持恒等生成元姿态。恒等姿态对应恒等置换，因此 cofold 在既有路径上不产生变化；非恒等的 `bm.simple_pi` 会显式失败。^[extended-block.md:60-61]

## 查询接口与下降分类

扩展块通过 `rank` 返回轨道数，通过 `orbit(s)` 和 `folded_generators` 提供轨道及折叠生成元访问，通过 `folded_cartan` 返回上游 `folded_diagram` 的整数矩阵形式。^[extended-block.md:73-77]

[[DescValue 扩展下降分类]]的 32 个变体按 `One*`、`Two*`、`Three*` 三族组织；其 `generator_length` 按族返回 1、2、3，表示折叠生成元长度。`link_count` 描述链接数量，`OneRealNonparity` 和 `OneImaginaryCompact` 等零链接类型不记录 cross action。^[extended-block.md:24-33]

## 证据范围

本页依据 `ext_block.rs` 结构切片的源码阅读材料，其快照记录的是 dirty 工作区字节。材料中的上游行号转述自源码注释，未独立重读上游；来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。扩展块正确性属于其自身的 HPC 证据链，本材料不扩展该范围。^[extended-block.md:9-15, extended-block.md:81-90]

## Sources

- [extended-block.md](../../sources/extended-block.md) — 扩展块：delta-不动部分与折叠生成元。
