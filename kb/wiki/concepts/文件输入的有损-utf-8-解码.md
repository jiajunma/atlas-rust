---
title: 文件输入的有损 UTF-8 解码
summary: FsProvider 以有损 UTF-8 方式提供文件内容，避免将游离的非 UTF-8 字节误判为文件打开失败。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:07.774Z"
updatedAt: "2026-10-09T14:25:07.774Z"
tags:
  - 文件输入
  - 字符编码
  - 错误处理
aliases:
  - 文件输入的有损-utf-8-解码
  - 文U解
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 文件输入的有损 UTF-8 解码

文件输入的有损 UTF-8 解码是 Atlas Rust CLI 的 `FsProvider` 所采用的读取策略。`FsProvider` 实现文件系统 `FileProvider`；由于上游读取字节，输入中游离的非 UTF-8 字节不得被当作打开文件失败，因此这里采用有损 UTF-8 解码。^[atlas-cli-main.md:18-19]

## 在会话输入中的位置

CLI 通过会话帧让文件参数或 stdin 共用一个会话。`<file` 包含依次按可重复指定的 `--path=DIR` 前缀、再按工作目录解析；路径解析与文件内容解码共同构成文件输入入口，可参见 [[包含文件的搜索路径解析]] 与 [[会话帧驱动的 CLI 执行模型]]。^[atlas-cli-main.md:9-13]

文件参数被刻意作为普通命令流输入；上游文件参数的 prelude-capture 语义不在此实现目标内。因此，有损解码这一读取策略本身不能说明文件参数具有完整的上游兼容语义。^[atlas-cli-main.md:30-31]

## 与字节输出的区别

输入侧采用有损 UTF-8 解码，而输出侧的 `OutputBytes` 和 `ReportBytes` 会将原始字节写入 stdout。二者属于不同环节，不能把输出的原始字节保留能力理解为文件输入也保留原始字节。相关输出机制见 [[SessionEvent 与字节保留输出]]。^[atlas-cli-main.md:18-25]

## 证据范围

来源仅明确说明有损解码策略及其动机，没有给出具体解码调用或替换细节。该文档属于结构性阅读，不声称语言验收；`main.rs` 本身没有测试，CLI 兼容性以 HPC 语料门为准。参见 [[CLI 兼容性的证据边界]]。^[atlas-cli-main.md:14-19, atlas-cli-main.md:39-44]

## Sources

- [atlas-cli-main.md](atlas-cli-main.md)
