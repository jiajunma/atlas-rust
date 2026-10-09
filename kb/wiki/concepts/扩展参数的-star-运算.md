---
title: 扩展参数的 star 运算
summary: star 返回指定根的 delta 轨道类型 DescValue 及邻接扩展参数列表，构成参数层下降计算的核心。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:47:11.450Z"
updatedAt: "2026-10-09T19:27:57.435Z"
tags:
  - 扩展参数
  - 下降运算
aliases:
  - 扩展参数的-star-运算
  - 扩S运
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 扩展参数的 star 运算

`star` 是扩展参数层的核心计算，接口为 `star(ctx, e, length, n_alpha)`，返回 `(DescValue, Vec<ExtParam>)`，描述根编号 `n_alpha` 的 delta-轨道类型及其邻接扩展参数。它移植自上游 `gkmod/ext_block.cpp` 的参数层，对应源码注释所指的 `ext_block.cpp:990–1705`。^[ext-param.md:19-24, ext-param.md:56-58]

## 上下文与参数

[[扩展表示上下文 ExtRepContext]] 通过扭曲对合 `delta` 扩展 `RepContext`。其中，`delta` 以根系置换表示，上下文还保存不动根集与诱导的单生成元 twist，并提供 `delta_of`、`is_delta_fixed_root`、`twisted` 等访问器，以及 `to_simple_shift`、`is_very_complex`、`shift_flip` 等高级判定。^[ext-param.md:33-38]

[[扩展参数值类型 ExtParam]] 包含 Weyl 元素 `tw`、余权 `l`、有理权 `gamma_lambda`、权 `tau`、余权 `t` 与翻转位。它可计算 `theta(ctx)`、`theta_id(ctx)`，并通过 `(tw, l mod 2)` 重建 KGB 元素。^[ext-param.md:40-47]

## 返回值与 finalisation

`star` 将类型判定与邻接参数一起返回：`DescValue` 表示指定根的 delta-轨道类型，`Vec<ExtParam>` 保存邻接扩展参数。相关类型体系见 [[DescValue 扩展下降分类]]。^[ext-param.md:56-58]

三个[[扩展参数的 finalisation 驱动]]——`extended_restrict_to_k`、`extended_finalise` 与 `scaled_extended_finalise`——在队列循环中重放 folded-orbit 反射与 `star` 下降，并跟踪相对[[扩展参数的默认扩展]]的净翻转。^[ext-param.md:25-26, ext-param.md:58-59]

`extended_finalise(ctx, sr)` 返回 `Vec<(StandardRepr, bool)>`，其输入须满足 standard 且 delta-fixed，前置条件由 `debug_assert` 检查。`scaled_extended_finalise(ctx, sr, factor_num, factor_den)` 返回 `(StandardRepr, bool)`，缩放 $\nu$ 并保持 $\lambda$ 固定。^[ext-param.md:59-62]

## 扩展块的符号调校

参数层还提供两个 StarOracle 实现：`ExtParamOracle` 服务于 `ExtBlock::tune_signs`，通过 `ext_param::def_ext` 重建每个父块元素的默认扩展；`PartialBlockOracle` 以 `PartialBlock` 为父块后端，用于 `ExtBlock::build_partial` 之后的 `tune_signs`。相关接口见 [[扩展块符号调校的 StarOracle 实现]]。^[ext-param.md:64-68]

## 算术与错误约定

`star` 所在模块遵循[[扩展参数层的 Rust 移植约定]]：所有 `Weight`、`Coweight` 与 `int` 算术采用二进制补码 wrapping i32，以匹配上游 `int` 算术；有理权分子保持 i64。上游 `assert` 转为 `debug_assert` 或仅在调试模式执行的 `validate`，真正的数据相关失败通过 [[StructureError 统一错误分类学|StructureError]] 暴露。^[ext-param.md:28-31]

## 证据范围

来源是对 `ext_param.rs` 的结构性阅读，所读字节来自 dirty 工作区并由快照记录。模块正确性属于其自身的 [[HPC 验收证据链]]，来源不重述或扩展这条证据链。^[ext-param.md:9-15]

来源未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论。所列上游行号转述自源码注释，未经独立重读上游核对，可能随版本演进而漂移。^[ext-param.md:70-78]

## Sources

- [ext-param.md](ext-param.md) — ext_param/star 层：扩展块的参数层。
