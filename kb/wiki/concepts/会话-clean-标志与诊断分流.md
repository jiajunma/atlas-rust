---
title: 会话 clean 标志与诊断分流
summary: 语法、类型、求值及真正的词法错误使会话变脏，而文件打开失败、Io 诊断、词法警告和 abandon 级联本身不改变 clean 状态。
sources:
  - atlas-core-session-frame.md
kind: concept
createdAt: "2026-10-09T14:34:26.450Z"
updatedAt: "2026-10-09T14:34:26.450Z"
tags:
  - 会话状态
  - 诊断
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 会话 clean 标志与诊断分流

`SessionFrame` 的 `clean` 标志记录会话是否发生影响退出状态的错误。它遵循上游的错误分类纪律：语法、类型和求值错误会将其置为 `false`，打开文件失败与 abandon 级联则保持原有状态。因此，包含流程中止并不必然意味着会话变为不干净。^[atlas-core-session-frame.md:13-16, atlas-core-session-frame.md:43-46]

## 诊断分流

在命令执行层，`Diagnostic` 事件按种类处理：`kind != Io` 时设置 `clean=false`，并使本命令返回 Abort；`Io` 诊断则不改变 `clean`。这使诊断的报告、命令中止和会话退出状态具有不同的判定条件。^[atlas-core-session-frame.md:50-58]

词法层另行区分 warning 与真正的词法错误。warning（例如未闭合字符串提示）只报告，不改变 `clean`，也不触发 Abort；真正的词法错误会将会话置为不干净，并在包含深度大于零时触发 [[嵌套包含的 abandon 级联]]。^[atlas-core-session-frame.md:69-70]

## 文件打开失败与包含中止

包含文件解析路径失败时，会产生 `Io` 诊断 `failed to open input file …` 并返回 Abort。由此引发的包含栈中止不会自行将 `clean` 置为 `false`；测试 `missing_file_aborts_the_include_stack_but_not_the_session` 明确断言，此时 `is_clean()` 仍为真。^[atlas-core-session-frame.md:36-46]

abandon 为各层包含文件报告放弃读取的位置，按最内层优先的顺序逐层补发。行号由 `lexer.offset()-1` 定位正在读取的行，再经 `line_map` 映射为物理行号；级联本身不改变 `clean`，其作用是呈现包含栈的中止位置。^[atlas-core-session-frame.md:43-46, atlas-core-session-frame.md:66-68]

## 输出重定向的失败路径

[[单命令输出重定向的执行纪律]]要求先将 `>file EXPR` 或 `>>file EXPR` 的命令体按表达式解析，再打开输出 sink。解析失败不会创建文件，但属于影响 `clean` 的语法错误；解析成功后若打开 sink 失败，则只向 stderr 输出一行 `Failed to open {name}`，会话保持 clean。^[atlas-core-session-frame.md:43-46, atlas-core-session-frame.md:59-65]

sink 在求值前打开，因此求值失败仍可能留下含部分输出的文件，并按求值错误纪律将会话置为不干净。重定向仅将 `Output`／`OutputBytes` 事件写入 sink，其余事件继续向上传递，且 `sink.close()` 无条件执行。^[atlas-core-session-frame.md:23-25, atlas-core-session-frame.md:43-46, atlas-core-session-frame.md:63-65]

## 诊断位置与证据边界

诊断显示使用预处理生成的 `line_map` 将重写后的行号映射回首个物理行。带 span 的诊断包含错误种类、来源、物理行号、列号、消息、源行及 caret；不带 span 的诊断仅使用裸 `{Kind} error:` 头。^[atlas-core-session-frame.md:72-79]

本文件的单元测试覆盖缺失文件中止、包含级联顺序、重定向打开失败后仍 clean、求值失败关闭 sink，以及物理行报告等行为。这些属于单元级行为锚点；端到端文件命令兼容性仍以 [[HPC 验收证据链]]中的差分语料为准，结构性阅读不构成语言或数学验收。^[atlas-core-session-frame.md:9-16, atlas-core-session-frame.md:81-97]

## Sources

- [会话帧：文件包含、输出重定向与顶层输出面（session_frame.rs）](atlas-core-session-frame.md)
