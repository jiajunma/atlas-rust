---
title: Grading shifts 的忠实性不变量
summary: 构造期无条件拒绝线性相关或为零的 grading shift 列，从而保证每个可实现 grading 对应唯一的伴随纤维元素。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:18.916Z"
updatedAt: "2026-10-09T19:29:22.074Z"
tags:
  - grading
  - 线性无关
  - 构造不变量
aliases:
  - grading-shifts-的忠实性不变量
  - GS的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Grading shifts 的忠实性不变量

Grading shifts 的忠实性不变量要求 `CartanGradingData` 的 shift 列在 F₂ 上线性无关。构造期的 `ensure_faithful_shifts` 无条件拒绝相关列，返回 `GradingShiftsNotFaithful`。这一检查保证每个可实现的 grading 对应唯一的 adjoint fiber 元素。^[grading.md:47-55, grading.md:61-67]

## Shift 的含义

[[Grading 的位向量类型纪律|Grading]] 的第 `i` 位对应所属模型的第 `i` 个 simple-imaginary 根，置位表示 noncompact。其索引属于 simple-imaginary 根列表，不能与 ambient coweight 坐标混用，即使二者维数相同。^[grading.md:17-27]

`grading_shifts[i]` 记录第 `i` 个伴随基代表与各 simple-imaginary 根的坐标奇性向量之间的 F₂ 配对。坐标奇性通过 `*coordinate % 2 != 0` 提取，包含负奇数。访问器 `grading_shift(adjoint_basis_index)` 以 `adjoint_fiber().dimension()` 为索引上界。^[grading.md:47-51, grading.md:68-69]

在 [[Quasisplit 规范化与 grading 的仿射线性求值|quasisplit 规范化]] 下，零 adjoint fiber 元素的 `base_grading` 为全一，即所有 simple-imaginary 根均为 noncompact。其他元素的 grading 通过 canonical ambient representative 作仿射线性求值：逐根计算 `!dot`，等价于将配对值与全一基点作 XOR。^[grading.md:35-38]

## 构造期检查

`ensure_faithful_shifts` 逐列检查独立性；若插入某列时 `insert` 返回 `false`，便以 `GradingShiftsNotFaithful` 拒绝构造。重复列和零列均由注入测试直接检查。源码注释将此要求对应到上游 `cartanclass.cpp:172` 的断言，而 Rust 实现将其改为无条件拒绝。^[grading.md:52-55]

源码注释说明，没有已知公共构造路径能够产生相关 shift 列，因此该检查属于防御性不变量；不能据此推断正常公共构造已知会触发这一错误。^[grading.md:52-55]

## 逆向求解与唯一性

[[通过增广消元反求 grading 对应元素|element_from_grading(target)]] 使用增广消元恢复 adjoint fiber 元素。每列同时携带 grading 位和位于 `imaginary_rank + adjoint_basis_index` 的 marker 位；归约右端时，marker 位累计所用 shift 的组合。右端为 `target XOR base`，由于基点全一，其置位恰好标记目标 grading 的 compact 位置。^[grading.md:61-65]

若归约结果的低 `imaginary_rank` 位仍有置位，则目标 grading 不可实现，返回 `StructureError::ImpossibleGrading`。否则，算法按 marker 位选取伴随基代表，通过 `xor_assign` 汇总出 ambient 代表。构造期的忠实性不变量保证可实现目标的解唯一，并不保证任意目标 grading 都可实现。^[grading.md:65-67]

## 测试与证据边界

相关测试锚点包括注入重复列和零列时拒绝构造，以及 A2 恒等对合下的四元素双射，其 shift 构成置换矩阵。A2 扭转情形拒绝全 compact grading；该例中伴随纤维维数为零，`grading_shift(0)` 返回 `None`。^[grading.md:52-55, grading.md:73-78]

来源属于结构性源码阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。记录的未覆盖分支包括 `build` 的两处 `IndexOutOfRange`、多数溢出与分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。^[grading.md:9-13, grading.md:80-81, grading.md:91-91]

## Sources

- [grading.md](grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量
