---
title: 形变缓存机器（domain_builtins.rs 2657–2933）——full/twisted full deformation 的递归、缓存纪律与协作截止
source: atlas-rust/atlas-core-deformation-cache
ingestedAt: 2026-10-09T16:00:00Z
---

# 形变缓存机器（domain_builtins.rs 2657–2933）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/domain_builtins.rs` 的形变机器：
`full_deform_key`/`cached_deformation`/`store_deformation`/
`full_deformation_terms`/`full_deformation_uncached`/`compute_full_deform`/
`compute_twisted_full_deform`，以及 `transpose_matrix`/`matrix_value`/
`cramer_solution`（2933–3022）。这是 ordinary-full-deform 弧的落点。
结构性阅读，不声称数学验收。

## 键与缓存纪律

- `FullDeformKey = (x, y_bits, gamma)`——canonical 单元（不是顶层参数键）。
- `cached_deformation`/`store_deformation`：`Mutex<DeformationCache>` 的
  锁**只在读/写瞬间**持有——绝不在递归调用间持锁（缓存或块属主都不
  跨递归锁定）。
- 只缓存**完整、canonical 排序**的结果；`active: HashSet<FullDeformKey>`
  显式检测递归环（"full deformation recursion revisited an active
  parameter"）。
- `deadline_expired` 在阶段间检查（协作式截止）；超限返回 `None`
  （不缓存部分结果）。

## 普通全形变（repr.cpp 的 `Rep_table::deformation`/`full_deformation`）

- 递推：`F(z) = L(z) + Σ_t c_t·(1-s)·F(t)`，遍历**每个**可约点；
  等于原版整数递推 `D(z) += c_t·L(t) + 2·c_t·D(t)`，因为
  `(1-s)² = 2(1-s)`。
- `full_deformation_terms`：分母超 alcove 界时先 `domain_alcove_center`
  取中；查缓存 → 入 active 集 → 递归 → 排序 → 缓存。
- `full_deformation_uncached`：scale-zero 基底保留**全部** final 项
  （`deformation_unit::set_LKTs`）；每个子项：`scale` →
  `deform_readjust` → `rep lookup` → `common_deformation_terms`
  （块修饰符 + 原始行 + gamma）→ 以 `c·(1-s)` 因子递归——**停在前一点
  并把它变成单个 K 型会丢弃其 Split 因子与后代**（Split 因子修复课）。
- `compute_full_deform`：对参数的每个 final 组分形变、按其系数缩放、
  合并、按 canonical `KTypePol` 项序排序。

## 扭曲全形变

`compute_twisted_full_deform`：`distinguished_twist` → `ExtRepContext` →
`extended_finalise`；**计时在 extended_finalise 之后开始**（axis.w:
8303-8308——setup 成本在协作截止之外）；`twisted_reducibility_lookup` +
`twisted_deformation_with_cancel`；finalise 翻转不同则系数为 `s`，否则
为 `1`；逐项合并排序。

## 矩阵辅助（2933–3022）

`transpose_matrix`（行主序转置）、`matrix_value`（行→Matrix 值）、
`cramer_solution`（精确 Cramer 解：分数自由变量消元）。

## 边界与限制

- `common_deformation_terms` 与块机器在派生臂区域；`RepContext`/
  `ExtRepContext` 在 `atlas-real-group`（各有包）；K 型/参数的系数
  契约见 [领域值包](atlas-core-domain-values.md)。
- 上游行号引用是**实现方移植陈述**；形变兼容以 HPC 差分门为准
  （普通全形变弧的 3845838/3845872 门，见根 AGENTS.md 的
  "Ordinary full deformation" 条目）。
- 字节数/哈希只标识本快照字节（git base `964f0033`）。
