---
title: 嵌套包含的 abandon 级联
summary: 包含流中止时，各层从最内层向外报告放弃读取的位置，通过 lexer 偏移回退和 line_map 将当前位置换算为物理行号。
sources:
  - atlas-core-session-frame.md
kind: concept
createdAt: "2026-10-09T14:34:00.426Z"
updatedAt: "2026-10-09T14:34:00.426Z"
tags:
  - 文件包含
  - 错误传播
  - 源码定位
aliases:
  - 嵌套包含的-abandon-级联
  - 嵌A级
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 嵌套包含的 abandon 级联

嵌套包含的 abandon 级联是 `SessionFrame` 在文件包含过程中中止读取、逐层报告放弃位置的机制。内层文件中止后，各外层文件分别补发 `Abandoning reading of file '{origin}' at line {physical}`，报告顺序由最内层向外展开。^[atlas-core-session-frame.md:66-68]

## 包含栈与中止条件

`SessionFrame` 的 `active` 栈记录当前包含链，同时承担循环守卫、深度检查和回溯命名。正常包含会打印 `Starting` 框架行、压栈并递归执行 `run_stream`；只有返回 `Finished` 时，才记录文件已完成并打印 `Completely` 框架行。^[atlas-core-session-frame.md:29-40]

输入文件解析路径失败时，会产生 `failed to open input file …` 的 Io 诊断并返回 `Abort`；缺失文件可中止当前包含栈。循环包含则静默跳过并视为成功；包含深度上限为 `MAX_INCLUDE_DEPTH = 64`，超出时产生 Io 诊断并中止。路径查找顺序另见 [[包含文件的搜索路径解析]]。^[atlas-core-session-frame.md:34-46]

命令执行产生非 Io 诊断时，本命令返回 `Abort`，并将会话标记为不 clean。词法层另有区分：warning 只报告，不中止也不弄脏会话；真正的词法错误会标记 dirty，并在包含深度大于零时触发 abandon 级联。^[atlas-core-session-frame.md:56-58, atlas-core-session-frame.md:69-70]

## 放弃位置与物理行号

`abandon` 使用 `lexer.offset()-1` 回退到正在读取的行，再经 `line_map` 换算物理行号。每个外层文件独立补发自己的放弃报告，因此级联既保留由内向外的顺序，也保留各层文件对应的读取位置。^[atlas-core-session-frame.md:66-68]

物理行映射来自预处理：每行先剥除尾部空白，若随后以 `\` 结尾，就与下一行直接拼接，不插入额外内容。`line_map` 将重写后的行映射到其首个物理行，供 `describe` 和 `abandon` 使用。^[atlas-core-session-frame.md:72-79]

## 与会话状态的关系

放弃包含栈与弄脏会话是两个不同的状态变化。语法、类型和求值错误会令 `clean=false`；打开失败与 abandon 级联本身刻意保持原有 clean 状态。测试 `missing_file_aborts_the_include_stack_but_not_the_session` 明确断言缺失文件导致包含栈中止后，`is_clean()` 仍为真。相关状态纪律见 [[clean 标志与 CLI 退出状态]]。^[atlas-core-session-frame.md:43-46]

包含中的 `quit` 则结束整个会话：设置 `quitting` 后，`run_top_level` 不再继续喂入流。这是 [[SessionFrame 会话帧与文件包含语义]] 中另一种终止路径。^[atlas-core-session-frame.md:47-48]

## 测试与证据边界

源文件中的单元测试覆盖缺失文件中止、级联内层优先、循环静默跳过、quit 传播和物理行报告等行为。这些属于单元级行为锚点；端到端文件命令兼容仍以 HPC 差分语料为准。源包为结构性阅读记录，不声称语言或数学验收，可结合 [[HPC 验收证据链]] 理解其证据范围。^[atlas-core-session-frame.md:9-16, atlas-core-session-frame.md:81-90, atlas-core-session-frame.md:96-97]

## Sources

- [atlas-core-session-frame.md](atlas-core-session-frame.md)
