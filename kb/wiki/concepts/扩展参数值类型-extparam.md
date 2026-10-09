---
title: 扩展参数值类型 ExtParam
summary: 保存 Weyl 元素、权与余权、有理权及翻转位，支持由 (tw, l mod 2) 重建 KGB 元素并恢复限制参数。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:46:56.920Z"
updatedAt: "2026-10-09T22:28:43.224Z"
tags:
  - 表示论
  - 值类型
aliases:
  - 扩展参数值类型-extparam
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 扩展参数值类型 ExtParam
summary: 保存 Weyl 元素、权与余权、有理权及翻转位，支持默认扩展构造、KGB 元素重建与限制操作，并参与 star 和 finalisation。
sources:
  - ext-param.md
kind: concept
tags:
  - 表示论
  - 参数表示
aliases:
  - 扩展参数值类型-extparam
provenanceState: extracted
---

# 扩展参数值类型 ExtParam

`ExtParam` 是扩展块参数层的值类型，移植自上游 `ext_block.h:293-364` 与 `ext_block.cpp:2283-2420`。它保存扩展参数，提供默认扩展构造、KGB 元素重建与限制操作，也是 `star` 运算返回的邻接参数类型。^[ext-param.md:19-24, ext-param.md:42-58]

## 数据与派生操作

`ExtParam` 包含 Weyl 元素 `tw`、余权 `l`（`Coweight`）、有理权 `gamma_lambda`（`RationalWeight`）、权 `tau`（`Weight`）、余权 `t`（`Coweight`）以及翻转位。^[ext-param.md:42-44]

派生操作包括 `theta(ctx)`、`theta_id(ctx)`、`x(ctx)`、`restrict_mod` 与 `restrict`。其中，`x(ctx)` 由 `(tw, l mod 2)` 重建 KGB 元素。^[ext-param.md:45-47]

相关的[[扩展表示上下文 ExtRepContext]] 由 twisting involution `delta` 扩展 `RepContext`；`delta` 以根系置换表示，并附有不动根集与诱导的单生成元 twist。^[ext-param.md:35-38]

## 默认扩展

`at` 在 KGB 元素 `x` 处构造默认扩展。[[扩展参数的默认扩展]]一族还包括 `default_extend`、`default_extend_srm`、`shifted_default_extension` 与 `is_default`；其中，`default_extend_srm` 要求 `gamma_lambda` 已在 `x` 处满足 `real_unique`。^[ext-param.md:44-52]

## star 与 finalisation

[[扩展参数的 star 运算]] `star(ctx, e, length, n_alpha)` 返回 `(DescValue, Vec<ExtParam>)`，分别给出根编号 `n_alpha` 的 `delta` 轨道类型与邻接扩展参数。^[ext-param.md:56-58]

三个[[扩展参数的 finalisation 驱动]]是 `extended_restrict_to_k`、`extended_finalise` 与 `scaled_extended_finalise`。它们的队列循环重放 folded-orbit 反射与 `star` 下降，并跟踪相对默认扩展的净翻转。^[ext-param.md:25-26, ext-param.md:58-59]

`extended_finalise(ctx, sr)` 返回 `Vec<(StandardRepr, bool)>`，其 standard 且 delta-fixed 的前置条件通过 `debug_assert` 检查。`scaled_extended_finalise(ctx, sr, factor_num, factor_den)` 返回 `(StandardRepr, bool)`，缩放 $\nu$ 而保持 $\lambda$ 固定。^[ext-param.md:59-62]

在[[扩展块符号调校的 StarOracle 实现]]中，`ExtParamOracle` 服务于 `ExtBlock::tune_signs`，通过 `ext_param::def_ext` 重建每个父块元素的默认扩展；`PartialBlockOracle` 以 `PartialBlock` 为后端，用于 `ExtBlock::build_partial` 之后的符号调校。^[ext-param.md:64-68]

## 算术与错误约定

本参数层的 `Weight`、`Coweight` 与 `int` 算术采用二进制补码 wrapping `i32`，以匹配上游 `int` 算术；有理权分子保持 `i64`。上游 `assert` 条件转为 `debug_assert` 或仅在调试模式运行的 `validate`，真正的数据相关失败通过 [[StructureError 统一错误分类学|StructureError]] 暴露。相关背景见[[扩展参数层的 Rust 移植约定]]。^[ext-param.md:28-31]

## 证据范围

来源是对 `ext_param.rs` 的结构性阅读，草稿经维护者逐条对照源码核对后改写。所读字节记录在 `snapshots/2026-10-03-ext-param.json`，对应 dirty 工作区；该材料不重述或扩展模块自身的 HPC 正确性证据链。^[ext-param.md:9-15]

上游文件行号转述自源码注释，未独立重读上游，可能随版本演进漂移。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[ext-param.md:72-78]

## Sources

- [ext-param.md](../../sources/ext-param.md) — ext_param/star 层：扩展块的参数层。
