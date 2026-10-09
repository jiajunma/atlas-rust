---
title: crate 根：atlas-core 语言门面（lib.rs）——15 个公开模块、1 个 crate 私有矩阵约化与兼容契约版本
source: atlas-rust/atlas-core-root
ingestedAt: 2026-10-09T09:48:46Z
---

# crate 根：atlas-core 语言门面（lib.rs）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/lib.rs`（26 行）+ 全部顶层模块与子目录文件的角色
定位。lib.rs 自述定位："Compatibility-oriented Atlas language runtime"——
公开模块刻意保持小且对应**可观察的语言边界**，实现工作遵循 `docs/` 的
契约。本包是结构性阅读，不声称任何语言或数学验收。

## 模块组织

- 15 个 `pub mod`：`coercions`、`diagnostic`、`domain_builtins`、
  `formula`、`frames`、`lex`、`linear_values`、`session`、
  `session_frame`、`source`、`syntax`、`typed`、`types`、`value`（按
  声明顺序）。
- 1 个 `pub(crate)`：`matreduc`（精确整数矩阵约化，移植自上游
  `utilities/matreduc.h`，服务 global.w 批内建）。
- 1 个 `#[cfg(test)]`：`session_fixture_tests`。
- 唯一常量导出：`COMPATIBILITY_VERSION: &str = "atlas-language-v0"`——
  语言兼容契约的版本标记。

## 模块角色地图（行数为本快照字节下的行数；角色描述取各自文件头自述）

| 模块 | 行数 | 角色（文件头自述，实现方陈述） |
| --- | --- | --- |
| `lex.rs` | 992 | 有状态词法器：Atlas 注释可嵌套，输入可按 token 逐个消费；`tokenize` 是兼容性便利接口。 |
| `syntax.rs` | 4659 | 语法前端：LALRPOP 生成文法 + 把有状态词法流适配成 spanned-token 流的独立适配层（`grammar.lalrpop` 1096 行）。 |
| `types.rs` | 961 | axis 类型模型（语言 phase B stage 1），移植 `type_expr`（axis-types.w:289-388）。 |
| `types/polymorphic.rs` | 1106 | 二阶类型机器（axis-types.w:2264+）。 |
| `types/recursive.rs` | 262 | 图级递归 typedef 安装（axis-types.w:1640-2010）：命名 RHS 槽位优先，环上的匿名后代保留身份。 |
| `types/revision_tests.rs` | 78 | 类型表修订测试。 |
| `typed.rs` | 18519 | 类型化管线核心：parsed→typed 可执行转换与求值（phase B stage B2 原地生长）；`convert_expr` 镜像 axis.w:272-487，检查与合成单遍完成；整数收窄用上游**逐字**错误文本（含笔误，bigint.cpp:142-162）。 |
| `typed/type_groups.rs` | 90 | 递归图构造前的组内名字解析；与 global.w:1632-1860 的校验顺序/诊断保持分离。 |
| `typed/command_cache_tests.rs` | 66 | 命令缓存测试。 |
| `typed/completion_tests.rs` | 40 | 补全测试。 |
| `frames.rs` | 310 | 类型化管线的求值帧（axis-types.w:2370-2400, 2830-2848）。 |
| `frames/completions.rs` | 112 | 会话持有的补全顺序，与当前名字可见性分离。 |
| `coercions.rs` | 426 | axis 强转表与类型邻近谓词（global.w:2526-2552 起）。 |
| `linear_values.rs` | 403 | vec/mat/ratvec 具体载荷与上游打印格式（axis-types.w:2028-2094）。 |
| `value.rs` | 359 | 求值器产生的值。 |
| `formula.rs` | 243 | 运算符优先级归约：结构解析器把式子看作交错的序列。 |
| `diagnostic.rs` | 179 | 解析器与求值器共享的结构化诊断。 |
| `source.rs` | 115 | 自持源文本与从 1 开始的行列位置。 |
| `session.rs` | 3889 | 有状态逐命令执行的外层循环：Atlas 词法分类依赖先前命令改变的状态，故**永不**预切分整个文件；拥有 `SessionEvent`（Value/Output/ReportLine/ReportBytes…）。 |
| `session_frame.rs` | 993 | 会话帧：文件包含、输出重定向与顶层输出面。 |
| `matreduc.rs` | 1559 | （crate 私有）矩阵精确约化：gcd+recorder、column_echelon 等，逐操作对应 pinned 上游 4d3e9449。 |
| `domain_builtins.rs` | 22771 | 领域内建：语言层到 `atlas-real-group` 的桥；命名函数应用在此派发；句柄是 Arc 持有的上下文束（急切校验种子 + 可失败惰性 KGB/表示属主），按（内类值、内部形式号、元素 id）**结构**比较，对应上游 memoized 句柄的可观察相等；打印串在上游形式稳定处逐字节一致。 |
| `domain_builtins/weyl_subgroup.rs` | 433 | 反射子群轨道与 ambient Weyl 见证。 |

## 边界与限制

- 本包只覆盖门面与模块地图；`typed.rs`/`domain_builtins.rs`/
  `session.rs`/`syntax.rs` 的内部实现各有独立来源包（待补）。
- "移植自 axis.w/global.w 的某行号"是**实现方的设计陈述**，不是已核验
  行为；语义等值以 HPC 差分门为准（例如 Weyl owner/dual 语义的 A1 限定
  验收见 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`）。
- 行数/哈希只标识本快照字节（git base `964f0033`，工作区干净）；
  字节正确性不由哈希证明。
- `matreduc` 虽私有但体量 1559 行且服务内建正确性，需要自己的包。
