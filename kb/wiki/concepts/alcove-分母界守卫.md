---
title: Alcove 分母界守卫
summary: denominator_exceeds_alcove_bound 在 rank 小于 63 时判断 denominator > 2^rank，在 rank ≥ 63 时返回 false，避免有符号 i64 移位造成阈值失真。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:02.795Z"
updatedAt: "2026-10-09T20:28:59.287Z"
tags:
  - 形变计算
  - 整数边界
  - Rust实现
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
summary: denominator_exceeds_alcove_bound 判断分母是否严格超过 2^rank；rank ≥ 63 时返回 false，避免有符号 i64 移位造成阈值失真。
sources:
  - alcove.md
kind: concept
tags:
  - 数值边界
  - deformation
  - Rust实现
aliases:
  - alcove-分母界守卫
---

# Alcove 分母界守卫

`denominator_exceeds_alcove_bound` 用于 deformation 的分母检查，判断分母是否严格超过数学阈值 \(2^{\mathrm{rank}}\)。它与 `alcove_center` 同为 alcove 模块的公开函数。^[alcove.md:17-24, alcove.md:91-97]

## 接口与判定规则

函数接收 `rank: usize` 与 `denominator: i64`，返回 `bool`。其判定表达式为 `rank < i64::BITS as usize - 1 && denominator > (1_i64 << rank)`。^[alcove.md:29-31, alcove.md:93-97]

当 `rank < 63` 时，函数使用严格不等式 `denominator > 2^rank`：分母等于阈值时返回 `false`，超过阈值才返回 `true`。^[alcove.md:93-97]

当 `rank ≥ 63` 时，数学阈值超出正 `i64` 的表示范围，函数一律返回 `false`。前置秩判断避免求值有符号左移 `1_i64 << 63`，因为该表达式会得到 `i64::MIN`，无法表示所需的正阈值。^[alcove.md:93-97]

## 测试与证据边界

单元测试覆盖 `rank = 62` 时分母分别取 \(2^{62}\) 与 \(2^{62}+1\) 的边界，以及 `rank = 63`、`rank = 64` 时分母取 `i64::MAX` 仍返回 `false` 的情况。^[alcove.md:93-97]

来源材料属于结构性源码阅读，不声称 alcove 计算已通过数学验收；其中的上游位置引用转录自代码注释，未独立核对上游文件字节。本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[alcove.md:9-13, alcove.md:185-190]

## 相关概念

同一模块中的 [[Alcove 重心计算与标准参数重建]] 说明 `alcove_center` 如何保留标准参数的 KGB 坐标与 `lambda_rho`，仅替换 infinitesimal character，并通过 `RepContext::sr_gamma` 重建规范参数。^[alcove.md:17-19, alcove.md:65-66, alcove.md:89-89]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。
