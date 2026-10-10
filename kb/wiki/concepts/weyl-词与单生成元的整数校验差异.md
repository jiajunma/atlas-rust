---
title: Weyl 词与单生成元的整数校验差异
summary: 词条目依次检查整数、负号、u64 收窄和半单秩界，单生成元先收窄至 i32 再检查范围，保留不同错误顺序与措辞。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:35.754Z"
updatedAt: "2026-10-10T01:44:26.043Z"
tags:
  - Weyl群
  - 输入校验
  - 诊断契约
aliases:
  - weyl-词与单生成元的整数校验差异
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Weyl 词与单生成元的整数校验差异

`domain_builtins.rs` 对 Weyl 词条目与单生成元采用不同的整数校验顺序：`check_weyl_word` 先提取整数、检查负号，再收窄到 `u64` 并检查半单秩边界；`check_weyl_generator` 则先收窄到 `i32`，再检查生成元范围。两条路径具有不同的整数表示边界、失败顺序与负数诊断。^[atlas-core-domain-seams.md:96-107]

## Weyl 词条目：先检查负号，再收窄

`check_weyl_word` 对每个条目先调用 `as_integer`。负数触发 `Negative integer where unsigned is required`；随后，若无法收窄到 `u64`，触发 `Integer value to big for conversion`；成功收窄后，若条目 $i \ge r$，则触发 `Illegal Weyl word entry {i} (should be <{r})`，其中 $r$ 为半单秩。详见 [[Weyl 词条目的有序校验]]。^[atlas-core-domain-seams.md:96-102]

转换错误中的 `to big` 是源码中的原有措辞，与别处的 `too big` 写法并存。该助手由 `validate` 的词校验路径调用，调用位置为 `domain_builtins.rs:10106`；源码注释引用 `atlas-types.w:2344-2359`。^[atlas-core-domain-seams.md:96-102]

## 单生成元：先收窄，再检查范围

`check_weyl_generator` 先把输入收窄到上游机器 `int` 对应的 `i32`，再做有符号范围检查。成功收窄的负数在 `usize::try_from` 处转换失败，落入 `Generator {g} out of range for Weyl group (should be <{r})` 诊断。因此，同一个可表示为 `i32` 的负整数，在词条目路径中得到无符号整数诊断，在单生成元路径中得到生成元越界诊断。^[atlas-core-domain-seams.md:96-107]

该助手由 `validate` 的生成元校验路径调用，调用位置为 `domain_builtins.rs:10097`；源码注释引用 `atlas-types.w:2447-2476`。来源未列出 `i32` 收窄失败时的具体诊断，因而不能据此将其文案等同于词条目的 `u64` 转换错误。^[atlas-core-domain-seams.md:103-107]

## 与实形式根下标校验的区别

另一个助手 `check_generator` 按实形式上下文的半单秩检查下标，错误文案为 `Illegal root index: {g}`。其注释说明，该措辞沿用上游 `get_reflection_index` 的用户下标表达，对应 `atlas-types.w:4481-4489`；`posroot`／负下标属于记录在案的 phase-1 暂缓事项。相关契约见 [[实形式生成元下标的校验契约]]。^[atlas-core-domain-seams.md:108-111]

## 证据边界

本页依据当前工作区字节的结构性阅读，不构成语言或数学验收。上游 C++／CWEB 行号均转述自 Rust 源码注释，未独立重读上游，可能随版本漂移；校验助手的诊断措辞以 HPC 语料门为行为权威，参见 [[HPC 验收证据链]]。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:115-118]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词／生成元校验。
