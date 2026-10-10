---
title: 通过增广消元反求 grading 对应元素
summary: 以 target XOR base 为右端，利用携带基索引标记位的增广消元恢复伴随基组合，不可实现时返回 ImpossibleGrading。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:50:19.229Z"
updatedAt: "2026-10-10T00:34:37.033Z"
tags:
  - grading
  - 模二线性代数
aliases:
  - 通过增广消元反求-grading-对应元素
  - 通G对
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 通过增广消元反求 grading 对应元素
summary: 以 target XOR base 为右端，通过携带伴随基索引标记位的增广消元恢复 fiber 元素；不可实现的目标返回 ImpossibleGrading，唯一性由 shift 列的忠实性保证。
sources:
  - grading.md
kind: concept
tags:
  - 模二线性代数
  - 增广消元
  - 紧致分级
---

# 通过增广消元反求 grading 对应元素

`CartanGradingData::element_from_grading(target)` 通过 F₂ 上的增广消元，求出具有指定 grading 的唯一伴随 Cartan fiber 元素。目标不可实现时返回 `StructureError::ImpossibleGrading`；可实现目标的唯一性由构造期检查的 [[Grading shifts 的忠实性不变量]] 保证。^[grading.md:61-67]

## 坐标与求解目标

[[Grading 的位向量类型纪律|Grading]] 的第 `i` 位对应所属模型的第 `i` 个 simple-imaginary 根，置位表示 noncompact。它与 ambient coweight 坐标有不同含义；即使维数相同，也必须通过类型区分。^[grading.md:17-27]

在 [[Quasisplit 规范化与 grading 的仿射线性求值|quasisplit 规范化]] 下，伴随 fiber 的零元素对应全一的 `base_grading`。其他元素的 grading 由 canonical ambient representative 与各单根的模二配对逐位取反得到，因此逆向求解的右端是 `target XOR base_grading`，其中置位恰好标记目标的 compact 位置。^[grading.md:35-38, grading.md:59-65]

## 增广消元过程

`grading_shifts[i]` 记录第 `i` 个伴随基代表与各单根奇性向量的 F₂ 配对。求解时，每列携带自身的 grading 位，并在第 `imaginary_rank + adjoint_basis_index` 位附加一个 marker 位，标记该列对应的伴随基。^[grading.md:47-51, grading.md:61-64]

算法以这些增广列归约右端 `target XOR base_grading`，同时累计求解所需的基组合。若归约后余数的低 `imaginary_rank` 位仍有置位，则目标无法由 shift 列组合实现，返回 `StructureError::ImpossibleGrading`。^[grading.md:62-66]

若低位余数全部为零，则按累计的 marker 位选取伴随基代表，通过 `xor_assign` 汇总为 ambient 代表，从而恢复具有目标 grading 的伴随 fiber 元素。^[grading.md:61-67]

## 唯一性与来源约束

构造阶段的 `ensure_faithful_shifts` 检查 shift 列的线性无关性；若插入某列时发现相关性，便无条件返回 `GradingShiftsNotFaithful`。这一检查保证可实现目标的解唯一，但不保证任意 grading 都可实现。来源将其描述为防御性检查：没有已知公共构造路径能产生相关列，注入测试直接覆盖重复列与零列。^[grading.md:52-55, grading.md:61-67]

正向接口 `grading(element)` 先取得 canonical representative，再逐根配对取反；外来纤维元素触发 `CartanFiberMismatch`。构造 grading 表还要求根系统与对合数据的 datum 一致，以及 adjoint 的 ambient fiber 对合与对合数据一致。构造器使用 adjoint descent 所依据的确切 ambient fiber 来源，详见 [[CartanGradingData 与纤维来源一致性]]。^[grading.md:40-45, grading.md:59-60]

## 测试与证据边界

相关测试锚点包括 SC A1 的双向往返、A2 恒等对合的四元素双射，以及 A2 扭转情形对全紧 grading 的拒绝。A2 扭转用例还检查 `adjoint.dimension() == 0`，且 `grading_shift(0)` 返回 `None`。^[grading.md:73-76]

`element_from_grading` 入口的 `RankMismatch` 尚未被测试覆盖，多数溢出与分配分支也未覆盖。来源属于结构性源码阅读，未执行构建、测试或原版运行；上述测试锚点不构成本次运行验证、数学验收或性能结论。^[grading.md:9-13, grading.md:80-81, grading.md:91-94]

## Sources

- [grading.md](../../sources/grading.md)：紧致 grading：simple-imaginary 根的紧致性位向量。
