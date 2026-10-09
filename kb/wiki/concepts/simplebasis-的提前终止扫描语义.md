---
title: simple_basis 的提前终止扫描语义
summary: simple_basis 仅接收正根；候选因反射结果非正而移除自身时终止整个外层扫描，后续候选不再检查，来源称此行为保留上游算法特征。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T21:07:15.509Z"
updatedAt: "2026-10-09T21:07:15.509Z"
tags:
  - 根子系统
  - 单根提取
  - 兼容性
aliases:
  - simplebasis-的提前终止扫描语义
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# simple_basis 的提前终止扫描语义

`simple_basis` 是 `real_weyl.rs` 的文件私有辅助函数，其调用方只传入正根。实现保留了上游的一项特殊扫描行为：当候选因反射结果非正而移除自身时，**整个外层扫描立即终止**，后续候选不再接受检查。来源将这一行为对应到上游 `rootdata.cpp:621–652`。^[real-weyl.md:49-54, real-weyl.md:135-137]

## 终止条件与作用范围

理解这一语义时，需要区分“移除当前候选”和“继续扫描其他候选”。在上述自身移除情形中，控制流会退出整个外层扫描，而不只是结束当前候选的处理。因此，后续候选未经检查是该实现保留的行为；将其改为移除后继续遍历，会改变来源明确记录的扫描语义。^[real-weyl.md:135-137]

正根输入是现有调用方遵守的前提。来源未给出任意含负根输入下的行为契约，因而不能把这项扫描规则推广为面向任意根集合的接口保证。^[real-weyl.md:135-137]

## 在实 Weyl 构造中的用途

在两侧共用的 `fiber_side` 中，算法先按上游键排序正虚根，再依据 grading 判定紧性，并通过 `compact_basis = simple_basis(compact)` 提取紧根基。这使提前终止规则成为紧根基构造所保留的实现细节；相关背景见 [[fiber grading 与 R-群核生成元]]。^[real-weyl.md:118-137]

`simple_complex` 则先选出同时与正虚根之和、正实根之和正交的正根，再求简单基并进行 Dynkin 分类。随后，它从对合配对的分量中各保留一个。简单基提取与后续分量筛选是不同步骤，可结合 [[实 Weyl 群的根列表与排序契约]] 阅读。^[real-weyl.md:139-142]

## 兼容性与证据边界

来源将提前终止明确记录为需要保留的上游特殊行为，但没有提供其数学必要性的证明。整份材料属于结构性源码阅读，不声称实 Weyl 层已通过数学验收；上游文件位置仅转录自代码注释，未独立核对上游字节。^[real-weyl.md:9-13, real-weyl.md:135-137]

材料列出了实 Weyl 打印层的七组 fixture，以及逐字节输出和错误路径锚点，但没有列出专门隔离 `simple_basis` 提前终止分支的测试。因此，这些整体回归锚点不能直接视为该分支的专项覆盖证明。^[real-weyl.md:159-168]

## Sources

- [real-weyl.md](../../sources/real-weyl.md) — 实 Weyl 群与块稳定子：`real_weyl.rs` 的构造、对偶 fiber 重放与打印层。
