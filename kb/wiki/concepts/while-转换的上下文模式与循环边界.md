---
title: while 转换的上下文模式与循环边界
summary: while 将循环层置于整棵 do 树外，由所需上下文选择 void、count 或 row 模式，row 路径先尝试 [*] 特化再回退 row_coercion。
sources:
  - atlas-core-convert-expr.md
kind: concept
createdAt: "2026-10-09T14:27:09.935Z"
updatedAt: "2026-10-09T22:13:30.172Z"
tags:
  - 控制流
  - 类型转换
aliases:
  - while-转换的上下文模式与循环边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: while 转换的上下文模式与循环边界
summary: while 将循环层置于整棵 do 树外，条件先转换再检查 bool；所需上下文决定 void、count 或 row 模式，row 路径先尝试 [*] 特化再回退 row_coercion。
sources:
  - atlas-core-convert-expr.md
kind: concept
tags:
  - 控制流
  - 类型检查
  - 类型上下文
aliases:
  - while-转换的上下文模式与循环边界
---

# while 转换的上下文模式与循环边界

`while` 属于 `convert_expr` 的 `loop` 转换族，与 `for`、`counted-for` 一同处理。转换遍同时执行类型检查与类型合成，`while` 的模式则由所需类型上下文决定；入口机制参见 [[convert_expr 的 in/out 类型模式与单遍转换]]。^[atlas-core-convert-expr.md:12-14, atlas-core-convert-expr.md:33-44]

## 循环边界与条件检查

源材料描述的上游 `axis.w` 规则将循环层置于整棵 `do` 树之外；压平守卫时不得将守卫移出这一循环边界。^[atlas-core-convert-expr.md:40-41]

条件先进行 a-priori 转换，再检查其类型是否为 `bool`。类型不匹配时，诊断沿用上游措辞 `found … while … was needed.`。^[atlas-core-convert-expr.md:41-42]

## 所需上下文与 WhileMode

`WhileMode` 由所需上下文决定：在 `void` 上下文中，循环体使用 `void` 上下文转换，并允许异构分支；在 `int` 上下文中，选择 `count` 模式；其他上下文选择 `row` 模式，并携带 `reversed` 标志。^[atlas-core-convert-expr.md:42-44]

`row` 路径先尝试 `[*]` 特化，失败后回退到 `row_coercion`。非行上下文中的列表显示也会查询 `row_coercion`，相关机制见 [[列表显示的行转换选择]]。^[atlas-core-convert-expr.md:14-15, atlas-core-convert-expr.md:44-44]

## 实现组织与证据边界

`loop` 是转换调度器的 12 个表达式族之一。全部族助手均标记为 `#[inline(never)]`，分区时保留原有分支体与已调整的共享转换上下文，属于不改变语义的机械划分。源材料指出，相关栈问题涉及分析帧，而非求值器。^[atlas-core-convert-expr.md:26-38]

本页依据结构性源码阅读，不构成语言兼容性验收。源材料中的上游行号对应关系属于实现方的移植陈述；转换行为的兼容性仍以 HPC 语料门为准，相关主题见 [[HPC 验收证据链]]。^[atlas-core-convert-expr.md:9-17, atlas-core-convert-expr.md:67-69]

## Sources

- [atlas-core-convert-expr.md](../../sources/atlas-core-convert-expr.md) — 转换遍 convert_expr（typed.rs 中部）。
