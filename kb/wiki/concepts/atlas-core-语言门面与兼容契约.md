---
title: atlas-core 语言门面与兼容契约
summary: atlas-core 按可观察语言边界组织模块，以 atlas-language-v0 标记兼容契约；正文列出 14 个公开模块，与标题的 15 个不一致。
sources:
  - atlas-core-root.md
kind: concept
createdAt: "2026-10-09T14:33:15.295Z"
updatedAt: "2026-10-09T22:18:37.530Z"
tags:
  - 语言运行时
  - 模块架构
  - 兼容契约
aliases:
  - atlas-core-语言门面与兼容契约
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: atlas-core 语言门面与兼容契约
summary: atlas-core 按可观察语言边界组织 14 个公开模块，以 atlas-language-v0 标记兼容契约；模块角色与移植约定属于结构性阅读证据，不构成行为或数学验收。
sources:
  - atlas-core-root.md
kind: concept
tags:
  - 语言运行时
  - 模块架构
  - 兼容契约
aliases:
  - atlas-core-语言门面与兼容契约
---

# atlas-core 语言门面与兼容契约

`atlas-core` 定位为“面向兼容性的 Atlas 语言运行时”（Compatibility-oriented Atlas language runtime）。crate 根 `lib.rs` 按可观察的语言边界组织公开模块，实现工作遵循 `docs/` 中的契约。本页依据门面与模块角色的结构性阅读，不构成语言或数学验收。^[atlas-core-root.md:7-13]

## 模块门面与版本标记

crate 根声明了 14 个 `pub mod`：`coercions`、`diagnostic`、`domain_builtins`、`formula`、`frames`、`lex`、`linear_values`、`session`、`session_frame`、`source`、`syntax`、`typed`、`types`、`value`。另有 crate 私有模块 `matreduc`，以及仅在 `#[cfg(test)]` 下启用的 `session_fixture_tests`。^[atlas-core-root.md:17-23]

唯一导出的常量为 `COMPATIBILITY_VERSION: &str = "atlas-language-v0"`，用于标记语言兼容契约版本。^[atlas-core-root.md:24-25]

## 可观察的语言边界

词法与语法层包含 `lex`、`syntax` 和 `formula`。`lex` 提供有状态、可逐 token 消费的词法器，支持[[可嵌套注释]]，并保留 `tokenize` 便利接口；`syntax` 通过独立适配层将词法流转换为带源码跨度的 token 流，接入 LALRPOP 生成的文法；`formula` 对结构解析器产生的交错序列执行运算符优先级归约。^[atlas-core-root.md:31-32, atlas-core-root.md:46-46]

类型与求值层包括 `types` 的 axis 类型模型、二阶类型机器和图级递归 typedef 安装，以及 `typed` 的 parsed→typed 可执行转换与求值。`convert_expr` 在单遍处理中完成检查与合成；递归类型组的名字解析发生在图构造之前，并与后续校验顺序及诊断保持分离。相关主题见[[Type 类型表示与语义等值]]和[[convert_expr 的 in/out 类型模式与单遍转换]]。^[atlas-core-root.md:33-38]

`frames` 提供求值帧，其补全支持由会话持有补全顺序，并将该顺序与当前名字可见性分离。`coercions` 提供强转表与类型邻近谓词；`linear_values` 承载 vec、mat、ratvec 及其上游打印格式；`value` 表示求值器产生的值。参见[[会话补全顺序与名字可见性分离]]及[[上游兼容的值打印约定]]。^[atlas-core-root.md:41-45]

会话层采用有状态的逐命令执行：Atlas 的词法分类依赖先前命令改变的状态，因此 `session` 永不预切分整个文件。它还拥有 `SessionEvent` 输出事件；`session_frame` 负责文件包含、输出重定向与顶层输出面。`diagnostic` 为解析器和求值器提供共享的结构化诊断，`source` 提供自持源文本及从 1 开始的行列位置。参见[[会话循环与文件会话帧的职责边界]]和[[SessionEvent 与字节保留输出]]。^[atlas-core-root.md:47-50]

## 领域桥接与兼容约定

`domain_builtins` 是语言层到 `atlas-real-group` 的桥梁，负责命名函数应用的派发。其句柄是由 `Arc` 持有的上下文束，结合急切校验的种子与可失败的惰性 KGB／表示属主；句柄按“内类值、内部形式号、元素 id”进行结构比较，以对应上游 memoized 句柄的可观察相等性。子模块 `weyl_subgroup` 负责反射子群轨道与 ambient Weyl 见证。参见[[函数调用分派与内建参数解包]]和[[ambient Weyl 见证词与作用方向]]。^[atlas-core-root.md:52-53]

文本兼容涉及诊断与打印。来源将 `typed` 的整数收窄错误描述为保留上游逐字文本，包括其中的笔误；领域内建的打印字符串则在上游形式稳定的位置保持逐字节一致。这些描述属于来源记录的实现约定。^[atlas-core-root.md:37-37, atlas-core-root.md:52-52]

crate 私有的 `matreduc` 为 `global.w` 批内建提供精确整数矩阵约化，包括 gcd 与 recorder、`column_echelon` 等操作。来源将其描述为移植自上游 `utilities/matreduc.h`，并逐操作对应固定上游版本 `4d3e9449`。该模块虽不公开，仍服务于内建正确性；来源指出其需要独立来源包。^[atlas-core-root.md:21-22, atlas-core-root.md:51-51, atlas-core-root.md:64-64]

## 证据范围与限制

本来源仅覆盖门面与模块地图，未展开 `typed.rs`、`domain_builtins.rs`、`session.rs`、`syntax.rs` 的内部实现。模块角色描述取自各文件头的实现方自述；“移植自 axis.w／global.w 某行”的说明属于设计陈述，不能直接视为已核验行为。语义等值以 HPC 差分门为准，相关原则见[[HPC 验收证据链]]。^[atlas-core-root.md:27-29, atlas-core-root.md:55-61]

来源仅举出 Weyl owner／dual 语义的 A1 限定验收作为例子，并未声明整个语言运行时通过验收。其记录的快照 Git base 为 `964f0033`，工作区干净；模块行数与哈希仅标识该快照的字节，不能证明字节正确性。^[atlas-core-root.md:59-63]

## Sources

- [atlas-core-root.md](../../sources/atlas-core-root.md) — crate 根：atlas-core 语言门面（lib.rs）。
