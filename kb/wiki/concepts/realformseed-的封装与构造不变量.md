---
title: RealFormSeed 的封装与构造不变量
summary: 私有字段将 grading offset、精确 square-class cocharacter 与基本对合处的归约种子绑定到共享表编号，并保持 grading_offset 等于 grading_of_simples(cocharacter)。
sources:
  - real-form-seed.md
kind: concept
createdAt: "2026-10-09T15:06:03.762Z"
updatedAt: "2026-10-10T00:47:01.268Z"
tags:
  - KGB
  - Rust
  - 构造不变量
aliases:
  - realformseed-的封装与构造不变量
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: RealFormSeed 的封装与构造不变量
summary: RealFormSeed 以私有字段绑定共享表编号下的 grading offset、精确平方类余特征与基本对合处的归约种子，并保持 grading 与余特征一致。
sources:
  - real-form-seed.md
kind: concept
tags:
  - KGB
  - 种子构造
  - 类型不变量
aliases:
  - realformseed-的封装与构造不变量
---

# RealFormSeed 的封装与构造不变量

`RealFormSeed` 表示一个弱实形式的 KGB 种子，将选定的 base-grading offset、精确的平方类余特征（square-class cocharacter），以及在基本对合（fundamental involution）处归约的种子元素封装在一起。这三部分使用同一个共享表的编号；调用方为每个内类维护一个仅追加（append-only）的表。^[real-form-seed.md:49-54]

## 私有字段与核心不变量

字段保持私有，避免调用方直接组装出彼此不匹配的三元组。核心构造不变量是 `grading_offset == grading_of_simples(cocharacter)`，即 grading offset 与余特征导出的简单根 grading 一致。种子验证采用上游自身的 x0-compacts 断言。^[real-form-seed.md:49-54]

公开访问器为 `form()`、`grading_offset()`、`square_class_cocharacter()` 和 `element()`，分别提供实形式标识、grading offset、平方类余特征和种子元素的访问。^[real-form-seed.md:64-65]

## 构造门控与自定义种子

`build` 首先检查共享表的内类相等；随后逐一比较 Weyl action 与根 involution 矩阵，检查 classification 的 fundamental class 是否归一化为该 datum 与 delta 上的恒等 twisted involution。后续门控涉及强层的计数一致性降级及 form id 越界检查，详见 [[RealFormSeed 的构建门控与自定义种子]]。^[real-form-seed.md:56-59]

`custom` 对应上游 `real_form_value::build` 的自定义分支，接收显式的 `(cocharacter, torus part)` 对。cocharacter 与简单根的配对须为整数，torus part 须复现指定 form 的 compact pattern。^[real-form-seed.md:59-62]

## 代表元选择的可观测影响

种子构造按两步落地：先实现精确有理数机制，包括 [[stable_log：选举的稳定对数]] 与 [[基本余权的精确构造]]，再实现 `RealFormSeed` 构建器。`stable_log` 选定的 adapted-basis 代表元固定 `g_rho_check`，进而固定每个下游 `torus_factor` 的有理数值，因此代表元选择承载可观测量。相关背景见 [[承载可观测量的适配基（adapted_basis）]]。^[real-form-seed.md:19-22]

## 证据范围

本页依据 `real_form_seed.rs` 的结构性阅读；来源包经过维护者逐条对照源码核对，所读字节来自 dirty 工作区，并由阅读快照记录。种子正确性属于 fundamental-lattice、seed gate 等独立 HPC 证据链，来源包不重述或扩展这些证据。^[real-form-seed.md:9-15]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。其中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[real-form-seed.md:72-79]

## Sources

- [real-form-seed.md](../../sources/real-form-seed.md) — KGB 种子 x0：stable_log、基本余权与 RealFormSeed。
