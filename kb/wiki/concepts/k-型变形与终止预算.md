---
title: K 型变形与终止预算
summary: made_dominant、made_theta_stable 和 normalised 通过交叉作用与反射消除负求值或复下降，分别使用权重缺陷、图大小及组合预算约束迭代。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:38.461Z"
updatedAt: "2026-10-10T00:39:47.477Z"
tags:
  - K型
  - 算法
  - 资源预算
aliases:
  - k-型变形与终止预算
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: K 型变形与终止预算
summary: made_dominant、made_theta_stable 与 normalised 分别以权重缺陷、图大小及组合预算限制变形迭代；展开与遍历另有控制机制。
sources:
  - ktype.md
kind: concept
tags:
  - K型
  - 变形算法
  - 资源预算
aliases:
  - k-型变形与终止预算
provenanceState: extracted
---

# K 型变形与终止预算

K 型的变形包括优势化、θ 稳定化和规范形构造，分别使用权重缺陷、图大小以及二者的组合限制迭代。`KType` 保存 KGB 元素 `x`、权重 `lam_rho` 和预计算的 `height`；crate 外通过 `sr_k` 构造，由 `lambda_unique` 选取 `(1−θ_x)X*` 陪集的规范代表。相关表示约定见 [[KType 表示参数与规范化构造]]。^[ktype.md:11-21, ktype.md:35-44]

## 优势化：`made_dominant`

`made_dominant` 要求输入为标准 K 型，非标准输入报错 `"standard K-type in make_dominant"`。算法对求值为负的复单根执行 cross 与反射，并在每轮末尾通过 `lambda_unique` 重新规范化。终止预算为 `weight_defect((1+θ)λ)`，超限时报错 `"dominance termination"`。^[ktype.md:35-38]

求值核 `theta_plus_1_eval(α)` 计算 \(\langle\lambda_\rho,\alpha^\vee\rangle+\operatorname{colevel}(\alpha)+\langle\lambda_\rho,(\theta\alpha)^\vee\rangle+\operatorname{colevel}(\theta\alpha)\)。`is_standard` 要求单虚余根上的求值非负，`is_dominant` 则要求所有单根上的求值非负；相关前提见 [[K 型谓词链与调用前提]]。^[ktype.md:22-30]

优势化原样携带 `height`，依据是源码注释所断言的 Weyl 共轭移动下 height 不变；这一不变性在来源中属于注释陈述。^[ktype.md:35-38]

## θ 稳定化：`made_theta_stable`

`made_theta_stable` 通过耗尽复下降完成变形，以图大小作为源码所称的“慷慨”终止界，对应的错误标识为 `"theta-stable termination"`。^[ktype.md:39-40]

## 典范纤维与规范形

`to_canonical_fiber` 沿 `InnerClass::canonicalize` 给出的词逐步执行 cross，要求每个生成元都是复单根，否则报错 `"canonical fiber cross"`。K 型等价判定先要求同属一个 Cartan 类，再比较各自进入典范纤维后的结果，详见 [[典范纤维与 K 型等价判定]]。^[ktype.md:31-31, ktype.md:41-42]

`normalised` 先进行典范化，再耗尽奇异复下降与负复求值。其终止预算为 `weight_defect + 图大小 + 1`，对应的错误标识为 `"normal form termination"`。^[ktype.md:43-44]

## 展开与遍历的控制机制

[[finals_for 带号重数展开]] 由工作栈驱动，没有显式终止计数。其 height 来源存在不对称：新入栈项经 `sr_k` 重算 height，而结果项通过 `KType::new` 构造，沿用待处理项的 height。^[ktype.md:45-53]

[[kgp_set 的 Levi 生成元遍历]] 先调用 `made_theta_stable`，再以 θ 稳定元素的实单根作为 Levi 生成元进行 BFS；无法映射回生成元下标的根会被静默跳过。BFS 由 `present` 位图限界，每个 KGB 元素至多入队一次。Real 分支计算 `shift = eval/2` 时不检查奇偶，final/semifinal 前提由调用方负责，源码注释称 wrapper 会先检查。^[ktype.md:56-61]

## 测试与证据边界

split A1 的冻结契约测试包含三个变形均保持输入不动的锚点，但全部终止预算错误分支、`to_canonical_fiber` 的错误分支和 `kgp_set` 整体均未覆盖。两个 su(2,1) 测试只有 `eprintln!`、没有断言，属于观察型测试，不构成机械验证锚点。参见 [[K 型实现的测试锚点与证据边界]]。^[ktype.md:65-74]

本页依据对 `ktype.rs` 的结构性阅读，不构成数学验收。来源记录的知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试覆盖描述不代表本次执行结果。^[ktype.md:9-14, ktype.md:83-87]

## Sources

- [ktype.md](../../sources/ktype.md)：K 型值与 RepContext 谓词/变形（ktype.rs）。
