---
title: 伴随 fiber 的分配前资源预算
summary: 目标矩阵分配前检查半单秩 r、持久条目估计 16r²+rn 与投影工作估计 2n²r，单次余特征投影另检查 nr 工作量。
sources:
  - cartan-fibers.md
kind: concept
createdAt: "2026-10-09T14:43:20.944Z"
updatedAt: "2026-10-10T00:29:25.372Z"
tags:
  - 伴随纤维
  - 资源预算
aliases:
  - 伴随-fiber-的分配前资源预算
  - 伴F的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 伴随 fiber 的分配前资源预算
summary: 伴随 fiber 在目标矩阵分配前检查半单秩、持久条目估计 16r²+rn 和投影工作估计 2n²r；单次余特征投影另检查 nr 工作量。
sources:
  - cartan-fibers.md
kind: concept
tags:
  - 资源预算
  - 伴随纤维
aliases:
  - 伴随-fiber-的分配前资源预算
provenanceState: extracted
---

# 伴随 fiber 的分配前资源预算

`AdjointCartanFiber::build` 在分配任何目标矩阵之前执行资源预算预检，限制伴随半单秩、持久存储条目数和投影运算量。预算类型 `AdjointFiberBudget` 包含 `integer_lattice`、`max_persistent_entries` 与 `max_projection_operations`。^[cartan-fibers.md:54-55, cartan-fibers.md:114-128]

## 检查顺序与拒绝条件

构造器先检查根系统与根对合的 datum 是否一致，不符时报 `DatumMismatch`；再检查源 fiber 的对合是否与根对合一致，不符时报 `CartanFiberInvolutionMismatch`。通过这两项检查后，才调用 `validate_adjoint_build_budget`，因此这两类一致性错误先于预算错误报告。^[cartan-fibers.md:114-123]

记 $r=\mathrm{semisimple\_rank}$、$n=\mathrm{lattice\_rank}$。预检依次检查下列条件；超限时返回 `AdjointFiberResourceLimit`，以资源名称区分失败原因。^[cartan-fibers.md:122-128]

| 检查对象 | 拒绝条件 | 错误资源名称 |
| --- | --- | --- |
| 伴随半单秩 | $r>\mathrm{max\_rank}$ | `"semisimple rank"` |
| 持久存储条目数 | $16r^2+rn>\mathrm{max\_persistent\_entries}$ | `"persistent entries"` |
| 投影运算量 | $2n^2r>\mathrm{max\_projection\_operations}$ | `"projection operations"` |

投影工作估计 $2n^2r$ 的注释依据是：每个源坐标至多对应一个分子基向量和一个分母基向量，而每次直接投影需要为每个伴随单根检查全部源坐标。持久条目估计使用常数 `ADJOINT_PERSISTENT_SQUARES = 16`，但其逐项构成未文档化。^[cartan-fibers.md:67-68, cartan-fibers.md:122-128]

## 与后续构造的关系

预算预检通过后，构造器建立伴随投影，计算根基作用矩阵及其转置所给出的余特征作用矩阵，再构造目标格对合和 Cartan fiber，最后验证投影能够沿子商下降。完整流程见 [[伴随 Cartan 纤维的构建与下降验证]]。^[cartan-fibers.md:129-136]

目标 fiber 复用 [[Cartan fiber 的先分母后分子构造]]：先计算整数负特征格并模二归约，在这一阶段执行整数格预算，再计算有限域分子。有限域侧不使用整数预算，而由 `try_reserve_exact` 与 `checked_*` 防护，失败可报告 `AllocationFailed` 或 `ArithmeticOverflow`。因此，伴随构造的预检之后仍有后续构造自身的预算和分配检查。^[cartan-fibers.md:72-80, cartan-fibers.md:134-136]

## 单次投影的工作量预算

构造期预检之外，`AdjointProjection::map_coweight` 每次调用还会先通过 `ensure_source` 验证输入的来源身份，不匹配时报 `DatumMismatch`；随后执行 `check_projection_work(1)`，再逐根计算配对。逐次预算要求“源秩 × 目标秩 × 向量数”不超过 `max_projection_operations`，对单个向量即检查 $nr$。^[cartan-fibers.md:105-112]

[[FiberToAdjoint 的按需投影]] 依次取得源元素的规范代表、执行模二投影、构造目标元素。它不保留稠密模二矩阵，也不缓存像；投影沿子商下降的验证已在构造阶段一次性完成。^[cartan-fibers.md:90-92, cartan-fibers.md:134-141]

## 测试与证据边界

来源记录了两个预算预检测试，锚定“拒绝早于目标矩阵分配”；另有 rank-33 动态伴随秩的构造测试，覆盖超过历史打包上限的情形。这些是结构性源码阅读记录的测试锚点，不构成 fiber 层的数学验收；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cartan-fibers.md:10-12, cartan-fibers.md:143-149, cartan-fibers.md:167-171]

`IntegerLatticeBudget` 四参数的完整语义不在此来源覆盖范围内，常数 `16` 的细分依据也未文档化。此外，构造入口的 `DatumMismatch` 门控没有专门测试锚点。^[cartan-fibers.md:122-128, cartan-fibers.md:155-163]

## Sources

- [cartan-fibers.md](../../sources/cartan-fibers.md) — Cartan fiber 与伴随 Cartan fiber（`cartan_fiber.rs` / `adjoint_fiber.rs`）。
