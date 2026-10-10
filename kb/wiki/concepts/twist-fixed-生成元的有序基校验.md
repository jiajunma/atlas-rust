---
title: twist-fixed 生成元的有序基校验
summary: verified_generator_map 校验固定简单生成元与伴随 fiber 的维数及实际有序基对应关系，并拒绝下标至少为 127 的生成元。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:05:19.599Z"
updatedAt: "2026-10-10T02:01:41.072Z"
tags:
  - 生成元
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
---

# twist-fixed 生成元的有序基校验

`verified_generator_map` 是 `real_form_order.rs` 中的生成元坐标校验机制。它要求 twist-fixed 简单生成元与伴随 fiber 的**实际有序基**逐位对应，而不只是存在抽象双射。这一顺序直接影响 `special_grading_key` 的位掩码数值比较，因此属于外部编号的排序契约。^[real-form-labels-order.md:73-80]

## 校验契约

twist-fixed 简单生成元按下标升序排列，其数量必须等于伴随 fiber 的维数；随后逐位校验实际有序基。生成元下标必须小于 `MAX_KEY_GENERATORS = 127`，下标达到或超过 127 时拒绝输入。仅有维数相等或抽象双射不足以满足该校验。^[real-form-labels-order.md:73-75]

排序键使用 twist-fixed 简单生成元上的无符号位集，生成元 0 对应最低位。因此，fiber 坐标必须对应到指定的生成元位位置，基顺序不能任意置换。`special_grading(external)` 返回这一平局排序位集，其位 g 表示生成元 g，值 1 的含义为 noncompact imaginary。^[real-form-labels-order.md:51-55, real-form-labels-order.md:73-85]

## 与 grading 排序键的关系

`special_grading_key` 使用 `specialGrading` 的 PARTITION 重载：以类代表为种子，升序扫描 `0..2^dimension`，用 `>=` 替换候选，从而在 popcount 最大的候选中选取最高 fiber 下标；随后在 fiber 秩内取补，再将补集展开到 twist-fixed 简单生成元的位置上。详见 [[specialGrading 的分区代表与位集编码]]。^[real-form-labels-order.md:77-80]

生成元下标与 fiber 宽度分别受 `MAX_KEY_GENERATORS` 和 `MAX_MASK_BITS` 限制。来源记录了全枚举行为及其对 `MAX_MASK_BITS` 的依赖，但未给出后者的数值；两项限制约束的对象不同。^[real-form-labels-order.md:73-80, real-form-labels-order.md:106-107]

`ExternalFormOrder` 先按 depth 升序排序，再以 grading 位集打破平局。实现要求 `(depth, grading)` 严格有序，出现并列时报 `"strict (depth, grading) order"` 不变量违例，并要求 quasisplit 居末。有序基校验维护第二排序键的坐标对应，完整机制见 [[弱实形式的外部编号与严格排序]]。^[real-form-labels-order.md:51-59, real-form-labels-order.md:73-75]

## 测试与证据边界

直接相关的测试锚点是 E6 twisted 生成元坐标回归，期望 `verified_generator_map == [1, 3]`。该期望来自给定置换，而非 Rust 输出。来源中的 E6 覆盖仅限生成元映射，不能据此认定整个外部编号流程已得到验证。^[real-form-labels-order.md:87-91, real-form-labels-order.md:108-109]

来源属于结构性源码阅读，不声称标签或编号层的数学验收。`real_form_order.rs` 几乎全部失败路径缺少测试锚点，因此上述拒绝条件是已记录的实现契约，并不表示错误分支均已通过测试验证。本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[real-form-labels-order.md:10-13, real-form-labels-order.md:103-105, real-form-labels-order.md:113-117]

## Sources

- [弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）](../../sources/real-form-labels-order.md)
