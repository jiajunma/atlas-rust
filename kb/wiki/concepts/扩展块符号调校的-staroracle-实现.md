---
title: 扩展块符号调校的 StarOracle 实现
summary: ExtParamOracle 通过重建父块元素的默认扩展服务 tune_signs，PartialBlockOracle 则以 PartialBlock 为后端支持部分扩展块构建后的符号调校。
sources:
  - ext-param.md
kind: concept
createdAt: "2026-10-09T14:47:19.857Z"
updatedAt: "2026-10-09T14:47:19.857Z"
tags:
  - 扩展块
  - 符号调校
  - 接口实现
aliases:
  - 扩展块符号调校的-staroracle-实现
  - 扩S实
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 扩展块符号调校的 StarOracle 实现

`ext_param` 参数层提供两个 `StarOracle` 实现：`ExtParamOracle` 与 `PartialBlockOracle`。它们用于扩展块的 `tune_signs` 符号调校，分别通过父块元素的默认扩展和 `PartialBlock` 后端提供支持。^[ext-param.md:64-68]

## 两种实现

`ExtParamOracle` 服务于 `ExtBlock::tune_signs`，每个父块元素的默认扩展通过 `ext_param::def_ext` 重建。这将符号调校与[[扩展参数的默认扩展]]联系起来。^[ext-param.md:66-67]

`PartialBlockOracle` 以 `PartialBlock` 父块为后端，用于 `ExtBlock::build_partial` 之后的 `tune_signs`。其使用顺序是先构建扩展块，再进行符号调校；父块结构可参见[[公共块的构造与元素编号（PartialBlock）]]。^[ext-param.md:67-68]

## 参数层基础

两种实现所在的参数层包含[[扩展表示上下文 ExtRepContext]]与[[扩展参数值类型 ExtParam]]。前者由 twisting involution `delta` 扩展 `RepContext`，保存根系置换、不动根集和诱导的单生成元 twist；后者保存 Weyl 元素、权与余权数据以及翻转位，并支持默认扩展构造。^[ext-param.md:33-52]

[[扩展参数的 star 运算]]的接口为 `star(ctx, e, length, n_alpha)`，返回 `(DescValue, Vec<ExtParam>)`，表示根编号 `n_alpha` 的 delta-轨道类型及邻接扩展参数。该返回形态是理解参数层 `star` 计算的基础。^[ext-param.md:56-58]

## 实现约定与证据边界

本模块使用二进制补码 wrapping i32 处理 `Weight`、`Coweight` 与 `int` 算术，有理权分子保持 i64。上游断言转换为 `debug_assert` 或仅调试启用的 `validate`，真正的数据相关失败通过 `StructureError` 暴露；这些约定适用于理解两个 oracle 所依赖的参数层。^[ext-param.md:28-31]

现有材料明确了两个实现的后端与调用位置，但没有展开 `tune_signs` 的内部调校规则。材料属于结构性源码阅读，所读字节来自 dirty 工作区；未执行构建、测试或原版运行，因此不构成数学验收、性能或并行行为的证据。上游位置也仅转述源码注释，未独立重读核实。^[ext-param.md:9-15, ext-param.md:64-78]

## Sources

- [ext-param.md](ext-param.md)：`ext_param/star 层：扩展块的参数层`。
