---
title: 文件输入的有损 UTF-8 解码
summary: FsProvider 以有损 UTF-8 方式提供文件内容，避免将游离的非 UTF-8 字节误判为文件打开失败。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:07.774Z"
updatedAt: "2026-10-09T20:29:23.102Z"
tags:
  - 文件输入
  - 字符编码
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

# 文件输入的有损 UTF-8 解码

文件输入的有损 UTF-8 解码是 Atlas Rust CLI 的 `FsProvider` 采用的读取策略。`FsProvider` 是文件系统 `FileProvider`；由于上游读取字节，输入中游离的非 UTF-8 字节不得被当作文件打开失败，因此这里采用有损 UTF-8 解码。^[atlas-cli-main.md:18-19]

## 在会话输入中的位置

CLI 通过会话帧，在文件参数或 stdin 上运行一个共享会话。`<file` 包含依次按可重复指定的 `--path=DIR` 前缀、再按工作目录解析。相关入口机制见 [[包含文件的搜索路径解析]] 与 [[会话帧驱动的 CLI 执行模型]]。^[atlas-cli-main.md:9-13]

文件参数被刻意作为普通命令流输入；上游文件参数的 prelude-capture 语义不在此实现目标内。有损解码策略并不代表文件参数处理已经实现全部上游语义。^[atlas-cli-main.md:30-31]

## 与原始字节输出的区别

输入侧的 `FsProvider` 使用有损 UTF-8 解码，输出侧的 `OutputBytes` 和 `ReportBytes` 则直接将原始字节写入 stdout。这两项约定分别适用于读取和输出环节；输出保留原始字节的能力不能作为文件输入保留原始字节的依据。相关机制见 [[SessionEvent 与字节保留输出]]。^[atlas-cli-main.md:18-25]

## 错误处理边界

非 UTF-8 字节不得变成打开失败，是文件读取的错误分类要求。会话退出状态另由 clean 标志决定：语法、类型或运行时错误使退出状态非零，缺少包含文件本身则不设置这一错误状态。相关规则见 [[clean 标志与 CLI 退出状态]]。^[atlas-cli-main.md:18-19, atlas-cli-main.md:35-37]

## 证据范围

来源明确记录了有损 UTF-8 策略及其动机，但未给出具体解码调用或替换细节。该材料属于结构性阅读，不声称语言验收；`main.rs` 本身没有测试，行为覆盖与 CLI 兼容性以 HPC 语料门为准。^[atlas-cli-main.md:9-19, atlas-cli-main.md:39-44]

## Sources

- [atlas-cli-main.md](../../sources/atlas-cli-main.md)：CLI 前端（atlas-cli/main.rs）——会话帧驱动、--path 解析与 clean 退出状态。
