---
title: Weyl 词与单生成元的整数校验差异
summary: check_weyl_word 逐项检查整数、负号、u64 收窄及半单秩界，而 check_weyl_generator 先收窄到 i32 再检查范围，两者保留不同诊断措辞与失败顺序。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:35.754Z"
updatedAt: "2026-10-09T20:33:35.754Z"
tags:
  - Weyl群
  - 输入校验
  - 诊断兼容
aliases:
  - weyl-词与单生成元的整数校验差异
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# Weyl 词与单生成元的整数校验差异

`domain_builtins.rs` 对 Weyl 词条目与单生成元采用不同的整数校验顺序：`check_weyl_word` 先判断负数，再收窄到 `u64`，最后检查半单秩边界；`check_weyl_generator` 则先收窄到上游机器 `int` 对应的 `i32`，再检查生成元范围。这一区别影响负数和大整数触发的诊断。^[atlas-core-domain-seams.md:96-107]

## Weyl 词条目的校验

`check_weyl_word` 对每个条目先调用 `as_integer`，随后依次检查：负数报 `Negative integer where unsigned is required`；无法收窄到 `u64` 时，报 `Integer value to big for conversion`；成功收窄但条目 \(i\) 满足 \(i \ge r\) 时，报 `Illegal Weyl word entry {i} (should be <{r})`，其中 \(r\) 为半单秩。这里的 `to big` 是源码措辞，应原样保留。相关顺序见 [[Weyl 词条目的有序校验]]。^[atlas-core-domain-seams.md:96-102]

这一助手由 `validate` 的词校验路径消费，源码位置为 10106。源码注释引用 `atlas-types.w:2344-2359`，但来源未独立重读该上游区间。^[atlas-core-domain-seams.md:96-102, atlas-core-domain-seams.md:115-116]

## 单生成元的校验

`check_weyl_generator` 首先将输入收窄到 `i32`，然后执行有符号范围检查。对于已经成功表示为 `i32` 的负数，`usize::try_from` 转换失败会触发 `Generator {g} out of range for Weyl group (should be <{r})`；它不会使用词条目专用的负数诊断。^[atlas-core-domain-seams.md:103-107]

因此，两条路径的机器整数边界也不同：词条目使用 `u64` 收窄，单生成元使用 `i32` 收窄，且两者都在收窄后检查半单秩范围。来源没有给出单生成元在 `i32` 收窄失败时的具体诊断文案，不能将词条目的转换错误文案直接套用到该路径。^[atlas-core-domain-seams.md:96-107]

单生成元助手由 `validate` 的生成元校验路径消费，源码位置为 10097；其注释引用 `atlas-types.w:2447-2476`。^[atlas-core-domain-seams.md:103-107]

## 与实形式根下标校验的区别

另一个助手 `check_generator` 按实形式上下文的半单秩检查下标，错误文案为 `Illegal root index: {g}`。其注释说明该措辞沿用上游 `get_reflection_index` 的用户下标表达；`posroot`／负下标处理仍是记录在案的 phase-1 暂缓事项。该助手应与上述 Weyl 单生成元校验区分。^[atlas-core-domain-seams.md:108-111]

## 证据边界

本页依据当前工作区字节的结构性阅读，不声称语言或数学验收。上游 C++／CWEB 行号均转述自 Rust 源码注释，可能随版本漂移；校验助手的诊断措辞仍以 HPC 语料门为行为权威，参见 [[HPC 验收证据链]]。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:113-118]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词／生成元校验。
