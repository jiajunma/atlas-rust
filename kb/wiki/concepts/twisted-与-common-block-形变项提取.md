---
title: twisted 与 common-block 形变项提取
summary: twisted 路径为 final、delta-fixed 父块元素提取整数形变项并由 wrapper 转为 Split 系数，common 路径处理 lookup 返回的部分块。
sources:
  - deformation-drivers.md
kind: concept
createdAt: "2026-10-09T14:44:58.512Z"
updatedAt: "2026-10-09T20:50:22.165Z"
tags:
  - 形变计算
  - 部分公共块
aliases:
  - twisted-与-common-block-形变项提取
  - T与C形
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: twisted 与 common-block 形变项提取
summary: twisted_deformation_terms 为 final、delta-fixed 父块元素提取整数系数形变项，包装层将 c 转为 Split(c,-c)；common_deformation_terms 则沿 contributions 路径处理 lookup 返回的 partial block。
sources:
  - deformation-drivers.md
kind: concept
tags:
  - 形变项
  - 公共块
  - 系数转换
---

# twisted 与 common-block 形变项提取

`twisted_deformation_terms` 为 final、delta-fixed 父块元素提取 twisted 形变项；`common_deformation_terms` 对 `RepTable::lookup` 返回的 partial block 提取 common-block 形变项。两者分别对应上游 `repr.cpp:2426-2520` 与 `repr.cpp:1933-2025` 的计算路径。^[deformation-drivers.md:105-114]

## Twisted 形变项与系数转换

`twisted_deformation_terms` 的所述移植入口采用平凡的 [[BlockModifier 块修正子|block modifier]]。输入元素 `y` 必须是 final、delta-fixed，并使用 **PARENT 块编号**；输出为 `(StandardRepr, int)` 对，按 finals 的反向累积顺序（reverse-accumulated finals）返回。^[deformation-drivers.md:107-111]

包装层将每个整数系数 `c` 转为 `Split(c, -c)`，然后按 `SR_poly` 顺序排序。提取函数的返回顺序与包装层的多项式排序是两个不同约定。[[SplitInteger 分裂整数系数|SplitInteger]] 表示 \(a+bs\)，因此 `Split(c, -c)` 正是 \(c(1-s)\)。^[deformation-drivers.md:52-59, deformation-drivers.md:107-111]

来源采用的形变记号为 \(D(z)=\sum c\bigl(L(t)+2D(t)\bigr)\) 与 \(F(z)=L(z)+(1-s)D(z)\)。包装层的系数转换对应其中的 \(1-s\) 因子；`SplitInteger` 使用 `i32` 分量和 `wrapping_*` 算术，本类型不提供溢出防护。^[deformation-drivers.md:16-16, deformation-drivers.md:52-59]

## Common-block 路径与参数重构

`common_deformation_terms` 对 lookup 返回的 [[公共块的构造与元素编号（PartialBlock）|PartialBlock]] 求形变项，沿用 `contributions(block, block.singular(bm,gamma), y)` 路径，其中奇异集由 block modifier `bm` 与 `gamma` 参与确定。^[deformation-drivers.md:112-114]

在真积分子系统上，父块是 `PartialBlock`。每行参数通过该行存储的 `gamma_lambda` 与 lookup 返回的 block modifier，经 `RepContext::sr_with_modifier` 重构，对应上游 `common_block::sr`。^[deformation-drivers.md:34-37]

完整块路径则由调用方一次性提供 `lambda_rho`，要求所有形变项共享该值。这是实质性前提：完整块中逐元素的 `lambda_rho` 可以变化，例如 SL(2,R) 在 \(\gamma=2\rho\) 时，compact-Cartan 元素为 `[1]`，split 元素为 `[0]`。语言层使用 `rc.lambda_rho(p)`。^[deformation-drivers.md:29-33]

## 父块借用与奇异集

[[形变计算的父块抽象|KlSumParent]] 提供两种借用视图：`Full` 持有 `BlockGraph` 与调用方提供的常量 `lambda_rho`；`Partial` 持有 `PartialBlock` 与可选的 `BlockModifier`，按行重构自己的 `lambda_rho`。递归驱动使用拥有数据的 `DeformParent`，通过 `as_kl_sum_parent` 提供借用，并须在 `twisted_deformation_terms` 借用块视图期间保持父块存活。^[deformation-drivers.md:85-93]

平凡 block modifier 下，`simple_singular_flags` 对应 `common_block::singular(gamma)`，`singular_orbits_at` 给出 extended block 在 `gamma` 处的 singular-orbit flags；partial parent 则折叠其子系统生成元的 singular set。上游依赖 modifier 的奇异轨道计算与 plain simple-coroot singular set 的一致性，以 `bm` 平凡为条件，此时 `bm.simp_int` 按恒等下标排列，`bm.simple_pi` 为恒等置换。^[deformation-drivers.md:40-43, deformation-drivers.md:77-81]

## 积分子系统边界

[[积分块范围与奇异集|IntegralBlockScope]] 依据 coroot 与 `gamma` 是否整配对分类：等价判据是 `coroot · gamma.numerator()` 能被 `gamma.denominator()` 整除。无根整配对时，common block 是长度为零的单例 `{p}`，形变项为空；[[递归 twisted deformation 与取消语义|递归 twisted deformation]] 对这一情形也不调用 `lookup`。^[deformation-drivers.md:63-70, deformation-drivers.md:130-134]

来源在 `ProperSubsystem` 分类处要求：不支持 common-block 的完整块移植入口必须显式失败，不能默默改用完整块计算；这一限制需与另述的 partial-block 路径一同理解。来源给出的 A1 例子中，\(\nu=[1]/2\) 时完整块大小为 3，而积分块是单例，直接替用完整块会改变计算范围。^[deformation-drivers.md:34-39, deformation-drivers.md:73-75, deformation-drivers.md:112-114]

## 证据范围

本页依据 `deform.rs` 的结构性阅读，所记录字节来自 dirty 工作区快照。上游行号转述自源码注释，未独立重读上游；来源包未执行构建、测试或原版运行，不构成数学验收、性能或并行结论。形变计算的正确性属于独立的 HPC 证据链，本来源不重述或扩展其结论。^[deformation-drivers.md:9-16, deformation-drivers.md:138-146]

## Sources

- [deformation-drivers.md](../../sources/deformation-drivers.md) — 形变驱动：twisted 与 block 形变。
