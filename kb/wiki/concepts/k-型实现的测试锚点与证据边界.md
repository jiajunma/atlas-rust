---
title: K 型实现的测试锚点与证据边界
summary: 来源记录九项测试，其中两个 su(2,1) 测试仅打印而无断言；kgp_set 和多类错误路径缺少覆盖，本包未执行测试或完成数学验收。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:58.026Z"
updatedAt: "2026-10-10T00:40:13.848Z"
tags:
  - K型
  - 测试覆盖
  - 证据边界
aliases:
  - k-型实现的测试锚点与证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: K 型实现的测试锚点与证据边界
summary: 九项测试涉及 split A1 契约、带号展开和表示往返；两个 su(2,1) 测试仅打印观察值，kgp_set 及多类错误分支未覆盖。结构性阅读不构成测试通过或数学验收证据。
sources:
  - ktype.md
kind: concept
tags:
  - K型
  - 测试覆盖
  - 证据边界
---

# K 型实现的测试锚点与证据边界

`ktype.rs` 的来源材料记录了 9 个测试，涉及 K 型谓词、变形、展开和表示参数往返，其中两个测试只有打印输出、没有断言。材料属于结构性源码阅读，不声称数学验收；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[ktype.md:9-14, ktype.md:65-74, ktype.md:83-87]

## 回归测试锚点

split A1 的冻结契约锚点使用 `x=2`、权重为 `[0]` 的 K 型，检查六个谓词全真、三个变形均不改变参数，以及模 \(2X^*\) 的相等性。相关表示与调用前提见 [[KType 表示参数与规范化构造]] 和 [[K 型谓词链与调用前提]]。^[ktype.md:65-66]

[[finals_for 带号重数展开]] 的测试涉及三种形态：final 参数返回自身、奇异情形保留，以及负参数下降为两项 `x=0 [coef −1] + x=2 [coef +1]`。实现返回无序带号重数表，合并发生在语言层，因此这里的书写次序不表示返回顺序。^[ktype.md:45-51, ktype.md:66-67]

其他锚点包括 `reducibility_points` 的两个空结果，以及 [[StandardRepr 标准表示参数]] 的往返转换 `sr ↔ sr_k_of_standard/sr_of_ktype`。^[ktype.md:67-68]

su(2,1) 的非 final 锚点涉及 `x=4/5` 的当选代表。来源转录的注释记录：`lambda_unique` 在 release build 中不记录主元取负，`x=4` 的当选基为 `(2,−1), [1,0]`。这些细节限定了该案例中的代表元背景。^[ktype.md:68-70]

## 观察型测试

来源将测试 8/9，即 `su21_deform_*`、`su21_finals_for_singular_gamma_zero`，明确标为观察型：它们只有 `eprintln!`，没有断言，不构成机械锚点。^[ktype.md:70-72]

## 未覆盖路径与调用前提

来源明确列出的覆盖缺口包括：[[kgp_set 的 Levi 生成元遍历]] 整体、全部终止预算错误、`equivalent` 的异 Cartan 分支、`to_canonical_fiber` 的错误分支，以及各溢出和分配分支。^[ktype.md:72-74]

[[K 型变形与终止预算]] 中，`made_dominant` 的预算为 `weight_defect((1+θ)λ)`，`made_theta_stable` 使用图大小作为终止界，`normalised` 的预算为 `weight_defect + 图大小 + 1`。这些限制已有源码描述，但对应的终止预算错误未被本包所列测试覆盖。^[ktype.md:35-44, ktype.md:72-74]

[[典范纤维与 K 型等价判定]] 中，`equivalent` 先检查是否属于同一 Cartan 类，再分别转到典范纤维后严格比较；`to_canonical_fiber` 要求沿规范化词经过的每个生成元都是复单根，否则报 `"canonical fiber cross"`。异 Cartan 分支与该转换的错误分支均属于已列明的覆盖缺口。^[ktype.md:31-31, ktype.md:41-42, ktype.md:72-74]

`kgp_set` 的实根分支使用 `shift = eval/2`，自身不检查奇偶性；final/semifinal 前提由调用方负责，注释称 wrapper 会先检查。该函数整体仍在未覆盖清单中。^[ktype.md:56-61, ktype.md:72-74]

## 来源身份与验收边界

本包在未改变的源码字节上取代 2026-10-03 的初读包，沿用同一 SHA-256 并保留旧快照；补充内容包括 `finals_for` 分支结构、`kgp_set`、测试锚点普查和错误分支普查。读取身份记录于 `2026-10-06-ktype.json`，绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录，草案经维护者对照源码逐条核对改写。^[ktype.md:78-87]

本页描述源码中已有测试及其覆盖限制。来源明确限定为结构性阅读，并注明本次维护未执行测试或 benchmark，因此不提供本次测试通过、性能验证或数学验收的结论。^[ktype.md:13-14, ktype.md:83-87]

## Sources

- [ktype.md](../../sources/ktype.md) — K 型值与 RepContext 谓词/变形（ktype.rs）。
