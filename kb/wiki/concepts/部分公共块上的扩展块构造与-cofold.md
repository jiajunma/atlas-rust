---
title: 部分公共块上的扩展块构造与 cofold
summary: build_partial 使用 x 与 gamma_lambda 测试不动点，在整子系统上折叠生成元并处理 cofold；非恒等 bm.simple_pi 姿态尚未移植且显式失败。
sources:
  - extended-block.md
kind: concept
createdAt: "2026-10-09T14:47:46.305Z"
updatedAt: "2026-10-09T19:28:35.288Z"
tags:
  - 部分公共块
  - 生成元折叠
aliases:
  - 部分公共块上的扩展块构造与-cofold
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 部分公共块上的扩展块构造与 cofold

`ExtBlock::build_partial` 在真积分子系统上的[[公共块的构造与元素编号（PartialBlock）|部分公共块（PartialBlock）]]中构造扩展块。[[扩展块与 δ-不动部分|扩展块]]取父块的 $\delta$-不动部分，并将生成元折叠为 $\delta$-轨道；部分公共块路径采用专门的不动点测试、子系统上的生成元折叠，以及构造完成后的 cofold。^[extended-block.md:19-22, extended-block.md:52-61]

## 不动点测试

`build_partial` 使用 `transformed_twisted` 的 `x + gamma_lambda` 形式判定不动点。部分公共块的 `y` 是合成的子系统 y-计数，并非对偶 KGB 元素，因此全块路径中对偶 `kgb.twisted` 的测试不适用。^[extended-block.md:52-55]

相比之下，[[局部扩展类型识别与全父块构造|全父块构造]] `ExtBlock::build` 使用恒等 block modifier，此时 `transformed_twisted` 退化为对两个 KGB 坐标分别应用 `kgb.twisted`，随后经 `complete_construction` 与诱导置换 `induced` 完成构造。^[extended-block.md:46-50]

## 子系统上的生成元折叠

`build_partial` 中的 [[折叠生成元与 fold_orbits|fold_orbits]] 运行在**子系统 Cartan 矩阵与子系统生成元 twist** 上。`twist` 表示 $\delta$ 诱导的简单根置换，Cartan 矩阵遵循 `cartan[i][j] = <alpha_i, alpha_j^v>` 的约定。折叠轨道按 `s0` 递增输出；非对合的 `twist` 对应上游错误 “Not a distinguished involution”。^[extended-block.md:39-42, extended-block.md:56-57]

折叠生成元以 `ExtGen { kind: ExtGenKind, s0, s1 }` 表示。`kind` 分为 `One`、`Two`、`Three`，`length()` 分别返回 1、2、3；`One` 情形下，`s1` 取 `usize::MAX`，对应上游的 `~0`。^[extended-block.md:37-38]

## cofold 的顺序与作用

生成元姿态的 cofold 在 `complete_construction` **之后**执行。它通过 `induced(orbits, bm.simple_pi)` 置换 diagram、轨道和链接表，并使用 `bm.simple_pi` 重写各轨道成员的编号，使这些结构随生成元姿态同步调整。^[extended-block.md:58-60]

当前移植**仅支持恒等生成元姿态**。恒等姿态产生恒等置换，因此 cofold 在既有路径上是无操作；非恒等的 `bm.simple_pi` 会显式失败。上述 cofold 流程不表示当前实现已支持任意生成元姿态。^[extended-block.md:58-61]

## 构造结果的访问

构造结果提供 `rank` 查询折叠轨道数，通过 `orbit(s)`、`folded_generators` 和 `folded_cartan` 访问折叠结构，其中 `folded_cartan` 是上游 `folded_diagram` 的整数矩阵形式。[[扩展块与父块的索引映射]]由 `z(n)` 等访问器体现：`z(n)` 返回扩展元素的父索引，`length(n)` 等于 `parent.length(z(n))`。^[extended-block.md:73-77]

## 证据边界

本页依据 `ext_block.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；扩展块正确性属于独立的 [[HPC 验收证据链]]，包括 ext-KL/unitarity gate。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本变化。^[extended-block.md:9-15, extended-block.md:84-90]

## Sources

- [extended-block.md](extended-block.md) — 扩展块：delta-不动部分与折叠生成元。
