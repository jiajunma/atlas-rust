---
title: StarOracle 注入与符号调整调试门
summary: tune_signs 通过 StarOracle 注入后续切片的 star 与 ext_param 计算，并在 debug_assertions 下运行 check_quadratic 和 check_braid 调试验证。
sources:
  - extended-block.md
kind: concept
createdAt: "2026-10-09T14:47:49.118Z"
updatedAt: "2026-10-09T14:47:49.118Z"
tags:
  - 符号调整
  - Rust设计
  - 调试验证
aliases:
  - staroracle-注入与符号调整调试门
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# StarOracle 注入与符号调整调试门

`StarOracle` 是扩展块符号调整所使用的 trait 注入接口。`ExtBlock::tune_signs` 通过该接口接入逐生成元的 `star` 计算及其比较所需的 `ext_param` 值，使扩展块结构切片能够调用属于后续 `ext_param`/`star` 切片的功能。^[extended-block.md:63-69]

## 注入边界

[[扩展块与 δ-不动部分|扩展块]]是普通块的 $\delta$-不动部分，其生成元折叠进 $\delta$-轨道。`ext_block.rs` 的来源切片覆盖扩展块结构，而 `ext_param` 与 `star` 经 `StarOracle` trait 注入；元素编号、cross/Cayley 语义以及以 `None` 表示的 `UndefBlock` 编码仍沿用完整块图约定。^[extended-block.md:19-22]

`ExtBlock::tune_signs` 是上游 `ext_block.cpp:1707-1876` 的泛型移植。该边界将符号调整流程与具体的[[扩展参数的 star 运算|star 运算]]及扩展参数计算衔接起来；来源包未展开这些后续切片的实现细节。^[extended-block.md:65-69, extended-block.md:86-89]

## 符号调整中的调试验证

`check_quadratic` 与 `check_braid` 两个调试验证门已移植为同名项，对应上游 `ext_block.cpp:2140-2245`。它们在 `tune_signs` 内部受 `debug_assertions` 控制运行，对应上游的 `#ifndef NDEBUG` 块，因此其执行条件属于调试构建约定。^[extended-block.md:67-69]

## 证据与限制

本页依据对 `ext_block.rs` 的结构性阅读，所读字节来自 dirty 工作区，并由来源包所链接的快照记录。扩展块正确性属于独立的 [[HPC 验收证据链]]，包括 ext-KL/unitarity gate；本来源包不重述或扩展这些验收结论。^[extended-block.md:9-15]

来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。本来源包也未执行构建、测试或原版运行，因此上述内容说明接口边界与调试门的实现安排，不构成数学验收、性能或并行结论。^[extended-block.md:84-90]

## Sources

- [extended-block.md](extended-block.md) — 扩展块：delta-不动部分与折叠生成元。
