---
title: 扩展参数值类型 ExtParam
summary: ExtParam 保存 Weyl 元素、权与余权及翻转位，支持计算 theta、由 (tw, l mod 2) 重建 KGB 元素以及限制操作。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:46:56.920Z"
updatedAt: "2026-10-09T14:46:56.920Z"
tags:
  - 表示论
  - 扩展参数
  - Rust设计
aliases:
  - 扩展参数值类型-extparam
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 扩展参数值类型 ExtParam

`ExtParam` 是扩展块参数层中的值类型，移植自上游 `ext_block.h:293-364` 与 `ext_block.cpp:2283-2420`。它承载扩展参数的数据，并提供默认扩展构造、KGB 元素重建及限制操作。^[ext-param.md:19-24, ext-param.md:40-52]

## 数据结构与派生操作

`ExtParam` 的字段包括 Weyl 元素 `tw`、余权 `l`（`Coweight`）、有理权 `gamma_lambda`（`RationalWeight`）、权 `tau`（`Weight`）、余权 `t`（`Coweight`）以及翻转位。^[ext-param.md:42-44]

派生操作包括 `theta(ctx)`、`theta_id(ctx)`、`x(ctx)`、`restrict_mod` 和 `restrict`。其中，`x(ctx)` 由 `(tw, l mod 2)` 重建 KGB 元素。^[ext-param.md:45-47]

这些操作使用的[[扩展表示上下文 ExtRepContext]]，是在 `RepContext` 上加入 twisting involution `delta` 的上下文；`delta` 以根系置换表示，并附带不动根集与诱导的单生成元 twist。^[ext-param.md:33-38]

## 默认扩展

`at` 在 KGB 元素 `x` 处构造默认扩展。相关接口还包括 `default_extend`、`default_extend_srm`、`shifted_default_extension` 与 `is_default`；其中 `default_extend_srm` 要求 `gamma_lambda` 已在 `x` 处满足 `real_unique`。这些接口构成[[扩展参数的默认扩展]]一族。^[ext-param.md:44-52]

## 在 star 与 finalisation 中的作用

[[扩展参数的 star 运算]] `star(ctx, e, length, n_alpha)` 返回 `(DescValue, Vec<ExtParam>)`，给出根编号 `n_alpha` 的 `delta` 轨道类型及邻接扩展参数。`ExtParam` 因而也是该运算输出的邻接参数值类型。^[ext-param.md:56-58]

[[扩展参数的 finalisation 驱动]]通过队列循环重放 folded-orbit 反射与 `star` 下降，并跟踪相对默认扩展的净翻转。`extended_finalise(ctx, sr)` 返回 `Vec<(StandardRepr, bool)>`，其 standard 与 delta-fixed 前置条件通过 `debug_assert` 检查；`scaled_extended_finalise(ctx, sr, factor_num, factor_den)` 返回 `(StandardRepr, bool)`，缩放 $\nu$ 而保持 $\lambda$ 固定。^[ext-param.md:58-62]

## 算术与错误约定

本参数层的 `Weight`、`Coweight` 与 `int` 算术采用二进制补码 wrapping `i32`，以匹配上游 `int` 算术；有理权分子保持 `i64`。上游 `assert` 转为 `debug_assert` 或仅在调试模式运行的 `validate`，真正的数据相关失败则通过 [[StructureError 统一错误分类学|StructureError]] 暴露。^[ext-param.md:28-31]

## 证据范围

本文依据的源文档属于对 `ext_param.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。上游文件行号转述自源码注释，未独立重读上游，可能随版本漂移；源文档未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论。^[ext-param.md:9-15, ext-param.md:70-78]

## Sources

- [ext-param.md](ext-param.md) — ext_param/star 层：扩展块的参数层。
