---
title: 限制根的可乘性（is_multipliable）
summary: 通过二倍限制权类是否仍为限制根判定可乘性；来源列有 A2 可乘测试锚点，但未验收其与 BC 型非约化根系的数学关系。
sources:
  - layout-restricted-roots.md
kind: concept
createdAt: "2026-10-09T14:58:23.412Z"
updatedAt: "2026-10-09T22:37:29.714Z"
tags:
  - 限制根系
  - 可乘性
  - 证据边界
aliases:
  - 限制根的可乘性ismultipliable
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 限制根的可乘性（is_multipliable）
summary: 通过检查二倍限制权类是否仍为限制根判定可乘性；测试覆盖 split A1 不可乘与特定 A2 对合下可乘的实例，尚未验收与 BC 型非约化根系的数学关系。
sources:
  - layout-restricted-roots.md
kind: concept
tags:
  - 限制根系
  - 可乘性
  - 测试边界
aliases:
  - 限制根的可乘性ismultipliable
---

# 限制根的可乘性（is_multipliable）

`RestrictedRootSystem::is_multipliable(weight)` 判定给定限制权的二倍类是否也是限制根的类。该判定关注二倍类的成员关系；纤维重数（multiplicity）则是另一项信息，来源中的测试分别检查二者。^[layout-restricted-roots.md:57-66]

## 商类表示与加倍

`RestrictedWeight` 表示商格 \(X^*/\ker(1-\theta)\) 中的不透明元素，以 \((1-\theta)(weight)\) 的坐标编码等价类。这是商格到像格的单射表示，不能将编码直接解释为原来的环境权坐标。例如，split A1 中 \(\alpha\) 的类编码为 \(2\alpha\)，但它并不等于 \(2\alpha\) 的类。^[layout-restricted-roots.md:50-53]

`restrict` 通过逐坐标 `checked_sub` 计算限制编码；私有方法 `doubled` 仅供 `is_multipliable` 使用，通过逐坐标 `checked_mul(2)` 加倍编码。因此，可乘性检查针对的是二倍商类是否仍属于限制根集合。^[layout-restricted-roots.md:53-62]

## 限制根集合与查询

`RestrictedRootSystem::build` 先检查 datum 一致性与格秩，不匹配时分别产生 `DatumMismatch` 与 `RankMismatch`。随后按根系枚举序遍历，跳过限制为零的根，将其余根按 `RestrictedWeight` 键聚合成纤维。限制根由 `BTreeMap` 收集，按编码坐标的字典序升序排列；`root(weight)` 使用二分查找。^[layout-restricted-roots.md:57-62]

系统的 `rank` 取自对合的 `anti_invariant_rank`，即 \(-1\) 特征空间的秩，而非纤维数量，参见 [[对合的反不变秩]]。限制根模块使用 `RootInvolutionData::involution()`，与内类布局模块互不调用；两者的接口差异见 [[内类布局与限制根系的接口边界]]。^[layout-restricted-roots.md:59-60, layout-restricted-roots.md:68-72]

## 测试锚点

split A1 的测试给出秩为 1、两个限制根，且 \(\alpha\) 的纤维重数为 1、不可乘。compact A1 的测试确认没有限制根。^[layout-restricted-roots.md:64-66]

A2 在对合矩阵 \(\begin{pmatrix}0&-1\\-1&0\end{pmatrix}\) 下的测试给出秩为 1，且 \(\lambda\) 的纤维重数为 2、可乘。这是来源明确记录的可乘实例。^[layout-restricted-roots.md:64-66]

## 证据边界

来源属于结构性源码阅读，不构成数学或正确性验收。错误分支没有失败路径测试，`rank` 与限制根条数的一致性未作断言；`is_multipliable` 与 BC 型非约化根系的关系等数学性质，也未在该文件中断言或由源包验收。^[layout-restricted-roots.md:79-85]

源包中的上游引用仅转录自代码注释，未核对上游字节；此次知识维护未执行 Atlas、Cargo、测试或 benchmark。因此，上述测试锚点是源码中的覆盖记录，并非此次执行结果。^[layout-restricted-roots.md:10-13, layout-restricted-roots.md:89-93]

## Sources

- [内类布局与限制根系（layout.rs / restricted_roots.rs）](../../sources/layout-restricted-roots.md)
