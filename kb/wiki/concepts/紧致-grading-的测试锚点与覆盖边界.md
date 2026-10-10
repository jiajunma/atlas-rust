---
title: 紧致 grading 的测试锚点与覆盖边界
summary: 来源记录九项测试锚点及索引、秩、溢出和分配分支的覆盖缺口，本包未执行测试或形成数学验收。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T20:54:33.100Z"
updatedAt: "2026-10-10T00:34:57.051Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 紧致 grading 的测试锚点与覆盖边界
summary: grading.rs 的九个测试锚点覆盖规范化、互转、来源校验、模二归约和动态表示；部分错误路径未覆盖，结构性阅读不构成测试通过或数学验收证据。
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

`grading.rs` 的来源材料记录了九个测试锚点，涉及 grading 规范化、纤维元素互转、非法输入拒绝及动态位向量表示。这些记录来自结构性源码阅读；本来源未执行构建、测试或原版运行，不能据此声称测试已通过或数学正确性已获验收。^[grading.md:9-13, grading.md:71-81, grading.md:91-94]

## 测试所保护的语义

`Grading` 是 `ModTwoVector` 的 newtype，bit `i` 对应所属模型 simple-imaginary 根列表的第 `i` 项，置位表示 **noncompact（非紧）**。这些位置与 ambient coweight 或 fiber 坐标不同，即使维数相同也必须由类型区分，参见 [[Grading 的位向量类型纪律]]。^[grading.md:17-27]

[[Quasisplit 规范化与 grading 的仿射线性求值]] 规定零 adjoint fiber 元素对应全一 `base_grading`；其他元素通过 canonical ambient representative 与逐根配对值进行 XOR 求值。反向求解使用增广消元，构造期检查的 shift 忠实性保证每个可实现 grading 对应唯一的 adjoint fiber 元素。^[grading.md:35-38, grading.md:61-67]

## 九个测试锚点

来源列出以下九个测试，覆盖正常互转、退化情形、错误输入和动态表示；其中相关 shift 列的注入测试直接检查重复列与零列。^[grading.md:52-55, grading.md:73-79]

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

### 不可实现 grading 与退化情形

A2 扭转测试锚定不可实现的全紧 grading，以及零维 adjoint fiber 的 shift 访问边界；A1×A1 交换测试则覆盖没有 simple-imaginary 根的情形。反向求解时，右端为 `target XOR base`，标记目标 grading 的 compact 位置；若归约余数在低 `imaginary_rank` 位仍有置位，返回 `StructureError::ImpossibleGrading`。^[grading.md:61-69, grading.md:74-76]

### 模二归约与坐标区分

B2 测试以余根 \((2,-1)\) 归约为 \((0,1)\) 为锚点，确认 `coordinate % 2 != 0` 包含负奇数。含中心余权坐标的测试区分 ambient fiber 中的 `m_alpha` 与其伴随像，参见 [[m_alpha 的模二归约与伴随投影]]。^[grading.md:47-51, grading.md:76-78]

### 来源校验与 shift 忠实性

外来输入测试覆盖三种错误：构造时 datum 不一致返回 `DatumMismatch`，adjoint 的 ambient fiber 对合不一致返回 `CartanFiberInvolutionMismatch`，求 grading 时遇到外来纤维元素返回 `CartanFiberMismatch`。构造器使用 `AdjointCartanFiber::ambient_fiber` 的确切来源构建 `m_alpha`，相关约束见 [[CartanGradingData 与纤维来源一致性]]。^[grading.md:40-45, grading.md:59-60, grading.md:77-78]

`ensure_faithful_shifts` 在 shift 列线性相关时返回 `GradingShiftsNotFaithful`，将上游断言改为无条件拒绝。注入测试直接检查重复列与零列；源码注释称没有已知公共构造路径能产生相关列，因此这是防御性检查，参见 [[Grading shifts 的忠实性不变量]]。^[grading.md:52-55]

### 动态表示的范围

33 个 A1 因子的测试检查 `noncompact_indices().count() == 33`，来源将其作为动态表示的测试锚点。^[grading.md:79-79]

这一计数直接展示了 33 个非紧位置的处理，不能仅凭该用例声称已经测试超过 64 个位置的情形。

## 未覆盖路径

来源明确列出的覆盖缺口为 `build` 中两处 `IndexOutOfRange`、多数溢出与分配分支，以及 `element_from_grading` 入口处的 `RankMismatch`。现有测试锚点因此不能视为完整的错误路径覆盖。^[grading.md:80-81]

## 证据边界

材料记录了 2026-10-03 与 2026-10-06 两次结构性阅读，所读源码字节及 SHA-256 相同。grading 的正确性属于其独立的 HPC 证据链，包括 Cartan/seed gate 等；本来源不重述或扩展这些证据，也不提供数学验收、性能或并行结论。^[grading.md:9-13, grading.md:85-91]

## Sources

- [grading.md](../../sources/grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量。
