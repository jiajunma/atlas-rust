---
title: Split 系数算术与零因子筛选
summary: SplitValue 用整数对表示 e+f·s（s²=1），算术遵循机器位宽回绕；split_keeps 按零因子标量的湮灭点筛选项，其余标量保留所有项。
sources:
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T14:30:33.263Z"
updatedAt: "2026-10-09T14:30:33.263Z"
tags:
  - 系数环
  - 零因子
  - 算术语义
aliases:
  - split-系数算术与零因子筛选
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Split 系数算术与零因子筛选

`SplitValue` 表示形如 \(e+f\,s\)、满足 \(s^2=1\) 的系数，以二元组 \((e,f)\) 存储。其算术遵循上游 `Split_integer` 的机器位宽回绕语义。相关表示见 [[SplitInteger 分裂整数系数]]。^[atlas-core-domain-values.md:59-61]

## 算术与打印约定

机器位宽回绕是该系数类型的算术契约。源码包将这一行为对应到上游 `arithmetic.h:152-213`；打印形式为 `(e±|f|s)`，其中 \(f\) 的符号由分隔符表达，对应 `io/basic_io.cpp:150-154`。^[atlas-core-domain-values.md:59-61]

## 零因子筛选

`split_keeps` 根据标量是否为零因子决定保留哪些项。对于 \(1\mp s\) 的倍数这类零因子标量，它会删除在湮灭点取值为零的项；其余标量保留所有项。源码包将这一规则对应到上游 `atlas-types.w:5868-5900`。^[atlas-core-domain-values.md:62-63]

## 多项式中的系数

`KTypePolValue` 与 `ParamPolValue` 分别保存同一实形上的有序 `(Split, KType)` 项和 `(Split, StandardRepr)` 项。同类项合并，零系数项删除；这是多项式项维护的规则，与 `split_keeps` 根据零因子标量筛选项的规则分别描述。^[atlas-core-domain-values.md:62-67]

系数赋值只替换精确 final 键的系数，不累加，也不展开非 final 键。即使写入零或结果被丢弃，也执行校验，并在触碰目标项之前拒绝不兼容的实形属主。相关约束见 [[实形属主约束下的多项式系数契约]]。^[atlas-core-domain-values.md:99-102]

## 证据边界

本页依据源码包对 `domain_builtins.rs` 上部区域的结构性阅读。该包未覆盖派发表与测试；其中提及的 A1 限定 HPC 语义验收针对 Weyl owner/dual 修复，不能据此认定 Split 算术或零因子筛选已获得独立验收。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:119-126]

## Sources

- [atlas-core-domain-values.md](../../sources/atlas-core-domain-values.md) — 领域值与 Weyl 身份（domain_builtins.rs 上部）。
