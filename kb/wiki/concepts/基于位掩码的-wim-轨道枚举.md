---
title: 基于位掩码的 W_im 轨道枚举
summary: walk_mask_orbits 按掩码升序播种并用 LIFO 栈遍历，由基 grading 与配对奇偶判断非紧方向，再执行 m_alpha 掩码平移。
sources:
  - weak-real-form.md
kind: concept
createdAt: "2026-10-09T15:15:25.118Z"
updatedAt: "2026-10-10T00:54:46.329Z"
tags:
  - 轨道枚举
  - 位掩码
  - grading
aliases:
  - 基于位掩码的-wim-轨道枚举
  - 基W轨
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 基于位掩码的 W_im 轨道枚举
summary: walk_mask_orbits 按 canonical 坐标掩码升序播种，以 LIFO 栈遍历伴随 Cartan 纤维的 W_im 轨道，通过 grading 奇偶判定和 m_alpha 平移实现生成元作用。
sources:
  - weak-real-form.md
kind: concept
tags:
  - 轨道枚举
  - 位掩码
  - 模二线性代数
aliases:
  - 基于位掩码的-wim-轨道枚举
provenanceState: extracted
---

# 基于位掩码的 W_im 轨道枚举

`walk_mask_orbits` 以位掩码枚举伴随 Cartan 纤维的 $W_{im}$ 轨道，为 `WeakRealFormPartition` 构造类表和确定性代表元。每个轨道对应内类在该 Cartan 对合处的一个弱实形式；实形标签由 `RealFormLabels` 管理，square-class 与强实层另行处理。背景见 [[弱实形式的伴随 Cartan 纤维轨道划分]]。^[weak-real-form.md:20-23, weak-real-form.md:37-43]

## 播种与生成元作用

算法按 canonical 坐标的整数掩码升序播种，以后进先出（LIFO）栈遍历轨道。对当前 `mask` 和生成元 `i`，转移规则与 `FiberAction` 一致：使用 `base[i] XOR parity(mask ∧ alpha_columns[i])` 判定非紧性，判为非紧时以 `m_alpha_masks[i]` 作平移。生成元作用的对合性依赖配对 $\langle\alpha,\alpha^\vee\rangle=2$ 的模二性质，实现中仅以 `debug_assert` 检查。^[weak-real-form.md:37-43]

类号按各轨道最小掩码的升序指派，首次出现的掩码即该类代表元。因此，代表元是轨道在 canonical-coordinate 整数序下的最小元。class 0 是单位元所在轨道，对应 quasisplit normalization。^[weak-real-form.md:27-28, weak-real-form.md:41-43, weak-real-form.md:53-54]

## 确定性编号与上游对齐

来源记载，stage-(d) 排序审计证明该编号与上游内部 `RealFormNbr` 一致：两者使用相同的升序轨道播种、低主元 RREF 子商基和坐标提取方式。这些 id 不属于延后由 adapter 处理的部分；只有解释器外部 `FormNumberMap` 顺序需要 adapter 置换。fundamental Cartan 划分产生的 id 同时充当 crate 的全局实形编号。参见 [[WeakRealFormId 的确定性编号与上游对齐]] 与 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[weak-real-form.md:27-32]

## 容量与预算守卫

掩码采用 `u64`，`MAX_MASK_BITS = 63` 限制伴随纤维维数，以保证 `1 << dimension` 移位在范围内。构建入口 `WeakRealFormPartition::build(grading, max_elements)` 对已校验 grading 表背后的伴随纤维进行划分，调用方通过 `max_elements` 限制枚举规模 $2^{\mathrm{dimension}}$。^[weak-real-form.md:34-35, weak-real-form.md:49-51]

预算检查使用 `u128` 计算 `1 << dimension`，再与拓宽后的 `max_elements` 比较。这里刻意不采用饱和计算，以防 `dimension == 64` 借 `usize::MAX` 上限漏过检查。^[weak-real-form.md:37-39]

类表采用 `Vec<u32>`，保留 `u32::MAX` 作为 `CLASS_SENTINEL`。若类序号无法用 `u32` 表示，或恰好等于哨兵，`seeded_class` 返回 `limit_error("classes", …)`。在没有伴随 `m_alpha`、每个掩码各自成类的单连通积情形中，这一守卫原则上可达。^[weak-real-form.md:41-45]

## 查询接口

`class_count` 返回类数，`classes` 提供升序的 `ExactSizeIterator`；`class_of` 与 `class_of_mask` 提供类查询，其中 canonical 坐标掩码查询支撑语言级 `fiber_partition`。`class_representative` 返回该类最小代表元，`quasisplit_class` 返回 class 0，`adjoint_fiber` 提供对应的伴随纤维。^[weak-real-form.md:49-54]

## 测试覆盖与证据边界

来源列出的划分测试锚点包括 A2 恒等情形的 2 类、B2 恒等情形的 3 类及精确代表元、单连通 A1 平凡作用的 2 个单元类，以及 A2 图扭转的 1 类。边界测试包括外来元素与预算不足的拒绝、rank-33 的预算拒绝、rank-64 的 mask-bits 拒绝，以及 `seeded_class` 哨兵守卫的直接测试。rank-33 的拒绝体现枚举预算门，不能解释为固定秩上限。^[weak-real-form.md:84-89]

尚未覆盖的分支包括 `class_of_mask` 越界、各 `ArithmeticOverflow` 转换分支，以及 `max_elements == 2^dimension` 的显式边界断言。参见 [[弱实形式划分与归因的测试覆盖边界]]。^[weak-real-form.md:89-91]

来源属于结构性源码阅读，两次阅读的源码字节及 SHA-256 相同。来源包未执行构建、测试或原版运行，也不重述或扩展既有 HPC 正确性证据链；上述测试锚点的记载不构成新的数学验收、性能或并行结论。^[weak-real-form.md:9-16, weak-real-form.md:106-109]

## Sources

- [weak-real-form.md](../../sources/weak-real-form.md) — 弱实形式划分：adjoint fiber 的 W_im 轨道。
