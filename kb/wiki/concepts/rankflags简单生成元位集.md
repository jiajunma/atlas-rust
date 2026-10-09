---
title: RankFlags：简单生成元位集
summary: RankFlags 使用非 Copy 的 u32 位集表示至多 32 个简单生成元，提供超集判定、交集、差集和最低置位查询，set/is_set 自身不检查边界。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:35.187Z"
updatedAt: "2026-10-09T14:55:35.187Z"
tags:
  - 位集
  - Rust设计
  - 秩限制
aliases:
  - rankflags简单生成元位集
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# RankFlags：简单生成元位集

`RankFlags` 是表示简单生成元集合的 `u32` 位集，与上游 KL 算法的同名类型语义一致，支持的秩上限为 **rank ≤ 32**。在 [[KlSupport：逐块 KL 支撑数据]] 中，它用于表示下降集与 good-ascent 集，支撑极值性、本原性等判定。^[kl-support.md:16-28]

## 表示与集合操作

`RankFlags` 使用私有字段 `bits: u32` 存储位集，且不是 `Copy` 类型。`set` 与 `is_set` 不进行边界检查；在 `KlSupport` 的使用路径上，rank ≤ 32 的约束由构造门控保证。^[kl-support.md:25-28]

`contains(other)` 判断当前集合是否为 `other` 的**超集**。`first_bit` 在固定范围 `0..32` 内按升序寻找最低置位；`intersect` 与 `difference` 分别计算交集与差集，并返回新值。^[kl-support.md:27-28]

## 在 KL 支撑判定中的作用

`KlSupport` 对每个块元素逐生成元分类：满足 `is_descent()` 的生成元进入下降集；不满足该条件且类型不是 `ImaginaryTypeII` 的生成元进入 good-ascent 集。因此，`ImaginaryTypeII` 既不是下降，也不是 good ascent。相关语义见 [[下降集、good ascent 与本原性]]。^[kl-support.md:32-39]

记下降集为 \(\operatorname{desc}(x)\)，good-ascent 集为 \(\operatorname{good}(x)\)。`ascent_descent(x, y)` 寻找 \(\operatorname{desc}(y)\setminus\operatorname{desc}(x)\) 的最低置位；`is_extremal(x, desc_y)` 检查 \(\operatorname{desc}(x)\supseteq\operatorname{desc}_y\)；`is_primitive(x, desc_y)` 检查 \(\operatorname{good}(x)\cap\operatorname{desc}_y=\varnothing\)。这些判定直接利用位集的差集、包含与交集语义。^[kl-support.md:41-43]

## 边界与验证范围

[[KlSupport 的拓扑构造门控]] 会拒绝 rank 超过 32 的输入。这一保护属于 `KlSupport` 的构造路径，不能视为 `RankFlags::set` 或 `RankFlags::is_set` 自身提供了边界检查。^[kl-support.md:25-26, kl-support.md:32-35]

源材料记录了 `RankFlags` 基本位操作的测试，其中包含“空集被任何集合包含”的情形，也记录了 rank 33 被构造器拒绝的测试。`KlSupport` 的成功构造路径、全部判定方法及整个本原索引机制没有单元测试；源材料注明它们经 KL 层的 HPC 门覆盖，但该文档仅报告结构性阅读，不声称数学验收，也未核对所引用的上游源码字节。^[kl-support.md:9-12, kl-support.md:57-62]

## Sources

- [kl-support.md](kl-support.md) — 逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）
