---
title: RankFlags：简单生成元位集
summary: RankFlags 以非 Copy 的 u32 位集表示至多 32 个生成元，contains 表示超集判定，set 与 is_set 的边界安全依赖调用方。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:35.187Z"
updatedAt: "2026-10-09T20:58:17.875Z"
tags:
  - 位集
  - 容量约束
aliases:
  - rankflags简单生成元位集
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: RankFlags：简单生成元位集
summary: RankFlags 使用非 Copy 的 u32 位集表示至多 32 个简单生成元，提供超集判定、交集、差集和最低置位查询；set/is_set 自身不检查边界。
sources:
  - kl-support.md
kind: concept
tags:
  - 位集
  - Rust设计
  - 秩限制
aliases:
  - rankflags简单生成元位集
---

# RankFlags：简单生成元位集

`RankFlags` 是表示简单生成元集合的 `u32` 位集，支持的秩上限为 **rank ≤ 32**。[[KlSupport：逐块 KL 支撑数据]] 使用它表示下降集与 good-ascent 集，并据此进行极值性、本原性等判定。源材料将其语义描述为与上游 KL 算法的同名类型一致。^[kl-support.md:16-28, kl-support.md:41-43]

## 表示与集合操作

`RankFlags` 使用私有字段 `bits: u32` 存储位集，且不是 `Copy` 类型。`set` 与 `is_set` 自身不进行边界检查；在 `KlSupport` 使用路径上，rank ≤ 32 的约束由构造门控保证。^[kl-support.md:25-28]

`contains(other)` 判断当前集合是否为 `other` 的**超集**。`first_bit` 在固定范围 `0..32` 内升序寻找最低置位；`intersect` 与 `difference` 分别返回交集与差集的新值。^[kl-support.md:27-28]

## 在 KL 支撑判定中的作用

`KlSupport` 对每个块元素逐生成元分类：满足 `is_descent()` 的生成元进入下降集；否则，只有类型不是 `ImaginaryTypeII` 时才进入 good-ascent 集。因此，`ImaginaryTypeII` 既不是下降，也不是 good ascent。相关定义见 [[下降集、good ascent 与本原性]]。^[kl-support.md:32-39]

记下降集为 \(\operatorname{desc}(x)\)，good-ascent 集为 \(\operatorname{good}(x)\)。`ascent_descent(x, y)` 取差集 \(\operatorname{desc}(y)\setminus\operatorname{desc}(x)\) 的最低置位；`is_extremal(x, desc_y)` 检查 \(\operatorname{desc}(x)\supseteq\operatorname{desc}_y\)；`is_primitive(x, desc_y)` 检查 \(\operatorname{good}(x)\cap\operatorname{desc}_y=\varnothing\)。^[kl-support.md:41-43]

## 容量与验证边界

[[KlSupport 的拓扑构造门控]] 在构造时拒绝 rank 超过 32 的输入。这一保护属于 `KlSupport` 的构造路径，不是 `RankFlags::set` 或 `RankFlags::is_set` 自身的边界检查。^[kl-support.md:25-26, kl-support.md:32-35]

源材料记录了基本位操作测试，其中包括“空集被任何集合包含”，以及 rank 33 被构造器拒绝的测试。`KlSupport` 的成功构造路径、全部判定方法和整个本原索引机制没有单元测试；材料注明它们经 KL 层的 HPC 门覆盖。更多范围说明见 [[KL 支撑层的测试覆盖与证据边界]]。^[kl-support.md:57-62]

本页依据的来源属于结构性源码阅读，不声称 KL 支撑层已获数学验收。材料中的上游引用仅转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[kl-support.md:9-12, kl-support.md:75-79]

## Sources

- [kl-support.md](../../sources/kl-support.md) — 逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）。
