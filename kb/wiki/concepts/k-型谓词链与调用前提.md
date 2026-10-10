---
title: K 型谓词链与调用前提
summary: 标准性、支配性、非零性、半终结性、正规性和终结性由根配对及 KGB 状态判定，部分前提依赖调用方而不在谓词内检查。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:26.038Z"
updatedAt: "2026-10-10T00:39:46.825Z"
tags:
  - K型
  - 谓词
  - 调用契约
aliases:
  - k-型谓词链与调用前提
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: K 型谓词链与调用前提
summary: K 型的六个谓词通过根配对与 KGB 状态判定标准性、支配性、非零性、半终结性、正规性和终结性；部分调用前提由调用方保证。
sources:
  - ktype.md
kind: concept
tags:
  - K型
  - 根系
  - 调用契约
aliases:
  - k-型谓词链与调用前提
---

# K 型谓词链与调用前提

K 型是[[StandardRepr 标准表示参数]]去掉 ν 后的 K-限制，表示为 `KType { x: KgbId, lam_rho: Weight, height: u32 }`。`lam_rho` 存储其 \((1-\theta_x)X^*\) 陪集中由 `lambda_unique` 选出的规范代表，`height` 在构造时预计算。六个谓词检查不同性质，其中部分调用前提不在谓词内部验证。^[ktype.md:11-14, ktype.md:24-30]

## 构造与共同求值核

crate 外只能通过 `sr_k(rc, x, λ−ρ)` 构造 K 型。该入口执行 `lambda_unique` 规范化，并预存 \((1+\theta)\lambda\) 的 height，其中 \((1+\theta)\lambda=\lambda_\rho+\theta\lambda_\rho+(1+\theta)\rho\)。crate 内的原始构造器 `new` 不校验不变量，可以装入任意 height，构造纪律依赖调用方。参见[[KType 表示参数与规范化构造]]。^[ktype.md:18-21]

私有求值核 `theta_plus_1_eval(α)` 计算 \(\langle\lambda_\rho,\alpha^\vee\rangle+\operatorname{colevel}(\alpha)+\langle\lambda_\rho,(\theta\alpha)^\vee\rangle+\operatorname{colevel}(\theta\alpha)\)。谓词集对这一求值只使用符号或零测试。^[ktype.md:22-23]

## 六个谓词及其前提

`is_standard` 要求单虚余根上的求值非负；`is_dominant` 要求所有单根的 `theta_plus_1_eval` 非负。两者的检查范围不同。^[ktype.md:24-25]

`is_nonzero` 检查不存在奇异紧单虚根。它**假设 `is_standard` 成立，但不自行检查**，标准性由调用方保证。^[ktype.md:25-26]

`is_semifinal` 检查权重 \(2\lambda_\rho+2\rho-2\rho_R\) 在实单根上的配对是否同余于 \(0\pmod 4\)。^[ktype.md:26-27]

`is_normal` 检查不存在奇异复下降。上游断言四联前提；Rust 移植的注释说明，由于计算是 total，入口不检查这些前提。源包未列出四联前提的具体组成。^[ktype.md:27-28]

`is_final` 遇到 `eval < 0` 即拒绝；当 `eval == 0` 时，按 KGB 状态分派：`ic` 拒绝，Real 的奇配对情形拒绝，Complex 下降拒绝，`inc` 放行。相关状态见[[KGB 生成元状态与下降判定]]。^[ktype.md:29-30]

## 变形入口的调用契约

`made_dominant` 显式检查标准性，非标准输入报错 `"standard K-type in make_dominant"`。随后对负求值的复单根执行 cross 与反射，每轮末尾重新执行 `lambda_unique` 规范化。其终止预算为 `weight_defect((1+θ)λ)`，超限报错 `"dominance termination"`；height 按注释所述的 Weyl 共轭不变性原样携带。参见[[K 型变形与终止预算]]。^[ktype.md:35-38]

`kgp_set` 先执行 `made_theta_stable`，其 Real 分支使用 `shift = eval/2`，**不检查奇偶性**。final/semifinal 前提由调用方负责，源码注释称 wrapper 会先检查。参见[[kgp_set 的 Levi 生成元遍历]]。^[ktype.md:56-61]

## 测试与证据边界

split A1 的冻结契约锚点包含 `x=2`、参数 `[0]` 的 K 型，六个谓词全部为真。源包还记录了 su(2,1) 的非 final 锚点，但测试 8/9（`su21_deform_*`、`su21_finals_for_singular_gamma_zero`）仅有 `eprintln!`，没有断言，属于观察型测试，不构成机械验证锚点。^[ktype.md:65-72]

现有测试未覆盖 `kgp_set` 整体、全部终止预算错误、`equivalent` 的异 Cartan 分支、`to_canonical_fiber` 的错误分支，以及各溢出和分配分支。参见[[K 型实现的测试锚点与证据边界]]。^[ktype.md:72-74]

本页依据结构性源码阅读，不声称数学验收。源包经过维护者对照源码逐条核对改写；该次知识维护未执行 Atlas、Cargo、测试或 benchmark，上述测试描述不代表本次执行结果。^[ktype.md:9-14, ktype.md:83-87]

## Sources

- [ktype.md](../../sources/ktype.md)：《K 型值与 RepContext 谓词/变形（ktype.rs）》。
