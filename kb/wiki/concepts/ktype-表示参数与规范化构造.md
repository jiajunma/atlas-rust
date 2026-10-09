---
title: KType 表示参数与规范化构造
summary: KType 保存去掉 ν 的 K-限制参数；sr_k 选取 lambda_unique 陪集代表并预计算 height，内部 new 不校验不变量。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:25.774Z"
updatedAt: "2026-10-09T22:36:03.991Z"
tags:
  - 表示论
  - K型
  - 规范化
aliases:
  - ktype-表示参数与规范化构造
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KType 表示参数与规范化构造
summary: KType 保存去掉 ν 的标准表示参数之 K-限制；sr_k 选取规范陪集代表并预计算 height，内部 new 依赖调用方维护不变量。
sources:
  - ktype.md
kind: concept
tags:
  - 表示论
  - Rust设计
  - 规范化
aliases:
  - ktype-表示参数与规范化构造
---

# KType 表示参数与规范化构造

`KType` 表示[[StandardRepr 标准表示参数]]去掉 \(\nu\) 后的 K-限制，结构为 `KType { x: KgbId, lam_rho: Weight, height: u32 }`。其存储约定是：`lam_rho` 为所属 \((1-\theta_x)X^*\) 陪集经 `lambda_unique` 选出的规范代表，`height` 在构造时预计算。^[ktype.md:10-14]

## 构造入口与不变量

`KType::new` 是 `pub(crate)` 可见的原始构造器，不校验不变量。crate 内部可通过它装入任意 `height`，正确性依赖调用方纪律；crate 外部只能通过 `sr_k` 构造。^[ktype.md:18-19]

`sr_k(rc, x, λ−ρ)` 先用 `lambda_unique` 规范化，再预存 \((1+\theta)\lambda\) 的 height。计算使用恒等式 \((1+\theta)\lambda=\lambda_\rho+\theta\cdot\lambda_\rho+(1+\theta)\rho\)，`theta_plus_1_lambda` 也采用同一公式。^[ktype.md:20-21]

规范代表的陪集对应[[对合的 (1−θ)X* 图像基对|对合的图像格]]。来源所述“规范化只在 `sr_k` 发生一次”针对构造入口；后续变形仍可能重新调用 `lambda_unique`，例如 `made_dominant` 在每轮末尾重新规范化。^[ktype.md:11-14, ktype.md:35-38]

## 变形中的规范化与 height

`made_dominant` 要求输入为标准 K 型，对负求值的复单根执行 cross 与反射，并在每轮末尾重新规范化。它原样携带 `height`，依据是源码注释断言的 Weyl 共轭移动下 height 不变性。其终止预算与失败行为见[[K 型变形与终止预算]]。^[ktype.md:35-38]

[[finals_for 带号重数展开|finals_for]] 的 height 来源具有不对称性：工作栈中的新项通过 `sr_k` 重新计算 height，结果项则用 `KType::new(x, normalized, 沿用待处理项的 height)` 构造。这是来源记录的结构性阅读观察。^[ktype.md:45-53]

## 参数等价性

`equivalent` 先要求两个参数属于同一 Cartan 类，再分别调用 `to_canonical_fiber`，最后进行严格相等比较。典范纤维转换沿 `InnerClass::canonicalize` 给出的词执行 cross，每个生成元必须是复单根，否则触发 `"canonical fiber cross"` 错误。详见[[典范纤维与 K 型等价判定]]。^[ktype.md:31-31, ktype.md:41-42]

## 测试与证据边界

来源列出的测试锚点包括 split A1 中 `x=2`、权重 `[0]` 的 K 型，六个谓词全真、三个变形保持不动、模 \(2X^*\) 的相等性，以及通过 `sr_k_of_standard` 和 `sr_of_ktype` 实现的 `StandardRepr` 往返。su(2,1) 的非 final 锚点涉及 `x=4/5` 的当选代表。^[ktype.md:65-70]

两个 su(2,1) 测试仅含 `eprintln!` 而无断言，属于观察型用例，不构成机械锚点。未覆盖的分支包括 `equivalent` 的异 Cartan 分支、`to_canonical_fiber` 的错误分支、全部终止预算错误及溢出和分配分支。详见[[K 型实现的测试锚点与证据边界]]。^[ktype.md:70-74]

本页依据 `ktype.rs` 的结构性阅读，不构成数学验收。来源记录了维护者对照源码逐条核对的过程；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[ktype.md:9-14, ktype.md:83-87]

## Sources

- [ktype.md](../../sources/ktype.md) — K 型值与 RepContext 谓词/变形（ktype.rs）。
