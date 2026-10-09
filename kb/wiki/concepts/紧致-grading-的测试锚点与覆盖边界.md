---
title: 紧致 grading 的测试锚点与覆盖边界
summary: 来源记录九项测试锚点及秩检查、索引、分配与溢出分支的覆盖缺口；本包未执行测试，也不构成数学验收。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T20:54:33.100Z"
updatedAt: "2026-10-09T22:31:45.679Z"
tags:
  - 测试覆盖
  - 证据边界
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

---
title: 紧致 grading 的测试锚点与覆盖边界
summary: grading.rs 的九个测试锚点涉及规范化、双向互转、不可实现 grading、来源校验、模二归约及动态表示；部分错误路径未覆盖，来源仅为结构性阅读，不构成测试执行或数学验收证据。
sources:
  - grading.md
kind: concept
tags:
  - 测试覆盖
  - 证据边界
  - 分级
aliases:
  - 紧致-grading-的测试锚点与覆盖边界
provenanceState: extracted
---

# 紧致 grading 的测试锚点与覆盖边界

`grading.rs` 的九个测试锚点涉及位向量语义、quasisplit 规范化、grading 与纤维元素互转，以及构造和输入校验。来源依据结构性源码阅读整理这些测试，未执行构建、测试或原版运行，因此不能将测试锚点视为已通过的测试报告或数学验收结果。^[grading.md:9-13, grading.md:71-81, grading.md:91-94]

## 测试所保护的约定

`Grading` 是 `ModTwoVector` 的 newtype，bit `i` 对应所属模型第 `i` 个 simple-imaginary 根，置位表示 **noncompact（非紧）**。grading 位置与 ambient coweight 或 fiber 坐标含义不同，即使维数相同也必须通过类型区分；参见 [[Grading 的位向量类型纪律]]。^[grading.md:17-27]

[[Quasisplit 规范化与 grading 的仿射线性求值]] 令零 adjoint fiber 元素对应全一 `base_grading`，其余 grading 通过 canonical ambient representative 与逐根配对值进行 XOR 求值。反向求解依赖 grading shifts 的线性无关性，保证每个可实现 grading 对应唯一的 adjoint fiber 元素。^[grading.md:35-38, grading.md:61-67]

## 九个测试锚点

来源列出以下九个测试，覆盖正常互转、退化情形、非法输入和动态表示；重复列与零列的注入细节另见忠实性检查说明。^[grading.md:52-55, grading.md:73-79]

| 测试情形 | 主要检查 |
| --- | --- |
| 单连通 A1 | quasisplit 规范化；`m_alpha` 非平凡、伴随像平凡；grading 与元素双向往返 |
| A2 恒等对合 | 四元素双射；根序 index 0 为 \(\alpha_2\)；shift 为置换矩阵 |
| A2 扭转对合 | 全紧 grading 返回 `ImpossibleGrading`；`adjoint.dimension() == 0`；`grading_shift(0)` 为 `None` |
| A1×A1 交换对合 | `imaginary_rank == 0` |
| 含中心余权坐标 | 区分 `m_alpha` 与其伴随像 |
| B2 | 负奇余根坐标的模二归约 |
| 外来输入 | 拒绝 datum、involution 和 fiber 不匹配 |
| 注入相关 shift 列 | 拒绝重复列与零列 |
| 33 个 A1 因子 | 动态表示；`noncompact_indices().count() == 33` |

### 规范化、根序与不可实现 grading

A1 和 A2 恒等对合测试分别锚定规范化与四元素双射；A2 还固定了根序 index 0 为 \(\alpha_2\) 的约定。A2 扭转测试覆盖不可实现的全紧 grading，以及零维 adjoint fiber 的 shift 访问边界；A1×A1 交换测试覆盖没有 simple-imaginary 根的情形。^[grading.md:73-76]

`element_from_grading` 以 `target XOR base` 为增广消元右端；由于 base 全一，右端标记 target 的 compact 位置。若消元余数在低 `imaginary_rank` 位仍有置位，则返回 `StructureError::ImpossibleGrading`；否则按累计的 marker 位组合伴随基代表，得到对应元素。^[grading.md:61-67]

### 模二归约与坐标区分

B2 测试锚定余根 \((2,-1)\) 归约为 \((0,1)\)，确认 `coordinate % 2 != 0` 包含负奇数。含中心余权坐标的测试则区分 ambient fiber 中的 `m_alpha` 与其伴随投影，参见 [[m_alpha 的模二归约与伴随投影]]。^[grading.md:47-51, grading.md:76-78]

### 来源校验与忠实性

外来输入测试覆盖三种拒绝路径：构造时 datum 不一致返回 `DatumMismatch`，adjoint 的 ambient fiber 对合不一致返回 `CartanFiberInvolutionMismatch`，求 grading 时外来纤维元素返回 `CartanFiberMismatch`。构造器使用 `AdjointCartanFiber::ambient_fiber` 的确切来源构建 `m_alpha`，相关约束见 [[CartanGradingData 与纤维来源一致性]]。^[grading.md:40-45, grading.md:59-60, grading.md:77-78]

注入测试直接检查重复 shift 列和零列被拒绝。`ensure_faithful_shifts` 在列线性相关时返回 `GradingShiftsNotFaithful`，将上游断言改为无条件拒绝；源码注释称没有已知公共构造路径能产生相关列，因此这是防御性检查。参见 [[Grading shifts 的忠实性不变量]]。^[grading.md:52-55, grading.md:78-79]

### 动态表示的范围

33 个 A1 因子的测试检查 `noncompact_indices().count() == 33`，锚定来源所述的动态表示。该用例直接展示了 33 个非紧位置的处理，不能仅凭此计数声称已经测试超过 64 个位置的情形。^[grading.md:79-79]

## 未覆盖路径与证据边界

来源明确列出的缺口包括 `build` 中两处 `IndexOutOfRange`、多数溢出与分配分支，以及 `element_from_grading` 入口处的 `RankMismatch`。这些缺口限定了现有测试锚点的错误路径覆盖范围。^[grading.md:80-81]

材料记录了 2026-10-03 与 2026-10-06 两次结构性阅读，源码字节和 SHA-256 未变。grading 正确性属于其独立的 HPC 证据链，包括 Cartan/seed gate 等；本来源不重述或扩展这些证据，也不提供数学验收、性能或并行结论。^[grading.md:9-13, grading.md:85-91]

## Sources

- [grading.md](../../sources/grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量。
