---
title: 扩展表示上下文 ExtRepContext
summary: ExtRepContext 以 twisting involution delta 扩展 RepContext，提供根系置换、不动根、诱导 twist 及移位翻转等判定。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:46:46.972Z"
updatedAt: "2026-10-09T14:46:46.972Z"
tags:
  - 表示论
  - 参数上下文
  - 根系
aliases:
  - 扩展表示上下文-extrepcontext
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 扩展表示上下文 ExtRepContext

`ExtRepContext` 是由 twisting involution `delta` 扩展的 [[RepContext 借用上下文与一致性约束|RepContext]]，属于 `ext_param.rs` 的扩展参数层。其上游对应类型为 `repr::Ext_rep_context`，来源包标注的实现位置为 `repr.h:682-714` 与 `repr.cpp:2786-2836`。^[ext-param.md:19-24, ext-param.md:33-38]

## 数据表示与接口

`delta` 以根系置换表示；上下文还附带不动根集，以及由该对合诱导的单生成元 twist。相关访问器包括 `rc()`、`delta()`、`delta_of`、`is_delta_fixed_root` 和 `twisted`，用于访问表示上下文及其扩展结构。^[ext-param.md:35-38]

高级判定接口包括 `to_simple_shift`、`is_very_complex` 和 `shift_flip`。来源包分别将它们对应到上游 `repr.h:706-708`、`repr.cpp:2804-2813` 和 `repr.cpp:2824-2836`，但未展开各判定的公式或算法步骤。^[ext-param.md:35-38]

## 在扩展参数计算中的作用

[[扩展参数值类型 ExtParam|ExtParam]] 的派生计算通过上下文参数 `ctx` 进行，包括 `theta(ctx)`、`theta_id(ctx)` 和 `x(ctx)`；其中 `x(ctx)` 由 `(tw, l mod 2)` 重建 KGB 元素。这些接口将扩展参数值与上下文中的根理论结构联系起来。^[ext-param.md:40-47]

[[扩展参数的 star 运算|star]] 以 `star(ctx, e, length, n_alpha)` 为入口，返回 `(DescValue, Vec<ExtParam>)`，给出根编号 `n_alpha` 的 delta-轨道类型与邻接扩展参数。[[扩展参数的 finalisation 驱动|finalisation 驱动]] 的队列循环重放 folded-orbit 反射与 `star` 下降，并跟踪相对默认扩展的净翻转。^[ext-param.md:54-62]

`extended_finalise(ctx, sr)` 返回 `Vec<(StandardRepr, bool)>`，其 standard 且 delta-fixed 的前置条件通过 `debug_assert` 检查；`scaled_extended_finalise(ctx, sr, factor_num, factor_den)` 返回 `(StandardRepr, bool)`，缩放 $\nu$ 而保持 $\lambda$ 固定。^[ext-param.md:59-62]

## 移植约定与证据边界

本模块的 `Weight`、`Coweight` 与 `int` 算术采用二进制补码 wrapping i32，有理权分子保持 i64。上游 `assert` 转为 `debug_assert` 或仅在 debug 模式执行的 `validate`；真正的数据相关失败通过 [[StructureError 统一错误分类学|StructureError]] 暴露。这些是模块级移植约定。^[ext-param.md:28-31]

来源包属于结构性源码阅读，记录的是 dirty 工作区中的源码字节。上游位置来自源码注释，未独立重读上游，行号可能随版本漂移；该包没有执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论。^[ext-param.md:9-15, ext-param.md:70-78]

## Sources

- [ext-param.md](ext-param.md) — ext_param/star 层：扩展块的参数层。
