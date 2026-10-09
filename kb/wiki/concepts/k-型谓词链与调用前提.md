---
title: K 型谓词链与调用前提
summary: 标准性、支配性、非零性、半终结性、正规性和终结性由根配对及 KGB 状态判定，其中部分前提依赖调用方而不在谓词内检查。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:26.038Z"
updatedAt: "2026-10-09T14:56:26.038Z"
tags:
  - 表示论
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# K 型谓词链与调用前提

K 型是[[StandardRepr 标准表示参数]]去掉 ν 后的 K-限制，表示为 `KType { x: KgbId, lam_rho: Weight, height: u32 }`。其存储约定是：`lam_rho` 为 `(1−θ_x)X*` 陪集中由 `lambda_unique` 选出的规范代表，`height` 在构造时预计算。谓词依赖这些表示约定，但并非每个入口都会检查调用前提。^[ktype.md:11-14, ktype.md:18-30]

## 构造与共同求值核

crate 外通过 `sr_k(rc, x, λ−ρ)` 构造 K 型：它执行 `lambda_unique` 规范化，并预存 `(1+θ)λ` 的 height，其中 `(1+θ)λ = λ_ρ + θ·λ_ρ + (1+θ)ρ`。crate 内的原始构造器 `new` 不校验不变量，可以装入任意 height，因此内部调用方仍须遵守构造纪律。参见[[KType 表示参数与规范化构造]]。^[ktype.md:18-21]

谓词共享私有求值核 `theta_plus_1_eval(α)`，其值为
\[
\langle \lambda_\rho,\alpha^\vee\rangle+\operatorname{colevel}(\alpha)
+\langle \lambda_\rho,(\theta\alpha)^\vee\rangle+\operatorname{colevel}(\theta\alpha).
\]
谓词集只使用该值的符号或零测试。^[ktype.md:22-23]

## 各谓词的判定与前提

`is_standard` 要求单虚余根上的求值非负；`is_dominant` 则要求所有单根的 `theta_plus_1_eval` 非负。二者的检查范围不同。^[ktype.md:24-25]

`is_nonzero` 检查不存在奇异紧单虚根。它**假设 `is_standard` 已成立，却不自行检查**；调用方必须保证这一前提。^[ktype.md:25-26]

`is_semifinal` 检查权重 `2λ_ρ + 2ρ − 2ρ_R` 与实单根的配对是否满足模 4 同余于 0。^[ktype.md:26-27]

`is_normal` 检查不存在奇异复下降。上游对此断言四联前提；Rust 移植的注释说明，由于计算是 total，入口不检查这些前提。源包没有列出四联前提的具体组成。^[ktype.md:27-28]

`is_final` 首先拒绝求值小于 0 的情形；求值等于 0 时，按 KGB 状态分派：拒绝 `ic`，拒绝 Real 的奇配对情形，拒绝 Complex 下降，而允许 `inc`。相关状态可参见[[KGB 生成元状态与下降判定]]。^[ktype.md:29-30]

## 变形入口如何处理前提

`made_dominant` 会显式拒绝非标准输入，报错 `"standard K-type in make_dominant"`。随后，它对负求值的复单根执行 cross 与反射，每轮末尾重新进行 `lambda_unique` 规范化；终止预算为 `weight_defect((1+θ)λ)`，超限报错 `"dominance termination"`。参见[[K 型变形与终止预算]]。^[ktype.md:35-38]

`kgp_set` 的 Real 分支使用 `shift = eval/2`，**不检查奇偶性**。其 final/semifinal 前提由调用方负责；源码注释称 wrapper 会先检查。因此，不能把该分支的整数除法视为对输入前提的验证。参见[[kgp_set 的 Levi 生成元遍历]]。^[ktype.md:56-61]

## 测试与证据边界

split A1 的冻结契约锚点包含 `x=2`、参数 `[0]` 的 K 型，验证六个谓词全部为真。源包也记录了 su(2,1) 的非 final 锚点，但其中两个 `su21` 测试仅有 `eprintln!`、没有断言，属于观察型测试，不能作为机械验证锚点。^[ktype.md:65-72]

这些材料来自结构性源码阅读，不声称数学验收。测试尚未覆盖 `kgp_set` 整体、全部终止预算错误及若干错误、溢出和分配分支；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。参见[[K 型实现的测试锚点与证据边界]]。^[ktype.md:13-14, ktype.md:72-74, ktype.md:83-87]

## Sources

- [ktype.md](ktype.md)：《K 型值与 RepContext 谓词/变形（ktype.rs）》
