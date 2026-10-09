---
title: 扩展参数的 finalisation 驱动
summary: 三个驱动通过队列重放折叠轨道反射与 star 下降并跟踪净翻转，缩放版本改变 ν 而保持 λ 固定。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:47:11.864Z"
updatedAt: "2026-10-09T19:28:01.193Z"
tags:
  - 扩展参数
  - 参数终结化
aliases:
  - 扩展参数的-finalisation-驱动
  - 扩F驱
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 扩展参数的 finalisation 驱动

扩展参数层提供三个 finalisation 驱动：`extended_restrict_to_k`、`extended_finalise` 与 `scaled_extended_finalise`。它们属于 `ext_param.rs` 对上游 `gkmod/ext_block.cpp` 参数层的移植，与 [[扩展参数值类型 ExtParam]]、[[扩展表示上下文 ExtRepContext]] 及 `star` 运算共同构成该层的实现。^[ext-param.md:19-26]

## 队列处理与翻转跟踪

三个驱动通过队列循环重放 folded-orbit 反射与 `star` 下降，并跟踪相对默认扩展的净翻转。这里的翻转以默认扩展为参照，相关构造见 [[扩展参数的默认扩展]]。^[ext-param.md:56-62]

`star(ctx, e, length, n_alpha)` 返回 `(DescValue, Vec<ExtParam>)`，给出根编号 `n_alpha` 的 delta-轨道类型及邻接扩展参数，供驱动执行下降步骤。具体运算见 [[扩展参数的 star 运算]]。^[ext-param.md:56-58]

## 接口与前置条件

`extended_finalise(ctx, sr)` 返回 `Vec<(StandardRepr, bool)>`，将 [[StandardRepr 标准表示参数]] 与翻转标记配对。输入须满足 standard 且 delta-fixed，这些前置条件通过 `debug_assert` 检查。^[ext-param.md:58-61]

`scaled_extended_finalise(ctx, sr, factor_num, factor_den)` 返回单个 `(StandardRepr, bool)`。其参数变换缩放 $\nu$，同时保持 $\lambda$ 固定。^[ext-param.md:61-62]

`extended_restrict_to_k` 对应上游 `ext_block.cpp:2435-2547`，同样列为参数层 finalisation 驱动；来源未进一步给出其完整签名、返回类型或独立前置条件。^[ext-param.md:25-26]

## 算术与失败处理

这些驱动所在模块遵循统一的 [[扩展参数层的 Rust 移植约定]]：`Weight`、`Coweight` 和 `int` 算术采用二进制补码 wrapping i32，有理权分子保持 i64。上游 `assert` 转为 `debug_assert` 或仅在调试模式执行的 `validate`，对应上游在 `NDEBUG` 下消除断言的行为；真正的数据相关失败通过 `StructureError` 暴露。^[ext-param.md:28-31]

## 证据范围

本页依据参数层的结构性阅读材料，所读源码来自 dirty 工作区快照。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；模块正确性属于其自身的 [[HPC 验收证据链]]，包括 unitarity gate 等。^[ext-param.md:9-15, ext-param.md:78-78]

来源中的上游行号转述自源码注释，未独立重读上游文件，可能随版本演进而漂移。^[ext-param.md:74-75]

## Sources

- [ext-param.md](ext-param.md) — ext_param/star 层：扩展块的参数层。
