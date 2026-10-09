---
title: twist-fixed 生成元的有序基校验
summary: verified_generator_map 要求 twist-fixed 简单生成元数量等于伴随 fiber 维数，并逐位验证实际有序基以保障 grading 掩码的数值比较语义，拒绝下标大于等于 127 的生成元。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:05:19.599Z"
updatedAt: "2026-10-09T15:05:19.599Z"
tags:
  - 生成元映射
  - 有序基
  - 输入校验
aliases:
  - twist-fixed-生成元的有序基校验
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# twist-fixed 生成元的有序基校验

`verified_generator_map` 校验 twist-fixed 简单生成元与伴随 fiber 坐标之间的**实际有序基对应**。它属于 `real_form_order.rs` 的外部编号机制：由于 `special_grading_key` 按生成元编号比较位掩码，仅有抽象双射不足以保证排序所需的坐标含义。^[real-form-labels-order.md:68-70]

## 校验契约

校验要求按升序排列的 twist-fixed 简单生成元数量等于伴随 fiber 的维数，并逐位核对实际有序基。生成元下标必须小于 `MAX_KEY_GENERATORS = 127`；下标达到或超过 127 时拒绝输入。^[real-form-labels-order.md:68-70]

这里的顺序具有可观测意义：grading 掩码以生成元 0 为最低位，并按无符号位集比较。因此，生成元与 fiber 坐标的对应不仅要覆盖正确的对象，还必须保持排序键使用的编号约定。^[real-form-labels-order.md:51-56, real-form-labels-order.md:68-75]

## 与 specialGrading 排序键的关系

`special_grading_key` 使用 `specialGrading` 的 PARTITION 重载：以类代表为种子，升序扫描 `0..2^dimension`，以 `>=` 条件替换候选，从而在最大 popcount 的候选中选取最高 fiber 下标。随后在 fiber 秩内取补，并将补集展开到 twist-fixed 简单生成元的位置上。此过程依赖已校验的有序基映射；fiber 宽度另受 `MAX_MASK_BITS` 限制。参见 [[specialGrading 的分区代表与位集编码]]。^[real-form-labels-order.md:68-75]

`ExternalFormOrder` 先按 depth 升序排列弱实形式，再用该 grading 位集打破平局。实现要求 `(depth, grading)` 构成严格序，任何并列都触发 `"strict (depth, grading) order"` 不变量违例，并要求 quasisplit 居末。有序基校验因此支撑了 [[弱实形式的外部编号与严格排序]] 中第二排序键的坐标语义。^[real-form-labels-order.md:51-58, real-form-labels-order.md:68-70]

## 测试与证据边界

直接相关的回归锚点是 E6 twisted 生成元坐标测试，期望 `verified_generator_map == [1, 3]`。该期望来自给定置换，而非 Rust 输出；E6 的测试覆盖仅限生成元映射，不能扩展为整个外部编号流程的验证。^[real-form-labels-order.md:82-86, real-form-labels-order.md:103-104]

来源属于结构性阅读，不声称标签或编号层的数学验收。`real_form_order.rs` 几乎全部失败路径缺少测试锚点，因此上述拒绝条件是已记录的实现契约，不代表相应错误分支均经过测试验证。^[real-form-labels-order.md:9-13, real-form-labels-order.md:98-104]

## Sources

- [弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）](real-form-labels-order.md)
