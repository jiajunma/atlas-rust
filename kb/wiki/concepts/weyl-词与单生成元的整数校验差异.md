---
title: Weyl 词与单生成元的整数校验差异
summary: Weyl 词条目依次检查整数、负号、u64 收窄及半单秩界，单生成元则先收窄为 i32 再检查范围，二者保留不同诊断与失败顺序。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:35.754Z"
updatedAt: "2026-10-09T22:15:54.396Z"
tags:
  - Weyl群
  - 输入校验
  - 兼容性
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

---
title: Weyl 词与单生成元的整数校验差异
summary: check_weyl_word 依次检查整数、负号、u64 收窄及半单秩界；check_weyl_generator 先收窄到 i32 再检查范围，保留不同的失败顺序与诊断措辞。
sources:
  - atlas-core-domain-seams.md
kind: concept
tags:
  - Weyl群
  - 输入校验
  - 诊断兼容
aliases:
  - weyl-词与单生成元的整数校验差异
provenanceState: extracted
---

# Weyl 词与单生成元的整数校验差异

`domain_builtins.rs` 对 Weyl 词条目与单生成元采用不同的整数校验顺序。`check_weyl_word` 先提取整数、检查负号，再收窄到 `u64` 并检查半单秩边界；`check_weyl_generator` 则先收窄到上游机器 `int` 对应的 `i32`，再检查生成元范围。两条路径的整数表示边界及负数诊断不同。^[atlas-core-domain-seams.md:96-107]

## Weyl 词条目的校验

`check_weyl_word` 对每个条目先调用 `as_integer`。负数触发 `Negative integer where unsigned is required`；非负整数无法收窄到 `u64` 时，触发 `Integer value to big for conversion`；成功收窄后，若条目 \(i\) 满足 \(i \ge r\)，则触发 `Illegal Weyl word entry {i} (should be <{r})`，其中 \(r\) 为半单秩。这些检查按上述顺序执行。^[atlas-core-domain-seams.md:96-102]

转换错误中的 `to big` 是源码保留的措辞，与别处的 `too big` 写法并存。该助手由 `validate` 的词校验路径消费，调用位置为 `domain_builtins.rs:10106`；源码注释引用 `atlas-types.w:2344-2359`。^[atlas-core-domain-seams.md:96-102]

## 单生成元的校验

`check_weyl_generator` 先将输入收窄到 `i32`，再做有符号范围检查。对于成功收窄的负数，`usize::try_from` 失败，同样触发 `Generator {g} out of range for Weyl group (should be <{r})`。因此，负 `i32` 使用生成元越界诊断，而非 Weyl 词条目的无符号整数诊断。^[atlas-core-domain-seams.md:103-107]

来源没有列出 `i32` 收窄失败时的具体诊断文案，因此不能把词条目的 `u64` 转换错误文案套用到单生成元路径。该助手由 `validate` 的生成元校验路径消费，调用位置为 `domain_builtins.rs:10097`；源码注释引用 `atlas-types.w:2447-2476`。^[atlas-core-domain-seams.md:103-107]

## 与实形式根下标校验的区别

另一个助手 `check_generator` 按实形式上下文的半单秩检查下标，错误文案为 `Illegal root index: {g}`。其注释说明该措辞沿用上游 `get_reflection_index` 的用户下标表达，对应 `atlas-types.w:4481-4489`；`posroot`／负下标仍属于记录在案的 phase-1 暂缓事项。相关主题见 [[实形式生成元下标的校验契约]]。^[atlas-core-domain-seams.md:108-111]

## 证据边界

本页依据当前工作区字节的结构性阅读，不构成语言或数学验收。上游 C++／CWEB 行号均转述自 Rust 源码注释，未独立重读上游，可能随版本漂移；校验助手的诊断措辞以 HPC 语料门为行为权威，参见 [[HPC 验收证据链]]。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:115-118]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词／生成元校验。
