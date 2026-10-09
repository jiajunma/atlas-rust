---
title: 典范纤维与 K 型等价判定
summary: to_canonical_fiber 沿 canonicalize 给出的词进行复单根 cross；equivalent 先检查 Cartan 类，再比较双方典范纤维中的结果。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:35.142Z"
updatedAt: "2026-10-09T14:56:35.142Z"
tags:
  - 表示论
  - 等价判定
  - 典范化
aliases:
  - 典范纤维与-k-型等价判定
  - 典K型
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 典范纤维与 K 型等价判定

K 型的 `equivalent` 判定先检查两个参数是否属于同一 Cartan 类，再分别通过 `to_canonical_fiber` 搬运到典范纤维，最后作严格相等比较。典范纤维因此提供了同一 Cartan 类内比较 K 型的共同位置。^[ktype.md:31-31]

## K 型的规范表示

`KType { x: KgbId, lam_rho: Weight, height: u32 }` 表示标准表示参数去掉 ν 后的 K-限制。`lam_rho` 存储其 `(1−θ_x)X*` 陪集经 `lambda_unique` 选出的规范代表，`height` 在构造时预计算；crate 外只能通过 `sr_k` 构造，crate 内的原始构造器 `new` 不校验不变量。相关表示约定见 [[KType 表示参数与规范化构造]]。^[ktype.md:11-14, ktype.md:18-21]

## 典范纤维搬运

`to_canonical_fiber` 沿 `InnerClass::canonicalize` 给出的词逐生成元执行 cross。每个生成元都必须对应复单根；不满足这一要求时，触发 `"canonical fiber cross"` 错误。这一约束是搬运过程的明确检查点。^[ktype.md:41-42]

典范纤维搬运与完整规范化具有不同职责：`normalised` 在典范化之后，还会耗尽奇异复下降与负复求值，并使用 `weight_defect + 图大小 + 1` 作为终止预算；`equivalent` 的记载流程则是同 Cartan 类检查、分别搬运和严格相等比较。相关变形过程见 [[K 型变形与终止预算]]。^[ktype.md:31-31, ktype.md:43-44]

## 等价判定流程

`equivalent` 的比较顺序为：先以 Cartan 类是否相同作为门槛；通过后，对两侧分别调用 `to_canonical_fiber`；最后比较搬运结果是否严格相等。因此，原始参数的直接比较并不是该方法记录的完整等价判定流程。^[ktype.md:31-31]

## 测试与证据边界

现有测试包含 split A1 的模 `2X*` 相等性锚点，但未覆盖 `equivalent` 的异 Cartan 分支，也未覆盖 `to_canonical_fiber` 的错误分支。不能将该锚点视为这些分支已经得到验证；更完整的覆盖说明见 [[K 型实现的测试锚点与证据边界]]。^[ktype.md:65-74]

本文依据的是 `ktype.rs` 的结构性阅读记录，不代表数学验收。该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[ktype.md:10-14, ktype.md:83-87]

## Sources

- [ktype.md](../../sources/ktype.md) — K 型值与 RepContext 谓词/变形（ktype.rs）
