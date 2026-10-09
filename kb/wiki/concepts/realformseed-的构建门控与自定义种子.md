---
title: RealFormSeed 的构建门控与自定义种子
summary: build 检查 inner class、fundamental class 归一化、强层计数一致性降级及 form id 边界；custom 要求显式 cocharacter 的 simple pairings 为整数且 torus part 复现该 form 的 compact pattern。
sources:
  - real-form-seed.md
kind: concept
createdAt: "2026-10-09T15:06:12.096Z"
updatedAt: "2026-10-09T15:06:12.096Z"
tags:
  - 种子构造
  - 输入验证
  - 实形式
aliases:
  - realformseed-的构建门控与自定义种子
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# RealFormSeed 的构建门控与自定义种子

`RealFormSeed` 表示一个弱实形式的 KGB 种子，包含选定的 base-grading offset、精确的 square-class cocharacter，以及在 fundamental involution 处归约的种子元素。这些数据使用同一个共享表的编号；调用方为每个 inner class 维护一个 append-only 表。^[real-form-seed.md:47-54]

## 构造不变量

`RealFormSeed` 的字段保持私有，以避免调用方组装出彼此不匹配的三元组。其构造不变量为 `grading_offset == grading_of_simples(cocharacter)`，种子验证采用上游自身的 x0-compacts 断言。相关封装见 [[RealFormSeed 的封装与构造不变量]]。^[real-form-seed.md:49-54]

## `build` 的门控链

`build` 首先检查共享表的 inner class 是否相等，采用 TitsCoset idiom。随后检查 classification 的 fundamental class 是否归一化为该 datum 与 delta 上的恒等 twisted involution；这一检查逐一比较 Weyl action 与根 involution 矩阵。门控链还包含强层的计数一致性降级与 form id 越界检查。相关背景见 [[Twisted involution 表与 Cartan 轨道存储]] 与 [[Cartan 类的强实层 StrongRealData]]。^[real-form-seed.md:56-59]

## `custom` 自定义种子

`custom` 对应上游 `real_form_value::build` 的 custom 分支，接受显式的 `(cocharacter, torus part)` 对。输入须满足两个条件：cocharacter 与简单根的配对为整数，且 torus part 必须复现该 form 的 compact pattern。^[real-form-seed.md:59-62]

## 代表元选择与访问接口

种子构造中的代表元选择承载可观测量：`stable_log` 的 adapted-basis 代表元固定 `g_rho_check`，继而固定每个下游 `torus_factor` 的有理数值。因此，[[stable_log：选举的稳定对数]] 是理解种子代表元及其下游影响的相关概念。^[real-form-seed.md:19-22]

构造后的种子通过 `form()`、`grading_offset()`、`square_class_cocharacter()` 与 `element()` 提供访问接口。^[real-form-seed.md:64-65]

## 证据边界

本页依据的来源包是对 `real_form_seed.rs` 的结构性阅读，记录的源码字节来自 dirty 工作区快照。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；种子正确性属于独立的 [[HPC 验收证据链]]。其中上游位置来自 Rust 源码注释的转述，未独立重读上游，行号可能随版本变化。^[real-form-seed.md:9-15, real-form-seed.md:69-79]

## Sources

- [real-form-seed.md](real-form-seed.md) — KGB 种子 x0：stable_log、基本余权与 RealFormSeed
