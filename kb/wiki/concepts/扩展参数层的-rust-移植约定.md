---
title: 扩展参数层的 Rust 移植约定
summary: Weight、Coweight 与 int 使用 wrapping i32 算术，有理权分子保留 i64，上游断言转为调试检查，数据相关失败通过 StructureError 暴露；本包仅提供结构性阅读证据。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:46:46.011Z"
updatedAt: "2026-10-09T22:29:07.431Z"
tags:
  - Rust
  - 移植契约
  - 证据边界
aliases:
  - 扩展参数层的-rust-移植约定
  - 扩R移
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 扩展参数层的 Rust 移植约定
summary: 扩展参数层使用 wrapping i32 整数算术，有理权分子保持 i64；上游断言转为调试检查，数据相关失败通过 StructureError 暴露。来源仅提供结构性阅读证据。
sources:
  - ext-param.md
kind: concept
tags:
  - Rust移植
  - 整数算术
  - 错误处理
  - 证据边界
---

# 扩展参数层的 Rust 移植约定

`ext_param.rs` 移植上游 `gkmod/ext_block.cpp` 的参数层，采用两条保真约定：明确整数算术的宽度与回绕行为，以及区分调试断言和数据相关失败。移植范围涵盖 [[扩展表示上下文 ExtRepContext]]、[[扩展参数值类型 ExtParam]]、比较与对齐辅助函数、共轭词构造、复根 cross 作用、[[扩展参数的 star 运算]]及三个 finalisation 驱动。^[ext-param.md:19-31]

## 整数算术

所有 `Weight`、`Coweight` 和 `int` 算术使用二进制补码 wrapping `i32`，以匹配上游 `int` 算术；有理权分子保持 `i64`。这两种宽度是参数层明确区分的移植约定。^[ext-param.md:28-29]

## 断言与错误边界

上游 `assert` 条件转换为 Rust 的 `debug_assert`，或仅在调试模式启用的 `validate`，对应上游在 `NDEBUG` 下编译时消除断言的行为。真正的数据相关失败通过 [[StructureError 统一错误分类学|StructureError]] 暴露。^[ext-param.md:29-31]

`extended_finalise(ctx, sr)` 展示了这一检查约定：输入须满足 standard 且 delta-fixed，前置条件由 `debug_assert` 检查，返回值为 `Vec<(StandardRepr, bool)>`。相关流程见 [[扩展参数的 finalisation 驱动]]。^[ext-param.md:59-61]

## 移植范围与驱动行为

比较与对齐辅助函数包括 `same_standard_reps`、`same_sign`、`z_align` 和 `level_a`；其他移植内容包括共轭词构造 `fixed_conjugate_simple`、复根 cross 作用 `complex_cross` 和核心 `star` 计算。参数层还提供 `extended_restrict_to_k`、`extended_finalise` 与 `scaled_extended_finalise` 三个 finalisation 驱动。^[ext-param.md:19-26]

`star(ctx, e, length, n_alpha)` 返回 `(DescValue, Vec<ExtParam>)`，描述根编号 `n_alpha` 的 delta-轨道类型及邻接扩展参数。三个驱动通过队列循环重放 folded-orbit 反射与 `star` 下降，并跟踪相对默认扩展的净翻转。`scaled_extended_finalise(ctx, sr, factor_num, factor_den)` 返回 `(StandardRepr, bool)`，缩放 $\nu$ 而保持 $\lambda$ 固定。^[ext-param.md:56-62]

## 证据边界

本页依据结构性源码阅读材料。所读 `ext_param.rs` 字节来自 dirty 工作区，记录于快照 `snapshots/2026-10-03-ext-param.json`，SHA-256 为 `0eabb0696791338ceb0d55ed4408eb661e022d4cbc3728c40cc09af80b5c8029`。材料不重述或扩展参数层自身的 HPC 正确性证据链，包括 unitarity gate 等。^[ext-param.md:9-15]

上游文件行号转述自源码注释，未独立重读上游，可能随版本演进漂移。来源材料未执行构建、测试或原版运行，因此不包含数学验收、性能或并行结论。^[ext-param.md:74-78]

## Sources

- [ext-param.md](../../sources/ext-param.md) — ext_param/star 层：扩展块的参数层。
