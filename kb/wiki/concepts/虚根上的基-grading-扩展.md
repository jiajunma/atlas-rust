---
title: 虚根上的基 grading 扩展
summary: 精确求解转置 bracket 子 Cartan 系统，以虚单根坐标系数和的奇偶扩展 grading，并显式检查虚根性，因为坐标整性不足以判定虚根。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:04:50.267Z"
updatedAt: "2026-10-09T21:05:09.376Z"
tags:
  - 虚根
  - grading
  - 精确线性代数
aliases:
  - 虚根上的基-grading-扩展
  - 虚G扩
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 虚根上的基 grading 扩展
summary: base_grading_extension 精确求解转置的 bracket 索引子 Cartan 系统，以虚单根坐标系数和的奇偶扩展基本 grading；虚根性须显式校验，不能由坐标整性替代。
sources:
  - real-form-labels-order.md
kind: concept
tags:
  - 根系
  - grading
  - 精确线性代数
---

# 虚根上的基 grading 扩展

虚根上的基 grading 扩展由 crate 内可见的 `base_grading_extension` 函数实现，是 `RealFormLabels` 的 grading 标签计算机制的一环。它将基本 grading 扩展到任意虚根，取该根在虚单根基下的坐标系数之和的奇偶作为 grading 值。^[real-form-labels-order.md:26-35]

## 坐标求解与虚根门控

若虚根表示为 \(\beta=\sum_i c_i\beta_i\)，其中 \(\beta_i\) 为虚单根，则扩展值为 \(\sum_i c_i\bmod 2\)。坐标通过精确求解**转置的 bracket 索引子 Cartan 系统**得到：系统的第 \(j\) 行记录各基根与第 \(j\) 个余根的配对。这一转置方向是坐标求解约定的一部分。^[real-form-labels-order.md:32-35]

实现显式检查根是否为虚根。非虚根的投影也可能具有整数坐标，因此坐标整性不能替代[[对合下的虚根、实根与复根分类|虚根类别判定]]。^[real-form-labels-order.md:33-35]

## 在弱实形式标签计算中的作用

`RealFormLabels` 将一个 Cartan 的局部弱实类映射到 fundamental 分区的全局实形式编号。计算首先确定 Cayley 回拉的翻转位：对每个 Cartan 侧虚单根，若使 `root + alpha` 仍为根的 Cayley 根 `alpha` 数量为奇数，就翻转该位。随后将 `cartan_imaginary ++ cayley` 经 cross 作用运送；每个根像都必须相对于 distinguished involution 为虚根，否则触发 `"fundamental imaginary"` 违规。^[real-form-labels-order.md:17-19, real-form-labels-order.md:26-31]

扩展后的基 grading 为模二求解提供比较基准。算法将回拉 grading 与翻转位异或，在 Cayley 块附加全 noncompact 条件，再与扩展基 grading 求差。增广 `ModTwoSubspace` 包含根位置和哨兵位：若 `quotient_representative` 的余量仍在根位置上置位，则返回 `ImpossibleGrading`；否则用哨兵位异或恢复 ambient 代表，并经 fundamental 分区的 `class_of` 得到标签。相关过程见[[Cayley 回拉与模二 grading 求解]]与[[弱实形式的局部到全局标签映射]]。^[real-form-labels-order.md:36-40]

## 测试与证据边界

基 grading 扩展的直接测试锚点包括 A2、B2 各两个根的手算奇偶校验，以及实根输入的拒绝测试。整个 labels 层的测试仅涉及 A1、A2、B2；`ImpossibleGrading` 与 `"quasisplit anchor"` 失败分支没有测试锚点。^[real-form-labels-order.md:44-47, real-form-labels-order.md:98-104]

以上说明来自源码结构性阅读，不构成标签或编号层的数学验收。来源中的上游引用转录自代码注释，本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-form-labels-order.md:9-13, real-form-labels-order.md:108-112]

## Sources

- [real-form-labels-order.md](../../sources/real-form-labels-order.md)：弱实形式标签与外部编号。
