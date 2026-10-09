---
title: finals_for 带号重数展开
summary: finals_for 用无显式终止计数的工作栈按根类型执行反射、Cayley 变换、墙投影和分裂，输出无序带号重数表，新任务与结果项的 height 来源不同。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:53.383Z"
updatedAt: "2026-10-09T20:59:11.497Z"
tags:
  - K型
  - 带号展开
  - Cayley变换
aliases:
  - finalsfor-带号重数展开
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: finals_for 带号重数展开
summary: finals_for 以工作栈按根类型、求值和下降状态展开 K 型，返回由语言层合并的无序带号重数表；实现没有显式终止计数，新任务与结果项的 height 来源不同。
sources:
  - ktype.md
kind: concept
tags:
  - 表示论
  - 展开算法
  - 重数
---

# finals_for 带号重数展开

`finals_for` 是 K 型层的带号重数展开过程，源包将其对应到上游 `K_repr.cpp:290–396`。它以工作栈驱动，根据根类型、求值和下降状态执行反射、Cayley 变换或墙投影，生成**无序的带号重数表**，合并由语言层负责。实现没有显式终止计数。^[ktype.md:45-51]

## 参数与求值

[[KType 表示参数与规范化构造|KType]] 表示标准表示参数去掉 ν 后的 K-限制，保存 `x: KgbId`、`lam_rho: Weight` 和预计算的 `height: u32`。`lam_rho` 存储其 `(1−θ_x)X*` 陪集经 `lambda_unique` 选出的代表；crate 外只能通过 `sr_k` 构造，由它完成规范化并计算 height。^[ktype.md:11-14, ktype.md:18-21]

分支使用的 `eval` 来自私有核 `theta_plus_1_eval(α)`，其表达式为 \(\langle\lambda_\rho,\alpha^\vee\rangle+\operatorname{colevel}(\alpha)+\langle\lambda_\rho,(\theta\alpha)^\vee\rangle+\operatorname{colevel}(\theta\alpha)\)。[[K 型谓词链与调用前提|谓词集]]使用这一求值的符号或零值。^[ktype.md:22-30]

## 按根类型展开

### 紧虚根 `ic`

当 `eval < 0` 时，执行双反射并将系数 `coef` 取负；当 `eval == 0` 时，因奇异紧因子而丢弃该项。^[ktype.md:45-47]

### 非紧虚根 `inc`

当 `eval < 0` 时，将 Cayley 像项压入工作栈；如果 cross 不动，即 type-2 情形，还压入 `λ_ρ + α` 的移位项。随后执行 cross，并将系数取负。^[ktype.md:47-49]

### 复根 `Complex`

当 `eval < 0` 或存在下降时，执行 cross 与反射。^[ktype.md:49-50]

### 实根 `Real`

当 \(\langle\lambda_\rho,\alpha^\vee\rangle\) 为奇数时，以 `shift = (eval + 1) / 2` 投影到墙，再按逆 Cayley 变换分裂。没有双值时只压入 `first`；若返回 `None`，则报错 `"parity real inverse Cayley"`。相关链接结构见 [[Cross、Cayley 与逆 Cayley 链接]]。^[ktype.md:50-51]

## 规范化与 height 传递

新任务与结果项的 height 来源不同：工作栈中的新项通过 `sr_k` 重算 height，而结果项通过 `KType::new(x, normalized, height)` 构造，沿用待处理项的 height。源包将这种不对称标记为源码阅读观察。^[ktype.md:52-53]

`simple_reflect` 的第三参数也区分两类权重：对 `im_wt` 使用 `0`，对 `lr` 使用 `1`。这一调用约定贯穿文件，参数语义位于 `RepContext` 侧。^[ktype.md:54-55]

## 测试与证据边界

源包记录了 `finals_for` 的三种测试形态：final 参数返回自身、奇异保留，以及负参数下降为两项 `x=0 [coef −1] + x=2 [coef +1]`。这些记录属于具体案例的测试锚点。^[ktype.md:65-67]

su(2,1) 相关的测试 8/9（`su21_deform_*`、`su21_finals_for_singular_gamma_zero`）只有 `eprintln!`，没有断言，属于观察型测试，不构成机械锚点。源包还列出各溢出与分配分支未覆盖；更多背景见 [[K 型实现的测试锚点与证据边界]]。^[ktype.md:68-74]

本页依据结构性源码阅读，不代表数学验收；上游行号仅转录自注释。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[ktype.md:13-14, ktype.md:83-87]

## Sources

- [ktype.md](ktype.md)：K 型值与 RepContext 谓词/变形（ktype.rs）。
