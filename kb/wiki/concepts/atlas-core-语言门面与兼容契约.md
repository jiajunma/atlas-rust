---
title: atlas-core 语言门面与兼容契约
summary: atlas-core 以可观察的语言边界组织模块，并导出兼容版本 atlas-language-v0；公开模块共 14 个。
sources:
  - atlas-core-root.md
kind: concept
createdAt: "2026-10-09T14:33:15.295Z"
updatedAt: "2026-10-09T14:33:15.295Z"
tags:
  - Rust架构
  - 语言运行时
  - 兼容契约
aliases:
  - atlas-core-语言门面与兼容契约
confidence: 0.99
provenanceState: ambiguous
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# atlas-core 语言门面与兼容契约

`atlas-core` 将自身定位为“面向兼容性的 Atlas 语言运行时”（Compatibility-oriented Atlas language runtime）。crate 根 `lib.rs` 的公开模块刻意保持较小，并对应可观察的语言边界；实现工作遵循 `docs/` 中的契约。当前来源是门面与模块角色的结构性阅读，不构成语言或数学验收。^[atlas-core-root.md:7-13]

## 门面与版本标记

crate 根唯一导出的常量是 `COMPATIBILITY_VERSION: &str = "atlas-language-v0"`，用于标记语言兼容契约版本。模块组织还包含 crate 私有的 `matreduc`，以及仅在 `#[cfg(test)]` 下启用的 `session_fixture_tests`。^[atlas-core-root.md:21-25]

公开模块共 14 个：`coercions`、`diagnostic`、`domain_builtins`、`formula`、`frames`、`lex`、`linear_values`、`session`、`session_frame`、`source`、`syntax`、`typed`、`types`、`value`。（来源包原误记为 15 个；经对照 git base `964f0033` 与当前字节的实际声明，均为 14 个，包已更正。）^[atlas-core-root.md:17-20]

## 可观察的语言边界

词法与语法层由 `lex`、`syntax` 和 `formula` 协作组成。`lex` 提供有状态、可逐 Token 消费的词法器，并支持[[可嵌套注释]]；`syntax` 将词法流适配为带源码跨度的 Token 流，接入 LALRPOP 生成的文法；`formula` 负责对交错序列进行运算符优先级归约。相关主题见 [[Atlas 语法前端与运算符优先级归约]]。^[atlas-core-root.md:31-32, atlas-core-root.md:46-46]

类型与求值层包括 `types` 的 axis 类型模型、二阶类型机器和递归 typedef 图构造，以及 `typed` 的 parsed→typed 可执行转换与求值。`convert_expr` 在单遍转换中完成检查与合成；`frames` 提供求值帧，`coercions` 提供强转表与类型邻近谓词，`value` 表示求值结果，`linear_values` 承载 vec、mat、ratvec 及其上游打印格式。相关主题见 [[Atlas 类型模型与递归类型图]]、[[convert_expr 的 in/out 类型模式与单遍转换]]。^[atlas-core-root.md:33-45]

会话层的关键约束是逐命令执行：Atlas 的词法分类依赖先前命令改变的状态，因此 `session` 不预切分整个文件。它还拥有 `SessionEvent` 输出事件；`session_frame` 负责文件包含、输出重定向与顶层输出面。共享的 `diagnostic` 和 `source` 分别提供结构化诊断、自持源文本及从 1 开始的行列位置。相关主题见 [[会话循环与文件会话帧的职责边界]]、[[SessionEvent 与字节保留输出]]。^[atlas-core-root.md:47-50]

## 领域桥接与兼容行为

`domain_builtins` 是语言层到 `atlas-real-group` 的桥梁，负责命名函数应用的派发。其句柄由 `Arc` 持有上下文束，结合急切校验的种子与可失败的惰性 KGB／表示属主；相等性按“内类值、内部形式号、元素 id”进行结构比较，以对应上游 memoized 句柄的可观察相等性。在上游形式稳定的位置，打印字符串要求逐字节一致。相关主题见 [[函数调用分派与内建参数解包]]。^[atlas-core-root.md:52-53]

兼容性还体现在具体诊断文本上：来源将 `typed` 的整数收窄错误描述为保留上游逐字文本，包括其中的笔误。这类文本与打印行为共同构成运行时的可观察边界。^[atlas-core-root.md:37-37, atlas-core-root.md:52-52]

`matreduc` 虽不属于公开 API，却服务于 `global.w` 批内建的正确性。它提供精确整数矩阵约化，包括 gcd 与 recorder、`column_echelon` 等操作；来源将其描述为逐操作对应固定上游版本 `4d3e9449`，并指出这一内部模块需要独立来源包。^[atlas-core-root.md:21-22, atlas-core-root.md:51-51, atlas-core-root.md:64-64]

## 证据范围

本来源覆盖门面和模块地图，未展开 `typed.rs`、`domain_builtins.rs`、`session.rs`、`syntax.rs` 的内部实现。模块头中“移植自 axis.w／global.w 某行”的说明属于实现方设计陈述，不能直接视为已核验的行为等值；语义等值须以 [[HPC 验收证据链]]中的差分门为准。来源举出的 Weyl owner／dual 验收仅限 A1 范围。^[atlas-core-root.md:55-61]

该快照记录的 Git base 为 `964f0033`，工作区干净。行数与哈希仅用于标识快照字节，不能证明这些字节的正确性；同样，兼容契约版本标记也不应被解读为完整兼容性验收。^[atlas-core-root.md:9-13, atlas-core-root.md:24-25, atlas-core-root.md:62-63]

## Sources

- [atlas-core-root.md](../../sources/atlas-core-root.md) — crate 根：atlas-core 语言门面（lib.rs）。
