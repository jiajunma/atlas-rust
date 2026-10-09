---
title: Alcove 分母界守卫
summary: denominator_exceeds_alcove_bound 判断 denominator > 2^rank，并在 rank ≥ 63 时返回 false，以避免有符号 i64 移位导致阈值失真。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:02.795Z"
updatedAt: "2026-10-09T14:24:02.795Z"
tags:
  - 数值边界
  - deformation
  - Rust实现
aliases:
  - alcove-分母界守卫
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Alcove 分母界守卫

Alcove 分母界守卫 `denominator_exceeds_alcove_bound` 用于 deformation 的分母检查，判断分母是否严格超过数学阈值 \(2^{\mathrm{rank}}\)。它与 `alcove_center` 同为 alcove 模块的公开函数。^[alcove.md:17-24, alcove.md:91-97]

## 接口与判定规则

函数接收 `rank: usize` 与 `denominator: i64`，返回 `bool`，其签名与判定表达式如下。^[alcove.md:29-31, alcove.md:93-97]

```rust
pub fn denominator_exceeds_alcove_bound(rank: usize, denominator: i64) -> bool

rank < i64::BITS as usize - 1 && denominator > (1_i64 << rank)
```

当 `rank < 63` 时，守卫使用严格不等式 `denominator > 2^rank`：分母恰好等于阈值时返回 `false`，超过阈值才返回 `true`。^[alcove.md:93-97]

当 `rank ≥ 63` 时，数学阈值已超出正 `i64` 的表示范围，因此任何 `i64` 分母都不可能超过它，守卫一律返回 `false`。前置秩判断也避免求值有符号左移 `1_i64 << 63`，该表达式会得到 `i64::MIN`，不能用作正的分母阈值。^[alcove.md:93-97]

## 测试与证据边界

单元测试覆盖 `rank = 62` 时分母取 \(2^{62}\) 与 \(2^{62}+1\) 的边界，以及 `rank = 63`、`rank = 64` 时分母取 `i64::MAX` 仍返回 `false` 的情况。^[alcove.md:93-97]

这些测试提供了分母界守卫的边界覆盖；来源材料属于结构性源码阅读，不声称 alcove 计算已通过数学验收。材料中的上游引用转录自代码注释，未独立核对上游文件字节。^[alcove.md:9-13, alcove.md:173-176]

## 相关概念

同一模块中的 [[Alcove 重心计算与标准参数重建]] 解释 `alcove_center` 如何保留 KGB 坐标与 `lambda_rho`，仅替换 infinitesimal character；其求解和通分过程可参见 [[Alcove 算法中的精确有理线性代数]]。^[alcove.md:17-19, alcove.md:65-83]

## Sources

- [alcove.md](alcove.md)
