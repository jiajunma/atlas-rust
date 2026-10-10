---
title: Split 系数算术与零因子筛选
summary: SplitValue 以整数对表示 e+f·s（s²=1），使用机器位宽回绕算术，split_keeps 按零因子标量的湮灭点筛选项。
sources:
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T14:30:33.263Z"
updatedAt: "2026-10-10T00:19:22.378Z"
tags:
  - Split系数
  - 整数算术
  - 零因子
aliases:
  - split-系数算术与零因子筛选
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Split 系数算术与零因子筛选
summary: SplitValue 用整数对表示 e+f·s（s²=1），算术采用机器位宽回绕；split_keeps 按零因子标量的湮灭点筛选项，其余标量保留所有项。
sources:
  - atlas-core-domain-values.md
kind: concept
tags:
  - 系数环
  - 零因子
  - 算术语义
aliases:
  - split-系数算术与零因子筛选
---

# Split 系数算术与零因子筛选

`SplitValue` 表示形如 $e+f\,s$、满足 $s^2=1$ 的系数，以整数对 $(e,f)$ 存储。其算术遵循上游 `Split_integer` 的机器位宽回绕语义，相关表示见 [[SplitInteger 分裂整数系数]]。^[atlas-core-domain-values.md:59-61]

## 算术与打印约定

机器位宽回绕是该系数类型的算术契约，来源将其对应到上游 `arithmetic.h:152-213`。打印形式为 `(e±|f|s)`，其中 $f$ 的符号折入分隔符，对应上游 `io/basic_io.cpp:150-154`。^[atlas-core-domain-values.md:59-61]

## 零因子筛选

`split_keeps` 根据标量筛选项：对于 $1\mp s$ 的倍数这类零因子标量，删除在湮灭点取值为零的项；其余标量保留所有项。来源将这一规则对应到上游 `atlas-types.w:5868-5900`。^[atlas-core-domain-values.md:62-63]

## 多项式中的系数

`KTypePolValue` 与 `ParamPolValue` 分别保存同一实形上的有序 `(Split, KType)` 项和 `(Split, StandardRepr)` 项。同类项合并，零系数项删除，对应上游 `SR_poly::add_term` 的项维护规则。^[atlas-core-domain-values.md:64-67]

`assign_polynomial_coefficient` 只替换精确 final 键的系数，不累加，也不展开非 final 键。即使写入零或写入结果被丢弃，也执行校验；不兼容的实形属主会在目标项被触碰之前遭到拒绝。相关约束见 [[实形属主约束下的多项式系数契约]]。^[atlas-core-domain-values.md:99-102]

`loop_terms` 借用规范的非零项序，产出的键保留其实形属主，不复制多项式，也不以整数键替代领域键。参见 [[容器迭代的借用与规范项序]]。^[atlas-core-domain-values.md:106-107]

## 证据边界

本页依据来源对 `domain_builtins.rs` 上部区域的结构性阅读。来源明确排除了派发表与测试，并将提及的 A1 限定 HPC 语义验收关联到 Weyl owner/dual 修复；该材料未提供 Split 算术或零因子筛选的独立验收结果。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:119-126]

## Sources

- [atlas-core-domain-values.md](../../sources/atlas-core-domain-values.md) — 领域值与 Weyl 身份（domain_builtins.rs 上部）。
