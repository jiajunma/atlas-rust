---
title: twist-fixed 生成元的有序基校验
summary: verified_generator_map 验证 twist-fixed 简单生成元与伴随 fiber 的维数及实际有序基对应关系，以保障掩码数值排序语义，并拒绝下标至少为 127 的生成元。
sources:
  - real-form-labels-order.md
kind: concept
createdAt: "2026-10-09T15:05:19.599Z"
updatedAt: "2026-10-09T21:05:40.717Z"
tags:
  - 生成元
  - 有序基
  - 不变量校验
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
summary: verified_generator_map 按升序校验 twist-fixed 简单生成元与伴随 fiber 的实际有序基对应，以保持 grading 排序键的坐标语义，并拒绝下标达到 127 的生成元。
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

`verified_generator_map` 是 `real_form_order.rs` 外部编号机制中的坐标校验：它要求 twist-fixed 简单生成元与伴随 fiber 的**实际有序基**对应。由于 `special_grading_key` 按生成元编号比较位掩码，仅建立抽象双射不足以保证排序键的坐标语义。^[real-form-labels-order.md:68-75]

## 校验契约

twist-fixed 简单生成元按升序排列，其数量必须等于伴随 fiber 的维数；随后逐位校验实际有序基。生成元下标必须小于 `MAX_KEY_GENERATORS = 127`，达到或超过 127 时拒绝输入。^[real-form-labels-order.md:68-70]

这一顺序直接参与数值比较：grading 键是在 twist-fixed 简单生成元上编码的无符号位集，生成元 0 对应最低位。因此，有序基校验约束了 fiber 坐标与生成元位位置之间的具体对应关系。^[real-form-labels-order.md:51-56, real-form-labels-order.md:68-75]

## 与 grading 排序键的关系

`special_grading_key` 使用 `specialGrading` 的 PARTITION 重载。它以类代表为种子，升序扫描 `0..2^dimension`，用 `>=` 条件替换候选，从而在 popcount 最大的候选中选取最高 fiber 下标；随后在 fiber 秩内取补，并将补集展开到 twist-fixed 简单生成元的位置上。该坐标展开依赖上述有序基对应，详见 [[specialGrading 的分区代表与位集编码]]。^[real-form-labels-order.md:68-75]

生成元下标限制与 fiber 宽度限制是两项约束：前者由 `MAX_KEY_GENERATORS` 控制，后者由 `MAX_MASK_BITS` 控制。来源记录了全枚举的实现事实，但未给出 `MAX_MASK_BITS` 的数值，因此不能将其等同于生成元下标上限。^[real-form-labels-order.md:68-75, real-form-labels-order.md:101-102]

`ExternalFormOrder` 先按 depth 升序排序，再用 grading 位集打破平局。实现要求 `(depth, grading)` 严格有序，出现并列时报 `"strict (depth, grading) order"` 不变量违例，并要求 quasisplit 居末。有序基校验支撑了这一第二排序键的坐标语义，参见 [[弱实形式的外部编号与严格排序]]。^[real-form-labels-order.md:51-58, real-form-labels-order.md:68-70]

## 测试与证据边界

直接相关的测试锚点是 E6 twisted 生成元坐标回归，期望 `verified_generator_map == [1, 3]`。该期望来自给定置换，而非 Rust 输出；E6 的覆盖仅限生成元映射，不能据此认定整个外部编号流程已经得到验证。^[real-form-labels-order.md:82-86, real-form-labels-order.md:103-104]

来源属于结构性源码阅读，不声称标签或编号层的数学验收。`real_form_order.rs` 几乎全部失败路径缺少测试锚点，因此上述拒绝条件应理解为已记录的实现契约，而非全部经过测试验证的错误分支。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-form-labels-order.md:9-13, real-form-labels-order.md:98-104, real-form-labels-order.md:108-112]

## Sources

- [弱实形式标签与外部编号（real_form_labels.rs / real_form_order.rs）](real-form-labels-order.md)
