---
title: 扩展参数层的 Rust 移植约定
summary: ext_param 层以 wrapping i32 保持整数算术语义、有理权分子保留 i64，并将上游断言映射为调试检查，将数据相关失败暴露为 StructureError。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:46:46.011Z"
updatedAt: "2026-10-09T14:46:46.011Z"
tags:
  - Rust移植
  - 算术语义
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 扩展参数层的 Rust 移植约定

扩展参数层 `ext_param.rs` 移植上游 `gkmod/ext_block.cpp` 的参数层，涵盖 [[扩展表示上下文 ExtRepContext]]、[[扩展参数值类型 ExtParam]]、比较与对齐辅助函数、共轭词构造、复根 cross 作用、[[扩展参数的 star 运算]]以及三个 finalisation 驱动。移植采用两条保真约定：保留上游整数算术语义，并区分调试断言与数据相关失败。^[ext-param.md:19-31]

## 整数算术约定

所有 `Weight`、`Coweight` 和 `int` 算术使用二进制补码 wrapping `i32`，以匹配上游 `int` 算术；有理权分子则保持 `i64`。这一约定明确区分整数权、余权运算与有理权分子的表示宽度。^[ext-param.md:28-29]

## 断言与错误边界

上游 `assert` 条件转换为 Rust 的 `debug_assert`，或仅在调试模式启用的 `validate`，对应上游在 `NDEBUG` 下消除断言的行为。真正的数据相关失败通过 [[StructureError 统一错误分类学|StructureError]] 暴露。^[ext-param.md:29-31]

具体而言，`extended_finalise(ctx, sr)` 的前置条件是输入为 standard 且 delta-fixed，这些条件通过 `debug_assert` 检查；其返回值为 `Vec<(StandardRepr, bool)>`。这是调试断言约定在[[扩展参数的 finalisation 驱动]]中的具体应用。^[ext-param.md:59-61]

## 移植覆盖范围

除上下文和值类型外，移植范围还包括 `same_standard_reps`、`same_sign`、`z_align`、`level_a` 等比较与对齐辅助函数，`fixed_conjugate_simple` 共轭词构造、`complex_cross` 复根 cross 作用，以及核心 `star` 计算。参数层之上提供 `extended_restrict_to_k`、`extended_finalise` 和 `scaled_extended_finalise` 三个 finalisation 驱动。^[ext-param.md:19-26]

`star(ctx, e, length, n_alpha)` 返回 `(DescValue, Vec<ExtParam>)`，表示根编号 `n_alpha` 的 delta-轨道类型及邻接扩展参数。三个 finalisation 驱动通过队列循环重放 folded-orbit 反射与 `star` 下降，并跟踪相对默认扩展的净翻转；`scaled_extended_finalise` 缩放 $\nu$，同时保持 $\lambda$ 固定。^[ext-param.md:56-62]

## 证据边界

本页依据结构性源码阅读材料，所读 `ext_param.rs` 字节来自 dirty 工作区，阅读快照为 `snapshots/2026-10-03-ext-param.json`。该材料不重述或扩展参数层自身的 HPC 正确性证据链。^[ext-param.md:9-15]

上游文件及行号来自 Rust 源码注释，未独立重读上游，可能随版本演进漂移。来源材料未执行构建、测试或原版运行，因此这些移植约定不构成数学验收、性能或并行结论。^[ext-param.md:74-78]

## Sources

- [ext-param.md](ext-param.md) — ext_param/star 层：扩展块的参数层。
