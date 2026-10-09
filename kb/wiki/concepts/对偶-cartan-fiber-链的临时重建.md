---
title: 对偶 Cartan fiber 链的临时重建
summary: 对偶对合通常仅与典范对偶代表共轭，因此每次调用按子预算重建 fiber、grading、轨道分划与标签链；当前无缓存，来源未验证性能影响。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:07:21.471Z"
updatedAt: "2026-10-09T21:07:08.738Z"
tags:
  - 对偶
  - Cartan-纤维
  - 资源预算
aliases:
  - 对偶-cartan-fiber-链的临时重建
  - 对CF链
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 对偶 Cartan fiber 链的临时重建
summary: 对偶对合 −θ 通常仅与典范对偶 Cartan 代表元共轭，因此 dual_side 每次调用都在预算约束下重建 fiber、grading、弱实形式分区与标签链，当前无缓存。
sources:
  - real-weyl.md
kind: concept
tags:
  - Cartan理论
  - 对偶性
  - 预算管理
aliases:
  - 对偶-cartan-fiber-链的临时重建
provenanceState: extracted
---

# 对偶 Cartan fiber 链的临时重建

对偶 Cartan fiber 链的临时重建是 `RealWeylContext::real_weyl` 的对偶侧准备步骤。文件私有的 `dual_side` 每次调用都重新构造 fiber、grading、弱实形式分区及标签，不使用缓存；这些数据用于选择对偶代表元并完成实 Weyl 群或块稳定子的构造。^[real-weyl.md:49-54, real-weyl.md:83-97, real-weyl.md:104-116]

## 重建原因

上游直接读取 `cc.dualFiber()`，Rust crate 则不能直接复用对偶分类中存储的 fiber：当前 Cartan 对合的对偶对合 \(-\theta\) 一般只是典范对偶 Cartan 代表元的共轭，而非该代表元本身。来源以 `tw * w0` 描述相关转换，因此实现针对当前 Cartan 类临时重建整条对偶链。相关背景见[[跨对偶的 Cartan 类对应与扭曲代表元]]。^[real-weyl.md:106-116]

## 构造顺序与预算

重建首先调用 `dual_twisted_representative`，通过 `longest_action(对偶, weyl_budget)` 在右侧补上最长元；随后按固定顺序执行 `CartanFiber::build`、`AdjointCartanFiber::build`、`CartanGradingData::build`、`WeakRealFormPartition::build`、`CayleyCrossDecomposition::build` 和 `RealFormLabels::build`。其中分区与分解分别关联[[弱实形式的伴随 Cartan 纤维轨道划分]]和[[扭曲对合的 Cayley/Cross 分解]]。^[real-weyl.md:109-114]

各阶段的预算取自 `RealWeylContext.budget` 所持 `CartanClassificationBudget` 的相应子预算；primal 侧则读取已经受预算约束构建的 `classification`。若缺失对偶 fundamental 类，重建返回带有 `"dual fundamental class"` 标识的不变量错误。^[real-weyl.md:113-116]

## 代表元选择与后续计算

`real_weyl` 先检查 Cartan 类和实形式标签，并从分区取得 primal 代表元 `x`，随后重建对偶链，再选择对偶代表元 `y`。因此，对偶链重建位于 primal 代表元定位之后、对偶代表元选择之前。^[real-weyl.md:85-93]

当 `dual_form` 为 `None` 时，`y` 取对偶伴随 fiber 的零元，对应上游硬编码的 quasisplit 代表；当其为 `Some` 时，先通过对偶标签定位形式，再取得分区代表元。标签定位失败返回 `RealFormNotDefinedOnCartan`，代表元缺失返回 `"dual real-form representative"` 不变量错误。参见[[实 Weyl 群与块稳定子的代表元选择]]。^[real-weyl.md:91-93]

打印包装通过 `ExternalFormOrder` 将外部形式编号转换为内部 `WeakRealFormId`；其中 `block_stabilizer_print` 的 `dual_form` 属于对偶内类的外部编号，越界返回 `IndexOutOfRange`。^[real-weyl.md:99-102]

选定代表元后，两侧共用 `fiber_side` 逻辑。对偶侧根列表经 `primal_roots_of_dual` 映回 primal `RootId`，其依据是对偶根向量即 primal 余根向量；未找到对应根时返回 `"dual root correspondence"` 不变量错误。相关列表约定见[[实 Weyl 群的根列表与排序契约]]。^[real-weyl.md:58-62, real-weyl.md:94-96, real-weyl.md:118-120]

## 证据与限制

每次重建且无缓存是来源记录的实现行为，但这一取舍是否有意尚未确认。它只是性能调查线索，来源未给出性能结论；预算耗尽路径和非 quasisplit 对偶形式的行为也没有测试锚点。^[real-weyl.md:186-188]

本页依据结构性源码阅读，不构成实 Weyl 层的数学验收。材料中的上游行号转录自源码注释，未核对上游字节；此次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[real-weyl.md:9-13, real-weyl.md:192-196]

## Sources

- [real-weyl.md](../../sources/real-weyl.md)：实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
