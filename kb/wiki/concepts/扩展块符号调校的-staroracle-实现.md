---
title: 扩展块符号调校的 StarOracle 实现
summary: ExtParamOracle 重建父块元素的默认扩展，PartialBlockOracle 使用部分父块，两者均为 tune_signs 提供参数层支持。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:47:19.857Z"
updatedAt: "2026-10-09T19:28:09.595Z"
tags:
  - 扩展块
  - 符号调校
aliases:
  - 扩展块符号调校的-staroracle-实现
  - 扩S实
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 扩展块符号调校的 StarOracle 实现

`ext_param` 参数层提供两个 `StarOracle` 实现：`ExtParamOracle` 与 `PartialBlockOracle`。二者均服务于扩展块的 `tune_signs` 符号调校，分别涉及父块元素默认扩展的重建，以及以 `PartialBlock` 为后端的部分扩展块调校。^[ext-param.md:64-68]

## 两种实现与调用位置

`ExtParamOracle` 服务于 `ExtBlock::tune_signs`，通过 `ext_param::def_ext` 重建每个父块元素的默认扩展。其参数来源与[[扩展参数的默认扩展]]直接相关。^[ext-param.md:66-67]

`PartialBlockOracle` 以 `PartialBlock` 父块为后端，用于 `ExtBlock::build_partial` 之后的 `tune_signs`，即在部分扩展块构建完成后进行符号调校。相关父块结构见[[公共块的构造与元素编号（PartialBlock）]]。^[ext-param.md:67-68]

## 参数层基础

[[扩展表示上下文 ExtRepContext]] 由 twisting involution `delta` 扩展 `RepContext`。其中 `delta` 以根系置换表示，并附有不动根集与诱导的单生成元 twist。^[ext-param.md:33-38]

[[扩展参数值类型 ExtParam]] 保存 Weyl 元素 `tw`、余权 `l`、有理权 `gamma_lambda`、权 `tau`、余权 `t` 与翻转位。其 `at` 构造给出 KGB 元素 `x` 处的默认扩展；默认扩展一族还包括 `default_extend`、`default_extend_srm`、`shifted_default_extension` 与 `is_default`。^[ext-param.md:40-52]

同一参数层中的[[扩展参数的 star 运算]]采用接口 `star(ctx, e, length, n_alpha)`，返回 `(DescValue, Vec<ExtParam>)`，分别表示根编号 `n_alpha` 的 delta-轨道类型与邻接扩展参数。^[ext-param.md:54-58]

## 实现约定

参数层使用二进制补码 wrapping i32 处理 `Weight`、`Coweight` 与 `int` 算术，以匹配上游 `int` 算术；有理权分子保持 i64。上游 `assert` 转为 `debug_assert` 或仅调试启用的 `validate`，真正的数据相关失败则通过 `StructureError` 暴露。这些是两个 oracle 所在模块的移植约定，参见[[扩展参数层的 Rust 移植约定]]。^[ext-param.md:28-31]

## 证据边界

来源明确记载了两个实现的后端与调用位置，但未展开 `tune_signs` 内部的符号调校规则。该材料属于结构性源码阅读，读取的是 dirty 工作区快照；参数层的正确性归属其自身的 [[HPC 验收证据链]]，来源不重述或扩展该证据。^[ext-param.md:9-15, ext-param.md:64-68]

本次来源整理未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。上游行号转述自源码注释，未独立重读上游，可能随版本演进发生漂移。^[ext-param.md:70-78]

## Sources

- [ext-param.md](ext-param.md)：ext_param/star 层：扩展块的参数层。
