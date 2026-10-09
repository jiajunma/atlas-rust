---
title: 会话 clean 标志与诊断分流
summary: 语法、类型、求值及真正的词法错误使会话变脏，而打开失败、Io 诊断、词法警告及 abandon 级联本身不改变 clean。
sources:
  - atlas-core-session-frame.md
kind: concept
createdAt: "2026-10-09T14:34:26.450Z"
updatedAt: "2026-10-09T20:36:41.382Z"
tags:
  - 会话管理
  - 错误处理
aliases:
  - 会话-clean-标志与诊断分流
  - 会C标
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 会话 clean 标志与诊断分流
summary: 语法、类型、求值及真正的词法错误使会话变脏；文件打开失败、Io 诊断、词法警告和 abandon 级联本身不改变 clean 状态。
sources:
  - atlas-core-session-frame.md
kind: concept
tags:
  - 会话状态
  - 诊断
  - 错误处理
aliases:
  - 会话-clean-标志与诊断分流
  - 会C标
provenanceState: extracted
---

# 会话 clean 标志与诊断分流

`SessionFrame` 的 `clean` 标志参与决定会话退出状态。语法、类型和求值错误会将其置为 `false`；文件打开失败与 abandon 级联本身不改变该状态。因此，包含流程中止并不必然意味着会话变脏。^[atlas-core-session-frame.md:13-16, atlas-core-session-frame.md:43-46]

## 诊断分流

命令执行层按 `Diagnostic` 的种类处理错误：`kind != Io` 时设置 `clean=false`，并使本命令返回 Abort；`Io` 诊断不弄脏会话。这里应区分诊断报告、命令中止与 `clean` 状态变化。^[atlas-core-session-frame.md:50-58]

词法层另行区分 warning 与真正的词法错误。warning（例如未闭合字符串提示）只报告，不弄脏会话，也不触发 Abort；真正的词法错误会将会话置为 dirty，并在包含深度大于零时触发 [[嵌套包含的 abandon 级联]]。^[atlas-core-session-frame.md:69-70]

## 文件包含失败

包含文件的 `resolve` 失败时，会产生 `Io` 诊断 `failed to open input file …` 并返回 Abort。由此导致的包含栈中止不会自行将 `clean` 置为 `false`；测试 `missing_file_aborts_the_include_stack_but_not_the_session` 明确断言，该场景下 `is_clean()` 仍为真。相关包含流程见 [[SessionFrame 会话帧与文件包含语义]]。^[atlas-core-session-frame.md:36-46]

abandon 按最内层优先的顺序，为各层包含文件补发 `Abandoning reading of file '{origin}' at line {physical}`。实现用 `lexer.offset()-1` 定位正在读取的行，再经 `line_map` 换算为物理行号。级联报告本身不弄脏会话；触发级联的语法、类型、求值或真正的词法错误则有各自的置脏规则。^[atlas-core-session-frame.md:43-46, atlas-core-session-frame.md:66-70]

## 输出重定向的失败路径

[[单命令输出重定向的执行纪律]]要求先将 `>file EXPR` 或 `>>file EXPR` 的命令体按表达式解析，再打开输出 sink。解析失败不会创建文件，但语法错误会将 `clean` 置为 `false`；解析成功后若打开 sink 失败，则只向 stderr 输出一行 `Failed to open {name}`，不弄脏会话。这种打开失败不会把此前已为 `false` 的状态恢复为 `true`。^[atlas-core-session-frame.md:43-46, atlas-core-session-frame.md:59-65]

sink 在求值前打开，因此求值失败仍可能留下包含部分输出的文件，并按求值错误纪律将会话置为 dirty。重定向将 `Output`／`OutputBytes` 事件写入 sink，其余事件照常向上传递，且 `sink.close()` 无条件执行。^[atlas-core-session-frame.md:23-25, atlas-core-session-frame.md:43-46, atlas-core-session-frame.md:63-65]

## 诊断位置

预处理先剥除每行尾部空白，再将以反斜杠结尾的行与下一行直接拼接，同时生成“重写行→首个物理行”的 `line_map`。`describe` 与 `abandon` 使用该映射报告物理行号。带 span 的诊断显示错误种类、来源、物理行号、列号、消息、源行和 caret；无 span 的诊断只有裸 `{Kind} error:` 头。^[atlas-core-session-frame.md:72-79]

## 测试与证据边界

本文件的单元测试覆盖缺失文件中止、包含级联顺序、重定向打开失败后仍 clean、求值失败关闭 sink，以及物理行报告等行为。这些是单元级行为锚点；端到端文件命令兼容性仍以 HPC 差分语料为准。来源属于结构性阅读，上游行号与对应关系属于实现方的移植陈述，不构成语言或数学验收。^[atlas-core-session-frame.md:9-16, atlas-core-session-frame.md:81-97]

## Sources

- [会话帧：文件包含、输出重定向与顶层输出面（session_frame.rs）](atlas-core-session-frame.md)
