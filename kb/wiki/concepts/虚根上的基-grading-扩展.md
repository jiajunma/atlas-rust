---
title: 虚根上的基 grading 扩展
summary: base_grading_extension 精确求解转置的 bracket 索引子 Cartan 系统，以虚单根坐标系数和的奇偶扩展 grading，并显式检查虚根性，因为坐标整性不足以判定虚根。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:04:50.267Z"
updatedAt: "2026-10-09T15:04:50.267Z"
tags:
  - 根系
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 虚根上的基 grading 扩展

虚根上的基 grading 扩展由 `pub(crate)` 函数 `base_grading_extension` 实现，是 `RealFormLabels` 的 grading 标签计算机制的一环。它把基本 grading 扩展到任意虚根：该根的 grading 值等于其在虚单根基下的坐标系数之和的奇偶。^[real-form-labels-order.md:26-35]

## 坐标求解与虚根门控

扩展采用精确坐标求解。所解线性系统使用**转置的 bracket 索引子 Cartan 矩阵**：第 \(j\) 行记录各基根与第 \(j\) 个余根的配对。若虚根写成 \(\beta=\sum_i c_i\beta_i\)，其中 \(\beta_i\) 为虚单根，则扩展值为
\(\operatorname{base\_grading\_extension}(\beta)=\sum_i c_i \pmod 2\)。^[real-form-labels-order.md:32-35]

根的类别必须显式校验，不能以求得坐标是否为整数来判断虚根性：非虚根的投影也可能具有整数坐标。因此，整性不能替代[[对合下的虚根、实根与复根分类|虚根类别判定]]。^[real-form-labels-order.md:33-35]

## 在弱实形式标签计算中的作用

`RealFormLabels` 将某个 Cartan 的局部弱实类映射到 fundamental 分区的全局实形式编号。计算时，先确定 Cayley 回拉所需的 grading 翻转位，再将 `cartan_imaginary ++ cayley` 根列表经 cross 作用运送；每个根像都必须相对于 distinguished involution 为虚根，否则触发 `"fundamental imaginary"` 不变量违例。^[real-form-labels-order.md:17-19, real-form-labels-order.md:26-31]

随后，算法将回拉 grading 与翻转位异或，并为 Cayley 块附加全 noncompact 条件，再与扩展后的基 grading 求差。该差值进入增广 `ModTwoSubspace` 求解：若 `quotient_representative` 的余量仍在根位置上置位，则返回 `ImpossibleGrading`；否则由哨兵位异或恢复 ambient 代表，再通过 fundamental 分区的 `class_of` 得到标签。这一过程连接了[[Cayley 回拉与模二 grading 求解]]与[[弱实形式的局部到全局标签映射]]。^[real-form-labels-order.md:36-40]

## 测试与证据边界

基 grading 扩展的直接测试锚点包括 A2、B2 各两个根的手算奇偶校验，以及实根输入的拒绝测试。来源列出的 labels 测试仅涉及 A1、A2、B2；其中 `ImpossibleGrading` 与 `"quasisplit anchor"` 失败分支没有测试锚点。^[real-form-labels-order.md:44-47, real-form-labels-order.md:98-104]

这些结论来自源码结构性阅读，不构成标签或编号层的数学验收；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[real-form-labels-order.md:9-13, real-form-labels-order.md:108-112]

## Sources

- [real-form-labels-order.md](real-form-labels-order.md)
