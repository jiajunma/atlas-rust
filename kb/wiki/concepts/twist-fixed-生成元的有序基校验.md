---
title: twist-fixed 生成元的有序基校验
summary: verified_generator_map 校验 twist-fixed 简单生成元与伴随 fiber 的维数及实际有序基对应关系，并拒绝下标至少为 127 的生成元。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:05:19.599Z"
updatedAt: "2026-10-09T22:42:59.974Z"
tags:
  - 生成元
  - 构造校验
aliases:
  - twist-fixed-生成元的有序基校验
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: twist-fixed 生成元的有序基校验
summary: verified_generator_map 校验 twist-fixed 简单生成元与伴随 fiber 的实际有序基对应，保持 grading 排序键的坐标语义，并拒绝下标达到 127 的生成元。
sources:
  - real-form-labels-order.md
kind: concept
tags:
  - 生成元映射
  - 有序基
  - 输入校验
aliases:
  - twist-fixed-生成元的有序基校验
---

# twist-fixed 生成元的有序基校验

`verified_generator_map` 是 `real_form_order.rs` 外部编号机制中的坐标校验。它要求 twist-fixed 简单生成元与伴随 fiber 的**实际有序基**对应，而不只是存在抽象双射，因为 `special_grading_key` 按生成元编号形成的位掩码进行数值比较。^[real-form-labels-order.md:68-75]

## 校验契约

twist-fixed 简单生成元按升序排列，其数量必须等于伴随 fiber 的维数；随后逐位校验实际有序基。生成元下标必须小于 `MAX_KEY_GENERATORS = 127`，达到或超过 127 时拒绝输入。^[real-form-labels-order.md:68-70]

grading 排序键是 twist-fixed 简单生成元上的无符号位集，生成元 0 对应最低位。因此，这项校验保留了 fiber 坐标与生成元位位置之间的具体对应关系，而该对应关系直接影响掩码的比较结果。^[real-form-labels-order.md:51-56, real-form-labels-order.md:68-75]

## 与 grading 排序键的关系

`special_grading_key` 使用 `specialGrading` 的 PARTITION 重载：以类代表为种子，升序扫描 `0..2^dimension`，以 `>=` 替换候选，从而在 popcount 最大的候选中选择最高 fiber 下标；随后在 fiber 秩内取补，再将补集展开到 twist-fixed 简单生成元的位置上。相关机制见 [[specialGrading 的分区代表与位集编码]]。^[real-form-labels-order.md:72-75]

生成元下标与 fiber 宽度分别受 `MAX_KEY_GENERATORS` 和 `MAX_MASK_BITS` 约束。来源明确记录了全枚举行为及其对 `MAX_MASK_BITS` 的依赖，但未给出后者的数值；不能将 fiber 宽度限制直接等同于生成元下标上限。^[real-form-labels-order.md:68-75, real-form-labels-order.md:101-102]

`ExternalFormOrder` 先按 depth 升序排序，再用 grading 位集打破平局。实现要求 `(depth, grading)` 严格有序，并列时报告 `"strict (depth, grading) order"` 不变量违例，同时要求 quasisplit 居末。有序基校验保障第二排序键所依赖的坐标对应，详见 [[弱实形式的外部编号与严格排序]]。^[real-form-labels-order.md:51-58, real-form-labels-order.md:68-70]

## 测试与证据边界

直接相关的测试锚点是 E6 twisted 生成元坐标回归，期望 `verified_generator_map == [1, 3]`。该期望来自给定置换，而非 Rust 输出；E6 的测试覆盖仅限生成元映射，不能据此认定整个外部编号流程已得到验证。^[real-form-labels-order.md:82-86, real-form-labels-order.md:103-104]

来源属于结构性源码阅读，不声称标签或编号层的数学验收。`real_form_order.rs` 几乎全部失败路径缺少测试锚点，因此上述拒绝条件是已记录的实现契约，并不表示这些错误分支均已通过测试验证。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-form-labels-order.md:9-13, real-form-labels-order.md:98-104, real-form-labels-order.md:108-112]

## Sources

- [弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）](../../sources/real-form-labels-order.md)
