---
title: TitsCoset 逐步约化的类映射依据
summary: Rust 在每个中间目标对合处约化而上游仅在末尾约化，其最终约化类不变的依据是操作在模二商上为类映射；本包仅记录该注释论据，未进行数学验收。
sources:
  - minimal-torus.md
kind: concept
createdAt: "2026-10-09T15:01:28.752Z"
updatedAt: "2026-10-09T15:01:28.752Z"
tags:
  - TitsCoset
  - 模二商
  - 证据边界
aliases:
  - titscoset-逐步约化的类映射依据
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# TitsCoset 逐步约化的类映射依据

`minimal_torus_part` 在把强代表下降到基本纤维时，复用 stage-(c) 的 `TitsCoset` 操作，并在每一步的目标对合处约化中间环面部分。源码注释以这些操作是 mod-2 商上的类映射为依据，说明逐步约化不会改变最终约化类；上游实现则只在末尾约化。这是源材料转录的注释声明，尚不是该材料独立验证的数学结论。^[minimal-torus.md:22-29]

## 类映射依据与约化位置

这里的关键依据是：相关操作被声明为 mod-2 商上的类映射，因此中间环面部分在目标对合处的约化不改变最终约化类。注释将这一性质关联到 stage-(e) KGB 枚举的验证，但源材料明确没有重新验证该声明，也没有核对所引用的上游文件字节。相关背景可参见 [[KGB 图与弱实形式]]。^[minimal-torus.md:10-13, minimal-torus.md:27-29]

## 在下降流程中的应用

下降前，算法要求 `factor − coch` 的每个坐标为整数，并将奇数坐标编码为 `torus_part` 的置位。随后通过 `grading_of_simples` 的单根配对偶性构造 `TitsCoset`；若 `table.lookup(twisted)` 找不到对应对合，则报告具名错误 `"synthetic involution coverage"`。^[minimal-torus.md:50-56]

下降循环在 Weyl 部分为恒等时停止；否则，按 `interface.outward()` 的迭代顺序选取首个左下降生成元。若该生成元在当前对合下为 Real，则执行逆 Cayley，且返回 `Ok(None)` 也视为错误；否则执行 based twisted 共轭 `cross_pregated`。这些步骤构成 [[强代表下降到基本纤维]] 的具体路径，每一步均在目标对合处约化中间环面部分。^[minimal-torus.md:27-29, minimal-torus.md:56-60]

循环末尾仍调用幂等的 `coset.reduce`。如果输入的 `twisted` 已处于基本纤维，下降循环不会承担约化工作，此时末尾调用起决定作用。下降完成后，算法以 `coweight = coch + lift(tp)` 提升余权，再进入 [[最小环面部分的 grading 轨道搜索]]。^[minimal-torus.md:60-74]

## 证据边界

源材料属于结构性源码阅读，不声称数学或正确性验收。“逐步约化不动最终类”应保留为源码注释中的依据，不能据此直接认定 Rust 与上游实现在所有输入上已经通过等价性验证。^[minimal-torus.md:9-13, minimal-torus.md:95-96]

现有四个测试锚点均为 rank 2、紧致内类。三个正例都满足 `coch == factor`，没有以可区分的断言刻画非平凡运输行为；非紧致 distinguished、非平凡运输及多数具名错误分支也未覆盖。因此，[[最小环面算法的测试覆盖边界]] 限制了这些测试对逐步约化依据的支持范围。^[minimal-torus.md:78-98]

## Sources

- [minimal-torus.md](minimal-torus.md) — 合成实形的选定余特征与初始环面部分（minimal_torus.rs）。
