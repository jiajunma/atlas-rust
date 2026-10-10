---
title: KType 表示参数与规范化构造
summary: KType 表示去掉 ν 的 K-限制参数；sr_k 通过 lambda_unique 选取陪集代表并预计算 height，内部 new 不校验不变量。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:25.774Z"
updatedAt: "2026-10-10T00:39:46.702Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: KType 表示参数与规范化构造
summary: KType 保存去掉 ν 的标准表示参数之 K-限制；sr_k 选取规范陪集代表并预计算 height，内部 new 依赖调用方维护不变量。
sources:
  - ktype.md
kind: concept
tags:
  - 表示论
  - K型
  - 规范化
aliases:
  - ktype-表示参数与规范化构造
---

# KType 表示参数与规范化构造

`KType` 表示[[StandardRepr 标准表示参数]]去掉 \(\nu\) 后的 K-限制，结构为 `KType { x: KgbId, lam_rho: Weight, height: u32 }`。其中 `lam_rho` 存储其 \((1-\theta_x)X^*\) 陪集经 `lambda_unique` 选出的规范代表，`height` 在构造时预计算。^[ktype.md:10-14]

## 构造入口与不变量

`KType::new` 是 `pub(crate)` 可见的原始构造器，不校验不变量。crate 内部可以通过它装入任意 `height`，正确性依赖调用方纪律；crate 外部只能通过 `sr_k` 构造。^[ktype.md:18-19]

`sr_k(rc, x, λ−ρ)` 先执行 `lambda_unique` 规范化，再预存 \((1+\theta)\lambda\) 的 height。其计算公式为 \((1+\theta)\lambda=\lambda_\rho+\theta\cdot\lambda_\rho+(1+\theta)\rho\)，`theta_plus_1_lambda` 使用同一公式。^[ktype.md:20-21]

来源在表示约定中称规范化“只在 `sr_k` 发生一次”，同时明确记载后续 `made_dominant` 会在每轮末尾调用 `lambda_unique` 重新规范化。因此，构造时的规范代表约定需要与后续变形中的重新规范化一并理解。^[ktype.md:11-14, ktype.md:35-38]

## 变形中的规范化与 height

`made_dominant` 要求输入为标准 K 型，否则报 `"standard K-type in make_dominant"`。它对负求值的复单根执行 cross 与反射，每轮末尾重新规范化，并原样携带 `height`；height 在 Weyl 共轭移动下不变是源码注释中的断言。终止预算为 `weight_defect((1+θ)λ)`，超限报 `"dominance termination"`，参见[[K 型变形与终止预算]]。^[ktype.md:35-38]

[[finals_for 带号重数展开|finals_for]] 中的 height 来源具有不对称性：工作栈中的新项经 `sr_k` 重新计算 height，结果项则通过 `KType::new(x, normalized, 沿用待处理项的 height)` 构造。这是来源记录的结构性阅读观察。^[ktype.md:45-53]

## 参数等价性

`equivalent` 先检查两个参数是否属于同一 Cartan 类，再分别调用 `to_canonical_fiber`，最后进行严格相等比较。典范纤维转换沿 `InnerClass::canonicalize` 给出的词执行 cross，每个生成元必须是复单根，否则触发 `"canonical fiber cross"` 错误。详见[[典范纤维与 K 型等价判定]]。^[ktype.md:31-31, ktype.md:41-42]

## 测试与证据边界

来源列出的测试锚点包括 split A1 中 `x=2`、权重 `[0]` 的 K 型：六个谓词全真、三个变形保持不动，以及模 \(2X^*\) 的相等性。另有通过 `sr_k_of_standard` 和 `sr_of_ktype` 实现的 `StandardRepr` 往返测试，以及 su(2,1) 中涉及 `x=4/5` 当选代表的非 final 锚点。^[ktype.md:65-70]

两个 su(2,1) 测试只有 `eprintln!`、没有断言，属于观察型用例，不构成机械锚点。未覆盖范围包括 `kgp_set` 整体、全部终止预算错误、`equivalent` 的异 Cartan 分支、`to_canonical_fiber` 的错误分支及各溢出和分配分支。详见[[K 型实现的测试锚点与证据边界]]。^[ktype.md:70-74]

本页依据 `ktype.rs` 的结构性阅读，不构成数学验收。来源记录了维护者对照源码逐条核对的过程；该次知识维护未执行 Atlas、Cargo、测试或 benchmark，上述测试描述不代表本次执行结果。^[ktype.md:9-14, ktype.md:83-87]

## Sources

- [ktype.md](../../sources/ktype.md) — K 型值与 RepContext 谓词/变形（ktype.rs）。
