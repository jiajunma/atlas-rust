---
title: Alcove 修正的负对合不动子空间校验歧义
summary: 来源更正确认检查为 (I+θ)Δ=0，即 θΔ=−Δ；此前 θΔ=0 的描述漏读累加初值，原有歧义已消除，实现与错误命名一致。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-10T00:13:31.175Z"
updatedAt: "2026-10-10T01:44:02.831Z"
tags:
  - Alcove几何
  - 对合
  - 证据更正
aliases:
  - alcove-修正的负对合不动子空间校验歧义
confidence: 1
provenanceState: extracted
contradictedBy:
  - slug: alcove-修正的负对合不动子空间校验歧义
    reason: 索引保留的旧描述将检查写为 θΔ=0；来源于 2026-10-10 明确更正为 (I+θ)Δ=0。
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Alcove 修正的负对合不动子空间校验歧义

`alcove_center` 的“−θ 不动子空间校验”实际验证 $(I+\theta)\Delta=0$，其中 $\Delta=\mathrm{centered\_gamma}-\gamma$。来源此前将其误述为 $\theta\Delta=0$，原因是漏读了 `try_fold` 的非零初始项。2026-10-10 的更正记录确认：实现与错误命名一致，这一歧义来自阅读转述，并非该检查的实现缺陷。^[alcove.md:84-96]

## 实际判定条件

检查通过 `rc.theta(z)?` 获取对合，随后逐行累加。第 $i$ 行的 `try_fold` 初始值为 `difference.numerator()[i]`，再累加各项 $\theta_{ij}\operatorname{num}(\Delta)_j$，因此完整结果为 $\operatorname{num}(\Delta)_i+\sum_j\theta_{ij}\operatorname{num}(\Delta)_j$，即 $((I+\theta)\operatorname{num}(\Delta))_i$，而不是单独的矩阵乘积。^[alcove.md:84-88]

由于分母为正，分子上的零值条件等价于 $(I+\theta)\Delta=0$，也就是 $\theta\Delta=-\Delta$。因此，修正量属于 θ 的 $(-1)$-特征空间，恰为 −θ 的不动子空间；参数的连续坐标（环面因子侧）位于该空间，居中修正不得离开它。^[alcove.md:89-93]

## 在重心计算中的位置

[[Alcove 重心计算与标准参数重建]]先根据墙方程与 radical basis 约束求解重心，再通分构造 `centered_gamma`。负对合不动子空间校验发生在这些步骤之后、标准参数重建之前。^[alcove.md:70-97]

若修正量不满足条件，函数返回 `RepInvariantViolation`，错误文本为 `"alcove correction lies outside the -theta fixed subspace"`；通过检查后，调用 `rc.sr_gamma(z.x(), &lambda_rho, &centered_gamma)` 重建参数，保留 KGB 坐标与 `lambda_rho`，并保持打包扭数据与派生高度的规范性。^[alcove.md:65-66, alcove.md:84-97]

## 歧义的来源与更正

先前描述省略了折叠累加的初始项，将“差值本身加上 θ 对差值的作用”误读成“θ 对差值的作用”。更正的关键是把初始项纳入完整表达式；这里的初始值直接决定算子是 $I+\theta$。这一案例体现了 [[折叠累加初值的算法语义]]：阅读 fold 时，初始值也是算法语义的一部分。^[alcove.md:87-96]

## 证据边界

上述更正依据维护者对照源码的核对，消解了这一具体检查的转述歧义；来源整体仍属于结构性阅读，不构成 alcove 计算的数学验收。关联的上游位置 `alcoves.cpp:317-321` 来自代码注释，未独立核对上游字节。^[alcove.md:9-13, alcove.md:84-96]

`alcove_center` 端到端没有单元测试，来源也未覆盖 `RepContext` 等外部类型的完整契约；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。相关限制见 [[Alcove 算法的测试覆盖与失败边界]]。^[alcove.md:181-184, alcove.md:193-198]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。
