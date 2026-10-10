---
title: RankFlags：简单生成元位集
summary: RankFlags 使用非 Copy 的 u32 位集表示至多 32 个生成元，contains 判定超集关系，位访问的安全性依赖调用方保证下标范围。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:35.187Z"
updatedAt: "2026-10-10T00:38:54.579Z"
tags:
  - KL
  - 位集
aliases:
  - rankflags简单生成元位集
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: RankFlags：简单生成元位集
summary: RankFlags 使用非 Copy 的 u32 位集表示至多 32 个简单生成元，提供超集、交集、差集和最低置位查询；set 与 is_set 自身不检查边界。
sources:
  - kl-support.md
kind: concept
tags:
  - 位集
  - Rust设计
  - 容量约束
aliases:
  - rankflags简单生成元位集
---

# RankFlags：简单生成元位集

`RankFlags` 是简单生成元集合的 `u32` 位集表示，容量约束为 **rank ≤ 32**。[[KlSupport：逐块 KL 支撑数据]] 使用它表示下降集与 good-ascent 集，并据此判定极值性和本原性。源材料将其语义描述为与上游 KL 算法的同名类型一致。^[kl-support.md:16-28, kl-support.md:41-43]

## 表示与集合操作

`RankFlags` 使用私有字段 `bits: u32` 存储集合，且不是 `Copy` 类型。`set` 与 `is_set` 自身不检查边界；在 `KlSupport` 的使用路径上，rank 上限由构造门控保证。^[kl-support.md:25-28]

`contains(other)` 判断当前集合是否为 `other` 的**超集**。`first_bit` 在固定范围 `0..32` 内按升序寻找最低置位；`intersect` 和 `difference` 分别返回交集与差集的新值。^[kl-support.md:27-28]

## 在 KL 支撑判定中的作用

`KlSupport` 为每个块元素构造下降集与 good-ascent 集：满足 `is_descent()` 的生成元进入下降集；否则，仅当类型不是 `ImaginaryTypeII` 时进入 good-ascent 集。因此，`ImaginaryTypeII` 既不是下降，也不是 good ascent，参见 [[下降集、good ascent 与本原性]]。^[kl-support.md:32-39]

记元素 $x$ 的下降集为 $\operatorname{desc}(x)$，good-ascent 集为 $\operatorname{good}(x)$。`ascent_descent(x, y)` 返回差集 $\operatorname{desc}(y)\setminus\operatorname{desc}(x)$ 的最低置位；`is_extremal(x, desc_y)` 检查 $\operatorname{desc}(x)\supseteq\operatorname{desc}_y$；`is_primitive(x, desc_y)` 检查 $\operatorname{good}(x)\cap\operatorname{desc}_y=\varnothing$。^[kl-support.md:41-43]

## 容量与测试边界

[[KlSupport 的拓扑构造门控]] 在构造时检查 rank ≤ 32，拒绝超过容量的输入。这是 `KlSupport` 使用路径上的保证，不应理解为 `RankFlags::set` 或 `RankFlags::is_set` 自身具有边界检查。^[kl-support.md:25-26, kl-support.md:32-35]

源材料记录的测试包括 `RankFlags` 基本位操作，其中验证了“空集被任何集合包含”，以及 rank 33 被 `KlSupport` 构造器拒绝的情况。`KlSupport` 的成功构造路径、全部判定方法和整个本原索引机制均无单元测试；材料注明它们经 KL 层的 HPC 门覆盖，参见 [[KL 支撑层的测试覆盖与证据边界]]。^[kl-support.md:57-62]

## 证据范围

本页依据对 `crates/atlas-real-group/src/kl_support.rs` 的结构性阅读，不声称 KL 支撑层已经通过数学验收。来源中的上游行号仅转录自代码注释，未核对上游字节；该次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试描述不代表该次维护的执行结果。^[kl-support.md:9-12, kl-support.md:75-79]

## Sources

- [kl-support.md](../../sources/kl-support.md) — 逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）。
