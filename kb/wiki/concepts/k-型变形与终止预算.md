---
title: K 型变形与终止预算
summary: made_dominant、made_theta_stable 和 normalised 通过 cross、反射及下降消除实现变形，并分别用权重缺陷、图大小或组合预算限制迭代。
sources:
  - ktype.md
kind: concept
createdAt: "2026-10-09T14:56:38.461Z"
updatedAt: "2026-10-09T14:56:38.461Z"
tags:
  - 算法
  - 规范化
  - 终止性
aliases:
  - k-型变形与终止预算
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# K 型变形与终止预算

K 型的变形操作围绕优势化、θ 稳定化和规范形构造展开。`KType` 保存 KGB 元素 `x`、权重 `lam_rho` 与预计算的 `height`；`sr_k` 通过 `lambda_unique` 选取 `(1−θ_x)X*` 陪集的规范代表，并计算 `(1+θ)λ` 的 height。这些表示约定是理解变形中权重更新与 height 传递的基础，参见 [[KType 表示参数与规范化构造]]。^[ktype.md:11-14, ktype.md:18-21]

## 优势化：`made_dominant`

`made_dominant` 要求输入为标准 K 型；非标准输入报错 `"standard K-type in make_dominant"`。算法对求值为负的复单根执行 cross 与反射，并在每轮末尾通过 `lambda_unique` 重新规范化。其终止预算为 `weight_defect((1+θ)λ)`，超限时报错 `"dominance termination"`。^[ktype.md:35-38]

该操作原样携带 `height`，依据是源码注释所断言的 Weyl 共轭移动下 height 不变。这里的求值使用 `(1+θ)λ` 与根的配对信息；相关符号测试和调用前提见 [[K 型谓词链与调用前提]]。^[ktype.md:20-30, ktype.md:35-38]

## θ 稳定化：`made_theta_stable`

`made_theta_stable` 通过耗尽复下降完成变形。它以图大小作为源码所称的“慷慨”终止界，超限时报错 `"theta-stable termination"`。^[ktype.md:39-40]

## 典范纤维与规范形

`to_canonical_fiber` 沿 `InnerClass::canonicalize` 给出的词逐步执行 cross，要求每个生成元都是复单根；违反这一要求时报错 `"canonical fiber cross"`。该检查约束变形路径上的根类型，相关内容见 [[典范纤维与 K 型等价判定]]。^[ktype.md:41-42]

`normalised` 先进行典范化，再耗尽奇异复下降与负复求值。其终止预算为 `weight_defect + 图大小 + 1`，超限时报错 `"normal form termination"`。^[ktype.md:43-44]

## 其他展开与遍历的终止控制

[[finals_for 带号重数展开]] 使用工作栈驱动，没有显式终止计数。因此，不能把上述变形操作的预算机制套用到 `finals_for`。其 height 处理也存在来源不对称：新入栈项经 `sr_k` 重算 height，而结果项使用 `KType::new`，沿用待处理项的 height。^[ktype.md:45-53]

[[kgp_set 的 Levi 生成元遍历]] 先调用 `made_theta_stable`，随后以 θ 稳定元素的实单根作为 Levi 生成元进行 BFS。遍历由 `present` 位图限界，每个 KGB 元素至多入队一次；Real 分支的 `shift = eval/2` 不检查奇偶，final/semifinal 前提由调用方负责。^[ktype.md:56-61]

## 测试与证据边界

现有 split A1 冻结契约锚点包含三个变形均保持输入不动的检查，但全部终止预算错误分支、`to_canonical_fiber` 的错误分支以及 `kgp_set` 整体均未覆盖。两个 su(2,1) 测试只有 `eprintln!`、没有断言，属于观察型测试，不能作为机械验证锚点。参见 [[K 型实现的测试锚点与证据边界]]。^[ktype.md:65-74]

本页依据的是对 `ktype.rs` 的结构性阅读，不构成数学验收；此次知识维护未执行 Atlas、Cargo、测试或 benchmark。因此，预算与错误行为属于源码阅读结论，不能据此宣称已经获得运行验证。^[ktype.md:10-14, ktype.md:83-87]

## Sources

- [ktype.md](ktype.md)：K 型值与 RepContext 谓词/变形（ktype.rs）
