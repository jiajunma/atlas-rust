---
title: 全局绑定 cell 的重定义与赋值语义
summary: 每次 set 定义无条件创建新 cell，已转换代码保留分析期捕获的旧 cell；:= 修改既有 cell，读取未赋值的 None 状态产生运行时错误。
sources:
  - atlas-core-frames.md
kind: concept
createdAt: "2026-10-10T00:19:44.909Z"
updatedAt: "2026-10-10T00:19:44.909Z"
tags:
  - 全局绑定
  - 求值器
  - 绑定身份
aliases:
  - 全局绑定-cell-的重定义与赋值语义
  - 全C的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

# 全局绑定 cell 的重定义与赋值语义

全局绑定通过共享、可变且允许未初始化的 cell 保存值。其核心规则是：每个 `set` 式定义都无条件分配新 cell，而 `:=` 赋值写入既有 cell；转换后的代码持有它在分析期捕获的那个 cell。^[atlas-core-frames.md:50-55]

## cell 与未初始化状态

`GlobalCell` 以 `Rc` 共享所有权，通过 `RefCell` 提供内部可变性，并以 `Option<SharedValue>` 表示是否已有值；其中 `SharedValue = Rc<Value>`。`None` 表示全局绑定已声明但未赋值，读取这一状态会产生运行时错误。`unset_global` 与 `global_with` 是两个构造入口。^[atlas-core-frames.md:18-20, atlas-core-frames.md:50-55]

## 重定义与赋值的区别

`set` 式重定义建立新的 cell，不替换已转换代码所持有的旧 cell。因此，分析期捕获的绑定身份会保留在转换后的代码中；后续同名定义分配新 cell，并不会使这些代码自动改为访问新 cell。^[atlas-core-frames.md:52-55]

`:=` 则修改已有 cell 中的值。它保留 cell 的身份，因此持有该 cell 的代码仍通过同一绑定读取赋值后的内容。理解这一区别时，应区分“为定义分配 cell”和“更新 cell 内的值”。^[atlas-core-frames.md:52-55]

## 求值中的借用纪律

帧模块规定：读操作在短借用下克隆共享值，写操作在右侧表达式完全求值后才取得短借用，任何借用都不跨嵌套求值持有。控制流通过 `Result` 传递，上下文由作用域函数恢复。^[atlas-core-frames.md:28-32]

局部槽同样以 `Option<SharedValue>` 表示是否初始化，但 `take_local` 会搬出局部值并留下空槽，相关行为见 [[局部槽的移出与未初始化状态]]。全局 cell 的重定义规则则明确要求每次 `set` 分配新 cell。^[atlas-core-frames.md:25-26, atlas-core-frames.md:46-55]

## 测试与证据边界

来源列出的五个单元测试中，有一个覆盖全局 cell 的未赋值与已赋值状态区分；该测试清单未列出专门验证 `set` 重定义与 `:=` 赋值差异的测试。^[atlas-core-frames.md:66-71]

本页依据 `crates/atlas-core/src/frames.rs` 的结构性阅读。来源中的上游 CWEB 行号转述自源码注释，未独立重读上游；兼容性权威仍是 HPC 语料门，本材料不声称语言验收。^[atlas-core-frames.md:9-14, atlas-core-frames.md:73-77]

## Sources

- [atlas-core-frames.md](../../sources/atlas-core-frames.md) — 求值帧链与共享槽（frames.rs）：闭包捕获、借用纪律与全局 cell。
