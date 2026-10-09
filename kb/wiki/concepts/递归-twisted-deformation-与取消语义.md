---
title: 递归 twisted deformation 与取消语义
summary: 递归驱动无记忆化地计算 final、delta-fixed 参数的 K 型分裂系数项与净翻转，rank-0 跳过 lookup，取消时返回 Ok(None) 且不发布部分结果。
sources:
  - deformation-drivers.md
kind: concept
createdAt: "2026-10-09T14:45:09.699Z"
updatedAt: "2026-10-09T19:28:11.438Z"
tags:
  - 扭曲形变
  - 递归计算
  - 取消语义
aliases:
  - 递归-twisted-deformation-与取消语义
  - 递TD与
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 递归 twisted deformation 与取消语义

`twisted_deformation` 对 final、delta-fixed 的标准参数 `z`，沿上下文 `ctx` 的 delta 计算完整递归 twisted deformation。它移植自 `Rep_table::twisted_deformation`，不包含记忆化，返回 `(KType, SplitInteger)` 项及 net flip。可取消变体在递归或块级操作之间检查取消请求，取消时不发布部分多项式。^[deformation-drivers.md:124-134]

## 递归结果与系数

来源采用公式 $D(z)=\sum c\,(L(t)+2D(t))$ 与 $F(z)=L(z)+(1-s)D(z)$。结果中的 [[SplitInteger 分裂整数系数]] 表示 $a+bs$，其中 `Split(c, -c)` 表示 $c(1-s)$。其两个字段均为 `i32`，算术遵循上游的环绕语义，溢出防护不属于该类型的职责。^[deformation-drivers.md:16-16, deformation-drivers.md:50-59]

返回的 net flip 记录 shrink-wrap 到最后一个可约点的效果；没有 shrink-wrap 时，`flip == false`。冻结移植契约规定，当 `gamma.denominator() > 2^rank` 时，先应用 `weyl::alcove_center` 收缩，再进入递归 twisted deformation，相关主题见 [[Alcove 分母界守卫]]。^[deformation-drivers.md:44-45, deformation-drivers.md:126-130]

## 可约点查找与父块生命周期

在 INTEGRAL `gamma` 的可约点参数处，`lookup` 承担上游 `rt.lookup(zi, index, bm)` 与 `block.extended_block(bm, ...)` 的职责。rank-0 integral 子系统对应长度为零的单例 common block `{p}`；它不贡献形变项，也不调用 `lookup`。^[deformation-drivers.md:68-70, deformation-drivers.md:130-132]

递归查找返回拥有所有权的 `DeformParent`：完整块分支保存 `Box<BlockGraph>` 与 `Weight`，部分块分支保存 `Arc<PartialBlock>` 与 `BlockModifier`。驱动通过 `as_kl_sum_parent` 获得借用视图，并必须在 `twisted_deformation_terms` 借用该视图期间保持所选父块存活，详见 [[形变计算的父块抽象]]。^[deformation-drivers.md:85-93]

完整块视图使用调用方提供的常量 `lambda_rho`；部分块视图则逐行重构各自的 `lambda_rho`。完整块中不同元素的该值实际可能变化，因此调用方必须确保相关形变项共享所传值。对于 PROPER integral 子系统，部分块每行使用自己的 stored `gamma_lambda` 与查找所得的 block modifier 重构参数。^[deformation-drivers.md:29-37, deformation-drivers.md:85-89]

## 重算与 flip 处理

该移植以朴素重算替代上游 `Rep_table` 的 `deformation_unit`／`alcove_hash` 记忆化。由于每个结果直接针对正在报告其 flip 的参数计算，无共享池时不需要上游 memo-hit 的 flip 调整。这是 [[形变驱动的冻结移植契约]] 中明确记录的简化。^[deformation-drivers.md:46-48]

## 取消语义

`twisted_deformation_with_cancel` 在每次递归或块级操作之间检查取消 probe。取消时返回 `Ok(None)`，且不发布部分多项式。该接口的取消检查位置是操作之间。^[deformation-drivers.md:132-134]

这些检查点说明不足以保证单个块级操作能够即时中断，也不能据此推导取消响应时延上限。

## 证据范围

本页依据 `deform.rs` 的结构性阅读材料。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；形变计算的正确性属于独立的 [[HPC 验收证据链]]。材料中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而变化。^[deformation-drivers.md:9-16, deformation-drivers.md:138-146]

## Sources

- [deformation-drivers.md](deformation-drivers.md) — 形变驱动：twisted 与 block 形变。
