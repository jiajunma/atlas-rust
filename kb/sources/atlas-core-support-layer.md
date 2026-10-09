---
title: 支撑层（diagnostic.rs + source.rs + coercions.rs）——结构化诊断、源位置与强转表/邻近谓词
source: atlas-rust/atlas-core-support-layer
ingestedAt: 2026-10-09T11:45:00Z
---

# 支撑层（diagnostic.rs + source.rs + coercions.rs）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`diagnostic.rs`（179 行）、`source.rs`（115 行）、`coercions.rs`
（426 行）——诊断/源位置/强转三个支撑面。结构性阅读，不声称语言或
数学验收。

## `diagnostic.rs`：结构化诊断

- `SourceId(u64)`：源缓冲的稳定身份；`0` = 匿名（无需区分的输入）。
- `SourcePosition`：**从 1 开始**的行/列（Atlas 用户面约定）。
- `SourceSpan`：含头不含尾（inclusive start, exclusive end），构造断言
  `byte_start <= byte_end`。
- `ErrorKind`：`Lexical`/`Syntax`/`Name`/`Type`/`Program`（表达式分析
  失败，区别于类型合一）/`Runtime`/`Io`。**类别与措辞刻意分离**——
  调用方按类别比行为，不依赖措辞。
- `Diagnostic`：`kind` + `message` + `raw_message: Option<AtlasString>`
  （Atlas 数据非 UTF-8 时的**精确字节**；`message` 仍是转义的编辑器
  预览，永不是字节输出权威——`new_bytes` 分流）+ `span` + `warning`
  （报告用户但**不弄脏会话**：上游词法恢复打印到 stderr 而不置 clean
  标志，退出状态保持 0）+ `back_trace: Vec<AtlasString>`。
- `back_trace` 纪律：运行时错误展开时累积，**最外层在前**；`trace()`
  前插（上游 `error_base::trace` 的 push_front，axis-types.h:305）；
  命令层把非空 trace 拷进 `back_trace` 系统变量（global.w:1135-1148）。

## `source.rs`：自持源文本与位置

`SourceText { source_id, text, line_starts }`：构造时预存每行起始字节；
`position(byte)` 钳到文本长、退到字符边界、在 `line_starts` 上
partition_point 得行号（1 基），列 = 行首到该字节的 **Unicode 标量数**
+1（1 基）；`span(start, end)` 组装 `SourceSpan`。测试锚点：匿名 id 稳定、
id 随 span 保留、位置按 Unicode 标量与换行计数（越界钳到末尾）。

## `coercions.rs`：强转表 + 邻近谓词

移植上游注册（global.w:2526-2552 接 atlas-types.w:9137-9144），**保持
顺序**——`coercion_between` 与 `row_coercion` 都是**首中即返**的线性
扫描，所以 `mat` 上下文的列表显示必须先遇 `[vec]->mat` 而非任何
`[[int]]->mat` 项。

- 29 条注册（测试钉死数量），tag 是上游转换名（打印为转换节点上的
  `tag:expr`）：`QI`(int→rat)、`V[I]`、`Qv[Q]`、`[I]V`、`[Q]Qv`、
  `QvV`、`Qv[I]`、`[Q]V`、`[Q][I]`、`M[V]`([vec]→mat)、`M[[I]]`、
  `[V][[I]]`、`[[I]][V]`、`[V]M`、`[[I]]M`、`[Qv]M`、`[[Q]]M`、
  `[Qv][V]`、`[[Q]][V]`、`[Qv][[I]]`、`[[Q]][[I]]`、`LT`
  (string→LieType)、`RdIc`、`IcRf`、`RdRf`、`SpI`、`Sp(I,I)`、
  `KpolK`(KType→KTypePol)、`PolP`(Param→ParamPol)。表是
  `OnceLock` 单次构建。
- `row_coercion(final_type)`：非行上下文的列表显示取**第一个** from 为
  行且目标是它的条目，产出显示元素的分量类型（`mat` → `vec`，永不
  `[int]`）。
- `is_close(x, y) -> u8`（axis-types.w:3246-3285）：三比特——0x1 左可转
  右、0x2 右可转左、0x4 邻近；相等（含 void 与递归身份）→ 0x7 且**先于**
  边界检查与展开；void/`*` 只与自己邻近；递归名的 Tabled/Applied → 0，
  否则展开一层再查；原始端点查表；行逐分量；元组按分量取 AND。
- `broader_eq(a, b)`（axis-types.w:3339-3364，平衡序）：void 最宽、`*`
  最窄；原始类型吸收一切可转入者；行/元组逐分量；**函数要求参数类型
  相等**。
- 测试引用原版背书证据（3856293 零参重载原地替换、3856744 递归名
  名义身份）；转换**节点**随类型化管线到达，本模块只回答适用性。

## 边界

值面见 [值层包](atlas-core-value-layer.md)；类型模型见
[类型包](atlas-core-types.md)；诊断的渲染/打印在 `session_frame.rs` 的
`describe_bytes`（见 [会话帧包](atlas-core-session-frame.md)）。上游行号
引用是**实现方移植陈述**；行为兼容以 HPC 语料门为准。字节数/哈希只标识
本快照字节（git base `964f0033`）。
