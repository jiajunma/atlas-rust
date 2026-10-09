---
title: 全形变中的 Split 因子与后代保留
summary: scale-zero 基底保留全部 final 项，子项经 scale、deform_readjust、表示查找及 common_deformation_terms 后递归，避免压成单个 K 型而丢失 Split 因子与后代。
sources:
  - atlas-core-deformation-cache.md
kind: concept
createdAt: "2026-10-09T14:27:37.717Z"
updatedAt: "2026-10-09T14:27:37.717Z"
tags:
  - 全形变
  - Split系数
  - 算法正确性
aliases:
  - 全形变中的-split-因子与后代保留
  - 全S因
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 全形变中的 Split 因子与后代保留

普通全形变通过递归保留每个形变子项的 Split 因子及其后代。若在前一个可约点停止，并把该项直接变成单个 K 型，就会丢弃它的 Split 因子与后续递归贡献。^[atlas-core-deformation-cache.md:31-40]

## 递推关系

普通全形变遍历每个可约点，使用递推式
\[
F(z)=L(z)+\sum_t c_t(1-s)F(t).
\]
由于 \((1-s)^2=2(1-s)\)，这一写法与原版整数递推 \(D(z)\mathrel{+}=c_tL(t)+2c_tD(t)\) 等价。因子 \(1-s\) 因而参与后代贡献的递归累积；相关系数表示可参见 [[SplitInteger 分裂整数系数]]。^[atlas-core-deformation-cache.md:31-33]

## 基底与后代的保留

`full_deformation_uncached` 在 scale-zero 基底中保留全部 final 项，对应 `deformation_unit::set_LKTs`。处理每个子项时，依次执行 `scale`、`deform_readjust`、`rep lookup`，再以块修饰符、原始行和 `gamma` 调用 `common_deformation_terms`，最后带着 \(c(1-s)\) 因子继续递归。这条处理链保留的是子项的完整形变贡献。^[atlas-core-deformation-cache.md:36-40]

顶层 `compute_full_deform` 对参数的每个 final 组分分别形变，按该组分的系数缩放后合并，并按 canonical `KTypePol` 项序排序。这一聚合步骤与递归中的后代保留共同构成 [[全形变参数规范化与结果聚合]] 的处理过程。^[atlas-core-deformation-cache.md:41-42]

## 缓存与截止纪律

`full_deformation_terms` 在分母超过 alcove 界时先调用 `domain_alcove_center` 取中，再依次查缓存、加入 active 集、递归、排序并缓存。缓存键 `FullDeformKey = (x, y_bits, gamma)` 标识 canonical 单元，而非顶层参数；缓存只接收完整且按 canonical 项序排序的结果。^[atlas-core-deformation-cache.md:19-25, atlas-core-deformation-cache.md:34-35]

[[全形变缓存与递归环检测]] 使用 active 集显式检测递归环，缓存互斥锁仅在读写瞬间持有，不跨递归调用持锁。[[形变计算的协作式截止]] 在阶段间检查期限，超限返回 `None`，不缓存部分结果。^[atlas-core-deformation-cache.md:20-27]

## 证据边界

上述说明来自对形变机器的结构性阅读，不构成数学验收。源材料中的上游行号引用属于实现方的移植陈述；形变兼容性仍以 HPC 差分门为准，普通全形变弧对应门号为 `3845838/3845872`。相关验收范围参见 [[形变兼容性的证据边界]]。^[atlas-core-deformation-cache.md:9-15, atlas-core-deformation-cache.md:62-64]

## Sources

- [atlas-core-deformation-cache.md](atlas-core-deformation-cache.md)
