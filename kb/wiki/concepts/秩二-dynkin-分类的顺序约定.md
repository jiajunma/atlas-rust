---
title: 秩二 Dynkin 分类的顺序约定
summary: 秩二按非对角条目乘积分型，B2/C2 标签由给定顶点顺序决定且不换序，G2 则按条目方向调整为短根在前。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:26.234Z"
updatedAt: "2026-10-09T20:50:43.536Z"
tags:
  - Dynkin分类
  - 编号约定
  - 兼容性
aliases:
  - 秩二-dynkin-分类的顺序约定
  - 秩D分
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 秩二 Dynkin 分类的顺序约定
summary: 秩二分类按非对角 Cartan 条目的乘积分型；B2/C2 标签由给定顶点顺序决定且保序，G2 则按条目方向决定是否交换顶点，使短根在前。
sources:
  - dynkin.md
kind: concept
tags:
  - Dynkin分类
  - 编号约定
  - 兼容性
---

# 秩二 Dynkin 分类的顺序约定

秩二 Dynkin 分类同时确定类型字母与顶点顺序。对于 B2/C2，分类器根据有序 Cartan 条目选择标签，并保留给定顺序；对于 G2，分类器可能交换顶点，使短根在前。这些规则属于分类器的编号约定。^[dynkin.md:51-56]

## 分类规则

对秩二连通分量，分类器按 `support` 升序取顶点 `(i, j)`，计算乘积 `cartan[i][j] * cartan[j][i]`。乘积为 1 时归为 A 型；为 2 时，若 `cartan[i][j] == -1` 则归为 C 型，否则归为 B 型，顶点顺序均保持不变；为 3 时归为 G 型，并在 `cartan[i][j] != -1` 时交换位置，使短根在前。^[dynkin.md:51-56]

这里的“给定顺序”是顶点编号在 `support` 中的升序。B2/C2 的标签由这一顺序下的 Cartan 条目决定，来源将其明确联系到历史 B2/C2 编号教训。^[dynkin.md:51-56]

## 与 Bourbaki 置换的关系

分类器为各分量生成 `position` 向量，`bourbaki_permutation` 将这些向量按分量顺序拼接。返回值 `result[k]` 表示占据 Bourbaki 位置 `k` 的 datum 顶点，因此秩二分类中的保序或交换会体现在最终置换中，参见 [[Bourbaki 顶点排序与置换语义]]。^[dynkin.md:51-56, dynkin.md:74-76]

`bourbaki_permutation` 是该模块在 crate 外的唯一入口，语言层用它按 Bourbaki 编号打印 grading；`classify` 与 `DynkinComponent` 则仅在 crate 内可见。^[dynkin.md:19-21]

## 测试锚点

来源列出的测试覆盖 A2 分类为 A；`cartan[0][1] = -2` 的 B2 分类为 B 且不换序；`cartan[0][1] = -1` 的 C2 分类为 C。标准 B2/C2 的 Bourbaki 置换均为平凡置换。G2 的两种指向分别覆盖交换和不交换位置的情况，均遵循短根在前的约定。^[dynkin.md:89-96]

## 输入与证据边界

上述规则以输入满足 based-datum Cartan 不变量为前提。入口检查矩阵方形、对角元为 2、非对角元属于 `-3..=0`，以及零模式对称，但不校验非零条目的数值配对。完整输入契约见 [[Cartan 矩阵输入契约与连通分量划分]]。^[dynkin.md:38-41]

来源属于结构性源码阅读，未核对上游源码字节，也不声称分类器通过数学验收。“精确复现 upstream”是注释中的意图声明；现有六个测试全部覆盖成功路径，错误路径没有测试覆盖。相关结论应结合 [[Dynkin 分类器的测试覆盖与证据边界]] 理解。^[dynkin.md:10-12, dynkin.md:100-102]

## Sources

- [dynkin.md](dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）
