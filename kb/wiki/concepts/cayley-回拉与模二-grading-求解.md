---
title: Cayley 回拉与模二 grading 求解
summary: 通过 Cayley 翻转位、cross 根运送及带哨兵位的增广 ModTwoSubspace 恢复 fundamental 代表，根位置残留非零余量时报 ImpossibleGrading。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:04:50.826Z"
updatedAt: "2026-10-09T21:05:16.395Z"
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cayley 回拉与模二 grading 求解
summary: RealFormLabels 通过 Cayley 回拉翻转位、cross 根运送和带哨兵位的模二求解恢复 fundamental 分区标签；根位置存在不可消余量时返回 ImpossibleGrading。
sources:
  - real-form-labels-order.md
kind: concept
tags:
  - Cayley变换
  - 模二线性代数
  - grading
aliases:
  - cayley-回拉与模二-grading-求解
---

# Cayley 回拉与模二 grading 求解

Cayley 回拉与模二 grading 求解是 `RealFormLabels` 使用的 grading-based legacy mechanism，用于把一个 Cartan 的局部弱实类映射到 fundamental 分区的全局实形式编号。`label(k)` 给出按 `classes()` 顺序排列的局部类 \(k\) 的全局标签，是[[弱实形式的局部到全局标签映射]]的计算核心。^[real-form-labels-order.md:17-40]

## 构造前提

构造门控依次检查 datum 一致性、grading 所属对合、分解的 distinguished 因子化，以及分区与 fiber 的来源一致性。fundamental grading 必须位于本 inner class 的 distinguished involution，Cartan grading 必须位于分解的合成对合。对应失败分别为 `DatumMismatch`、`CartanFiberInvolutionMismatch`、`DistinguishedInvolutionMismatch`，以及外来 fiber 经元素 provenance 路径触发的 `CartanFiberMismatch`。^[real-form-labels-order.md:19-24]

## Cayley 回拉与根运送

首先，对 Cartan 侧每个虚单根 `root` 计算翻转位：统计使 `root + alpha` 仍为根的 Cayley 根 `alpha`，个数为奇数时翻转。随后，将拼接根列表 `cartan_imaginary ++ cayley` 经 cross 作用运送；每个像都必须相对于 distinguished involution 为虚根，否则触发 `"fundamental imaginary"` 不变量违例。^[real-form-labels-order.md:28-31]

对每个局部类，求解右端先取“回拉后的 grading XOR 翻转位”，再接上 Cayley 块的全 noncompact 条件。该右端随后与扩展基 grading 求差，作为模二求解的目标。^[real-form-labels-order.md:36-40]

## 基 grading 的扩展

`base_grading_extension` 是 `pub(crate)` 函数，将基本 grading 扩展到任意虚根：其值等于该根在虚单根基下的坐标系数和的奇偶。坐标通过精确求解**转置的 bracket 索引子 Cartan 系统**得到，其中第 \(j\) 行记录各基根与余根 \(j\) 的配对。相关概念见[[虚根上的基 grading 扩展]]。^[real-form-labels-order.md:32-35]

坐标整性不能作为虚根性判据，因为非虚根的投影也可能具有整数坐标。因此，实现显式检查根类，不能以坐标求解的整性替代这一门控。^[real-form-labels-order.md:33-35]

## 增广模二系统与标签恢复

求解器构造增广 `ModTwoSubspace`，坐标包含 `list_len` 个根位置和与维数相同数量的哨兵位，用来表达 fundamental fiber 基代表与运送根的关系。对每个局部类，将上述 grading 差交给 `quotient_representative` 归约；相关背景见[[F₂ 商空间的确定性代表元与陪集判定]]。^[real-form-labels-order.md:36-40]

若归约余量在根位置仍有置位，则返回 `ImpossibleGrading`。否则，由哨兵位通过 XOR 组合恢复 ambient 代表，再调用 fundamental 分区的 `class_of`，取得该局部类的全局标签。^[real-form-labels-order.md:39-40]

最后，首标签必须等于 fundamental 分区的 quasisplit 类，否则触发 `"quasisplit anchor"` 不变量违例；空分区也必然触发该检查。^[real-form-labels-order.md:41-42]

## 测试与证据边界

来源记录的测试锚点包括：A2 反射 Cartan 只标记 quasisplit；fundamental Cartan 给出恒等映射；单连通 A1 分裂 Cartan 标记 quasisplit；B2 与 twisted A2 的各 Cartan 检查标签计数、quasisplit 锚点和值域。此外，还有交叉或外来输入的三条拒绝路径，以及 A2/B2 各两个根的基扩展手算奇偶锚点和实根拒绝测试。^[real-form-labels-order.md:44-47]

这些证据属于结构性源码阅读，不构成数学或正确性验收。labels 测试仅触及 A1/A2/B2，`ImpossibleGrading` 与 `"quasisplit anchor"` 的失败路径没有测试锚点。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-form-labels-order.md:98-104, real-form-labels-order.md:108-112]

局部标签的输出与外部编号接口的输入同为 fundamental 分区的 `WeakRealFormId`。来源据此推断“局部类 → fundamental 内部编号 → 外部编号”在类型上可以串联，但两个文件互不导入，所读字节中没有执行该组合的代码或测试；不能将这一推断视为已经验证的接口组合。相关主题见[[弱实形式的外部编号与严格排序]]。^[real-form-labels-order.md:88-94]

## Sources

- [弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）](real-form-labels-order.md)
