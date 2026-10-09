---
title: 嵌套包含的 abandon 级联
summary: 包含流中止时利用 lexer 偏移与 line_map 定位物理行，各包含层从最内层向外报告放弃读取的位置。
sources:
  - atlas-core-session-frame.md
kind: concept
createdAt: "2026-10-09T14:34:00.426Z"
updatedAt: "2026-10-09T22:19:09.797Z"
tags:
  - 文件包含
  - 诊断回溯
aliases:
  - 嵌套包含的-abandon-级联
  - 嵌A级
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 嵌套包含的 abandon 级联
summary: 文件包含中止时，各层从最内层向外报告放弃位置，并通过 lexer 偏移和 line_map 定位物理行；级联本身不改变会话的 clean 状态。
sources:
  - atlas-core-session-frame.md
kind: concept
tags:
  - 文件包含
  - 错误传播
  - 源码定位
aliases:
  - 嵌套包含的-abandon-级联
provenanceState: extracted
---

# 嵌套包含的 abandon 级联

嵌套包含的 abandon 级联是 `SessionFrame` 在文件包含中止时逐层报告放弃位置的机制。内层中止后，各外层分别补发 `Abandoning reading of file '{origin}' at line {physical}`，报告顺序从最内层向外展开。^[atlas-core-session-frame.md:66-68]

## 包含栈与中止条件

`SessionFrame` 的 `active` 栈记录当前包含链，用于循环守卫、深度检查和回溯命名。正常包含会打印 `Starting` 框架行、压栈并递归执行 `run_stream`；返回 `Finished` 时，才将文件记入 `completed` 并打印 `Completely` 框架行。相关结构见 [[SessionFrame 会话帧与文件包含语义]]。^[atlas-core-session-frame.md:29-40]

`resolve` 查找失败会产生 `failed to open input file …` 的 Io 诊断并返回 `Abort`，缺失文件可因此中止当前包含栈。已经位于 `active` 栈中的文件则静默跳过并视为成功。包含深度上限为 `MAX_INCLUDE_DEPTH = 64`，超深同样产生 Io 诊断并中止；循环守卫处理真实递归，深度上限用于约束病态 provider。路径解析见 [[包含文件的搜索路径解析]]。^[atlas-core-session-frame.md:34-46]

命令执行产生非 Io 诊断时，会将会话标记为不 clean，并使本命令返回 `Abort`。词法层区分 warning 与真正的错误：warning 只报告，不中止也不弄脏会话；真正的词法错误会标记 dirty，并在包含深度大于零时触发 abandon 级联。^[atlas-core-session-frame.md:56-58, atlas-core-session-frame.md:69-70]

## 放弃位置与物理行号

`abandon` 使用 `lexer.offset()-1` 回退到正在读取的行，再经 `line_map` 换算物理行号。每个外层文件分别报告自己的放弃位置，因此级联保留了包含链各层的文件名与行号。^[atlas-core-session-frame.md:66-68]

`line_map` 来自输入预处理：每行先剥除尾部空白，若剥除后以 `\` 结尾，就与下一行直接拼接，不插入额外内容。因此，反斜杠后带空白的行仍可续行。映射将重写后的行对应到其首个物理行，供 `describe` 和 `abandon` 将诊断或放弃位置映射回物理行。^[atlas-core-session-frame.md:72-79]

## 与会话状态的关系

包含栈中止与会话变为不 clean 是两个不同的状态变化。语法、类型和求值错误会令 `clean=false`；打开失败与 abandon 级联本身不会弄脏会话。测试 `missing_file_aborts_the_include_stack_but_not_the_session` 明确断言：缺失文件导致包含栈中止后，`is_clean()` 仍为真。相关约定见 [[clean 标志与 CLI 退出状态]]。^[atlas-core-session-frame.md:43-46]

包含中的 `quit` 是另一条终止路径：它结束整个会话，`run_top_level` 在 `quitting` 后不再继续喂入流。^[atlas-core-session-frame.md:47-48]

## 测试与证据边界

源文件的单元测试包含缺失文件中止、级联内层优先、循环静默跳过、quit 传播及物理行报告等行为锚点。这些证据限于单元级行为；端到端文件命令兼容仍以 HPC 差分语料门为准，参见 [[HPC 验收证据链]]。^[atlas-core-session-frame.md:81-90]

源包属于结构性阅读记录，不声称语言或数学验收。其中上游 `buffer.w`、`main.w`、`parser.y` 的行号引用属于实现方的移植陈述，不能替代端到端可见行为的验证。^[atlas-core-session-frame.md:9-16, atlas-core-session-frame.md:94-99]

## Sources

- [atlas-core-session-frame.md](../../sources/atlas-core-session-frame.md) — 会话帧：文件包含、输出重定向与顶层输出面（session_frame.rs）。
