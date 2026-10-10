---
title: 典范纤维与 K 型等价判定
summary: equivalent 先检查 Cartan 类，再比较运输到典范纤维后的值；to_canonical_fiber 要求运输词的每个生成元对应复单根。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:35.142Z"
updatedAt: "2026-10-10T00:40:06.162Z"
tags:
  - K型
  - Cartan
  - 等价判定
aliases:
  - 典范纤维与-k-型等价判定
  - 典K型
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 典范纤维与 K 型等价判定
summary: equivalent 先检查 Cartan 类，再比较双方搬运到典范纤维后的值；搬运词中的每个生成元必须对应复单根，相关错误分支尚缺测试覆盖。
sources:
  - ktype.md
kind: concept
tags:
  - K型
  - 典范纤维
  - 等价判定
aliases:
  - 典范纤维与-k-型等价判定
  - 典K型
---

# 典范纤维与 K 型等价判定

K 型的 `equivalent` 按固定顺序判定等价性：先检查两个参数是否属于同一 Cartan 类，再分别调用 `to_canonical_fiber`，最后对搬运后的结果作严格相等比较。^[ktype.md:31-31]

## K 型的规范表示

`KType { x: KgbId, lam_rho: Weight, height: u32 }` 表示标准表示参数去掉 ν 后的 K-限制。`lam_rho` 保存其 \((1-\theta_x)X^*\) 陪集经 `lambda_unique` 选出的规范代表，`height` 在构造时预计算。crate 外只能通过 `sr_k` 构造；crate 内的原始构造器 `new` 不校验不变量，调用方须维护这些约束。详见 [[KType 表示参数与规范化构造]]。^[ktype.md:11-14, ktype.md:18-21]

## 典范纤维搬运

`to_canonical_fiber` 沿 `InnerClass::canonicalize` 给出的词逐生成元执行 cross。每个生成元都必须对应复单根；违反这一要求时，报 `"canonical fiber cross"` 错误。^[ktype.md:41-42]

`equivalent` 使用典范纤维搬运后的严格比较；`normalised` 则在典范化后继续耗尽奇异复下降与负复求值。后者的终止预算为 `weight_defect + 图大小 + 1`，对应错误为 `"normal form termination"`。相关变形流程见 [[K 型变形与终止预算]]。^[ktype.md:31-31, ktype.md:43-44]

## 测试与证据边界

现有测试包含 split A1 的模 \(2X^*\) 相等性锚点，但未覆盖 `equivalent` 的异 Cartan 分支，也未覆盖 `to_canonical_fiber` 的错误分支。相关覆盖范围见 [[K 型实现的测试锚点与证据边界]]。^[ktype.md:65-74]

本文依据 `ktype.rs` 的结构性阅读记录，不构成数学验收。来源说明维护者已对照源码逐条核对改写；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[ktype.md:9-14, ktype.md:83-87]

## Sources

- [ktype.md](../../sources/ktype.md) — K 型值与 RepContext 谓词/变形（ktype.rs）。
