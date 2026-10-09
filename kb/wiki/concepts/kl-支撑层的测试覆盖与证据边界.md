---
title: KL 支撑层的测试覆盖与证据边界
summary: 四项替身测试覆盖基本位操作与三条构造拒绝路径，未单测构造成功、判定及本原索引机制；本包未执行测试、核对上游字节或完成数学验收。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:56:19.193Z"
updatedAt: "2026-10-09T22:36:03.428Z"
tags:
  - 测试覆盖
  - 证据边界
aliases:
  - kl-支撑层的测试覆盖与证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KL 支撑层的测试覆盖与证据边界
summary: 四个测试锚点覆盖基本位操作与三条构造拒绝路径；构造成功路径、判定方法及本原索引机制缺少单元测试。来源记载其经 KL 层 HPC 门覆盖，但本包仅作结构性阅读，不构成数学验收。
sources:
  - kl-support.md
kind: concept
tags:
  - 测试覆盖
  - 证据边界
aliases:
  - kl-支撑层的测试覆盖与证据边界
---

# KL 支撑层的测试覆盖与证据边界

[[KlSupport：逐块 KL 支撑数据]]的覆盖说明来自对 `crates/atlas-real-group/src/kl_support.rs` 共 467 行的结构性阅读。源材料由 Kimi probe 起草，维护者对照源码逐条核对改写；这一编辑状态不代表 KL 支撑层已通过数学验收。^[kl-support.md:9-12, kl-support.md:75-79]

## 已有测试覆盖

源材料记录了 4 个经 `FakeTopology` 替身组织的测试锚点：[[RankFlags：简单生成元位集]]的基本位操作，以及三条构造拒绝路径。位操作包含“空集被任何集合包含”的语义；拒绝路径分别是 rank 为 33 时超出容量、元素长度未按非降序排列，以及 cross 链接目标越界。^[kl-support.md:57-62]

这些锚点仅覆盖[[KlSupport 的拓扑构造门控]]的部分检查。构造器还要求逐元素长度存在、逐生成元的 descent/cayley/inverse_cayley 数据存在，并检查 cross 与 Cayley 像的目标均在块内。^[kl-support.md:32-39, kl-support.md:57-62]

## 单元测试缺口与行为契约

`KlSupport::new` 的成功路径、全部判定方法和整个本原索引机制均无单元测试。源材料注明这些部分经 KL 层 HPC 门覆盖，但未在该段列出具体用例、运行报告或验收结果。^[kl-support.md:57-62]

缺少直接单元测试的判定行为包括[[下降集、good ascent 与本原性]]、extremal 判定及[[唯一上升像与本原元素回退]]。其中，`ImaginaryTypeII` 既不是下降，也不是 good ascent；`unique_ascent` 对复上升取 cross 像，对虚 I 型上升取第一个 Cayley 像，其余返回 `None`。`prim_back_up` 先自减再判定，失败时输入位置已被置为 0，原地修改属于接口契约。^[kl-support.md:35-45, kl-support.md:59-62]

本原索引机制包含幂等准备、降序扫描、`DEAD_END` 哨兵处理和最终索引反转。`prim_index`、`nr_of_primitives`、`col_size` 与 `self_index` 均要求先对同一下降集调用 `prepare_prim_index`，否则映射缺键会导致 panic；只有 `prim_index` 的文档注释显式说明了这一前置条件。^[kl-support.md:47-55]

## 不变量与源码覆盖边界

元素长度非降由构造门控强制保证。本原索引降序扫描所依赖的“上升像序号更大”则属于源码阅读观察，源材料指出没有显式防护，不能将其视为已经由构造器检查的不变量。^[kl-support.md:70-71]

`RankFlags` 使用私有 `u32` 位集，rank 上限为 32。`set` 和 `is_set` 本身不做边界检查，`KlSupport` 使用路径依靠构造门控保证容量约束；rank 33 的拒绝测试覆盖的是这一构造边界。^[kl-support.md:23-28, kl-support.md:32-33, kl-support.md:59-60]

本包未读取定义于 `crate::block` 的 `BlockDescent` 完整变体集及 `is_descent()` 定义。[[BlockTopology 只读块拓扑接口]]是 sealed trait，测试替身需要实现私有 `Sealed`。^[kl-support.md:68-69]

## 来源身份与验收限制

源材料中的 `klsupport.h/cpp`、`blocks.h` 和 `kl.cpp` 上游行号仅转录自代码注释，未核对上游文件字节。本包明确不进行数学或正确性验收，因此这些引用不能单独作为上游兼容性的验证结果。^[kl-support.md:10-12, kl-support.md:64-66]

精确读取身份记录于快照 `2026-10-06-kl-support.json`，绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录。本次知识维护未执行 Atlas、Cargo、测试或 benchmark；本页记录的是结构性阅读所得的覆盖情况与限制，而非本次运行验证结果。^[kl-support.md:73-79]

## Sources

- [逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）](../../sources/kl-support.md)
