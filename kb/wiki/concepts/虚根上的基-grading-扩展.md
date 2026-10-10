---
title: 虚根上的基 grading 扩展
summary: 精确求解转置 bracket 子 Cartan 系统，以虚单根坐标和的奇偶扩展基 grading，并显式检查虚根性。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:04:50.267Z"
updatedAt: "2026-10-10T00:46:12.508Z"
tags:
  - 根系
  - grading
aliases:
  - 虚根上的基-grading-扩展
  - 虚G扩
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 虚根上的基 grading 扩展
summary: base_grading_extension 精确求解转置的 bracket 索引子 Cartan 系统，以虚单根坐标系数和的奇偶扩展基本 grading，并显式校验虚根性。
sources:
  - real-form-labels-order.md
kind: concept
tags:
  - 虚根
  - grading
  - 精确线性代数
aliases:
  - 虚根上的基-grading-扩展
---

# 虚根上的基 grading 扩展

虚根上的基 grading 扩展由 crate 内可见的 `base_grading_extension`（`pub(crate)`）实现，是 `RealFormLabels` 标签计算的一环。它将基本 grading 扩展到任意虚根，取值为该根在虚单根基下的坐标系数之和的奇偶。^[real-form-labels-order.md:26-35]

## 坐标求解与根类型检查

设虚根为 $\beta=\sum_i c_i\beta_i$，其中 $\beta_i$ 为虚单根，则扩展值为 $\sum_i c_i\bmod 2$。坐标通过精确求解**转置的 bracket 索引子 Cartan 系统**获得：系统第 $j$ 行记录各基根与第 $j$ 个余根的配对。^[real-form-labels-order.md:32-35]

实现显式检查虚根性，因为非虚根的投影也可能具有整数坐标。因此，坐标整性不能替代[[对合下的虚根、实根与复根分类|根类型判定]]；求得整坐标本身不足以接受输入。^[real-form-labels-order.md:33-35]

## 在弱实形式标签计算中的作用

`RealFormLabels` 将一个 Cartan 的局部弱实类映射到 fundamental 分区的全局实形式编号，参见[[弱实形式的局部到全局标签映射]]。计算先确定 Cayley 回拉翻转位：对每个 Cartan 侧虚单根，统计使 `root + alpha` 仍为根的 Cayley 根 `alpha`，数量为奇数时翻转。随后将 `cartan_imaginary ++ cayley` 经 cross 作用运送，每个像都必须相对于 distinguished involution 为虚根，否则触发 `"fundamental imaginary"` 违规。^[real-form-labels-order.md:17-19, real-form-labels-order.md:26-31]

扩展基 grading 为后续模二求解提供比较基准。算法将回拉后的 grading 与翻转位异或，在 Cayley 块附加全 noncompact 条件，再与扩展基 grading 求差。增广 `ModTwoSubspace` 使用根位置与哨兵位表达 fundamental fiber 基代表同运送根的关系，详见[[Cayley 回拉与模二 grading 求解]]。^[real-form-labels-order.md:36-38]

若 `quotient_representative` 的余量在根位置仍有置位，则返回 `ImpossibleGrading`；否则由哨兵位异或出 ambient 代表，再通过 fundamental 分区的 `class_of` 得到标签。首标签必须是 fundamental quasisplit 类，否则触发 `"quasisplit anchor"` 违规。^[real-form-labels-order.md:39-42]

## 测试与证据边界

基 grading 扩展的直接测试锚点包括 A2、B2 各两个根的手算奇偶校验，以及实根输入的拒绝测试。labels 层整体测试仅涉及 A1、A2、B2；`ImpossibleGrading` 与 `"quasisplit anchor"` 的失败分支没有测试锚点。^[real-form-labels-order.md:44-47, real-form-labels-order.md:98-104]

来源属于经维护者对照源码核对的结构性阅读，不构成标签或编号层的数学验收。上游引用仅转录自代码注释；该次知识维护未执行 Atlas、Cargo、测试或 benchmark，上述测试锚点不代表该次执行结果。^[real-form-labels-order.md:9-13, real-form-labels-order.md:108-112]

## Sources

- [real-form-labels-order.md](../../sources/real-form-labels-order.md)：弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）。
