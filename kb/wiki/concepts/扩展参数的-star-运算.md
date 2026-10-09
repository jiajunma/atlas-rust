---
title: 扩展参数的 star 运算
summary: star(ctx, e, length, n_alpha) 返回 (DescValue, Vec<ExtParam>)，描述指定根的 delta 轨道类型及邻接扩展参数。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:47:11.450Z"
updatedAt: "2026-10-09T14:47:11.450Z"
tags:
  - 扩展块
  - 根系轨道
  - star运算
aliases:
  - 扩展参数的-star-运算
  - 扩S运
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 扩展参数的 star 运算

`star` 是扩展参数层的核心计算，接口为 `star(ctx, e, length, n_alpha)`，返回 `(DescValue, Vec<ExtParam>)`，描述根编号 `n_alpha` 的 delta-轨道类型及其邻接扩展参数。该计算移植自上游 `gkmod/ext_block.cpp` 的参数层。^[ext-param.md:19-24, ext-param.md:56-58]

## 上下文与参数表示

[[扩展表示上下文 ExtRepContext]] 由 twisting involution `delta` 扩展 `RepContext`；`delta` 以根系置换表示，同时提供不动根集与诱导的单生成元 twist。它提供 `delta_of`、`is_delta_fixed_root`、`twisted` 等访问器，以及 `to_simple_shift`、`is_very_complex`、`shift_flip` 等高级判定。^[ext-param.md:33-38]

[[扩展参数值类型 ExtParam]] 包含 Weyl 元素 `tw`、余权 `l`、有理权 `gamma_lambda`、权 `tau`、余权 `t` 与翻转位。其派生操作可计算 `theta`、`theta_id`，并通过 `(tw, l mod 2)` 重建 KGB 元素。^[ext-param.md:40-47]

## 返回值与 finalisation

`star` 的返回值将类型判定与邻接参数放在一起：`DescValue` 表示对应根的 delta-轨道类型，`Vec<ExtParam>` 承载邻接扩展参数。类型体系可参见 [[DescValue 扩展下降分类]]。^[ext-param.md:56-58]

三个[[扩展参数的 finalisation 驱动]]——`extended_restrict_to_k`、`extended_finalise` 与 `scaled_extended_finalise`——在队列循环中重放 folded-orbit 反射与 `star` 下降，并跟踪相对[[扩展参数的默认扩展]]的净翻转。^[ext-param.md:25-26, ext-param.md:58-59]

其中，`extended_finalise(ctx, sr)` 返回 `Vec<(StandardRepr, bool)>`，其 standard 且 delta-fixed 的前置条件由 `debug_assert` 检查；`scaled_extended_finalise(ctx, sr, factor_num, factor_den)` 返回 `(StandardRepr, bool)`，缩放 $\nu$ 并保持 $\lambda$ 固定。^[ext-param.md:59-62]

## 算术与错误约定

`star` 所在模块遵循[[扩展参数层的 Rust 移植约定]]：所有 `Weight`、`Coweight` 与 `int` 算术采用二进制补码 wrapping i32，有理权分子保留 i64。上游 `assert` 转为 `debug_assert` 或仅在调试模式执行的 `validate`；真正的数据相关失败通过 [[StructureError 统一错误分类学|StructureError]] 暴露。^[ext-param.md:28-31]

## 证据范围

本页依据对 `ext_param.rs` 的结构性阅读材料；所读字节来自 dirty 工作区，并由快照记录。材料未执行构建、测试或原版运行，不构成数学验收、性能或并行结论。上游位置来自源码注释，未独立重读上游，行号可能随版本变化。^[ext-param.md:9-15, ext-param.md:70-78]

## Sources

- [ext-param.md](ext-param.md)
