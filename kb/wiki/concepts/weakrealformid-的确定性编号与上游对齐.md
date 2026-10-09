---
title: WeakRealFormId 的确定性编号与上游对齐
summary: 轨道按典范坐标最小掩码升序编号，零号为 identity 轨道；来源报告内部编号已与上游对齐，外部 FormNumberMap 仍需置换。
sources:
  - weak-real-form.md
kind: concept
createdAt: "2026-10-09T15:15:19.607Z"
updatedAt: "2026-10-09T19:37:37.627Z"
tags:
  - 弱实形式
  - 编号约定
  - 上游兼容
aliases:
  - weakrealformid-的确定性编号与上游对齐
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# WeakRealFormId 的确定性编号与上游对齐

`WeakRealFormId` 标识 adjoint Cartan fiber 在 $W_{im}$ 作用下的轨道，轨道对应内类在指定 Cartan 对合处的弱实形式。`WeakRealFormPartition` 保存类表及每类的确定性代表元，实形标签由 `RealFormLabels` 保存。参见 [[弱实形式的伴随 Cartan 纤维轨道划分]]。^[weak-real-form.md:20-23]

## 确定性编号规则

各轨道按其 canonical-coordinate 整数序下的最小元升序编号，该最小元同时作为类的代表元。class 0 是 identity 元素所在的轨道，对应 quasisplit normalization；由 fundamental Cartan 的划分生成的 id，还兼任 crate 的全局实形编号。^[weak-real-form.md:27-32, weak-real-form.md:53-54]

`walk_mask_orbits` 按掩码升序播种，以 LIFO 栈遍历轨道。每个新轨道首次出现的掩码即其最小元，因此类号与代表元由固定坐标序确定。类表采用 `Vec<u32>`，并保留 `u32::MAX` 作为 `CLASS_SENTINEL`；新类序号无法放入 `u32` 或等于哨兵时，返回 `limit_error("classes", …)`。遍历机制参见 [[基于位掩码的 W_im 轨道枚举]]。^[weak-real-form.md:37-45]

## 与上游编号的对齐

来源记载，`SEED_X0_DESIGN.md` 的 stage-(d) 排序审计证明，该编号与上游内部 `RealFormNbr` 一致。对齐依赖三个相同的约定：升序轨道播种、low-pivot RREF subquotient 基，以及坐标提取方式。坐标结构参见 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[weak-real-form.md:27-30]

这些内部 id 的对齐不属于 adapter 延后处理范围；需要 adapter 置换的仅是解释器的外部 `FormNumberMap` 顺序。内部实形编号与解释器外部顺序因此应分别处理。^[weak-real-form.md:29-32]

## 局部类与全局编号

指定 Cartan 处的局部轨道通过该 Cartan 的标签映射到 fundamental weak-form 编号。在 [[代表元级弱实形式归因]] 中，grading 先经 `element_from_grading` 转为纤维元素，再由 `class_of` 得到局部类，最后通过 `labels().label(local)` 得到全局编号。^[weak-real-form.md:58-63, weak-real-form.md:80-82]

查询接口中，`class_count` 返回类数，`classes` 提供升序的 `ExactSizeIterator`；`class_of` 与 `class_of_mask` 查询元素或 canonical 坐标掩码所属类；`class_representative` 返回类的最小代表元，`quasisplit_class` 返回 class 0。^[weak-real-form.md:49-54]

## 容量约束

掩码采用 `u64`，`MAX_MASK_BITS = 63` 限制可用维数；构造参数 `max_elements` 则限制枚举规模 $2^{dimension}$。实现使用 `u128` 计算该规模并与扩宽后的预算比较，故意不采用饱和处理，以防 `dimension == 64` 借 `usize::MAX` 上限漏过检查。^[weak-real-form.md:34-39, weak-real-form.md:49-51]

## 测试与证据边界

来源列出的测试锚点包括 A2 恒等对合的两类、B2 恒等对合的三类及精确代表元、SC A1 平凡作用的两个单元类、A2 图扭转的一类，以及 `seeded_class` 哨兵守卫。尚未覆盖的分支包括 `class_of_mask` 越界、各 `ArithmeticOverflow` 转换分支，以及预算恰好等于 $2^{dimension}$ 的显式断言。^[weak-real-form.md:84-91]

本页依据结构性源码阅读包。该包未执行构建、测试或原版运行，也未独立重读上游源码；上述编号对齐是对来源所记排序审计的转述，不构成新的数学验收、性能或并行结论。^[weak-real-form.md:9-16, weak-real-form.md:27-32, weak-real-form.md:101-106]

## Sources

- [weak-real-form.md](weak-real-form.md) — 弱实形式划分：adjoint fiber 的 W_im 轨道。
