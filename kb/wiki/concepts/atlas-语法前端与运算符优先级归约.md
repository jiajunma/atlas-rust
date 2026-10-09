---
title: Atlas 语法前端与运算符优先级归约
summary: 语法前端结合 LALRPOP 文法与独立的带位置 token 流适配层，formula 模块负责对结构解析器产生的交错序列进行运算符优先级归约。
sources:
  - atlas-core-root.md
kind: concept
createdAt: "2026-10-09T14:33:15.650Z"
updatedAt: "2026-10-09T14:33:15.650Z"
tags:
  - 语法分析
  - LALRPOP
  - 运算符优先级
aliases:
  - atlas-语法前端与运算符优先级归约
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Atlas 语法前端与运算符优先级归约

Atlas 的语法前端由 `syntax.rs` 中的文法与词法流适配层，以及 `formula.rs` 中的运算符优先级归约组成。它们属于 `atlas-core` 的语言门面；该 crate 面向兼容性组织公开模块，使模块对应可观察的语言边界。^[atlas-core-root.md:9-13, atlas-core-root.md:31-32, atlas-core-root.md:46-46]

## 文法与词法流适配

`syntax.rs` 使用 LALRPOP 生成文法，并通过独立适配层将有状态词法流转换为带源码跨度的 token 流（spanned-token 流）。来源快照中，`syntax.rs` 为 4659 行，文法文件 `grammar.lalrpop` 为 1096 行。相关概念见 [[Atlas 词法 Token 模型]] 与 [[带源码跨度的表达式抽象语法树]]。^[atlas-core-root.md:31-32]

词法层支持嵌套注释，并允许逐 token 消费输入；`tokenize` 是兼容性便利接口。会话层逐命令执行，因为先前命令可能改变后续输入的词法分类，因此不会预先切分整个文件。这是理解语法前端输入方式的重要上下文，参见 [[可嵌套注释]] 与 [[会话循环与文件会话帧的职责边界]]。^[atlas-core-root.md:31-31, atlas-core-root.md:49-49]

## 运算符优先级归约

`formula.rs` 承担运算符优先级归约，结构解析器将表达式视为交错的序列。来源将这一职责与 `syntax.rs` 的文法和词法流适配职责分别列出；该快照中的 `formula.rs` 为 243 行。^[atlas-core-root.md:32-32, atlas-core-root.md:46-46]

## 后续类型化处理与诊断

语法解析之后，`typed.rs` 承担从 parsed 表达式到 typed 可执行表达式的转换与求值；其中 `convert_expr` 在单遍转换中完成类型检查与合成。解析器和求值器共享 `diagnostic.rs` 提供的结构化诊断，而 `source.rs` 提供自持源文本及从 1 开始的行列位置。相关概念见 [[convert_expr 的 in/out 类型模式与单遍转换]]、[[TypedExpr 可执行表达式树]] 与 [[CLI 事件输出与源码诊断]]。^[atlas-core-root.md:37-37, atlas-core-root.md:47-48]

## 证据范围

本页依据 crate 根的结构性模块地图，说明前端各部分的职责与衔接。来源未展开具体运算符优先级、结合性或归约步骤，并明确将 `syntax.rs` 等模块的内部实现留给独立来源包；因此不能据此确立完整的解析规则或语言兼容性结论。^[atlas-core-root.md:27-32, atlas-core-root.md:46-46, atlas-core-root.md:55-61]

来源中的上游移植对应关系属于实现方的设计陈述，不等同于已核验行为；语义等值仍以 HPC 差分门为准。该来源本身不声称语言或数学验收，参见 [[HPC 验收证据链]]。^[atlas-core-root.md:9-13, atlas-core-root.md:59-61]

## Sources

- [atlas-core-root.md](atlas-core-root.md) — crate 根：atlas-core 语言门面（lib.rs）
