---
title: 文件输入的有损 UTF-8 解码
summary: FsProvider 使用有损 UTF-8 解码读取文件，避免将游离的非 UTF-8 字节误判为文件打开失败。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:07.774Z"
updatedAt: "2026-10-09T22:11:52.640Z"
tags:
  - 文件输入
  - 文本编码
aliases:
  - 文件输入的有损-utf-8-解码
  - 文U解
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 文件输入的有损 UTF-8 解码
summary: FsProvider 使用有损 UTF-8 解码提供文件内容，避免将游离的非 UTF-8 字节视为文件打开失败。
sources:
  - atlas-cli-main.md
kind: concept
tags:
  - 文件输入
  - 字符编码
aliases:
  - 文件输入的有损-utf-8-解码
provenanceState: extracted
---

# 文件输入的有损 UTF-8 解码

文件输入的有损 UTF-8 解码是 Atlas Rust CLI 的文件系统 `FileProvider` 实现 `FsProvider` 所采用的读取策略。来源说明，上游读取字节，因此游离的非 UTF-8 字节不得变成文件打开失败；`FsProvider` 为此使用有损 UTF-8 解码。^[atlas-cli-main.md:18-19]

## 会话输入中的位置

CLI 通过会话帧，在文件参数或 stdin 上运行一个共享会话。`<file` 包含依次按可重复指定的 `--path=DIR` 前缀、再按工作目录解析。相关机制见 [[包含文件的搜索路径解析]] 与 [[会话帧驱动的 CLI 执行模型]]。^[atlas-cli-main.md:9-13]

文件参数被刻意作为普通命令流输入；上游文件参数的 prelude-capture 语义不在此实现目标内。stdin 非终端时读入并运行，终端输入则进入交互循环。^[atlas-cli-main.md:30-34]

## 输入解码与字节输出

输入侧的 `FsProvider` 使用有损 UTF-8 解码；输出侧的 `OutputBytes` 和 `ReportBytes` 则将原始字节写入 stdout。两种处理分别作用于输入和输出环节，相关输出机制见 [[SessionEvent 与字节保留输出]]。^[atlas-cli-main.md:18-25]

## 错误处理边界

游离的非 UTF-8 字节不得导致文件打开失败。会话退出状态则由 clean 标志决定：语法、类型或运行时错误使退出状态非零，缺少包含文件本身不设置该错误状态。详见 [[clean 标志与 CLI 退出状态]]。^[atlas-cli-main.md:18-19, atlas-cli-main.md:35-37]

## 证据范围

来源明确记录了有损 UTF-8 策略及其动机，但未给出具体解码调用或替换细节。材料属于结构性阅读，不声称语言验收；`main.rs` 本身没有测试，行为覆盖与 CLI 兼容性以 HPC 语料门为准。^[atlas-cli-main.md:9-19, atlas-cli-main.md:39-44]

## Sources

- [atlas-cli-main.md](../../sources/atlas-cli-main.md)：CLI 前端（atlas-cli/main.rs）——会话帧驱动、--path 解析与 clean 退出状态。
