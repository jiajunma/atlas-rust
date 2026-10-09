---
title: KL 支撑层的测试覆盖与证据边界
summary: 四个 FakeTopology 单元测试覆盖基本位操作和三条构造拒绝路径，构造成功路径、判定方法及本原索引机制无单元测试；来源称其经 KL 层 HPC 门覆盖，但本次未执行验证或核实上游引用。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:56:19.193Z"
updatedAt: "2026-10-09T14:56:19.193Z"
tags:
  - 测试覆盖
  - 证据边界
  - HPC验证
aliases:
  - kl-支撑层的测试覆盖与证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# KL 支撑层的测试覆盖与证据边界

KL 支撑层的现有证据来自对 `crates/atlas-real-group/src/kl_support.rs` 共 467 行的结构性阅读，以及源材料记录的测试锚点。草案由 Kimi probe 起草，维护者对照源码逐条核对改写；这一编辑状态不代表 KL 支撑层已经通过数学验收。^[kl-support.md:9-12]

## 单元测试覆盖

源材料记录了 4 个通过 `FakeTopology` 替身组织的测试锚点：[[RankFlags：简单生成元位集]]的基本位操作，以及三条构造拒绝路径。位操作覆盖包含“空集被任何集合包含”的语义；拒绝路径分别检查 rank 为 33 时超出容量、元素长度未按非降序排列，以及 cross 链接目标越界。^[kl-support.md:57-62]

这些拒绝测试涉及[[KlSupport 的拓扑构造门控]]的部分约束。构造器还检查逐元素长度存在、逐生成元的 descent/cayley/inverse_cayley 数据存在，以及 cross 与 Cayley 像的目标位于块内；现有测试锚点不构成对所有构造检查的完整覆盖。^[kl-support.md:32-39, kl-support.md:57-62]

## 覆盖缺口

`KlSupport::new` 的成功路径、全部判定方法和整个本原索引机制均无单元测试。源材料注明这些部分经 KL 层的 HPC 门覆盖，但未给出具体测试案例、报告或验收结果，因此不能据此描述其完整覆盖范围。^[kl-support.md:59-62]

判定方法涉及[[下降集、good ascent 与本原性]]、extremal 判定、唯一上升像和本原元素回退。其中，`ImaginaryTypeII` 既不是下降，也不是 good ascent；`prim_back_up` 先自减再判定，失败时仍将输入位置置为 0。这些分类与原地修改契约属于缺少直接单元测试的行为范围。^[kl-support.md:35-45, kl-support.md:59-62]

本原索引机制还包含幂等准备、降序扫描、`DEAD_END` 哨兵处理和最终索引反转。`prim_index`、`nr_of_primitives`、`col_size` 与 `self_index` 均要求预先对同一下降集调用 `prepare_prim_index`，否则会因映射缺键而 panic；源材料指出，只有 `prim_index` 的文档注释显式说明了这一前置条件。^[kl-support.md:47-55]

## 顺序与接口的证据边界

元素长度非降由构造门控强制保证；本原索引降序扫描所依赖的“上升像序号更大”则只是源码阅读观察，未见显式防护。二者的证据强度不同，不能将后者表述为已经由构造器验证的不变量。^[kl-support.md:70-71]

源材料未读取定义于 `crate::block` 的 `BlockDescent` 完整变体集及其 `is_descent()` 定义，因此不能凭本包独立确认全部下降类型语义。[[BlockTopology 只读块拓扑接口]]是 sealed trait，测试替身需要实现私有 `Sealed`。^[kl-support.md:68-69]

## 来源身份与验收限制

源材料中的 `klsupport.h/cpp`、`blocks.h` 和 `kl.cpp` 上游行号仅转录自代码注释，未核对上游文件字节。因此，这些引用提供的是追溯线索，不构成已核实的上游兼容性或数学正确性证据。^[kl-support.md:10-12, kl-support.md:64-66]

精确读取身份记录于快照 `2026-10-06-kl-support.json`，其中绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。本次知识维护未执行 Atlas、Cargo、测试或 benchmark；结构性阅读结论应与[[HPC 验收证据链]]中的运行及验收证据区分。^[kl-support.md:73-79]

## Sources

- [逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）](../../sources/kl-support.md)
