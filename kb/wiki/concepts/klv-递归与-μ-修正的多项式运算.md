---
title: KLV 递归与 μ-修正的多项式运算
summary: 多项式引擎提供加减、乘以 1+q、次数平移和带 μ 系数的修正运算，以及 q=-1 求值，支持 KLV 递归所需的计算。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T14:55:08.279Z"
updatedAt: "2026-10-09T14:55:08.279Z"
tags:
  - KLV多项式
  - 递归算法
  - μ修正
aliases:
  - klv-递归与-μ-修正的多项式运算
  - K递Μ
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# KLV 递归与 μ-修正的多项式运算

KLV 多项式引擎以 `KlPol(Vec<i32>)` 表示变量 $q$ 上的多项式，提供逐列递归、μ-修正和相关求值所需的运算。它与按块存储和填充的 `KlTable` 配合；多项式运算负责系数计算，列填充算法负责选择递归路径。^[kl-polynomial-table.md:11-13, kl-polynomial-table.md:21-27, kl-polynomial-table.md:99-110]

## 表示与基本约定

系数按次数从低到高排列，零多项式用空向量表示；非零多项式通过每次运算后的 `trim` 保持最高次系数非零。`degree()` 对零多项式也返回 0，因此调用方必须用 `is_zero()` 区分零多项式与常数多项式。非负性与首一性属于算法输出的外层性质，并非类型维护的不变量，详见 [[KLV 多项式的表示与不变量]]。^[kl-polynomial-table.md:21-27]

`coefficient()` 在越界时返回 0，`add` 与 `sub` 的正确性依赖这一行为；其文档中“越界时 panic”的描述已经过期，应以实现为准。^[kl-polynomial-table.md:55-56]

## 递归与 μ-修正运算

`add`、`sub` 提供普通加减法，`scaled(factor)` 提供标量倍乘。`shift()` 的含义是乘以 $1+q$；例如 $(1+2q)(1+q)=1+3q+2q^2$，这一展开也是现有测试锚点之一。^[kl-polynomial-table.md:33-40, kl-polynomial-table.md:118-121]

`add_shifted(other, d)` 计算 $P+q^d\,other$，用于构造 complex descent 的递归项，例如 $P_{sx,sy}+qP_{x,sy}$。`sub_shifted(other, d, mu)` 计算 $P-\mu q^d\,other$，承担 μ-修正项的减法；`add_shifted_scaled(other, d, mu)` 则用于累加 μ-求和贡献。^[kl-polynomial-table.md:35-39]

这些运算嵌入 [[KLV 表的幂等逐列填充算法]]：每列先准备对应 descent set 的 primitive 索引。若 `first_direct_recursion` 找到使 $y$ 具有 complex descent 或 real type-I descent 的生成元 $s$，则执行 `recursion_column`，随后执行 `complete_primitives`；否则进入 `new_recursion_column`，按 $x$ 区分 “nice and real” 与 “endgame” 情形，μ-修正由此进入一般路径。来源没有展开两条路径的完整公式与判定条件。^[kl-polynomial-table.md:101-114]

## 除法、整性与求值

`divide_by_2()` 用于 KLV 计算中应当恰好整除的情形。只要存在奇系数，就返回 `StructureError::RepInvariantViolation`，错误信息为 `"KL polynomial parity"`。这类整性约束可参见 [[KLV 多项式除法与整性检查]]。^[kl-polynomial-table.md:41-43]

`quotient_by_1_plus_q(bound)` 通过合成除法恢复 $(1+q)P$ 的商，使用交错部分和并截断到给定次数界。虽然签名返回 `Result`，当前函数体恒返回 `Ok`；这一签名形态用于对齐上游调用方式，不能据此推断实现会报告整除失败。^[kl-polynomial-table.md:44-45, kl-polynomial-table.md:57-58]

`evaluate_at_minus_one()` 计算 $q=-1$ 时的值，即系数的交错和；`from_coefficients` 支持逐系数构造，供 `ext_kl` 的 `extract_M` 等路径使用。^[kl-polynomial-table.md:46-48]

## 算术与证据边界

系数使用普通的、未检查溢出的 `i32` 算术，移位中的 `index + d` 下标计算也没有独立的溢出检查，模块不提供 `ArithmeticOverflow` 错误通道。因此，“运算不返回失败”不等于具备溢出安全保证；这种实现是否可接受取决于系数上界，来源未作结论。^[kl-polynomial-table.md:25-27, kl-polynomial-table.md:50-64]

现有四个测试覆盖池种子序号、`shift` 展开、$q=-1$ 的交错和以及 `sub_shifted` 的一个单项例子。`add`、`sub`、`add_shifted`、`scaled`、`divide_by_2` 的错误分支和 `quotient_by_1_plus_q` 等仍存在测试覆盖缺口。^[kl-polynomial-table.md:118-123]

本页依据结构性源码阅读，不构成 KLV 数学正确性验收。来源未执行构建、测试或原版运行，也不提供性能或并行结论；其中上游行号转述自源码注释，未独立核对上游，可能随版本漂移。相关验收应以独立的 [[HPC 验收证据链]] 为准。^[kl-polynomial-table.md:9-17, kl-polynomial-table.md:130-135]

## Sources

- [kl-polynomial-table.md](kl-polynomial-table.md) — KLV 多项式的存储与逐列计算。
