---
title: Cayley 回拉与模二 grading 求解
summary: 标签计算结合 Cayley 回拉翻转位、cross 根运送和带哨兵位的 ModTwoSubspace 求解恢复 fundamental 代表；根位置存在不可消余量时返回 ImpossibleGrading。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:04:50.826Z"
updatedAt: "2026-10-09T15:04:50.826Z"
tags:
  - Cayley变换
  - 模二线性代数
  - grading
aliases:
  - cayley-回拉与模二-grading-求解
  - C回G求
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cayley 回拉与模二 grading 求解

Cayley 回拉与模二 grading 求解是 `RealFormLabels` 使用的 grading-based legacy mechanism：它把某个 Cartan 的局部弱实类映射到 fundamental 分区中的全局实形式编号。`label(k)` 表示按 `classes()` 顺序排列的局部类 \(k\) 的全局标签，构成[[弱实形式的局部到全局标签映射]]的计算核心。^[real-form-labels-order.md:17-40]

## 构造前提

构造按顺序检查 datum 一致性、两侧 grading 所属的对合、分解的 distinguished 因子化，以及分区与 fiber 的来源一致性。fundamental grading 必须属于本 inner class 的 distinguished involution，Cartan grading 必须属于分解的合成对合；相应失败分别通过 `DatumMismatch`、`CartanFiberInvolutionMismatch`、`DistinguishedInvolutionMismatch` 或 `CartanFiberMismatch` 报告。^[real-form-labels-order.md:19-24]

## Cayley 回拉与根列表运送

首先，对 Cartan 侧每个虚单根计算翻转位：统计使 `root + alpha` 仍为根的 Cayley 根 `alpha`，个数为奇数时翻转该位。随后，将拼接根列表 `cartan_imaginary ++ cayley` 经 cross 作用运送；每个像都必须相对于 distinguished involution 为虚根，否则触发 `"fundamental imaginary"` 不变量违例。^[real-form-labels-order.md:28-31]

这些翻转位用于修正回拉后的 grading。求解右端先取“回拉 grading XOR 翻转位”，再接上 Cayley 块的全 noncompact 条件，从而把两类根上的 grading 要求放入同一个求解系统。^[real-form-labels-order.md:36-40]

## 基 grading 的扩展

`base_grading_extension` 将基本 grading 扩展到任意虚根：其值为该根在虚单根基下的坐标系数之和的奇偶。坐标通过精确求解转置的 bracket 索引子 Cartan 系统得到，其中第 \(j\) 行记录每个基根与余根 \(j\) 的配对。^[real-form-labels-order.md:32-35]

根类检查必须独立保留：非虚根的投影也可能具有整数坐标，因此坐标整性不能代替虚根性判定。实现显式门控根类，再使用这些坐标计算 grading。^[real-form-labels-order.md:33-35]

## 增广模二系统与标签恢复

求解器构造增广 `ModTwoSubspace`，坐标由 `list_len` 个根位置和与维数相同数量的哨兵位组成，用于表达 fundamental fiber 基代表与运送后根列表的关系。对每个局部类，将上述右端与扩展基 grading 求差，再调用 `quotient_representative` 进行归约；这一过程与[[F₂ 商空间的确定性代表元与陪集判定]]相关。^[real-form-labels-order.md:36-40]

若归约余量在根位置仍有置位，则 grading 条件无法满足，返回 `ImpossibleGrading`。否则，由哨兵位通过 XOR 组合得到 ambient 代表，再交给 fundamental 分区的 `class_of` 确定全局标签。^[real-form-labels-order.md:39-40]

最后，首标签必须等于 fundamental 分区的 quasisplit 类，否则触发 `"quasisplit anchor"` 不变量违例；空分区也必然触发该检查。^[real-form-labels-order.md:41-42]

## 测试与证据边界

记录的测试锚点包括：A2 反射 Cartan 与单连通 A1 分裂 Cartan 只标记 quasisplit；fundamental Cartan 给出恒等映射；B2 与 twisted A2 的各 Cartan 检查标签计数、quasisplit 锚点和值域。此外，还有交叉或外来输入的三条拒绝路径，以及 A2/B2 的基扩展手算奇偶锚点和实根拒绝测试。^[real-form-labels-order.md:44-47]

这些材料属于结构性源码阅读，不构成数学或正确性验收。labels 测试仅触及 A1/A2/B2，`ImpossibleGrading` 和 `"quasisplit anchor"` 的失败路径没有测试锚点；局部标签与[[弱实形式的外部编号与严格排序]]之间也没有组合测试。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-form-labels-order.md:98-104, real-form-labels-order.md:108-112]

## Sources

- [弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）](real-form-labels-order.md)
