---
title: RealFormSeed 的封装与构造不变量
summary: RealFormSeed 在共享 append-only 表的编号下绑定 grading offset、square-class cocharacter 与种子元素，以私有字段保证 grading_offset 等于 grading_of_simples(cocharacter)。
sources:
  - real-form-seed.md
kind: concept
createdAt: "2026-10-09T15:06:03.762Z"
updatedAt: "2026-10-09T15:06:03.762Z"
tags:
  - 实形式
  - KGB种子
  - Rust设计
aliases:
  - realformseed-的封装与构造不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# RealFormSeed 的封装与构造不变量

`RealFormSeed` 表示一个弱实形式的 KGB 种子，将选定的 base-grading offset、精确的 square-class cocharacter，以及在 fundamental involution 处归约的种子元素封装在一起。这些数据使用同一个共享表的编号；调用方须为每个 inner class 维护一个 append-only 表。^[real-form-seed.md:49-54]

## 封装与核心不变量

`RealFormSeed` 的字段为私有字段，以避免调用方直接组装出彼此不匹配的三元组。其核心构造不变量是 `grading_offset == grading_of_simples(cocharacter)`，即 grading offset 必须与 cocharacter 导出的简单根 grading 一致。种子验证采用上游自身的 x0-compacts 断言。^[real-form-seed.md:49-54]

公开访问器包括 `form()`、`grading_offset()`、`square_class_cocharacter()` 和 `element()`，分别提供实形式标识、grading offset、平方类余特征和种子元素的访问。^[real-form-seed.md:64-65]

## 构造门控

`build` 首先检查共享表的 inner class 相等，沿用 TitsCoset 的检查惯例；随后检查 classification 的 fundamental class 是否归一化为该 datum 与 delta 上的恒等 twisted involution，具体通过逐一比较 Weyl action 与根 involution 矩阵完成。后续门控还涉及强层的计数一致性降级，以及 form id 越界检查。相关背景见 [[RealFormSeed 的构建门控与自定义种子]]。^[real-form-seed.md:56-59]

`custom` 对应上游 `real_form_value::build` 的自定义分支，接收显式的 `(cocharacter, torus part)` 对。cocharacter 与简单根的配对必须为整数，torus part 则必须复现指定 form 的 compact pattern。^[real-form-seed.md:59-62]

## 代表元选择的可观测影响

种子构造分为精确有理数机制与 `RealFormSeed` 构建器两步，前者包括 [[stable_log：选举的稳定对数]] 和 [[基本余权的精确构造]]。其中，`stable_log` 选定的 adapted-basis 代表元固定 `g_rho_check`，进而固定每个下游 `torus_factor` 的有理数值，因此该代表元选择承载可观测量；参见 [[承载可观测量的适配基（adapted_basis）]]。^[real-form-seed.md:19-22]

## 证据范围

本页依据的来源包是对 `real_form_seed.rs` 的结构性阅读，记录的源码字节来自 dirty 工作区。该包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；种子正确性所属的 fundamental-lattice、seed gate 等 HPC 证据链也未在包中重述或扩展。^[real-form-seed.md:9-15, real-form-seed.md:79-79]

来源包中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[real-form-seed.md:72-73]

## Sources

- [real-form-seed.md](real-form-seed.md) — KGB 种子 x0：stable_log、基本余权与 RealFormSeed
