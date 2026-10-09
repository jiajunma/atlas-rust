---
title: twisted KL 和的长度函数选择
summary: twisted_kl_sum 与 twisted_kl_column_at_s 均计算 q=s 处的交错 twisted KL 和，但符号分别由 extended block 和 parent block 的长度函数决定。
sources:
  - deformation-drivers.md
kind: concept
createdAt: "2026-10-09T14:44:36.411Z"
updatedAt: "2026-10-09T14:44:36.411Z"
tags:
  - KL 多项式
  - 长度函数
  - 符号约定
aliases:
  - twisted-kl-和的长度函数选择
  - TK和
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# twisted KL 和的长度函数选择

twisted KL 和在 \(q=s\) 处计算交错和时，必须区分 extended block 与 parent block 的长度函数。`twisted_kl_sum` 使用 extended block 自身的长度；`twisted_kl_column_at_s` 使用对应 parent block 元素的长度。这是两个接口的关键差异。^[deformation-drivers.md:95-103]

## 两个接口的约定

`twisted_kl_sum` 对应上游自由函数 `twisted_KL_sum`（`repr.cpp:2304-2350`）。输入 `y` 使用 EXTENDED-block 编号，计算该元素在 \(q=s\) 处的交错 twisted KL 列和，符号依据 `eblock.length`。^[deformation-drivers.md:97-99]

`twisted_kl_column_at_s` 对应上游 `Rep_table::twisted_KL_column_at_s`（`repr.cpp:2371-2423`）。它计算 PARENT 块元素 `y0` 所对应参数的交错和，符号依据 `parent.length(eblock.z(x))`：先将 extended block 元素 `x` 映射到 parent block，再读取父块长度。相关编号关系见 [[扩展块与父块的索引映射]]。^[deformation-drivers.md:100-103]

## 父块与参数重构

twisted KL 和通过 `KlSumParent<'a>` 读取父 common block。`Full` 变体借用 `BlockGraph`，并使用调用方提供的常量 `lambda_rho`；`Partial` 变体借用 `PartialBlock`，携带可选的 `BlockModifier`，逐行重构各自的 `lambda_rho`。因此，理解父块长度的使用位置时，也需保留完整块与部分公共块的参数重构区别，参见 [[形变计算的父块抽象]]。^[deformation-drivers.md:83-93]

## 奇异集的适用条件

长度函数之外，奇异轨道的计算也受 block modifier 约定约束。在平凡 modifier 下，`simple_singular_flags` 对应 `common_block::singular(gamma)`，`singular_orbits_at` 给出 extended block 在 `gamma` 处的奇异轨道标志；partial parent 则折叠其子系统生成元的 singular set。上游按 modifier 索引的计算与 plain simple-coroot singular set 的一致性，以 `bm` 平凡为条件。参见 [[BlockModifier 块修正子]]。^[deformation-drivers.md:40-43, deformation-drivers.md:77-81]

## 证据边界

上述区别来自对 `deform.rs` 的结构性阅读及维护者逐条源码核对。来源中的上游位置转述自源码注释，未独立重读上游，行号可能随版本变化；本来源未执行构建、测试或原版运行，不构成数学验收、性能或并行结论。^[deformation-drivers.md:9-16, deformation-drivers.md:138-151]

## Sources

- [deformation-drivers.md](deformation-drivers.md) — 形变驱动：twisted 与 block 形变。
