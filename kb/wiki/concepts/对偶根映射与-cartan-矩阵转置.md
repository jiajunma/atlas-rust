---
title: 对偶根映射与 Cartan 矩阵转置
summary: 对偶根通过余根向量映回 primal RootId，实根及实紧根子系统使用转置 Cartan 矩阵确定类型，体现 B/C 类型互换。
sources:
  - real-weyl.md
kind: concept
createdAt: "2026-10-09T15:07:38.595Z"
updatedAt: "2026-10-09T22:45:06.272Z"
tags:
  - 根系对偶
  - Cartan矩阵
  - 根编号
aliases:
  - 对偶根映射与-cartan-矩阵转置
  - 对C矩
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 对偶根映射与 Cartan 矩阵转置
summary: 对偶根通过原侧余根向量映回 primal RootId，实根及实紧根子系统则使用转置 Cartan 矩阵判定类型，从而体现 B/C 类型互换。
sources:
  - real-weyl.md
kind: concept
tags:
  - 根系
  - 对偶性
  - Cartan矩阵
aliases:
  - 对偶根映射与-cartan-矩阵转置
provenanceState: extracted
---

# 对偶根映射与 Cartan 矩阵转置

在 [[实 Weyl 群与块稳定子的构造]] 中，对偶侧根列表映回原侧（primal）的 `RootId`，而对偶子系统的类型通过转置 Cartan 矩阵判定。这两项约定分别处理根的存储标识与子系统类型：使用原侧标识保存根，不改变其类型计算所需的对偶配对。^[real-weyl.md:58-70]

## 对偶根映回原侧

`RealWeyl` 的根列表统一保存 primal `RootId`。对偶侧的 `real_compact` 与 `real_orth` 通过余根向量映回原侧，其对应依据是：对偶根向量就是 primal 的余根向量。^[real-weyl.md:58-62]

构造时，两侧分别执行 `fiber_side`，再由 `primal_roots_of_dual` 将对偶侧根转换为 primal 根标识。如果找不到对应根，构造返回带有 `"dual root correspondence"` 标识的不变量错误。^[real-weyl.md:94-97]

根列表还保留特定的排序约定：`imaginary` 与 `real` 按上游 `RootNbr` 键排序，即先按高度、再按简单根坐标的反向字典序；`complex` 保留 `makeSimpleComplex` 的输出顺序。相关编号背景见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[real-weyl.md:58-62]

## Cartan 矩阵转置与类型判定

`real_type` 与 `real_compact_type` 调用 `subsystem_cartan(..., transposed=true)`，按对偶子系统计算类型：对偶子系统的 Cartan 矩阵是 primal 子系统 Cartan 矩阵的转置。其余三个类型使用 `transposed=false`。^[real-weyl.md:66-70]

B/C 类型互换在这一转置步骤进入结果。例如，在 C2 datum 上，`Sp(4,R)` 的 Cartan #3 打印 `W^R is a Weyl group of type B2`，对应测试锚点为 `(2,3)`。^[real-weyl.md:66-70]

## 对偶侧数据的构造来源

映射前的对偶数据来自临时重建的 fiber 链。所需的对偶 Cartan 对合 `−θ` 一般只是典范对偶 Cartan 代表元的共轭，因此实现不能直接复用对偶分类存储的 fiber。每次调用均从 `dual_twisted_representative` 开始，重建 Cartan fiber、伴随 fiber、grading、弱实形式分区、Cayley/Cross 分解和标签；该链不使用缓存，各阶段预算来自 `RealWeylContext.budget`。详见 [[对偶 Cartan fiber 链的临时重建]]。^[real-weyl.md:104-116]

对偶来源也体现在 R-群字段中：`real_r` 保存对偶侧的 R-群向量，`imaginary_r` 保存 primal 侧向量。根列表采用 primal 标识的约定，应与这些字段的计算来源分别理解。^[real-weyl.md:58-73]

## 测试证据与边界

来源中的测试注释声明，oracle 输出来自固定上游构建 `rev 4d3e9449`，于 2026-08-11 重新生成并逐字节复制到断言。其中包含 `Sp(4,R)` 的 B/C 互换锚点，是 [[实 Weyl 层的 oracle 测试与证据边界]] 中的具体覆盖案例。^[real-weyl.md:159-168]

来源属于结构性源码阅读，不构成数学或正确性验收；上游文件位置及对应关系转录自代码注释，未核对上游字节。预算耗尽路径和非 quasisplit 对偶形式的行为没有测试锚点，对偶链每次重建也仅被记录为性能线索，未形成性能结论。^[real-weyl.md:177-188]

## Sources

- [real-weyl.md](../../sources/real-weyl.md) — 实 Weyl 群与块稳定子：real_weyl.rs 的构造、对偶 fiber 重放与打印层。
