---
title: 扩展参数层的 Rust 移植约定
summary: 整数权算术采用 wrapping i32、有理权分子保持 i64，上游断言转为调试检查，数据相关失败通过 StructureError 返回。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:46:46.011Z"
updatedAt: "2026-10-09T19:28:11.366Z"
tags:
  - Rust
  - 移植兼容性
  - 错误处理
aliases:
  - 扩展参数层的-rust-移植约定
  - 扩R移
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 扩展参数层的 Rust 移植约定

扩展参数层 `ext_param.rs` 移植上游 `gkmod/ext_block.cpp` 的参数层，采用两条保真约定：使用固定宽度回绕算术保持上游整数运算语义，并将调试断言与数据相关失败分开处理。其覆盖范围包括 [[扩展表示上下文 ExtRepContext]]、[[扩展参数值类型 ExtParam]]、比较与对齐辅助函数、共轭词构造、复根 cross 作用、[[扩展参数的 star 运算]]及三个 finalisation 驱动。^[ext-param.md:19-31]

## 整数算术约定

所有 `Weight`、`Coweight` 和 `int` 算术使用二进制补码 wrapping `i32`，以匹配上游 `int` 算术；有理权分子则保持 `i64`。因此，整数权与余权运算的宽度约定和有理权分子的表示宽度有所区别。^[ext-param.md:28-29]

## 断言与错误边界

上游 `assert` 条件转换为 Rust 的 `debug_assert`，或仅在调试模式启用的 `validate`，对应上游在 `NDEBUG` 下编译时消除断言的行为。真正的数据相关失败通过 [[StructureError 统一错误分类学|StructureError]] 暴露。^[ext-param.md:29-31]

`extended_finalise(ctx, sr)` 是这一约定的具体应用：输入须满足 standard 且 delta-fixed，前置条件通过 `debug_assert` 检查，返回值为 `Vec<(StandardRepr, bool)>`。相关流程见 [[扩展参数的 finalisation 驱动]]。^[ext-param.md:59-61]

## 移植覆盖范围

比较与对齐辅助函数包括 `same_standard_reps`、`same_sign`、`z_align` 和 `level_a`；此外还移植了共轭词构造 `fixed_conjugate_simple`、复根 cross 作用 `complex_cross` 及核心 `star` 计算。参数层之上提供 `extended_restrict_to_k`、`extended_finalise` 和 `scaled_extended_finalise` 三个 finalisation 驱动。^[ext-param.md:19-26]

`star(ctx, e, length, n_alpha)` 返回 `(DescValue, Vec<ExtParam>)`，表示根编号 `n_alpha` 的 delta-轨道类型与邻接扩展参数。三个驱动通过队列循环重放 folded-orbit 反射与 `star` 下降，并跟踪相对默认扩展的净翻转；其中 `scaled_extended_finalise` 缩放 $\nu$，同时保持 $\lambda$ 固定。^[ext-param.md:56-62]

## 证据边界

上述约定来自结构性源码阅读，所读 `ext_param.rs` 字节取自 dirty 工作区，记录于快照 `snapshots/2026-10-03-ext-param.json`。来源材料不重述或扩展参数层自身的 [[HPC 验收证据链]]，包括 unitarity gate 等正确性证据。^[ext-param.md:9-15]

上游文件的行号转述自源码注释，未独立重读上游，可能随版本演进漂移。来源材料未执行构建、测试或原版运行，因此不包含数学验收、性能或并行结论。^[ext-param.md:74-78]

## Sources

- [ext-param.md](ext-param.md) — ext_param/star 层：扩展块的参数层。
