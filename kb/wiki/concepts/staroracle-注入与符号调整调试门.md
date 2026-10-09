---
title: StarOracle 注入与符号调整调试门
summary: tune_signs 通过 StarOracle 注入 star 与扩展参数计算，并仅在 debug_assertions 下运行 check_quadratic 和 check_braid 验证。
sources:
  - extended-block.md
kind: concept
createdAt: "2026-10-09T14:47:49.118Z"
updatedAt: "2026-10-09T19:28:36.913Z"
tags:
  - 符号调整
  - 调试验证
aliases:
  - staroracle-注入与符号调整调试门
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# StarOracle 注入与符号调整调试门

`StarOracle` 是扩展块符号调整所使用的 trait 注入接口。`ExtBlock::tune_signs` 通过该接口接入逐生成元的 `star` 计算及其比较所需的 `ext_param` 值，将扩展块结构与后续扩展参数功能衔接起来。^[extended-block.md:65-69]

## 注入边界

[[扩展块与 δ-不动部分|扩展块]]是普通块的 $\delta$-不动部分，其生成元折叠进 $\delta$-轨道。来源材料覆盖 `ext_block.rs` 的结构切片；`ext_param` 与 [[扩展参数的 star 运算|star 运算]]属于后续切片，在此通过 `StarOracle` trait 注入。元素编号、cross/Cayley 语义及以 `None` 表示的 `UndefBlock` 编码沿用完整块图约定。^[extended-block.md:19-22]

`ExtBlock::tune_signs` 是上游 `ext_block.cpp:1707-1876` 的泛型移植。来源说明了注入接口与符号调整流程的关系，但未展开具体 `ext_param`、`StarOracle` 或 `ext_kl` 的实现细节。^[extended-block.md:65-69, extended-block.md:86-89]

## 调试验证门

`check_quadratic` 与 `check_braid` 已移植为同名调试验证项，对应上游 `ext_block.cpp:2140-2245`。两者在 `tune_signs` 内部于 `debug_assertions` 条件下运行，与上游的 `#ifndef NDEBUG` 块对应；其执行受调试断言配置控制。^[extended-block.md:67-69]

## 证据与限制

上述说明依据对 `ext_block.rs` 的结构性阅读，所读源码来自 dirty 工作区，并由来源包的快照记录。扩展块的正确性另属 [[HPC 验收证据链]]，包括 ext-KL/unitarity gate；本来源包不重述或扩展这些验收结论。^[extended-block.md:9-15]

来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。本来源包未执行构建、测试或原版运行，因此只能支持接口边界与调试验证安排的说明，不提供数学验收、性能或并行结论。^[extended-block.md:84-90]

## Sources

- [extended-block.md](extended-block.md) — 扩展块：delta-不动部分与折叠生成元。
