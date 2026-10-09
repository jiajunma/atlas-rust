---
title: 紧致 grading 的测试锚点与覆盖边界
summary: 来源记录九项测试锚点，涵盖双向往返、不可实现 grading、来源拒绝、负奇坐标和动态维数，但入口秩错误及多类分配与溢出分支未覆盖，且本包未执行测试或数学验收。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T20:54:33.100Z"
updatedAt: "2026-10-09T20:54:33.100Z"
tags:
  - 测试覆盖
  - 证据边界
  - 分级
aliases:
  - 紧致-grading-的测试锚点与覆盖边界
  - 紧G的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 紧致 grading 的测试锚点与覆盖边界

紧致 grading 的测试锚点覆盖 `grading.rs` 的位向量语义、规范化、grading 与纤维元素互转，以及构造和输入校验。来源记录了 9 个测试，但依据是结构性源码阅读，并未执行构建、测试或原版运行；这些锚点不能直接视为数学验收结果。^[grading.md:9-13, grading.md:71-81, grading.md:91-94]

## 测试所保护的约定

`Grading` 是 `ModTwoVector` 的 newtype，bit `i` 对应所属模型的第 `i` 个 simple-imaginary 根，置位表示 **noncompact（非紧）**。它与 ambient coweight 或 fiber 坐标具有不同含义，即使维数相同也必须通过类型区分；详见 [[Grading 的位向量类型纪律]]。^[grading.md:17-27]

quasisplit 规范化令零 adjoint fiber 元素对应全一 `base_grading`，其余 grading 通过 canonical ambient representative 与逐根配对值进行仿射线性求值。反向求解依赖 grading shifts 的线性无关性，以保证可实现 grading 对应唯一的 adjoint fiber 元素。^[grading.md:35-38, grading.md:59-67]

## 九个测试锚点

来源列出的测试覆盖如下，涵盖正常构造、退化情形、非法输入及防御性不变量。^[grading.md:73-79]

| 测试情形 | 主要检查 |
| --- | --- |
| 单连通 A1 | quasisplit 规范化；`m_alpha` 非平凡而伴随像平凡；grading 与元素双向往返 |
| A2 恒等对合 | 四元素双射；根序 index 0 为 \(\alpha_2\)；shift 为置换矩阵 |
| A2 扭转对合 | 全紧 grading 返回 `ImpossibleGrading`；`adjoint.dimension() == 0`；`grading_shift(0)` 为 `None` |
| A1×A1 交换对合 | `imaginary_rank == 0` |
| 含中心余权坐标 | 区分 `m_alpha` 与其伴随像 |
| B2 | 负奇余根坐标的模二归约 |
| 外来输入 | 分别拒绝 datum、involution 和 fiber 不匹配 |
| 注入相关 shift 列 | 拒绝重复列与零列 |
| 33 个 A1 因子 | 保持动态表示；`noncompact_indices().count() == 33` |

### 规范化、根序与不可实现 grading

A1 与 A2 恒等对合的测试分别锚定规范化和四元素双射；A2 还明确固定了 \(\alpha_2\) 位于根序 index 0 的约定。A2 扭转测试则覆盖不可实现的全紧 grading，以及零维 adjoint fiber 的 shift 访问边界；A1×A1 交换测试覆盖没有 simple-imaginary 根的情形。^[grading.md:73-76]

这些用例对应 [[Quasisplit 规范化与 grading 的仿射线性求值]] 和 [[通过增广消元反求 grading 对应元素]]：反向求解以 `target XOR base` 为右端，若消元后的余数在低 `imaginary_rank` 位仍有置位，就返回 `StructureError::ImpossibleGrading`。^[grading.md:35-38, grading.md:61-67]

### 模二归约与坐标区分

B2 测试锚定余根 \((2,-1)\) 归约为 \((0,1)\)，确保 `coordinate % 2 != 0` 包含负奇数。含中心余权坐标的测试则区分 ambient fiber 中的 `m_alpha` 与其伴随投影，参见 [[m_alpha 的模二归约与伴随投影]]。^[grading.md:47-51, grading.md:76-78]

### 输入来源与忠实性检查

外来输入测试覆盖三种拒绝路径：构造时 datum 不一致返回 `DatumMismatch`，ambient fiber 的 involution 不一致返回 `CartanFiberInvolutionMismatch`，求 grading 时外来纤维元素返回 `CartanFiberMismatch`。这些检查与 [[CartanGradingData 与纤维来源一致性]] 的构造约束对应。^[grading.md:40-45, grading.md:59-60, grading.md:77-78]

注入测试直接检查重复 shift 列和零列被拒绝。`ensure_faithful_shifts` 在列线性相关时返回 `GradingShiftsNotFaithful`；这是将上游断言改为无条件拒绝的防御性检查，源码注释称没有已知公共构造路径能产生相关列。详见 [[Grading shifts 的忠实性不变量]]。^[grading.md:52-55, grading.md:78-79]

### 动态表示

33 个 A1 因子的测试检查 `noncompact_indices().count() == 33`，用于锚定表示保持动态，而不受固定 32/64 位打包限制。^[grading.md:79-79]

## 未覆盖路径与证据限制

来源明确列出的缺口包括：`build` 中两处 `IndexOutOfRange`、多数溢出与分配分支，以及 `element_from_grading` 入口处的 `RankMismatch`。因此，现有锚点并不构成错误路径的完整覆盖。^[grading.md:80-81]

该材料记录了两次字节未变、SHA-256 相同的结构性阅读。grading 的正确性仍属于其独立的 HPC 证据链，包括 Cartan/seed gate 等；本材料不重述或扩展这些证据，也不提供数学验收、性能或并行结论。^[grading.md:9-13, grading.md:85-91]

## Sources

- [grading.md](../../sources/grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量。
