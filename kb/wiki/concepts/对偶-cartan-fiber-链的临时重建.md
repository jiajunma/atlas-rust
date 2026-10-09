---
title: 对偶 Cartan fiber 链的临时重建
summary: 由于对偶对合 −θ 通常仅与典范代表共轭，dual_side 每次调用都在预算约束下重建 fiber、grading、实形分划与标签链，当前无缓存。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:07:21.471Z"
updatedAt: "2026-10-09T15:07:21.471Z"
tags:
  - Cartan理论
  - 对偶性
  - 预算管理
aliases:
  - 对偶-cartan-fiber-链的临时重建
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 对偶 Cartan fiber 链的临时重建

对偶 Cartan fiber 链的临时重建是 `RealWeylContext::real_weyl` 构造中的对偶侧准备步骤。私有辅助逻辑 `dual_side` 每次调用都重新构造 fiber、grading、弱实形式分区和标签等数据，不使用缓存。它为[[实 Weyl 群与块稳定子的构造]]提供对偶侧代表元及后续计算所需的上下文。^[real-weyl.md:83-97, real-weyl.md:104-116, real-weyl.md:186-187]

## 为什么需要重建

上游直接读取 `cc.dualFiber()`，Rust crate 则不能复用对偶分类中存储的 fiber：当前 Cartan 对合对应的对偶对合 `-θ`，一般只是典范对偶 Cartan 代表元的共轭，而非该代表元本身。源码注释以 `tw * w0` 描述相关转换，因此实现针对当前 Cartan 类临时重建整条对偶链。^[real-weyl.md:106-116]

## 构造链与预算

重建从 `dual_twisted_representative` 开始，借助 `longest_action(对偶, weyl_budget)` 在右侧补上最长元，然后依次执行 `CartanFiber::build`、`AdjointCartanFiber::build`、`CartanGradingData::build`、`WeakRealFormPartition::build`、`CayleyCrossDecomposition::build` 和 `RealFormLabels::build`。这些步骤涉及[[伴随 Cartan 纤维的构建与下降验证]]、[[弱实形式的伴随 Cartan 纤维轨道划分]]及[[扭曲对合的 Cayley/Cross 分解]]。^[real-weyl.md:109-114]

各阶段使用 `RealWeylContext.budget` 中 `CartanClassificationBudget` 的相应子预算；primal 侧则读取已经受预算约束构建的 `classification`。若缺失对偶 fundamental 类，构造返回带有 `"dual fundamental class"` 标识的不变量错误。^[real-weyl.md:113-116]

## 在实 Weyl 构造中的位置

`real_weyl` 先验证 Cartan 类和实形式标签，并取得 primal 代表元 `x`，随后调用 `dual_side` 重建对偶链，再选择对偶代表元 `y`。因此，对偶链重建发生在 primal 代表元定位之后、对偶代表元选择之前。^[real-weyl.md:85-93]

当 `dual_form` 为 `None` 时，`y` 取对偶伴随 fiber 的零元，对应上游硬编码的 quasisplit 代表；当其为 `Some` 时，先通过对偶标签定位形式，再取得代表元。标签缺失返回 `RealFormNotDefinedOnCartan`，代表元缺失则返回 `"dual real-form representative"` 不变量错误。块稳定子打印包装接收的 `dual_form` 是对偶内类的外部形式编号，并将其转换为内部 `WeakRealFormId`。^[real-weyl.md:91-93, real-weyl.md:99-102]

完成代表元选择后，两侧共用 `fiber_side` 逻辑。对偶侧根列表经 `primal_roots_of_dual` 映回 primal `RootId`，映射依据是对偶根向量即 primal 余根向量；找不到对应根时返回 `"dual root correspondence"` 不变量错误。^[real-weyl.md:58-62, real-weyl.md:94-96, real-weyl.md:118-120]

## 证据与限制

每次调用重建且无缓存是已记录的实现行为，但该取舍是否有意尚未确认；它只能视为性能调查线索，不能据此断言性能损失或缓存收益。预算耗尽路径和非 quasisplit 对偶形式的行为均无测试锚点。^[real-weyl.md:186-188]

本页依据结构性阅读材料，不构成数学正确性验收。材料中的上游行号来自源码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。相关测试背景见[[实 Weyl 层的 oracle 测试与证据边界]]。^[real-weyl.md:9-13, real-weyl.md:179-180, real-weyl.md:192-196]

## Sources

- [real-weyl.md](real-weyl.md)：实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
