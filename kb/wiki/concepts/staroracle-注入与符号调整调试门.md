---
title: StarOracle 注入与符号调整调试门
summary: tune_signs 通过 StarOracle 注入 star 与扩展参数计算，仅在 debug_assertions 下运行 check_quadratic 和 check_braid。
sources:
  - extended-block.md
kind: concept
createdAt: "2026-10-09T14:47:49.118Z"
updatedAt: "2026-10-09T20:52:31.479Z"
tags:
  - 符号调整
  - 依赖注入
  - 调试验证
aliases:
  - staroracle-注入与符号调整调试门
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: StarOracle 注入与符号调整调试门
summary: ExtBlock::tune_signs 通过 StarOracle 接入逐生成元的 star 计算及扩展参数值，并在 debug_assertions 下运行 check_quadratic 与 check_braid。
sources:
  - extended-block.md
kind: concept
tags:
  - 符号调整
  - 调试验证
aliases:
  - staroracle-注入与符号调整调试门
---

# StarOracle 注入与符号调整调试门

`StarOracle` 是扩展块符号调整所使用的 trait 注入接口。`ExtBlock::tune_signs` 通过该接口接入逐生成元的 `star` 计算及比较所需的 `ext_param` 值，使扩展块结构切片能够衔接后续扩展参数功能。^[extended-block.md:65-69]

## 注入边界

[[扩展块与 δ-不动部分|扩展块]]是普通块的 $\delta$-不动部分，其生成元折叠进 $\delta$-轨道。来源材料覆盖 `ext_block.rs` 的结构切片；`ext_param` 与 [[扩展参数的 star 运算|star 运算]]属于后续切片，在此经 `StarOracle` trait 注入。元素编号、cross/Cayley 语义及以 `None` 表示的 `UndefBlock` 编码沿用完整块图约定。^[extended-block.md:19-22]

`ExtBlock::tune_signs` 是上游 `ext_block.cpp:1707-1876` 的泛型移植。来源说明了注入接口与符号调整的关系，但未展开具体 `ext_param`、`StarOracle` 或 `ext_kl` 实现；这些内容属于后续来源包。^[extended-block.md:65-69, extended-block.md:86-89]

## 调试验证门

`check_quadratic` 与 `check_braid` 已移植为同名调试验证项，对应上游 `ext_block.cpp:2140-2245`。两者在 `tune_signs` 内部于 `debug_assertions` 条件下运行，与上游的 `#ifndef NDEBUG` 块对应。因此，这些验证项的执行受调试断言配置控制。^[extended-block.md:67-69]

## 证据与限制

上述说明依据对 `ext_block.rs` 的结构性阅读，所读字节来自 dirty 工作区，由来源包的阅读快照记录。扩展块正确性另属其 HPC 证据链，包括 ext-KL/unitarity gate；本来源包不重述或扩展这些验收结论。^[extended-block.md:9-15]

来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。本来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[extended-block.md:84-90]

## Sources

- [extended-block.md](../../sources/extended-block.md) — 扩展块：delta-不动部分与折叠生成元。
