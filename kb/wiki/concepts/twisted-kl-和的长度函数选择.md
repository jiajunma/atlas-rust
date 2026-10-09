---
title: twisted KL 和的长度函数选择
summary: twisted_kl_sum 与 twisted_kl_column_at_s 均在 q=s 处求交错和，但符号分别依据扩展块长度与父块长度计算。
sources:
  - deformation-drivers.md
kind: concept
createdAt: "2026-10-09T14:44:36.411Z"
updatedAt: "2026-10-09T19:27:27.997Z"
tags:
  - 扭曲KL
  - 长度函数
aliases:
  - twisted-kl-和的长度函数选择
  - TK和
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# twisted KL 和的长度函数选择

twisted KL 和在 \(q=s\) 处计算交错和时，两个接口采用不同的长度函数：`twisted_kl_sum` 的符号来自 extended block 自身的长度，`twisted_kl_column_at_s` 的符号来自对应 parent block 元素的长度。这是两者的关键差异。^[deformation-drivers.md:95-103]

## 两个接口的编号与符号约定

`twisted_kl_sum` 对应上游自由函数 `twisted_KL_sum`（`repr.cpp:2304-2350`），对 extended block 元素 `y` 计算 \(q=s\) 处的交错 twisted KL 列和。这里 `y` 使用扩展块编号，符号依据 `eblock.length`。^[deformation-drivers.md:97-99]

`twisted_kl_column_at_s` 对应上游 `Rep_table::twisted_KL_column_at_s`（`repr.cpp:2371-2423`），计算 parent block 元素 `y0` 所对应参数的交错和。符号依据 `parent.length(eblock.z(x))`：先通过 `eblock.z(x)` 将扩展块元素映射到父块，再读取父块长度。相关编号关系见 [[扩展块与父块的索引映射]]。^[deformation-drivers.md:100-103]

## 父块与参数重构

twisted KL 和通过 `KlSumParent<'a>` 读取父 common block。`Full` 变体借用 `BlockGraph` 和调用方提供的常量 `lambda_rho`；`Partial` 变体借用 `PartialBlock`，携带可选的 `BlockModifier`，逐行重构各自的 `lambda_rho`。两种父块视图的具体职责见 [[形变计算的父块抽象]]。^[deformation-drivers.md:85-93]

完整块上的逐元素 `lambda_rho` 实际可能变化，因此使用调用方一次性提供的常量时，调用方必须保证所有形变项共享该值。真积分子系统的 partial parent 则按每行存储的 `gamma_lambda` 与 lookup 得到的 block modifier 重构参数。^[deformation-drivers.md:29-37]

## 奇异轨道的适用条件

除长度函数外，奇异轨道计算还受 [[BlockModifier 块修正子]] 的约定约束。上游按 modifier 索引的 singular-orbit 计算，在 `bm` 平凡时与 `ExtBlock::singular_orbits` 使用的 plain simple-coroot singular set 一致；此时 `bm.simp_int` 是按恒等下标排列的简单根列表，`bm.simple_pi` 是恒等置换。^[deformation-drivers.md:40-43]

`simple_singular_flags` 对应平凡 modifier 下的 `common_block::singular(gamma)`；`singular_orbits_at` 给出扩展块在 `gamma` 处的奇异轨道标志。partial parent 同样通过折叠其子系统生成元的 singular set 得到相应标志。^[deformation-drivers.md:77-81]

## 证据边界

上述接口区别来自对 `deform.rs` 的结构性阅读及维护者逐条源码核对。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移；来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。形变计算的正确性另属其 [[HPC 验收证据链]]。^[deformation-drivers.md:9-16, deformation-drivers.md:138-151]

## Sources

- [deformation-drivers.md](deformation-drivers.md) — 形变驱动：twisted 与 block 形变。
