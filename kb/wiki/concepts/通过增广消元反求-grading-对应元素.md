---
title: 通过增广消元反求 grading 对应元素
summary: 以 target XOR base 为右端，使用附带基索引标记位的增广消元恢复伴随基组合，不可实现的目标返回 ImpossibleGrading。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:19.229Z"
updatedAt: "2026-10-09T20:54:20.132Z"
tags:
  - 增广消元
  - 分级
  - 错误处理
aliases:
  - 通过增广消元反求-grading-对应元素
  - 通G对
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 通过增广消元反求 grading 对应元素
summary: 以 target XOR base 为右端，通过携带伴随基索引标记位的 F₂ 增广消元反求 fiber 元素；不可实现的目标返回 ImpossibleGrading，可实现目标的唯一性由 shift 列的忠实性保证。
sources:
  - grading.md
kind: concept
tags:
  - grading
  - 增广消元
  - 有限域
---

# 通过增广消元反求 grading 对应元素

`CartanGradingData::element_from_grading(target)` 通过 F₂ 上的增广消元，求出具有指定 grading 的唯一伴随 Cartan fiber 元素。目标不可实现时返回 `StructureError::ImpossibleGrading`；可实现目标的唯一性由构造阶段检查的 [[Grading shifts 的忠实性不变量]] 保证。^[grading.md:61-67]

## 坐标与求解目标

[[Grading 的位向量类型纪律|Grading]] 的第 `i` 位对应所属模型的第 `i` 个 simple-imaginary 根，置位表示 noncompact。这个索引与 ambient coweight 的格坐标不同；即使维数相同，也必须通过类型区分。^[grading.md:17-27]

在 [[Quasisplit 规范化与 grading 的仿射线性求值|quasisplit 规范化]] 下，伴随 fiber 的零元素对应全一的 `base_grading`。其他元素的 grading 由其 canonical ambient representative 与各单根逐根进行模二配对，再对结果取反。因此，逆向求解的右端是 `target XOR base_grading`，其中置位恰好标记目标 grading 的 compact 位置。^[grading.md:35-38, grading.md:59-65]

## 增广消元过程

`grading_shifts[i]` 记录第 `i` 个伴随基代表与各单根奇性向量的 F₂ 配对。求解时，每个 shift 列携带自身的 grading 位，并附加一个位于 `imaginary_rank + adjoint_basis_index` 的 marker 位，用于追踪该列对应的伴随基索引。^[grading.md:47-51, grading.md:61-64]

算法用这些增广列归约右端 `target XOR base_grading`，同时通过 marker 位累计所用 shift 的组合。归约完成后，若余数的低 `imaginary_rank` 位仍有置位，则目标无法实现，返回 `StructureError::ImpossibleGrading`。^[grading.md:62-66]

若低位余数全部为零，则按累计的 marker 位选取伴随基代表，用 `xor_assign` 汇总为 ambient 代表，得到所请求 grading 对应的伴随 fiber 元素。^[grading.md:61-67]

## 存在性与唯一性

构造阶段的 `ensure_faithful_shifts` 检查 shift 列是否线性无关；发现相关列时，无条件返回 `GradingShiftsNotFaithful`。该不变量保证可实现目标的解唯一，但不保证所有 grading 都能实现。来源将此检查描述为防御性措施，并指出没有已知公共构造路径能产生相关列；注入测试直接覆盖重复列与零列。^[grading.md:52-55, grading.md:61-67]

正向接口 `grading(element)` 先取得 canonical representative，再逐根配对取反；外来纤维元素会触发 `CartanFiberMismatch`。正反向转换的上下文约束可结合 [[CartanGradingData 与纤维来源一致性]] 理解。^[grading.md:40-45, grading.md:59-67]

## 测试与证据边界

相关源码测试锚点包括 SC A1 的双向往返、A2 恒等对合的四元素双射，以及 A2 扭转情形对全紧 grading 的拒绝。A2 扭转用例还检查 `adjoint.dimension() == 0`，且 `grading_shift(0)` 返回 `None`。这些用例分别覆盖可实现目标的转换与不可实现目标的错误行为。^[grading.md:73-76]

`element_from_grading` 入口处的 `RankMismatch` 尚未被测试覆盖，多数溢出与分配分支也未覆盖。来源属于结构性源码阅读，未执行构建、测试或原版运行；上述测试锚点不构成本次运行验证、数学验收或性能结论。详见 [[紧致 grading 的测试锚点与覆盖边界]]。^[grading.md:9-13, grading.md:80-81, grading.md:91-94]

## Sources

- [grading.md](../../sources/grading.md)：紧致 grading：simple-imaginary 根的紧致性位向量。
