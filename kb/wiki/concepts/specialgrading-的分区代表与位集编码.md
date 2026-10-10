---
title: specialGrading 的分区代表与位集编码
summary: 全枚举 fiber 下标，选取最大 popcount 中的最高下标，在 fiber 秩内取补后映射到 twist-fixed 简单生成元位集，生成元 0 对应最低位。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:05:15.013Z"
updatedAt: "2026-10-10T02:01:40.513Z"
tags:
  - 分级
  - 位集编码
  - 实形式排序
aliases:
  - specialgrading-的分区代表与位集编码
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# specialGrading 的分区代表与位集编码

`special_grading_key` 实现 `specialGrading` 的 PARTITION 重载，将分区类编码为 twist-fixed 简单生成元上的无符号位集。在[[弱实形式的外部编号与严格排序]]中，该位集用作 depth 相同时的排序键；生成元 0 对应最低位，因此生成元编号与坐标映射直接影响比较结果。^[real-form-labels-order.md:51-57, real-form-labels-order.md:73-80]

## 分区代表的选择与编码

算法以类代表为种子，按升序扫描 fiber 下标 `0..2^dimension`，选取该类中 popcount（置位数）最大的代表。更新条件使用 `>=`，因此置位数相同时，后扫描到的候选会替换当前代表，最终选中最大 popcount 候选中 fiber 下标最高者。^[real-form-labels-order.md:77-80]

选定代表后，算法先在 fiber 秩限定的位宽内取补，再通过 unslice 将补集映射到 twist-fixed 简单生成元上。取补使用 fiber 坐标，输出位集则以简单生成元编号作为位位置，生成元 0 为最低位。^[real-form-labels-order.md:77-80]

访问器 `special_grading(external)` 返回对应实形式的同深度排序位集。根据文档与字段注释，位 g 对应生成元 g，置位表示 noncompact imaginary（非紧虚）。^[real-form-labels-order.md:82-85]

## 有序基与位宽约束

`verified_generator_map` 要求升序排列的 twist-fixed 简单生成元数量等于伴随 fiber 维数，并逐位校验实际有序基。这里需要验证具体坐标对应关系，而不只是抽象双射，因为排序键按生成元数值顺序比较掩码；详见[[twist-fixed 生成元的有序基校验]]。^[real-form-labels-order.md:73-75]

生成元下标达到或超过 127（`MAX_KEY_GENERATORS`）时会被拒绝。fiber 位宽另受 `MAX_MASK_BITS` 限制，全枚举的安全性依赖该限制；来源包没有给出这一常量的数值。^[real-form-labels-order.md:73-80, real-form-labels-order.md:106-107]

## 在外部编号中的作用

`ExternalFormOrder` 首先按 depth 升序排序，再按上述无符号位集比较。depth 对应 distinguished involution 处极大正交非紧虚根集的大小，其计算机制见[[非紧虚根正交集的深度计算]]。^[real-form-labels-order.md:51-55, real-form-labels-order.md:61-65]

上游使用不稳定的 `std::sort`，Rust 移植则断言 `(depth, grading)` 构成严格序：排序键并列时报告 `"strict (depth, grading) order"` 不变量违例，并要求 quasisplit 居末，否则报告 `"quasisplit last"`。“compact（深度 0）为 external 0”属于文档声明，代码没有显式断言首项深度为 0。^[real-form-labels-order.md:55-59]

## 测试与证据边界

Spin(8) 测试覆盖同 depth 时通过 grading 键获得严格排序，并检查 compact 居首、split 居末。E6 twisted 测试覆盖生成元坐标映射，期望 `verified_generator_map == [1, 3]`；该期望来自给定置换，而非 Rust 输出。^[real-form-labels-order.md:87-91]

来源属于结构性源码阅读，不构成标签或编号层的数学验收，上游位置引用仅转录自代码注释。外部编号层几乎全部失败路径没有测试锚点，E6 覆盖仅限生成元映射，标签层与外部编号层没有组合测试。本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试描述不代表本次执行结果。^[real-form-labels-order.md:10-13, real-form-labels-order.md:103-109, real-form-labels-order.md:113-117]

## Sources

- [弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）](../../sources/real-form-labels-order.md)
