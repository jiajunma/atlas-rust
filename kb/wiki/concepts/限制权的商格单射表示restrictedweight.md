---
title: 限制权的商格单射表示（RestrictedWeight）
summary: 以 (1−θ)(weight) 编码 X*/ker(1−θ) 的等价类，坐标属于像格表示；split A1 中 α 类编码为 2α，并不表示它等于 2α 的类。
sources:
  - layout-restricted-roots.md
kind: concept
createdAt: "2026-10-09T14:58:02.607Z"
updatedAt: "2026-10-09T14:58:02.607Z"
tags:
  - 限制权
  - 商格
  - Rust设计
aliases:
  - 限制权的商格单射表示restrictedweight
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 限制权的商格单射表示（RestrictedWeight）

`RestrictedWeight` 是商格 \(X^*/\ker(1-\theta)\) 中元素的不透明表示，以私有 `Vec<i32>` 保存坐标。它通过 \((1-\theta)(weight)\) 编码权的等价类；这些坐标表示商格到像格的单射，而不是原始权的环境坐标。^[layout-restricted-roots.md:48-55]

## 商类与编码坐标

编码坐标必须与其代表的商类区分。例如，在 split A1 中，`alpha` 的类编码为 `2alpha`，但这不意味着它等同于 `2alpha` 的类。这里的倍增来自 \(1-\theta\) 的编码作用，不能据此将编码结果直接解释为原始权的商类。^[layout-restricted-roots.md:50-55]

`restrict` 使用逐坐标 `checked_sub` 计算限制；私有方法 `doubled` 使用逐坐标 `checked_mul(2)`，仅供 `is_multipliable` 使用。两种操作都采用受检查的整数算术。^[layout-restricted-roots.md:53-55]

## 排序、纤维与查询

`RestrictedWeight` 派生 `Ord` 和 `Hash`，可作为 `BTreeMap` 的键。`RestrictedRootSystem::build` 在校验 datum 一致性与格秩后，按根系枚举顺序遍历根，跳过限制为零的根，并将限制权相同的根聚合成纤维。收集得到的限制根按编码坐标的字典序升序排列，`root(weight)` 据此进行二分查找。^[layout-restricted-roots.md:50-62]

限制根系的 `rank` 取自对合的 `anti_invariant_rank`，即 \(-1\) 特征空间的秩，而不是纤维数量；相关背景见[[对合的反不变秩]]。`is_multipliable(weight)` 则检查该权的二倍类本身是否也是一个限制根的类。^[layout-restricted-roots.md:57-62]

## 测试锚点与证据边界

现有测试覆盖三种情形：split A1 的秩为 1，有两个限制根，`alpha` 纤维的 multiplicity 为 1 且不可乘；compact A1 没有限制根；A2 在对合矩阵 \(\begin{pmatrix}0&-1\\-1&0\end{pmatrix}\) 下秩为 1，`lambda` 纤维的 multiplicity 为 2 且可乘。^[layout-restricted-roots.md:64-66]

这些记录属于结构性阅读与测试锚点说明，不构成数学或正确性验收。错误分支没有失败路径测试；`RestrictedRootSystem::rank` 与限制根条数的一致性未作断言，`is_multipliable` 与 BC 型非约化根系等数学性质的关系也未在该文件中断言。本次知识维护未执行测试或 benchmark。^[layout-restricted-roots.md:77-85, layout-restricted-roots.md:89-93]

## Sources

- [内类布局与限制根系（layout.rs / restricted_roots.rs）](layout-restricted-roots.md)
