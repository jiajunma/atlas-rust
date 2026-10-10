---
title: twisted KL 和的长度函数选择
summary: twisted_kl_sum 与 twisted_kl_column_at_s 均在 q=s 处求交错和，但符号分别依据扩展块和父块的长度函数。
sources:
  - deformation-drivers.md
kind: concept
createdAt: "2026-10-09T14:44:36.411Z"
updatedAt: "2026-10-10T00:30:15.921Z"
tags:
  - KL多项式
  - 扩展块
aliases:
  - twisted-kl-和的长度函数选择
  - TK和
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: twisted KL 和的长度函数选择
summary: twisted_kl_sum 与 twisted_kl_column_at_s 在 q=s 处计算交错和，符号分别依据扩展块长度与父块长度。
sources:
  - deformation-drivers.md
kind: concept
tags:
  - KL多项式
  - 长度函数
  - 符号约定
aliases:
  - twisted-kl-和的长度函数选择
  - TK和
provenanceState: extracted
---

# twisted KL 和的长度函数选择

在 \(q=s\) 处计算交错 twisted KL 和时，`twisted_kl_sum` 与 `twisted_kl_column_at_s` 的关键差异是**确定符号所用的长度函数**：前者使用扩展块自身的长度，后者使用父块的长度。^[deformation-drivers.md:95-103]

## 接口与编号约定

`twisted_kl_sum` 对应上游自由函数 `twisted_KL_sum`（`repr.cpp:2304-2350`），计算扩展块元素 `y` 在 \(q=s\) 处的交错 twisted KL 列和。输入 `y` 使用扩展块编号，符号取自 `eblock.length`。^[deformation-drivers.md:97-99]

`twisted_kl_column_at_s` 对应上游 `Rep_table::twisted_KL_column_at_s`（`repr.cpp:2371-2423`），计算父块元素 `y0` 所对应参数的交错和。符号取自 `parent.length(eblock.z(x))`：先通过 `eblock.z(x)` 将扩展块元素映射到父块，再读取父块长度。相关编号关系见 [[扩展块与父块的索引映射]]。^[deformation-drivers.md:100-103]

## 父块视图与参数重构

twisted KL 和通过 `KlSumParent<'a>` 读取父 common block。`Full` 变体借用 `BlockGraph` 与调用方提供的常量 `lambda_rho`；`Partial` 变体借用 `PartialBlock`，携带可选的 `BlockModifier`，并逐行按上游 `common_block::sr` 的方式重构各自的 `lambda_rho`。参见 [[形变计算的父块抽象]]。^[deformation-drivers.md:83-93]

完整块中各元素的 `lambda_rho` 可能不同，因此一次性提供该值的接口要求调用方传入所有形变项共享此值的参数。真积分子系统使用 `PartialBlock` 时，每行依据自身存储的 `gamma_lambda` 与 lookup 得到的 block modifier 重构参数。^[deformation-drivers.md:29-37]

## 奇异轨道约定

上游按 [[BlockModifier 块修正子|block modifier]] 索引的奇异轨道计算，在 `bm` 平凡时，与 `ExtBlock::singular_orbits` 使用普通单余根奇异集的计算一致。此时 `bm.simp_int` 是按恒等下标排列的简单根列表，`bm.simple_pi` 是恒等置换。^[deformation-drivers.md:40-43]

`simple_singular_flags` 对应平凡 modifier 下的 `common_block::singular(gamma)`；`singular_orbits_at` 给出扩展块在 `gamma` 处的奇异轨道标志。partial parent 同样通过折叠其子系统生成元的奇异集得到相应标志。^[deformation-drivers.md:77-81]

## 证据边界

两个 twisted KL 和的长度函数差异已由维护者对照源码核对。本页依据的是 `deform.rs` 的结构性阅读，所读字节属于快照记录的 dirty 工作区；形变计算正确性另属 ordinary-deform、G2、PSp4R 等 [[HPC 验收证据链]]，本来源不重述或扩展这些结果。^[deformation-drivers.md:9-16, deformation-drivers.md:147-151]

来源中的上游行号转述自源码注释，未独立重读上游文件，可能随版本演进而漂移。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[deformation-drivers.md:138-146]

## Sources

- [deformation-drivers.md](../../sources/deformation-drivers.md) — 形变驱动：twisted 与 block 形变。
