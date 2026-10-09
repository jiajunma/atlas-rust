---
title: 全形变中的 Split 因子与后代保留
summary: scale-zero 基底保留全部 final 项，子项经缩放、修正与块查找后以 c·(1-s) 递归，避免单个 K 型替代导致 Split 因子及后代丢失。
sources:
  - atlas-core-deformation-cache.md
kind: concept
createdAt: "2026-10-09T14:27:37.717Z"
updatedAt: "2026-10-09T22:14:09.177Z"
tags:
  - 形变计算
  - Split系数
aliases:
  - 全形变中的-split-因子与后代保留
  - 全S因
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 全形变中的 Split 因子与后代保留
summary: 普通全形变在 scale-zero 基底保留全部 final 项，并以 c(1-s) 因子递归处理每个子项，避免丢失 Split 因子及后代贡献。
sources:
  - atlas-core-deformation-cache.md
kind: concept
tags:
  - 形变计算
  - K型
  - 分裂整数
---

# 全形变中的 Split 因子与后代保留

普通全形变必须保留形变子项的 Split 因子及其后代贡献。`full_deformation_uncached` 在 scale-zero 基底保留全部 final 项，并以 \(c(1-s)\) 因子递归处理子项；若停在前一个可约点并将其变成单个 K 型，就会丢弃 Split 因子与后代。^[atlas-core-deformation-cache.md:36-40]

## 递推关系

普通全形变遍历每个可约点，采用递推式 \(F(z)=L(z)+\sum_t c_t(1-s)F(t)\)。由于 \((1-s)^2=2(1-s)\)，来源将其对应于原版整数递推 \(D(z)\mathrel{+}=c_tL(t)+2c_tD(t)\)。这里的 \(1-s\) 因子须随递归贡献保留，相关系数表示见 [[SplitInteger 分裂整数系数]]。^[atlas-core-deformation-cache.md:29-40]

## 基底、子项与结果聚合

`full_deformation_uncached` 的 scale-zero 基底保留**全部 final 项**，对应 `deformation_unit::set_LKTs`。对子项的处理顺序是 `scale` → `deform_readjust` → `rep lookup` → `common_deformation_terms`；其中 `common_deformation_terms` 使用块修饰符、原始行与 `gamma`，随后以 \(c(1-s)\) 因子继续递归。完整执行这条链条，是保留 Split 因子与后代贡献的关键。^[atlas-core-deformation-cache.md:36-40]

顶层 `compute_full_deform` 对输入参数的每个 final 组分分别计算形变，按组分系数缩放后合并，再按 canonical `KTypePol` 项序排序。相关处理见 [[全形变参数规范化与结果聚合]]。^[atlas-core-deformation-cache.md:41-42]

## 完整结果的缓存纪律

`full_deformation_terms` 在分母超过 alcove 界时先调用 `domain_alcove_center` 取中，然后依次查缓存、加入 active 集、递归、排序并缓存。缓存键 `FullDeformKey = (x, y_bits, gamma)` 标识 canonical 单元，而非顶层参数；只缓存完整且按 canonical 顺序排列的结果。^[atlas-core-deformation-cache.md:19-25, atlas-core-deformation-cache.md:34-35]

[[全形变缓存与递归环检测]] 通过 `active: HashSet<FullDeformKey>` 显式检测递归环。缓存互斥锁只在读写瞬间持有，缓存与块属主均不跨递归调用持锁。[[形变计算的协作式截止]] 在阶段间检查期限，超限返回 `None`，不缓存部分结果。^[atlas-core-deformation-cache.md:20-27]

## 证据边界

本页依据 `domain_builtins.rs` 形变机器的结构性阅读，不构成数学验收。来源中的上游行号引用属于实现方的移植陈述；形变兼容性仍以 HPC 差分门为准，材料列出的普通全形变弧门号为 `3845838/3845872`。相关验收要求参见 [[HPC 验收证据链]]。^[atlas-core-deformation-cache.md:9-15, atlas-core-deformation-cache.md:62-64]

## Sources

- [atlas-core-deformation-cache.md](../../sources/atlas-core-deformation-cache.md) — 形变缓存机器：普通与扭曲全形变的递归、缓存纪律与协作截止。
