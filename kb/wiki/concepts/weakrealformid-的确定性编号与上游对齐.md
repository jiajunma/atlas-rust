---
title: WeakRealFormId 的确定性编号与上游对齐
summary: 按轨道最小 canonical-coordinate 掩码升序编号，identity 轨道为 class 0；来源记载内部编号已完成上游对齐，外部 FormNumberMap 顺序仍需 adapter 置换。
sources:
  - weak-real-form.md
kind: concept
createdAt: "2026-10-09T15:15:19.607Z"
updatedAt: "2026-10-09T15:15:19.607Z"
tags:
  - 弱实形式
  - 确定性编号
  - 上游对齐
aliases:
  - weakrealformid-的确定性编号与上游对齐
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# WeakRealFormId 的确定性编号与上游对齐

`WeakRealFormId` 标识 adjoint Cartan fiber 在 $W_{im}$ 作用下的轨道。这些轨道对应内类在指定 Cartan 对合处的弱实形式；`WeakRealFormPartition` 保存类表及每类的确定性代表元，实形标签则由 `RealFormLabels` 保存。参见 [[弱实形式的伴随 Cartan 纤维轨道划分]]。^[weak-real-form.md:20-23]

## 确定性编号规则

编号按各轨道在 canonical-coordinate 整数序下的最小元升序指派，轨道最小元同时作为该类的代表元。class 0 是 identity 元素所在的轨道，对应 quasisplit normalization。由 fundamental Cartan 的划分生成的 id，还兼任 crate 的全局实形编号。^[weak-real-form.md:27-32, weak-real-form.md:53-54]

`walk_mask_orbits` 按掩码升序播种，并以 LIFO 栈遍历轨道；每个新轨道首次出现的掩码就是其最小元，因此类号与代表元均由固定坐标序确定。类表使用 `Vec<u32>`，其中 `u32::MAX` 保留为 `CLASS_SENTINEL`；若新类序号无法放入 `u32` 或等于哨兵，则返回 `limit_error("classes", …)`。参见 [[基于位掩码的 W_im 轨道枚举]]。^[weak-real-form.md:37-45]

## 与上游内部编号的对齐

来源记载，`SEED_X0_DESIGN.md` 的 stage-(d) 排序审计证明，该编号与上游内部 `RealFormNbr` 一致。对齐依据包括相同的升序轨道播种、相同的 low-pivot RREF subquotient 基，以及相同的坐标提取方式。相关坐标结构见 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[weak-real-form.md:27-30]

这些内部 id 无需等待 adapter 置换；需要 adapter 置换的是解释器的外部 `FormNumberMap` 顺序。因此，内部编号一致性与外部展示顺序是不同的契约。参见 [[弱实形式的外部编号与严格排序]]。^[weak-real-form.md:29-32]

## 局部编号与全局标签

指定 Cartan 处的局部轨道通过该 Cartan 的标签映射到 fundamental weak-form 编号。在代表元级归因流程中，grading 经 `element_from_grading` 转为纤维元素，再由 `class_of` 得到局部类，最后通过 `labels().label(local)` 得到全局编号。参见 [[弱实形式的局部到全局标签映射]]与 [[代表元级弱实形式归因]]。^[weak-real-form.md:58-63, weak-real-form.md:80-82]

查询接口包括按升序返回类的 `classes`、查询元素或 canonical 坐标掩码所属类的 `class_of` 与 `class_of_mask`、返回最小代表元的 `class_representative`，以及返回 class 0 的 `quasisplit_class`。这些接口使编号顺序、类归属和代表元选择可直接查询。^[weak-real-form.md:49-54]

## 容量约束与证据边界

掩码采用 `u64`，`MAX_MASK_BITS = 63` 限制可用维数；调用方的 `max_elements` 则约束枚举规模 $2^{dimension}$。实现用 `u128` 计算枚举规模并与扩宽后的预算比较，故意不做饱和处理，以防 `dimension == 64` 借 `usize::MAX` 上限漏过检查。^[weak-real-form.md:34-39, weak-real-form.md:49-51]

源码测试锚点包括 A2 恒等对合的两类、B2 恒等对合的三类及精确代表元、SC A1 平凡作用的两个单元类、A2 图扭转的一类，以及类号哨兵守卫。来源同时记录，`class_of_mask` 越界、部分算术溢出转换和预算恰等于 $2^{dimension}$ 的显式断言尚未覆盖。^[weak-real-form.md:84-91]

本页依据结构性源码阅读包；该包未执行构建、测试或原版运行，也未独立重读上游源码。因此，上述编号对齐是对来源所记排序审计的转述，不构成新的数学验收、性能或并行结论。^[weak-real-form.md:9-16, weak-real-form.md:29-32, weak-real-form.md:101-106]

## Sources

- [weak-real-form.md](weak-real-form.md) — 弱实形式划分：adjoint fiber 的 W_im 轨道。
