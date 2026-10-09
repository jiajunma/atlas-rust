---
title: 扩展块符号调校的 StarOracle 实现
summary: ExtParamOracle 重建父块元素的默认扩展，PartialBlockOracle 使用部分父块后端，两者为 ExtBlock::tune_signs 提供参数层支持。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:47:19.857Z"
updatedAt: "2026-10-09T22:29:00.475Z"
tags:
  - 扩展块
  - 符号调校
  - 接口适配
aliases:
  - 扩展块符号调校的-staroracle-实现
  - 扩S实
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 扩展块符号调校的 StarOracle 实现
summary: ExtParamOracle 重建父块元素的默认扩展，PartialBlockOracle 以部分父块为后端，两者支持扩展块的 tune_signs 符号调校。
sources:
  - ext-param.md
kind: concept
tags:
  - 扩展块
  - 符号调校
  - 接口适配
---

# 扩展块符号调校的 StarOracle 实现

`ext_param` 参数层提供两个 `StarOracle` 实现：`ExtParamOracle` 与 `PartialBlockOracle`。两者均用于扩展块的 `tune_signs` 符号调校，来源分别说明了默认扩展的重建方式，以及部分父块后端的使用位置。^[ext-param.md:64-68]

## 两种实现

`ExtParamOracle` 服务于 `ExtBlock::tune_signs`，通过 `ext_param::def_ext` 重建每个父块元素的默认扩展。相关构造见 [[扩展参数的默认扩展]]。^[ext-param.md:66-67]

`PartialBlockOracle` 以 `PartialBlock` 父块为后端，用于 `ExtBlock::build_partial` 之后的 `tune_signs`。其调用位置是部分扩展块构建完成后的符号调校阶段；父块结构可参见 [[公共块的构造与元素编号（PartialBlock）]]。^[ext-param.md:67-68]

## 参数层基础

[[扩展表示上下文 ExtRepContext]] 由 twisting involution `delta` 扩展 `RepContext`。其中 `delta` 以根系置换表示，并附有不动根集与诱导的单生成元 twist。^[ext-param.md:33-38]

[[扩展参数值类型 ExtParam]] 保存 Weyl 元素 `tw`、余权 `l`、有理权 `gamma_lambda`、权 `tau`、余权 `t` 与翻转位。`at` 在 KGB 元素 `x` 处构造默认扩展；默认扩展一族还包括 `default_extend`、`default_extend_srm`、`shifted_default_extension` 与 `is_default`，其中 `default_extend_srm` 要求 `gamma_lambda` 已在 `x` 处满足 `real_unique`。^[ext-param.md:40-52]

同一参数层中的 [[扩展参数的 star 运算]] 采用接口 `star(ctx, e, length, n_alpha)`，返回 `(DescValue, Vec<ExtParam>)`，表示根编号 `n_alpha` 的 delta-轨道类型与邻接扩展参数。^[ext-param.md:56-58]

## 实现约定

两个 oracle 所在模块以二进制补码 wrapping i32 处理 `Weight`、`Coweight` 与 `int` 算术，以匹配上游 `int` 算术；有理权分子保持 i64。上游 `assert` 转为 `debug_assert` 或仅在调试模式启用的 `validate`，真正的数据相关失败通过 `StructureError` 暴露。详见 [[扩展参数层的 Rust 移植约定]]。^[ext-param.md:28-31]

## 证据边界

来源对两个 oracle 的说明限于默认扩展重建、后端与调用位置，未展开 `tune_signs` 内部的符号调校规则。材料属于结构性源码阅读，所读证据为 dirty 工作区的 `ext_param.rs` 字节；参数层正确性归属其自身的 HPC 证据链，本材料不重述或扩展该证据。^[ext-param.md:9-15, ext-param.md:64-68]

来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。所列上游行号转述自源码注释，未独立重读上游文件，可能随版本演进发生漂移。^[ext-param.md:70-78]

## Sources

- [ext-param.md](../../sources/ext-param.md)：ext_param/star 层：扩展块的参数层。
