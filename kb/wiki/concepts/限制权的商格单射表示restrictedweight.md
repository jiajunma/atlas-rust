---
title: 限制权的商格单射表示（RestrictedWeight）
summary: 用 (1−θ)(weight) 编码 X*/ker(1−θ) 的等价类，坐标属于像格表示，限制与倍增分别使用受检减法和乘法。
sources:
  - layout-restricted-roots.md
kind: concept
createdAt: "2026-10-09T14:58:02.607Z"
updatedAt: "2026-10-09T21:00:09.786Z"
tags:
  - 限制权
  - 商格
  - 整数算术
aliases:
  - 限制权的商格单射表示restrictedweight
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 限制权的商格单射表示（RestrictedWeight）
summary: RestrictedWeight 以 (1−θ)(weight) 编码商格 X*/ker(1−θ) 的元素；编码坐标属于像格表示，不能直接解释为原始权的商类。
sources:
  - layout-restricted-roots.md
kind: concept
tags:
  - 限制权
  - 商格
  - Rust设计
aliases:
  - 限制权的商格单射表示restrictedweight
---

# 限制权的商格单射表示（RestrictedWeight）

`RestrictedWeight` 是商格 \(X^*/\ker(1-\theta)\) 中元素的不透明表示，以私有 `Vec<i32>` 保存坐标。它通过 \((1-\theta)(weight)\) 编码权的等价类：这些坐标给出商格到像格的单射表示，而不是原始权的环境坐标。^[layout-restricted-roots.md:48-55]

## 商类与编码坐标

必须区分商类与其编码。例如，在 split A1 中，`alpha` 的类编码为 `2alpha`，但并不等于 `2alpha` 的类。这里的倍增来自 \(1-\theta\) 的编码作用，不能把编码坐标直接当作原始权的商类。^[layout-restricted-roots.md:50-55]

`restrict` 通过逐坐标 `checked_sub` 计算限制；私有方法 `doubled` 通过逐坐标 `checked_mul(2)` 计算倍增，仅供 `is_multipliable` 使用。两种操作都采用受检查的整数算术。^[layout-restricted-roots.md:53-55]

## 纤维聚合与有序查询

`RestrictedWeight` 派生 `Ord` 和 `Hash`，可作为 `BTreeMap` 的键。`RestrictedRootSystem::build` 先检查 datum 一致性与格秩，分别对应 `DatumMismatch` 和 `RankMismatch`；随后按根系枚举顺序遍历，跳过限制为零的根，将限制权相同的根聚合成纤维。^[layout-restricted-roots.md:50-60]

限制根从 `BTreeMap` 收集，因此按编码坐标的字典序升序排列；`root(weight)` 使用二分查找。相关机制见 [[限制根系的纤维聚合与有序查询]]。^[layout-restricted-roots.md:57-62]

限制根系的 `rank` 取自对合的 `anti_invariant_rank`，即 \(-1\) 特征空间的秩，而非纤维数量，参见 [[对合的反不变秩]]。`is_multipliable(weight)` 则判断该限制权的二倍类本身是否也是限制根的类，参见 [[限制根的可乘性（is_multipliable）]]。^[layout-restricted-roots.md:59-62]

## 测试锚点与证据边界

现有测试包含三个锚点：split A1 的秩为 1，有两个限制根，`alpha` 纤维的重数为 1 且不可乘；compact A1 没有限制根；A2 在对合矩阵 \(\begin{pmatrix}0&-1\\-1&0\end{pmatrix}\) 下秩为 1，`lambda` 纤维的重数为 2 且可乘。^[layout-restricted-roots.md:64-66]

来源属于结构性源码阅读，不构成数学或正确性验收，上游引用也未核对。错误分支没有失败路径测试；`rank` 与限制根条数的一致性未作断言，`is_multipliable` 与 BC 型非约化根系的关系等数学性质也未在该文件中断言。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[layout-restricted-roots.md:9-13, layout-restricted-roots.md:77-85, layout-restricted-roots.md:89-93]

## Sources

- [内类布局与限制根系（layout.rs / restricted_roots.rs）](layout-restricted-roots.md)
