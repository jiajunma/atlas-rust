---
title: KLV 多项式引擎的测试覆盖与证据边界
summary: 源码四项测试锚定池种子、乘以 1+q、q=-1 求值及移位减法，去重、除法和多项基础运算仍缺测试；本次结构性阅读未执行测试，也不扩展独立 HPC 验收结论。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T19:32:16.192Z"
updatedAt: "2026-10-09T19:32:16.192Z"
tags:
  - KLV多项式
  - 测试覆盖
  - 证据边界
aliases:
  - klv-多项式引擎的测试覆盖与证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# KLV 多项式引擎的测试覆盖与证据边界

KLV 多项式引擎的测试覆盖集中于 `kl_polynomial.rs` 的四个局部测试，涉及池种子序号、乘以 \(1+q\)、在 \(q=-1\) 处求值以及一次移位减法。来源材料对这些测试进行了结构性阅读，但未执行构建、测试或原版运行，因此测试锚点的存在不能视为本次运行通过或数学验收的证据。^[kl-polynomial-table.md:118-123, kl-polynomial-table.md:135-135]

## 已有测试锚点

[[KLV 多项式去重池]]的种子测试检查 `get(0)` 为零多项式、`get(1).as_slice() == &[1]`，对应池索引 0 表示零、索引 1 表示常数一的约定。该测试覆盖种子序号，但来源明确将 `match_pol` 去重路径列为未测试面。^[kl-polynomial-table.md:68-72, kl-polynomial-table.md:118-123]

`shift` 测试验证 \((1+2q)(1+q)=1+3q+2q^2\)，对应此接口乘以 \(1+q\) 的语义。`sub_shifted` 测试验证 \((1+q)-q\cdot1=1\)，覆盖移位减法的一个单项用例；相关运算用于 [[KLV 递归与 μ-修正的多项式运算]]。^[kl-polynomial-table.md:34-38, kl-polynomial-table.md:118-121]

在 \(q=-1\) 处求值的测试检查系数向量 `[1,−1,2]` 的交错和为 4，以及 \((1+q)^2\) 的求值结果为 0。这些用例直接检验 `evaluate_at_minus_one()` 的交错求和行为。^[kl-polynomial-table.md:46-47, kl-polynomial-table.md:120-121]

## 未覆盖的接口与错误分支

来源列出的未测试面包括 `add`、`sub`、`add_shifted`、`scaled`、`divide_by_2` 的错误分支、`quotient_by_1_plus_q`、`match_pol` 去重路径及 `get` 越界行为；现有四个测试并未覆盖多项式引擎的完整接口。^[kl-polynomial-table.md:118-123]

[[KLV 多项式除法与整性检查]]尤其需要区分接口语义与错误分支覆盖：`divide_by_2()` 遇到任一奇系数会返回 `StructureError::RepInvariantViolation`，错误信息为 `"KL polynomial parity"`，但该分支未被上述测试覆盖。`quotient_by_1_plus_q` 虽返回 `Result`，函数体却恒返回 `Ok`，其返回类型用于保持与上游调用形态一致。^[kl-polynomial-table.md:41-45, kl-polynomial-table.md:57-58, kl-polynomial-table.md:122-123]

## 结构复核揭示的边界

`coefficient()` 的文档声称越界会 panic，实际实现却返回 0，且 `add`、`sub` 的正确性依赖这一行为。理解 [[KLV 多项式的零延拓系数访问与零多项式判别]]时应以实现为准，不能把过期文档当作已经验证的接口契约。^[kl-polynomial-table.md:55-56]

`KlHashTable::default()` 产生没有 0、1 号种子的空池，与 `new()` 的语义不同；若存在 `default()` 调用点，就会破坏零与一的固定池索引约定。来源将其记录为条件性风险，并未确认存在这样的调用点。^[kl-polynomial-table.md:59-61]

[[KLV 多项式算术的溢出错误边界]]也不由现有测试解决：实现采用未检查的普通算术，包括 `i32` 系数运算及 `index + d` 下标计算，没有 `ArithmeticOverflow` 错误通道。其可接受性取决于系数上界，来源对此不作断言。^[kl-polynomial-table.md:62-64]

## 证据适用范围

来源包解释 `kl_polynomial.rs` 与 `kl_table.rs` 的代码结构。KLV 计算的正确性属于独立的 [[HPC 验收证据链]]，包括 F4/E6 修复与 rank6 inventory；本包不重述或扩展这些结论。一般递归路径中 endgame 的历史修复也有独立的 tests-first 证据链，不能由上述四个多项式测试替代。^[kl-polynomial-table.md:11-17, kl-polynomial-table.md:112-114]

来源中的上游文件行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。本包未执行任何构建、测试或原版运行，也不提供数学验收、性能或并行结论。^[kl-polynomial-table.md:130-135]

## Sources

- [kl-polynomial-table.md](kl-polynomial-table.md) — KLV 多项式的存储与逐列计算。
