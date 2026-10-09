---
title: specialGrading 的分区代表与位集编码
summary: 全枚举 fiber 下标，在最大 popcount 候选中取最高下标，于 fiber 秩内取补后映射至 twist-fixed 简单生成元位集，生成元 0 对应最低位。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:05:15.013Z"
updatedAt: "2026-10-09T21:05:34.595Z"
tags:
  - grading
  - 位集编码
  - 轨道枚举
aliases:
  - specialgrading-的分区代表与位集编码
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: specialGrading 的分区代表与位集编码
summary: special_grading_key 在分区类中选取 popcount 最大且 fiber 下标最高的代表，在 fiber 秩内取补，再映射为 twist-fixed 简单生成元位集，用于外部编号的同深度排序。
sources:
  - real-form-labels-order.md
kind: concept
tags:
  - grading
  - 位集编码
  - 分区代表
aliases:
  - specialgrading-的分区代表与位集编码
---

# specialGrading 的分区代表与位集编码

`special_grading_key` 实现 `specialGrading` 的 PARTITION 重载，将分区类转换为 twist-fixed 简单生成元上的无符号位集。在[[弱实形式的外部编号与严格排序]]中，该位集用于打破 depth 相同的并列；生成元 0 对应最低位，因此生成元编号直接参与排序键的数值比较。^[real-form-labels-order.md:51-57, real-form-labels-order.md:72-75]

## 分区代表的选择

算法以类代表为种子，按升序扫描 fiber 下标 `0..2^dimension`，在该类中选择 popcount（置位数）最大的代表。更新条件使用 `>=`，因此 popcount 相同时由后扫描到的候选替换当前代表，最终选中最大 popcount 候选中 fiber 下标最高者。^[real-form-labels-order.md:72-75]

选定代表后，算法在 fiber 秩限定的位宽内取补，再将补集 unslice 到 twist-fixed 简单生成元上。取补的范围是 fiber 秩，最终编码的位位置则使用简单生成元编号。^[real-form-labels-order.md:72-75]

## 有序基与位集含义

这一编码依赖实际的有序基。`verified_generator_map` 要求按升序排列的 twist-fixed 简单生成元数量等于伴随 fiber 维数，并逐位校验实际基的对应关系。仅验证抽象双射并不足够，因为 `special_grading_key` 会按生成元的数值顺序比较掩码，详见[[twist-fixed 生成元的有序基校验]]。^[real-form-labels-order.md:68-70]

`special_grading(external)` 返回相应实形式的排序位集。其文档和字段注释规定：位 g 对应生成元 g，置位表示 noncompact imaginary；生成元 0 为最低位。^[real-form-labels-order.md:72-80]

生成元下标达到或超过 127（`MAX_KEY_GENERATORS`）时会被拒绝。fiber 位宽另受 `MAX_MASK_BITS` 限制；全枚举的安全性依赖这一限制，但来源包未给出该常量的数值。^[real-form-labels-order.md:68-75, real-form-labels-order.md:101-102]

## 在外部编号中的作用

`ExternalFormOrder` 首先按 depth 升序排序，再按上述位集的无符号数值排序。这里的 depth 指 distinguished involution 处极大正交 noncompact 虚根集的大小，其计算细节见[[非紧虚根正交集的深度计算]]。^[real-form-labels-order.md:51-55, real-form-labels-order.md:61-66]

Rust 移植要求 `(depth, grading)` 构成严格序：若两个排序键完全相同，则报告 `"strict (depth, grading) order"` 不变量违例；同时显式要求 quasisplit 居末。相比之下，“compact（深度 0）为 external 0”属于文档声明，代码没有对首项深度为 0 的显式断言。^[real-form-labels-order.md:55-59]

## 测试与证据边界

Spin(8) 测试覆盖同 depth 时经 grading 键获得严格排序，并检查 compact 居首、split 居末。E6 twisted 测试覆盖生成元坐标映射，期望 `verified_generator_map == [1, 3]`；期望值来自给定置换，而非 Rust 输出。^[real-form-labels-order.md:82-86]

来源属于结构性源码阅读，不构成数学正确性验收，上游引用也仅转录自代码注释。外部编号层几乎全部失败路径尚无测试锚点，E6 覆盖仅限生成元映射，标签层与外部编号层没有组合测试；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-form-labels-order.md:10-13, real-form-labels-order.md:98-104, real-form-labels-order.md:108-112]

## Sources

- [弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）](real-form-labels-order.md)
