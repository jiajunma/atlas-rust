---
title: 实 Weyl 层的移植范围与刻意省略
summary: 生成元直接提供 canonical word，移植省略反射词构造机制、调试尺寸断言及 printDualRealWeyl，但仍计算后者所需的虚根与实根生成元列表。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T21:07:39.527Z"
updatedAt: "2026-10-09T21:07:39.527Z"
tags:
  - Rust-移植
  - 实-Weyl-群
  - 实现边界
aliases:
  - 实-weyl-层的移植范围与刻意省略
  - 实W层
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 实 Weyl 层的移植范围与刻意省略

实 Weyl 层位于 `crates/atlas-real-group/src/real_weyl.rs`，移植了实 Weyl 群与块稳定子所需的数据构造、生成元及打印逻辑。其范围包括上游 `realweyl::RealWeyl`、`realweyl::RealWeylGenerators`、`realweyl_io::common_print`，以及 `output::printRealWeyl`／`printBlockStabilizer` 的代表元选取。^[real-weyl.md:10-13, real-weyl.md:17-25]

## 已移植的职责

`RealWeyl` 针对 Cartan 类及实形、对偶实形代表元，收集简单虚根、实根、复根及其子系统类型、紧根基与两个 R-群。打印包装中，实形代表取 `G_C.representative(rf,cn)`；实 Weyl 群使用对偶伴随 fiber 的零元，块稳定子使用指定对偶形式的代表元，详见 [[实 Weyl 群与块稳定子的代表元选择]]。^[real-weyl.md:17-25]

公开入口由 `RealWeylContext` 提供，包括 `real_weyl`、`real_weyl_print` 与 `block_stabilizer_print`。`RealWeyl` 和 `RealWeylGenerators` 的字段私有，仅通过 getter 暴露；`DualSide`、`FiberSide` 及辅助函数保持文件私有，`twisted_orbit_size` 仅为 `pub(crate)`。^[real-weyl.md:29-54, real-weyl.md:181-182]

## 刻意省略的上游机制

### 反射词与优势化机器

实现未移植 `reflection_word`／`to_dominant` 机制，因为生成元按构造即具有 canonical 表示。每个根的反射经 `WeylAction::root_reflection` 转为 `WeylElement`，打印直接使用 `WeylElement::canonical_word`；来源说明其结果与上游 `WeylGroup::word` 所得规范词相同。相关约定见 [[Weyl 元素的规范词]]。^[real-weyl.md:75-81, real-weyl.md:172-172]

### 调试尺寸断言

上游打印末尾受 `#ifndef NDEBUG` 控制的尺寸断言及其 `weylsize` 计算未移植。这一省略针对打印路径的调试检查；模块仍有 `twisted_orbit_size`，以虚、实、复三个子系统的 Weyl 群阶之积计算稳定子阶，并将整除性失败作为不变量错误。^[real-weyl.md:144-147, real-weyl.md:173-173]

### 对偶实 Weyl 群打印入口

`printDualRealWeyl` 未移植，原因是没有内建包装使用它。文件私有的 `PrintKind` 仅包含 `RealWeyl` 与 `BlockStabilizer`，没有上游的 `dual_real_W` 分支；不过，对偶打印所需的 imaginary／real 生成元列表仍在计算，来源将后续补齐描述为仅涉及打印层的改动。^[real-weyl.md:49-50, real-weyl.md:174-175]

### gkmod 包装子系统

上游 gkmod 的 `blockstabilizer` 子系统未作为独立子系统移植，因为该包装只转发 `(rf, cn, drf)`。块稳定子的构造与打印能力仍由实 Weyl 层入口提供。^[real-weyl.md:22-25, real-weyl.md:38-43]

## 保留的兼容契约

打印层保留了上游可观察的格式细节，包括两类输出的不同头行、摘要行数、非平凡 `W^C` 的 `isomorphic to ` 前缀，以及生成元节标题中不一致的冒号。空词打印为 `e`，非空词按从 1 开始的编号以逗号连接；每行终止，摘要与节之间恰好一个空行，末节之后无多余字节。^[real-weyl.md:149-157]

对偶侧并未因省略对偶打印入口而省去计算。由于所需对偶 Cartan 对合通常只是典范对偶代表元的共轭，实现每次调用都临时重建对偶 fiber、grading、弱实形式分区及标签链，并使用 `RealWeylContext.budget` 的相应子预算。该路径没有缓存；来源未确认这一取舍是否有意，也未据此作性能声明，详见 [[对偶 Cartan fiber 链的临时重建]]。^[real-weyl.md:104-116, real-weyl.md:186-187]

## 证据与覆盖边界

测试注释声明，七个 fixture 的预期输出来自固定上游构建 `rev 4d3e9449`，于 2026-08-11 重新生成并逐字节复制进断言。锚点覆盖 B/C 类型互换、复因子打印前缀、rank-2 R-群核生成元顺序，以及形式无定义和编号越界错误；预算耗尽与非 quasisplit 对偶形式的行为没有测试锚点。^[real-weyl.md:161-168, real-weyl.md:188-188]

来源属于结构性源码阅读，不构成实 Weyl 层的数学验收。上游行号仅转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。因此，移植范围、刻意省略和已有测试锚点应与已验证的数学正确性保持区分。^[real-weyl.md:9-13, real-weyl.md:192-196]

## Sources

- [real-weyl.md](../../sources/real-weyl.md) — 实 Weyl 群与块稳定子：`real_weyl.rs` 的构造、对偶 fiber 重放与打印层。
