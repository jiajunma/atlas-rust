---
title: 递归 twisted deformation 与取消语义
summary: twisted_deformation 无记忆化地递归处理 final、delta-fixed 参数，返回 KType 分裂系数项与 net flip；rank-0 不调用 lookup，可取消变体返回 Ok(None) 且不发布部分多项式。
sources:
  - deformation-drivers.md
kind: concept
createdAt: "2026-10-09T14:45:09.699Z"
updatedAt: "2026-10-09T14:45:09.699Z"
tags:
  - 递归算法
  - 形变计算
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 递归 twisted deformation 与取消语义

`twisted_deformation` 对 final、delta-fixed 的标准参数 `z`，沿上下文 `ctx` 的 delta 计算完整递归 twisted deformation。它移植自 `Rep_table::twisted_deformation`，返回 `(KType, SplitInteger)` 项及一个记录 shrink-wrap 效果的 net flip；可取消变体则允许在递归或块级操作之间终止计算，且不发布部分多项式。^[deformation-drivers.md:124-134]

## 递归结果与系数

来源沿用数学记号
$D(z)=\sum c\,(L(t)+2D(t))$、
$F(z)=L(z)+(1-s)D(z)$。结果中的 [[SplitInteger 分裂整数系数]] 表示 $a+bs$；形变项系数 `c` 可转换为 `Split(c, -c)`，即 $c(1-s)$。该类型的字段为 `i32`，算术采用环绕语义，不承担溢出防护。^[deformation-drivers.md:16-16, deformation-drivers.md:50-59, deformation-drivers.md:107-111]

返回的 net flip 记录 shrink-wrap 到最后一个 reducibility point 的效果；没有 shrink-wrap 时，`flip == false`。冻结移植契约还规定：当 `gamma.denominator() > 2^rank` 时，先应用 `weyl::alcove_center` 收缩，再进入递归 twisted deformation。^[deformation-drivers.md:44-45, deformation-drivers.md:126-130]

## 可约点查找与父块生命周期

在 INTEGRAL `gamma` 的 reducibility-point 参数处，`lookup` 承担上游 `rt.lookup(zi, index, bm)` 与扩展块构造的职责。rank-0 integral 子系统对应长度为零的单例 common block；它不贡献形变项，也不调用 `lookup`。^[deformation-drivers.md:68-70, deformation-drivers.md:130-132]

递归查找返回拥有所有权的 `DeformParent`：完整块分支保存 `Box<BlockGraph>` 与 `Weight`，部分块分支保存 `Arc<PartialBlock>` 与 `BlockModifier`。驱动通过 `as_kl_sum_parent` 获得借用视图，并必须在 `twisted_deformation_terms` 使用该视图期间保持所选父块存活，详见 [[形变计算的父块抽象]]。^[deformation-drivers.md:83-93]

完整块视图使用调用方提供的常量 `lambda_rho`；部分块视图按行重构各自的 `lambda_rho`。完整块中各元素的 `lambda_rho` 实际可能不同，因此调用方必须满足相关形变项共享所传值的限制，不能将常量假设无条件推广到所有完整块参数。^[deformation-drivers.md:29-37, deformation-drivers.md:85-89]

## 重算与 flip 处理

该移植不包含上游 `Rep_table` 的 `deformation_unit`／`alcove_hash` 记忆化，而采用朴素重算。由于每个结果直接为当前报告 flip 的参数计算，无共享池时不需要上游 memo-hit 的 flip 调整。这是 [[形变驱动的冻结移植契约]] 中明确记录的简化。^[deformation-drivers.md:46-48, deformation-drivers.md:126-129]

## 取消语义

`twisted_deformation_with_cancel` 在每次递归或块级操作之间检查取消 probe。取消时返回 `Ok(None)`，且不发布部分多项式；调用方应将这一返回值理解为计算被取消，而非已经获得完整结果。^[deformation-drivers.md:132-134]

取消检查点位于操作之间。来源没有给出单个块级操作内部的检查频率或取消响应时延保证，因此不能据此声称计算能够即时中断。^[deformation-drivers.md:132-134]

## 证据范围

本页依据的是 `deform.rs` 的结构性阅读材料，其快照记录了 dirty 工作区字节。来源未执行构建、测试或原版运行；形变正确性属于独立的 [[HPC 验收证据链]]，本文不构成数学验收、性能或并行行为结论。上游位置由源码注释转述，未独立重读上游，行号可能随版本变化。^[deformation-drivers.md:9-16, deformation-drivers.md:138-146]

## Sources

- [deformation-drivers.md](deformation-drivers.md) — 形变驱动：twisted 与 block 形变。
