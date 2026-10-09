---
title: 扩展参数的 star 运算
summary: star(ctx, e, length, n_alpha) 返回根的 delta 轨道类型 DescValue 与邻接扩展参数列表，供参数层下降计算使用。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:47:11.450Z"
updatedAt: "2026-10-09T22:28:55.735Z"
tags:
  - 扩展参数
  - 下降算法
aliases:
  - 扩展参数的-star-运算
  - 扩S运
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 扩展参数的 star 运算
summary: star 返回指定根的 delta 轨道类型及邻接扩展参数，供参数层 finalisation 驱动执行下降计算。
sources:
  - ext-param.md
kind: concept
tags:
  - 扩展参数
  - 下降运算
aliases:
  - 扩展参数的-star-运算
provenanceState: extracted
---

# 扩展参数的 star 运算

`star` 是扩展参数层的核心计算，接口为 `star(ctx, e, length, n_alpha)`，返回 `(DescValue, Vec<ExtParam>)`：前者表示根编号 `n_alpha` 的 delta-轨道类型，后者包含邻接扩展参数。该计算移植自上游 `gkmod/ext_block.cpp` 的参数层，源码注释标注对应 `ext_block.cpp:990–1705`。^[ext-param.md:19-24, ext-param.md:56-58]

## 上下文与参数

[[扩展表示上下文 ExtRepContext]] 以 twisting involution `delta` 扩展 `RepContext`。其中 `delta` 由根系置换表示，上下文还保存不动根集与诱导的单生成元 twist，并提供 `delta_of`、`is_delta_fixed_root`、`twisted` 等访问器，以及 `to_simple_shift`、`is_very_complex`、`shift_flip` 等高级判定。^[ext-param.md:33-38]

[[扩展参数值类型 ExtParam]] 包含 Weyl 元素 `tw`、余权 `l`、有理权 `gamma_lambda`、权 `tau`、余权 `t` 与翻转位。其派生计算包括 `theta(ctx)`、`theta_id(ctx)`，以及由 `(tw, l mod 2)` 重建 KGB 元素的 `x(ctx)`。^[ext-param.md:40-47]

## 返回值与下降驱动

`star` 同时返回类型判定和邻接参数，将 [[DescValue 扩展下降分类]] 与后续参数处理衔接起来。三个[[扩展参数的 finalisation 驱动]]——`extended_restrict_to_k`、`extended_finalise` 和 `scaled_extended_finalise`——在队列循环中重放 folded-orbit 反射与 `star` 下降，并跟踪相对[[扩展参数的默认扩展]]的净翻转。^[ext-param.md:25-26, ext-param.md:56-59]

`extended_finalise(ctx, sr)` 返回 `Vec<(StandardRepr, bool)>`，输入须满足 standard 且 delta-fixed，前置条件通过 `debug_assert` 检查。`scaled_extended_finalise(ctx, sr, factor_num, factor_den)` 返回 `(StandardRepr, bool)`，缩放 $\nu$ 并保持 $\lambda$ 固定。^[ext-param.md:59-62]

## 与扩展块符号调校的衔接

参数层提供两个[[扩展块符号调校的 StarOracle 实现|StarOracle 实现]]。`ExtParamOracle` 服务于 `ExtBlock::tune_signs`，通过 `ext_param::def_ext` 重建每个父块元素的默认扩展；`PartialBlockOracle` 以 `PartialBlock` 为父块后端，用于 `ExtBlock::build_partial` 之后的 `tune_signs`。^[ext-param.md:64-68]

## 算术与错误约定

`star` 所在模块遵循[[扩展参数层的 Rust 移植约定]]：所有 `Weight`、`Coweight` 与 `int` 算术采用二进制补码 wrapping i32，以匹配上游 `int` 算术；有理权分子保持 i64。上游 `assert` 转为 `debug_assert` 或仅在调试模式执行的 `validate`，真正的数据相关失败通过 [[StructureError 统一错误分类学|StructureError]] 暴露。^[ext-param.md:28-31]

## 证据范围

本页依据对 `ext_param.rs` 的结构性阅读。所读字节来自 dirty 工作区，由 `snapshots/2026-10-03-ext-param.json` 记录；模块正确性属于其自身的 HPC 证据链，来源不重述或扩展该证据链。^[ext-param.md:9-15]

来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。上游行号转述自源码注释，未经独立重读上游核对，可能随版本演进而漂移。^[ext-param.md:70-78]

## Sources

- [ext-param.md](../../sources/ext-param.md) — ext_param/star 层：扩展块的参数层。
