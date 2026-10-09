---
title: 基于位掩码的 W_im 轨道枚举
summary: walk_mask_orbits 按掩码升序播种并以 LIFO 栈遍历，根据基 grading 与配对奇偶判定非紧方向，再用 m_alpha 掩码平移。
sources:
  - weak-real-form.md
kind: concept
createdAt: "2026-10-09T15:15:25.118Z"
updatedAt: "2026-10-09T19:37:43.722Z"
tags:
  - 轨道枚举
  - 位掩码
  - 分级
aliases:
  - 基于位掩码的-wim-轨道枚举
  - 基W轨
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 基于位掩码的 W_im 轨道枚举

`walk_mask_orbits` 以位掩码枚举伴随 Cartan fiber 中的 $W_{im}$ 轨道，是 `WeakRealFormPartition` 划分的实现核心。每个轨道对应内类在该 Cartan involution 处的一个弱实形式；划分保存类表和每类的确定性代表元，实形标签由 `RealFormLabels` 管理。参见 [[弱实形式的伴随 Cartan 纤维轨道划分]]。^[weak-real-form.md:20-23, weak-real-form.md:37-43]

## 播种与转移规则

算法按 canonical 坐标的整数掩码升序播种，并使用 LIFO 栈游走轨道。对当前 `mask` 和生成元 `i`，`FiberAction` 通过 `base[i] XOR parity(mask ∧ alpha_columns[i])` 判定非紧性；判定为非紧时，以 `m_alpha_masks[i]` 作平移。生成元作用的对合性依据根与余根配对 $\langle\alpha,\alpha^\vee\rangle=2$ 在模二下的性质，实现仅以 `debug_assert` 检查。^[weak-real-form.md:37-43]

类号按各轨道最小掩码的升序指派，首次出现的掩码即该类代表元。因此，代表元是轨道在 canonical-coordinate 整数序下的最小元；class 0 是单位元所在轨道，对应 quasisplit normalization。^[weak-real-form.md:27-28, weak-real-form.md:39-43]

## 确定性编号

来源记载，stage-(d) 排序审计证明该编号与上游内部 `RealFormNbr` 一致，依据是相同的升序轨道播种、低主元 RREF 子商基和坐标提取方式。仅解释器外部的 `FormNumberMap` 顺序需要 adapter 置换；由 fundamental Cartan 划分产生的 id 同时充当 crate 的全局实形编号。相关约定见 [[WeakRealFormId 的确定性编号与上游对齐]] 与 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[weak-real-form.md:27-32]

## 容量与预算边界

掩码采用 `u64`，并设置 `MAX_MASK_BITS = 63`，约束维数以保证 `1 << dimension` 的移位有效。构建入口 `WeakRealFormPartition::build(grading, max_elements)` 对已校验 grading 表背后的伴随 fiber 执行划分，`max_elements` 是调用方对枚举规模 $2^{\mathrm{dimension}}$ 给出的上界。^[weak-real-form.md:34-35, weak-real-form.md:49-51]

预算检查使用 `u128` 计算 `1 << dimension`，再与拓宽后的 `max_elements` 比较。此处刻意不采用饱和计算，以防 `dimension == 64` 借 `usize::MAX` 上限漏过检查。^[weak-real-form.md:37-39]

类表使用 `Vec<u32>`，并保留 `u32::MAX` 作为 `CLASS_SENTINEL`。若类序号无法放入 `u32` 或等于哨兵，`seeded_class` 返回 `limit_error("classes", …)`。在没有伴随 `m_alpha`、每个掩码各自成类的单连通积情形下，这一守卫原则上可达。^[weak-real-form.md:41-45]

## 查询接口

构建后的划分通过 `class_count` 提供类数，通过 `classes` 提供升序的 `ExactSizeIterator`。`class_of` 和 `class_of_mask` 支持 canonical 坐标查询，并支撑语言级 `fiber_partition`；`class_representative` 返回该类最小代表元，`quasisplit_class` 返回 class 0，`adjoint_fiber` 提供对应 fiber。^[weak-real-form.md:49-54]

## 测试锚点与证据边界

来源列出的测试锚点包括 A2 恒等情形的 2 类、B2 恒等情形的 3 类及其精确代表元、单连通 A1 平凡作用的 2 个单元类，以及 A2 图扭转的 1 类。边界测试覆盖外来元素与预算不足的拒绝、rank-33 的预算拒绝、rank-64 的 mask-bits 拒绝，以及 `seeded_class` 哨兵守卫的直接测试。^[weak-real-form.md:84-89]

尚未覆盖的分支包括 `class_of_mask` 越界、各 `ArithmeticOverflow` 转换分支，以及 `max_elements == 2^dimension` 的显式边界断言。来源属于结构性阅读，未执行构建、测试或原版运行；这些测试锚点的记载不构成新的数学验收、性能或并行结论。^[weak-real-form.md:89-91, weak-real-form.md:106-106]

## Sources

- [weak-real-form.md](weak-real-form.md) — 弱实形式划分：adjoint fiber 的 W_im 轨道。
