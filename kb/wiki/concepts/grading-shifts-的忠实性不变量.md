---
title: Grading shifts 的忠实性不变量
summary: 构造期检查 grading shift 列线性无关，遇到相关列或零列即拒绝，从而保证可实现 grading 对应的 adjoint fiber 元素唯一。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:18.916Z"
updatedAt: "2026-10-09T14:50:18.916Z"
tags:
  - 线性无关
  - 构造期验证
  - 唯一性
aliases:
  - grading-shifts-的忠实性不变量
  - GS的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Grading shifts 的忠实性不变量

Grading shifts 的忠实性不变量要求 `CartanGradingData` 的 shift 列在 F₂ 上线性无关。构造时，`ensure_faithful_shifts` 无条件拒绝线性相关的列，并返回 `GradingShiftsNotFaithful`；这一不变量保证可实现的 grading 对应唯一的 adjoint fiber 元素。^[grading.md:47-55, grading.md:61-67]

## Shift 的含义

[[Grading 的位向量类型纪律|Grading]] 的第 `i` 位对应所属模型的第 `i` 个 simple-imaginary 根，置位表示 noncompact。它的索引不同于 ambient coweight 坐标，即使两者维数相同，也必须通过类型加以区分。^[grading.md:17-27]

`grading_shifts[i]` 记录第 `i` 个伴随基代表与各 simple-imaginary 根的坐标奇性向量之间的 F₂ 配对。根坐标通过 `*coordinate % 2 != 0` 取奇性，因此负奇数也保留为置位。`grading_shift(adjoint_basis_index)` 以 `adjoint_fiber().dimension()` 为索引上界。^[grading.md:47-51, grading.md:68-69]

在 [[Quasisplit 规范化与 grading 的仿射线性求值|quasisplit 规范化]] 下，零 adjoint fiber 元素的 `base_grading` 为全一，即每个 simple-imaginary 根均为 noncompact。其余元素的 grading 由 canonical ambient representative 的仿射线性求值得到：逐根计算 `!dot`，也就是将配对值与全一基点作 XOR。^[grading.md:35-38]

## 构造期检查

`ensure_faithful_shifts` 检查 shift 列的独立性：若插入某列时 `insert` 返回 `false`，就以 `GradingShiftsNotFaithful` 拒绝构造。源码注释将这一要求对应到上游 `cartanclass.cpp:172` 的断言；Rust 实现将其改为无条件拒绝。注释同时说明，没有已知公共构造路径会产生相关列，因此该检查属于防御性不变量。^[grading.md:52-55]

## 逆向求解与唯一性

`element_from_grading(target)` 使用增广消元恢复 adjoint fiber 元素。每个 shift 列携带 grading 位，以及位于 `imaginary_rank + adjoint_basis_index` 的 marker 位；消元在归约右端的同时累计所用 shift 的组合。右端为 `target XOR base`，因为基点全一，其中置位恰好标记目标 grading 的 compact 位置。^[grading.md:61-65]

若归约后的低 `imaginary_rank` 位仍有置位，目标 grading 无法实现，返回 `StructureError::ImpossibleGrading`；否则，按 marker 位选取伴随基代表，通过 `xor_assign` 汇总出 ambient 代表。构造期的忠实性检查保证求解结果唯一，但不保证任意目标 grading 都可实现。^[grading.md:65-67]

## 测试与证据边界

测试通过注入重复列和零列直接检查相关 shift 被拒绝的行为。其他锚点包括 A2 恒等对合的四元素双射，其 shift 构成置换矩阵；A2 扭转情形则拒绝全 compact grading，并在伴随纤维维数为零时使 `grading_shift(0)` 返回 `None`。^[grading.md:52-55, grading.md:73-78]

来源属于结构性源码阅读，未执行构建、测试或原版运行，不构成数学验收或性能证据。记录的覆盖缺口包括 build 中两处 `IndexOutOfRange`、多数溢出与分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。^[grading.md:9-13, grading.md:80-81, grading.md:91-91]

## Sources

- [grading.md](grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量
