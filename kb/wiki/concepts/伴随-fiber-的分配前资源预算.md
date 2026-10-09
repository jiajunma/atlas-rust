---
title: 伴随 fiber 的分配前资源预算
summary: 伴随构造在目标矩阵分配前检查半单秩 r、持久条目估计 16r²+rn 与投影工作估计 2n²r，并在单次余特征投影时另行检查 nr 的工作量上限。
sources:
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:43:20.944Z"
updatedAt: "2026-10-09T14:43:20.944Z"
tags:
  - 资源预算
  - 内存管理
  - 构造门控
aliases:
  - 伴随-fiber-的分配前资源预算
  - 伴F的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 伴随 fiber 的分配前资源预算

`AdjointCartanFiber::build` 在分配任何目标矩阵之前执行资源预算预检，限制伴随半单秩、持久存储条目数和投影运算量。预算由 `AdjointFiberBudget` 承载，包含 `integer_lattice`、`max_persistent_entries` 与 `max_projection_operations`。^[cartan-fibers.md:54-55, cartan-fibers.md:114-128]

## 预检顺序与阈值

构造器先检查根系统与根对合的 datum 是否一致，再检查源 fiber 的对合是否与实际根对合一致；对应失败分别返回 `DatumMismatch` 和 `CartanFiberInvolutionMismatch`。只有通过这些检查，才进入 `validate_adjoint_build_budget`，因此来源或对合不匹配的错误先于预算错误报告。^[cartan-fibers.md:114-123]

记 \(r=\text{semisimple_rank}\)、\(n=\text{lattice_rank}\)，预算预检按以下顺序拒绝超限输入；各项错误均为 `AdjointFiberResourceLimit`，以不同的资源名称区分。^[cartan-fibers.md:122-128]

| 检查对象 | 拒绝条件 | 错误资源名称 |
| --- | --- | --- |
| 伴随半单秩 | \(r>\text{max_rank}\) | `"semisimple rank"` |
| 持久存储条目数 | \(16r^2+rn>\text{max_persistent_entries}\) | `"persistent entries"` |
| 投影运算量 | \(2n^2r>\text{max_projection_operations}\) | `"projection operations"` |

投影运算量估计的依据是：每个源坐标至多对应一个分子基向量和一个分母基向量，而每次直接投影都要为每个伴随单根检查全部源坐标。持久存储估计使用常数 `ADJOINT_PERSISTENT_SQUARES = 16`，但来源没有记录该常数的逐项构成。^[cartan-fibers.md:67-68, cartan-fibers.md:122-128]

## 与构造流程及逐次预算的关系

预检通过后，构造器才建立伴随投影与根基作用矩阵，导出余权作用矩阵，构造目标格对合和 Cartan fiber，最后验证投影能够沿子商下降。目标 fiber 复用 [[Cartan fiber 的先分母后分子构造]]：整数分母计算执行整数格预算，有限域部分另由 `try_reserve_exact` 与 `checked_*` 防护，失败可报告 `AllocationFailed` 或 `ArithmeticOverflow`。^[cartan-fibers.md:72-80, cartan-fibers.md:129-136]

构造期预检之外，`AdjointProjection::map_coweight` 在验证输入的来源身份后，还执行 `check_projection_work(1)`。其逐次预算条件为“源秩 × 目标秩 × 向量数”不超过 `max_projection_operations`，之后才逐根计算配对。因此，构造预算与单次余权投影预算分别约束不同阶段的工作量。^[cartan-fibers.md:105-112]

通过构造期的下降验证后，[[FiberToAdjoint 的按需投影]] 每次依次取得源元素的规范代表、执行模二投影、构造目标元素；它不保留稠密模二矩阵，也不缓存像。^[cartan-fibers.md:138-141]

## 测试与证据边界

来源记录了两个预算预检测试，锚定“拒绝早于目标矩阵分配”；另有 rank-33 动态伴随秩的构造测试，覆盖超过历史打包上限的情形。这些属于结构性阅读所记录的测试锚点，不构成 fiber 层的数学验收，也不是本次知识维护执行测试所得的结果。^[cartan-fibers.md:10-12, cartan-fibers.md:143-149, cartan-fibers.md:167-171]

`IntegerLatticeBudget` 四参数的完整语义不在此来源覆盖范围内；持久存储常数 `16` 的细分依据也未文档化。因此，本页仅描述已记录的预算公式、检查顺序与失败行为。^[cartan-fibers.md:122-128, cartan-fibers.md:155-158]

## Sources

- [cartan-fibers.md](cartan-fibers.md) — Cartan fiber 与伴随 Cartan fiber（`cartan_fiber.rs` / `adjoint_fiber.rs`）。
