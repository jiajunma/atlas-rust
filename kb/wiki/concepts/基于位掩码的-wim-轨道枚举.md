---
title: 基于位掩码的 W_im 轨道枚举
summary: walk_mask_orbits 按掩码升序播种并用 LIFO 栈遍历，通过基 grading 与配对奇偶判断非紧方向，再以 m_alpha 掩码平移。
sources:
  - weak-real-form.md
kind: concept
createdAt: "2026-10-09T15:15:25.118Z"
updatedAt: "2026-10-09T21:12:57.063Z"
tags:
  - 轨道枚举
  - 模二线性代数
aliases:
  - 基于位掩码的-wim-轨道枚举
  - 基W轨
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 基于位掩码的 W_im 轨道枚举
summary: walk_mask_orbits 按 canonical 坐标掩码升序播种，以 LIFO 栈遍历伴随 Cartan fiber 的 W_im 轨道，并通过 grading 奇偶判定和 m_alpha 平移生成轨道。
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

`walk_mask_orbits` 用位掩码枚举伴随 Cartan 纤维（adjoint Cartan fiber）中的 $W_{im}$ 轨道，为 `WeakRealFormPartition` 提供划分。每个轨道对应内类在该 Cartan 对合处的一个弱实形式；划分保存类表及每类的确定性代表元，实形标签另由 `RealFormLabels` 管理。参见 [[弱实形式的伴随 Cartan 纤维轨道划分]]。^[weak-real-form.md:20-23, weak-real-form.md:37-43]

## 播种与生成元作用

算法按 canonical 坐标的整数掩码升序播种，并以 LIFO 栈遍历轨道。对当前 `mask` 和生成元 `i`，`FiberAction` 使用 `base[i] XOR parity(mask ∧ alpha_columns[i])` 判定非紧性；判定为非紧时，以 `m_alpha_masks[i]` 作平移。生成元作用的对合性来自根与余根配对 $\langle\alpha,\alpha^\vee\rangle=2$ 的模二性质，实现中仅用 `debug_assert` 检查。^[weak-real-form.md:37-43]

类号按各轨道最小掩码的升序指派，首次出现的掩码即该类代表元。因此，`class_representative` 返回轨道在 canonical-coordinate 整数序下的最小元。class 0 是单位元所在轨道，对应 quasisplit normalization。^[weak-real-form.md:27-28, weak-real-form.md:39-43, weak-real-form.md:53-54]

## 确定性编号与上游对齐

来源记载，stage-(d) 排序审计证明该编号与上游内部 `RealFormNbr` 一致：两者采用相同的升序轨道播种、低主元 RREF 子商基和坐标提取方式。只有解释器外部 `FormNumberMap` 的顺序需要 adapter 置换；由 fundamental Cartan 划分产生的 id 同时用作 crate 的全局实形编号。相关约定见 [[WeakRealFormId 的确定性编号与上游对齐]] 与 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[weak-real-form.md:27-32]

## 容量与预算边界

掩码使用 `u64`，`MAX_MASK_BITS = 63` 限制伴随纤维的维数，保证 `1 << dimension` 的移位有效。构建入口 `WeakRealFormPartition::build(grading, max_elements)` 对已校验 grading 表背后的伴随纤维进行划分；`max_elements` 是调用方对枚举规模 $2^{\mathrm{dimension}}$ 设置的上界。^[weak-real-form.md:34-35, weak-real-form.md:49-51]

预算检查用 `u128` 计算 `1 << dimension`，再与拓宽后的 `max_elements` 比较。此处刻意不采用饱和计算，防止 `dimension == 64` 借 `usize::MAX` 上限漏过检查。^[weak-real-form.md:37-39]

类表采用 `Vec<u32>`，保留 `u32::MAX` 作为 `CLASS_SENTINEL`。若类序号无法表示为 `u32` 或等于哨兵，`seeded_class` 返回 `limit_error("classes", …)`。在没有伴随 `m_alpha`、每个掩码各自成类的单连通积情形下，这一守卫原则上可达。^[weak-real-form.md:41-45]

## 查询接口

划分通过 `class_count` 提供类数，通过 `classes` 提供升序的 `ExactSizeIterator`。`class_of` 与 `class_of_mask` 支持 canonical 坐标掩码查询，支撑语言级 `fiber_partition`；`class_representative` 返回该类最小代表元，`quasisplit_class` 返回 class 0，`adjoint_fiber` 提供对应的伴随纤维。^[weak-real-form.md:49-54]

## 测试锚点与证据边界

来源列出的轨道划分测试包括：A2 恒等情形的 2 类、B2 恒等情形的 3 类及精确代表元、单连通 A1 平凡作用的 2 个单元类，以及 A2 图扭转的 1 类。边界测试涉及外来元素与预算不足的拒绝、rank-33 的预算拒绝、rank-64 的 mask-bits 拒绝，以及 `seeded_class` 哨兵守卫的直接测试。rank-33 的拒绝体现枚举预算门，不能据此解释为固定的秩上限。^[weak-real-form.md:84-89]

未覆盖的分支包括 `class_of_mask` 越界、各 `ArithmeticOverflow` 转换分支，以及 `max_elements == 2^dimension` 的显式边界断言。相关覆盖限制见 [[弱实形式划分与归因的测试覆盖边界]]。^[weak-real-form.md:89-91]

来源属于结构性源码阅读，两次阅读记录的源码字节未变；该来源包未执行构建、测试或原版运行，也不重述或扩展既有 HPC 正确性证据链。因此，上述测试锚点的记载不构成新的数学验收、性能或并行结论。^[weak-real-form.md:9-16, weak-real-form.md:106-109]

## Sources

- [weak-real-form.md](../../sources/weak-real-form.md) — 弱实形式划分：adjoint fiber 的 W_im 轨道。
