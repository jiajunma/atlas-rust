---
title: KLV 递归与 μ-修正的多项式运算
summary: 多项式引擎提供加减、乘以 1+q、次数平移、带 μ 系数的修正及 q=-1 求值，供 KLV 递归使用。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T14:55:08.279Z"
updatedAt: "2026-10-09T22:35:00.198Z"
tags:
  - KLV多项式
  - 递归算法
aliases:
  - klv-递归与-μ-修正的多项式运算
  - K递Μ
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KLV 递归与 μ-修正的多项式运算
summary: KlPol 提供 KLV 逐列递归、μ-修正、除法和求值所需的多项式运算；整性检查与算术溢出具有不同的错误边界。
sources:
  - kl-polynomial-table.md
kind: concept
tags:
  - KLV多项式
  - 递归算法
  - μ修正
aliases:
  - klv-递归与-μ-修正的多项式运算
  - K递Μ
provenanceState: extracted
---

# KLV 递归与 μ-修正的多项式运算

`KlPol` 是 KLV 多项式计算的运算层，提供递归与 μ-修正所需的加减、次数平移、标量倍乘、除法和求值操作。`kl_polynomial.rs` 实现多项式引擎，`kl_table.rs` 负责按块存储与逐列填充。^[kl-polynomial-table.md:11-13, kl-polynomial-table.md:29-48]

## 表示与基本约定

`KlPol(Vec<i32>)` 按次数从低到高存储系数，零多项式为空向量；每次运算通过 `trim` 保持非零多项式的最高次系数非零。`degree()` 对零多项式返回 0，调用方须用 `is_zero()` 将其与常数多项式区分。非负性与首一性属于算法输出的外层性质，并非类型维护的不变量，参见 [[KLV 多项式的表示与不变量]]。^[kl-polynomial-table.md:21-27]

`coefficient()` 在下标越界时返回 0，`add` 与 `sub` 的正确性依赖这一行为。文档中“越界时 panic”的描述已过期，应以实现为准。^[kl-polynomial-table.md:55-56]

## 递归与 μ-修正

`add`、`sub` 实现普通多项式加减，`scaled(factor)` 实现标量倍乘。`shift()` 的含义是乘以 $1+q$；现有测试包含 $(1+2q)(1+q)=1+3q+2q^2$ 的展开。^[kl-polynomial-table.md:33-40, kl-polynomial-table.md:118-121]

`add_shifted(other, d)` 计算 $P+q^d\,other$，用于 complex descent 递归项，例如 $P_{sx,sy}+qP_{x,sy}$。`sub_shifted(other, d, mu)` 计算 $P-\mu q^d\,other$，用于减去 μ-修正项；`add_shifted_scaled(other, d, mu)` 则用于累加 μ-求和贡献。^[kl-polynomial-table.md:35-39]

这些操作服务于 [[KLV 表的幂等逐列填充算法]]。每列先为其 descent set 准备 primitive 索引；若 `first_direct_recursion` 找到使 $y$ 具有 complex descent 或 real type-I descent 的生成元 $s$，便执行 `recursion_column`，随后执行 `complete_primitives`。否则进入 `new_recursion_column`，按 $x$ 区分 “nice and real” 与 “endgame” 情形，μ-修正由此进入一般路径。来源未展开两条路径的完整公式与判定条件。^[kl-polynomial-table.md:101-114]

## 除法、整性检查与求值

`divide_by_2()` 用于 KLV 语境下应当恰好整除的计算。若任一系数为奇数，则返回 `StructureError::RepInvariantViolation`，错误信息为 `"KL polynomial parity"`，详见 [[KLV 多项式除法与整性检查]]。^[kl-polynomial-table.md:41-43]

`quotient_by_1_plus_q(bound)` 使用合成除法，以交错部分和恢复 $(1+q)P$ 的商，并截断到给定次数界。该函数虽然返回 `Result`，但当前函数体恒返回 `Ok`；这一签名用于对齐上游调用形态，并不意味着实现会报告整除失败。^[kl-polynomial-table.md:44-45, kl-polynomial-table.md:57-58]

`evaluate_at_minus_one()` 计算 $q=-1$ 时的值，即系数的交错和。`from_coefficients` 支持逐系数构造，供 `ext_kl` 的 `extract_M` 等路径使用。^[kl-polynomial-table.md:46-48]

## 算术错误边界

系数使用未经溢出检查的普通 `i32` 算术，移位涉及的 `index + d` 下标计算也没有溢出检查；模块没有 `ArithmeticOverflow` 错误通道。除法接口之外的运算不通过返回值报告失败，不能据此认定其具有溢出安全保证。是否可接受取决于系数上界，来源未作结论，参见 [[KLV 多项式算术的溢出错误边界]]。^[kl-polynomial-table.md:25-27, kl-polynomial-table.md:50-64]

## 测试与证据范围

来源列出的四个测试覆盖池种子序号、`shift` 展开、$q=-1$ 的交错和，以及 `sub_shifted` 的单项例子 $(1+q)-q\cdot1=1$。尚未覆盖的路径包括 `add`、`sub`、`add_shifted`、`scaled`、`divide_by_2` 错误分支、`quotient_by_1_plus_q`、`match_pol` 去重和 `get` 越界等。^[kl-polynomial-table.md:118-123]

本页依据结构性源码阅读，不构成 KLV 数学正确性验收。来源未执行构建、测试或原版运行，不含性能或并行结论；其上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。KLV 正确性属于独立的 [[HPC 验收证据链]]，本页不扩展其验收范围。^[kl-polynomial-table.md:9-17, kl-polynomial-table.md:130-135]

## Sources

- [kl-polynomial-table.md](../../sources/kl-polynomial-table.md) — KLV 多项式的存储与逐列计算。
