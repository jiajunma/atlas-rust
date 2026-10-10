---
title: KLV 多项式引擎的测试覆盖与证据边界
summary: 四项测试锚点覆盖池种子、乘以 1+q、q=-1 求值及移位减法；本包未执行测试，不能扩展独立 HPC 数学验收结论。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T19:32:16.192Z"
updatedAt: "2026-10-10T00:38:42.869Z"
tags:
  - KLV
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: KLV 多项式引擎的测试覆盖与证据边界
summary: 四个局部测试覆盖池种子、乘以 1+q、q=-1 求值及移位减法；多项基础运算、去重与除法仍有覆盖缺口，结构性阅读不构成测试通过或数学验收证据。
sources:
  - kl-polynomial-table.md
kind: concept
tags:
  - KLV多项式
  - 测试覆盖
  - 证据边界
aliases:
  - klv-多项式引擎的测试覆盖与证据边界
provenanceState: extracted
---

# KLV 多项式引擎的测试覆盖与证据边界

`kl_polynomial.rs` 的测试覆盖集中于四个局部测试：池种子序号、乘以 \(1+q\)、在 \(q=-1\) 处求值及移位减法。来源材料仅对测试与实现进行了结构性阅读，未执行构建、测试或原版运行；这些测试锚点不代表本次运行通过，也不构成数学验收证据。^[kl-polynomial-table.md:118-123, kl-polynomial-table.md:135-135]

## 已有测试锚点

[[KLV 多项式去重池]]的种子测试检查 `get(0)` 为零多项式、`get(1).as_slice() == &[1]`，对应索引 0 表示零、索引 1 表示常数一的初始化约定。种子序号测试不覆盖 `match_pol` 的内容去重路径，后者被来源明确列为未测试面。^[kl-polynomial-table.md:68-72, kl-polynomial-table.md:118-123]

`shift` 测试验证 \((1+2q)(1+q)=1+3q+2q^2\)，检验该接口乘以 \(1+q\) 的语义。`sub_shifted` 测试验证 \((1+q)-q\cdot1=1\)，覆盖移位减法的一个单项用例；该运算用于 KLV 递归中的 μ-修正。^[kl-polynomial-table.md:34-38, kl-polynomial-table.md:118-121]

`evaluate_at_minus_one()` 测试检查系数向量 `[1,−1,2]` 在 \(q=-1\) 处的求值为 4，以及 \((1+q)^2\) 的求值为 0，直接检验系数交错求和行为。^[kl-polynomial-table.md:46-47, kl-polynomial-table.md:120-121]

## 未覆盖的接口与错误分支

来源列出的未测试面包括 `add`、`sub`、`add_shifted`、`scaled`、`divide_by_2` 的错误分支、`quotient_by_1_plus_q`、`match_pol` 去重路径及 `get` 越界行为。因此，现有四个测试并未覆盖多项式引擎的完整接口。^[kl-polynomial-table.md:118-123]

[[KLV 多项式除法与整性检查]]需要区分实际错误路径与返回类型：`divide_by_2()` 遇到任一奇系数会返回 `StructureError::RepInvariantViolation`，错误信息为 `"KL polynomial parity"`，但该分支未被上述测试覆盖。`quotient_by_1_plus_q` 虽返回 `Result`，函数体却恒返回 `Ok`，其签名用于与上游调用形态对齐。^[kl-polynomial-table.md:41-45, kl-polynomial-table.md:57-58, kl-polynomial-table.md:122-123]

## 结构复核发现的边界

`coefficient()` 的文档声称越界会 panic，实际实现却返回 0，且 `add`、`sub` 的正确性依赖这一行为。来源将该文档标为过期，并明确以实现为准；相关表示约定参见 [[KLV 多项式的表示与不变量]]。^[kl-polynomial-table.md:55-56]

`KlHashTable::default()` 产生没有 0、1 号种子的空池，与 `new()` 的语义不同。如果存在 `default()` 调用点，就会破坏零与一的固定池索引约定；来源将其记录为条件性风险，并未确认这样的调用点存在。^[kl-polynomial-table.md:59-61]

[[KLV 多项式算术的溢出错误边界]]同样需要单独审视：实现采用未检查的普通算术，包括 `i32` 系数运算及 `index + d` 下标计算，没有 `ArithmeticOverflow` 错误通道。其可接受性取决于系数上界，来源对此不作断言。^[kl-polynomial-table.md:62-64]

## 证据适用范围

来源包解释 `kl_polynomial.rs` 与 `kl_table.rs` 的代码结构。材料记录了两次结构性阅读，其中重读仅覆盖 `kl_polynomial.rs`，所读字节与初读相同；源码阅读与快照记录不应被表述为测试执行结果。^[kl-polynomial-table.md:9-17, kl-polynomial-table.md:124-129, kl-polynomial-table.md:135-135]

KLV 计算正确性属于独立的 [[HPC 验收证据链]]，包括 F4/E6 修复与 rank6 inventory，来源包不重述或扩展这些结论。一般递归路径中 endgame 的历史修复，包括 `first_endgame_pair` 与 real-II cross 的 `UndefBlock` 边界，也有自己的 tests-first 证据链。^[kl-polynomial-table.md:11-13, kl-polynomial-table.md:112-114]

来源中的上游文件行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。本包未执行任何构建、测试或原版运行，不提供数学验收、性能或并行结论。^[kl-polynomial-table.md:130-135]

## Sources

- [kl-polynomial-table.md](../../sources/kl-polynomial-table.md) — KLV 多项式的存储与逐列计算。
