---
title: Alcove 分母界守卫
summary: denominator_exceeds_alcove_bound 在 rank 小于 63 时判断 denominator > 2^rank，在 rank 至少为 63 时返回 false，避免有符号 i64 移位失真。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:02.795Z"
updatedAt: "2026-10-09T22:11:28.624Z"
tags:
  - arithmetic-safety
  - deformation
aliases:
  - alcove-分母界守卫
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Alcove 分母界守卫
summary: denominator_exceeds_alcove_bound 在 rank 小于 63 时判断分母是否严格超过 2^rank，在 rank ≥ 63 时返回 false，避免有符号 i64 移位造成阈值失真。
sources:
  - alcove.md
kind: concept
tags:
  - 形变计算
  - 整数边界
  - Rust实现
aliases:
  - alcove-分母界守卫
---

# Alcove 分母界守卫

`denominator_exceeds_alcove_bound` 是 deformation 的分母守卫，用于判断分母是否严格超过阈值 \(2^{\mathrm{rank}}\)。它与 `alcove_center` 同为 alcove 模块的公开函数；后者的参数处理见 [[Alcove 重心计算与标准参数重建]]。^[alcove.md:17-24, alcove.md:91-97]

## 接口与判定规则

函数接收 `rank: usize` 和 `denominator: i64`，返回 `bool`。实现使用表达式 `rank < i64::BITS as usize - 1 && denominator > (1_i64 << rank)`，先检查秩，再比较分母。^[alcove.md:29-31, alcove.md:93-97]

当 `rank < 63` 时，函数判断 `denominator > 2^rank`。这里使用严格不等式：分母等于阈值时返回 `false`，超过阈值才返回 `true`。^[alcove.md:93-97]

当 `rank ≥ 63` 时，数学阈值超出正 `i64` 的表示范围，函数一律返回 `false`。前置条件避免执行不适用的移位：有符号 `i64` 左移 63 位会得到 `i64::MIN`，无法表示所需的正阈值。^[alcove.md:93-97]

## 测试覆盖

单元测试固定了两个边界：`rank = 62` 时，分母取 \(2^{62}\) 返回 `false`，取 \(2^{62}+1\) 返回 `true`；`rank = 63` 或 `rank = 64` 时，即使分母取 `i64::MAX`，仍返回 `false`。^[alcove.md:93-97]

## 证据边界

来源材料是对 `crates/atlas-real-group/src/alcove.rs` 的结构性阅读，不声称 alcove 计算已通过数学验收。上游引用仅转录自代码注释，未独立核对上游文件字节；本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试覆盖描述不代表本次执行结果。^[alcove.md:9-13, alcove.md:185-190]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。
