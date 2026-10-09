---
title: RealFormSeed 的构建门控与自定义种子
summary: build 检查内类、基本 Cartan 类归一化、强层计数及形式编号；custom 要求显式余特征的简单根配对为整数且环面部分复现指定形式的紧根模式。
sources:
  - real-form-seed.md
kind: concept
createdAt: "2026-10-09T15:06:12.096Z"
updatedAt: "2026-10-09T19:34:32.375Z"
tags:
  - KGB种子
  - 输入校验
aliases:
  - realformseed-的构建门控与自定义种子
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# RealFormSeed 的构建门控与自定义种子

`RealFormSeed` 表示一个弱实形式的 KGB 种子，包含选定的 base-grading offset、精确的 square-class cocharacter，以及在 fundamental involution 处归约的种子元素。这些数据使用同一个共享表的编号；调用方为每个 inner class 维护一个仅追加的表。^[real-form-seed.md:49-54]

## 封装与构造不变量

`RealFormSeed` 的字段保持私有，以避免公开组装出彼此不匹配的三元组。构造不变量为 `grading_offset == grading_of_simples(cocharacter)`；种子验证采用上游自身的 x0-compacts 断言。相关封装见 [[RealFormSeed 的封装与构造不变量]]。^[real-form-seed.md:49-54]

## `build` 的门控链

`build` 首先检查共享表的 inner class 是否相等，沿用 TitsCoset idiom。随后检查 classification 的 fundamental class 是否归一化为该 datum 与 delta 上的恒等 twisted involution；检查通过逐一比较 Weyl action 与根 involution 矩阵完成。共享表的背景见 [[Twisted involution 表与 Cartan 轨道存储]]。^[real-form-seed.md:56-58]

后续门控包括强层的计数一致性降级与 form id 越界检查。来源仅概述这些门控，没有展开降级的具体行为；强层背景可参见 [[Cartan 类的强实层 StrongRealData]]。^[real-form-seed.md:56-59]

## `custom` 自定义种子

`custom` 对应上游 `real_form_value::build` 的 custom 分支，接受显式的 `(cocharacter, torus part)` 对。cocharacter 与简单根的配对必须为整数，torus part 必须复现该 form 的 compact pattern。^[real-form-seed.md:59-62]

## 代表元选择与访问接口

种子构造中的代表元选择承载可观测量：`stable_log` 的 adapted-basis 代表元固定 `g_rho_check`，继而固定每个下游 `torus_factor` 的有理数值。相关机制见 [[stable_log：选举的稳定对数]] 与 [[承载可观测量的适配基（adapted_basis）]]。^[real-form-seed.md:19-22]

构造后的种子提供 `form()`、`grading_offset()`、`square_class_cocharacter()` 与 `element()` 四个访问器。^[real-form-seed.md:64-65]

## 证据边界

本页依据对 `real_form_seed.rs` 的结构性阅读，所读源码来自 dirty 工作区快照。种子正确性属于其自身的 [[HPC 验收证据链]]，包括 fundamental-lattice 与 seed gate 等；来源包不重述或扩展这条证据链。^[real-form-seed.md:9-15]

来源包未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[real-form-seed.md:69-79]

## Sources

- [real-form-seed.md](real-form-seed.md) — KGB 种子 x0：stable_log、基本余权与 RealFormSeed
