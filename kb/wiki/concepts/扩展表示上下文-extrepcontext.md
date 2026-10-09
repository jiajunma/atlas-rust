---
title: 扩展表示上下文 ExtRepContext
summary: 以 twisting involution delta 扩展 RepContext，保存根置换、不动根集与生成元 twist，并提供移位和翻转判定。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:46:46.972Z"
updatedAt: "2026-10-09T20:51:40.436Z"
tags:
  - 表示论
  - Rust设计
aliases:
  - 扩展表示上下文-extrepcontext
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 扩展表示上下文 ExtRepContext
summary: 以 twisting involution delta 扩展 RepContext，保存根置换、不动根集和生成元 twist，并提供移位与翻转判定。
sources:
  - ext-param.md
kind: concept
tags:
  - 表示论
  - 扩展参数
aliases:
  - 扩展表示上下文-extrepcontext
---

# 扩展表示上下文 ExtRepContext

`ExtRepContext` 以扭曲对合（twisting involution）`delta` 扩展 [[RepContext 借用上下文与一致性约束|RepContext]]，属于 `ext_param.rs` 的扩展参数层。其上游对应类型为 `repr::Ext_rep_context`，来源标注的位置为 `repr.h:682-714` 与 `repr.cpp:2786-2836`。^[ext-param.md:19-24, ext-param.md:35-38]

## 数据表示与接口

`delta` 以根系置换表示；上下文还附带不动根集，以及由该对合诱导的单生成元 twist。访问器包括 `rc()`、`delta()`、`delta_of`、`is_delta_fixed_root` 和 `twisted`，提供基础上下文及扩展结构的访问。^[ext-param.md:35-38]

高级判定接口包括 `to_simple_shift`、`is_very_complex` 和 `shift_flip`。来源分别将其对应到上游 `repr.h:706-708`、`repr.cpp:2804-2813` 和 `repr.cpp:2824-2836`，但未展开判定公式或算法步骤。^[ext-param.md:35-38]

## 在扩展参数计算中的作用

[[扩展参数值类型 ExtParam|ExtParam]] 的派生计算使用上下文参数 `ctx`，包括 `theta(ctx)`、`theta_id(ctx)` 和 `x(ctx)`；其中 `x(ctx)` 由 `(tw, l mod 2)` 重建 KGB 元素。^[ext-param.md:42-47]

[[扩展参数的 star 运算|star]] 的入口为 `star(ctx, e, length, n_alpha)`，返回 `(DescValue, Vec<ExtParam>)`，给出根编号 `n_alpha` 的 delta-轨道类型与邻接扩展参数。[[扩展参数的 finalisation 驱动|finalisation 驱动]] 的队列循环重放 folded-orbit 反射与 `star` 下降，并跟踪相对默认扩展的净翻转。^[ext-param.md:56-59]

`extended_finalise(ctx, sr)` 返回 `Vec<(StandardRepr, bool)>`，其输入须为 standard 且 delta-fixed，这些前置条件通过 `debug_assert` 检查。`scaled_extended_finalise(ctx, sr, factor_num, factor_den)` 返回 `(StandardRepr, bool)`，缩放 $\nu$ 而保持 $\lambda$ 固定。^[ext-param.md:59-62]

## 移植约定

[[扩展参数层的 Rust 移植约定|模块级移植约定]]要求所有 `Weight`、`Coweight` 与 `int` 算术采用二进制补码 wrapping i32，以匹配上游 `int` 算术；有理权分子保持 i64。上游 `assert` 转为 `debug_assert` 或仅在 debug 模式执行的 `validate`，真正的数据相关失败通过 [[StructureError 统一错误分类学|StructureError]] 暴露。^[ext-param.md:28-31]

## 证据边界

来源属于结构性源码阅读，记录的是 dirty 工作区中的源码字节。参数层的正确性归于其自身的 HPC 证据链（如 unitarity gate），本来源不重述或扩展该证据链。^[ext-param.md:9-15]

所列上游位置均转述自源码注释，未独立重读上游，行号可能随版本演进而漂移。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[ext-param.md:70-78]

## Sources

- [ext-param.md](../../sources/ext-param.md) — ext_param/star 层：扩展块的参数层。
