---
title: finals_for 带号重数展开
summary: finals_for 用工作栈按根类型与求值执行反射、Cayley 变换、墙投影和分裂，生成无序带号重数表；它没有显式终止计数，且新任务与结果项的 height 来源不同。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:53.383Z"
updatedAt: "2026-10-09T14:56:53.383Z"
tags:
  - 表示论
  - 展开算法
  - 重数
aliases:
  - finalsfor-带号重数展开
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# finals_for 带号重数展开

`finals_for` 是 K 型层的带号重数展开过程，对应上游 `K_repr.cpp:290–396`。它由工作栈驱动，按根类型、求值和下降状态变换待处理项，返回**无序的带号重数表**；合并由语言层负责，算法没有显式终止计数。^[ktype.md:45-51]

## 参数与求值

[[KType 表示参数与规范化构造|KType]] 表示标准表示参数去掉 ν 后的 K-限制，保存 `x: KgbId`、`lam_rho: Weight` 和预计算的 `height: u32`。存储的 `lam_rho` 是其 `(1−θ_x)X*` 陪集经 `lambda_unique` 选出的规范代表；对外构造入口 `sr_k` 负责规范化并计算 height。^[ktype.md:11-14, ktype.md:18-21]

分支中的 `eval` 来自 `theta_plus_1_eval(α)`，其公式为
\[
\langle\lambda_\rho,\alpha^\vee\rangle+\operatorname{colevel}(\alpha)
+\langle\lambda_\rho,(\theta\alpha)^\vee\rangle+\operatorname{colevel}(\theta\alpha).
\]
谓词集使用这一求值的符号或零值。相关前提见 [[K 型谓词链与调用前提]]。^[ktype.md:22-30]

## 展开分支

对于紧虚根状态 `ic`，若 `eval < 0`，执行双反射并将系数 `coef` 取负；若 `eval == 0`，则因奇异紧因子而丢弃该项。^[ktype.md:45-47]

对于非紧虚根状态 `inc`，若 `eval < 0`，先将 Cayley 像项压入工作栈；若 cross 不动，即 type-2 情形，还会压入 `λ_ρ + α` 的移位项。随后执行 cross，并将系数取负。^[ktype.md:47-49]

对于复根状态 `Complex`，若 `eval < 0` 或存在下降，则执行 cross 与反射。^[ktype.md:49-49]

对于实根状态 `Real`，若 \(\langle\lambda_\rho,\alpha^\vee\rangle\) 为奇数，则以 `shift = (eval + 1) / 2` 投影到墙，再按逆 Cayley 变换分裂。若无双值，只压入 `first`；若返回 `None`，则报错 `"parity real inverse Cayley"`。相关链接结构见 [[Cross、Cayley 与逆 Cayley 链接]]。^[ktype.md:50-51]

## 规范化与 height 的来源

工作栈新项与结果项的 height 来源不同：新压入的待处理项通过 `sr_k` 重算 height；结果项则通过 `KType::new(x, normalized, height)` 构造，沿用待处理项的 height。这是源码阅读记录的实现差异。^[ktype.md:52-53]

反射调用还区分两类权重：`simple_reflect` 对 `im_wt` 使用第三参数 `0`，对 `lr` 使用第三参数 `1`；这一约定贯穿文件，参数语义由 `RepContext` 定义。^[ktype.md:54-55]

## 测试与证据边界

测试记录覆盖 `finals_for` 的三种形态：final 参数返回自身、奇异保留，以及负参数下降为两项 `x=0 [coef −1] + x=2 [coef +1]`。这些是具体案例锚点，不能据此推定全部分支都已验证。^[ktype.md:65-67]

su(2,1) 相关测试中的 `su21_deform_*` 与 `su21_finals_for_singular_gamma_zero` 只有 `eprintln!`，没有断言，属于观察型测试。源包还记录了溢出与分配分支等覆盖缺口；完整背景见 [[K 型实现的测试锚点与证据边界]]。^[ktype.md:68-74]

本页依据结构性源码阅读，不代表数学验收；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[ktype.md:13-14, ktype.md:83-87]

## Sources

- [ktype.md](ktype.md)：K 型值与 RepContext 谓词/变形（ktype.rs）。
