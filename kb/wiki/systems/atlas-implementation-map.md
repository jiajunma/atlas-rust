---
id: atlas-implementation-map
type: implementation
note_status: draft
aliases: [Atlas architecture, 解释器结构]
source_snapshot: sources/snapshots/2026-10-01-initial.json
---

# Rust 系统结构与兼容性边界

Atlas Rust 的兼容性目标位于 Atlas 语言的可观察边界。本页以 Rust 系统结构及其演进为主，解释语言执行和数学领域实现的分工。原版是追溯行为和算法来源的 baseline，不要求 C++ 文件与 Rust 文件逐一配对。

## 原版的知识来源

仓库 [LANGUAGE](../../../docs/LANGUAGE.md) 将上游分为 CWEB 解释器与语法、普通 C++ 领域库、文档等区域。研究具体数学算法时还要查看上游 `atlas-scripts`，不能由低层函数名推断整条脚本计算流程。这里是来源地图，不是对各区域的完整源码审查。

## 当前 Rust 的实际边界

[Cargo.toml](../../../Cargo.toml) 列出三个 workspace 成员：

| crate | 本次读取到的职责与入口 |
| --- | --- |
| atlas-cli | `main` 创建 `SessionFrame`，接入文件和输出，处理命令行及进程退出状态 |
| atlas-core | `SessionFrame` 处理有状态会话和包含文件；`session` 按命令推进词法、解析和执行；`TypedContext` 保存类型、全局绑定、重载和求值状态 |
| atlas-real-group | 提供根数据、Weyl、KGB、KL 等领域模块；`atlas-core` 通过实际 Cargo 依赖使用它 |

`session.rs` 明确要求按命令推进词法，因为先前命令会改变后续输入的解释。`SessionEvent` 的值、报告、输出和诊断交给外层处理；退出状态由 CLI 结合会话状态决定。

## 早期设计与现状的区别

[DESIGN](../../../docs/DESIGN.md) 提供设计边界，但部分名称和表示不能直接作为当前结构：

- 它讨论的 `atlas-coxeter`、`atlas-io` 不在当前 workspace；`rustcox-core` 在该文档中是候选适配方向。
- 当前 core 导出 `session`、`session_frame`、`typed`、`frames` 等模块，不能把早期的 `eval`、`resolve` 等逻辑分层写成已经存在的同名模块。
- 当前 `Frame` 由 `Rc` 链接，槽位使用 `RefCell`；因此不能把早期 arena/handle 描述直接当成实际内存方案。
- 当前 `SessionEvent` 有 `Value`、`Output`、`ReportLine`、`ReportBytes`、`OutputBytes`、`Diagnostic`；设计中列出的 `FileWrite` 和 `Exit` 不是这个枚举当前的变体。

这些区别说明 KB 应随源码迭代。它们本身不证明兼容或不兼容；具体行为仍需对应原版证据。

## 来源与关联

- [core/lib.rs](../../../crates/atlas-core/src/lib.rs) 与 [core/Cargo.toml](../../../crates/atlas-core/Cargo.toml)：实际模块和领域依赖。
- [session.rs](../../../crates/atlas-core/src/session.rs)：`SessionEvent`、`run_source_with_context`、`execute_tokens`。
- [session_frame.rs](../../../crates/atlas-core/src/session_frame.rs)：`SessionFrame`、`run_top_level`。
- [typed.rs](../../../crates/atlas-core/src/typed.rs)：`TypedContext`；[frames.rs](../../../crates/atlas-core/src/frames.rs)：`Frame`、`EvaluationContext`。
- [cli/main.rs](../../../crates/atlas-cli/src/main.rs)：`main`；[real-group/lib.rs](../../../crates/atlas-real-group/src/lib.rs)：领域模块。
- [兼容性契约](../../../docs/COMPATIBILITY.md)、[来源快照](../../sources/snapshots/2026-10-01-initial.json)。
- 具体领域示例：[根坐标](../math/root-coordinates.md)与 [Ladder 两版比较](../comparisons/root-ladder-cpp-rust.md)。
