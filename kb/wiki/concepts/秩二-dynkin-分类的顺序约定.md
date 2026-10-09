---
title: 秩二 Dynkin 分类的顺序约定
summary: 秩二分类按两个非对角 Cartan 条目的乘积分型，其中 B2/C2 标签由给定顶点顺序决定且保持顺序，G2 则按条目方向决定是否交换位置以使短根在前。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:26.234Z"
updatedAt: "2026-10-09T14:45:26.234Z"
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 秩二 Dynkin 分类的顺序约定

秩二 Dynkin 分类不仅确定类型字母，还规定顶点的输出顺序。对于 B2/C2，同构的秩二系统采用哪个标签，由有序 Cartan 条目决定，分类器保留给定顺序；对于 G2，分类器可能交换顶点，使短根在前。^[dynkin.md:51-56]

## 分类规则

对一个秩二连通分量，分类器按 `support` 的升序取顶点 `(i, j)`，再根据乘积 `cartan[i][j] * cartan[j][i]` 分派。乘积为 1 时归为 A 型；乘积为 2 时，若 `cartan[i][j] == -1` 则归为 C 型，否则归为 B 型，两者都不交换顶点；乘积为 3 时归为 G 型，若 `cartan[i][j] != -1` 则交换位置，使短根在前。^[dynkin.md:51-56]

这里的“给定顺序”具体体现为原始顶点编号在 `support` 中的升序。B2/C2 的规则明确保留这一顺序，而不是通过交换顶点统一类型标签；来源将其标记为历史 B2/C2 编号教训的落点。^[dynkin.md:51-56]

## 与 Bourbaki 置换的关系

分类结果的 `position` 向量参与构造 [[Bourbaki 顶点排序与置换语义|Bourbaki 置换]]：`bourbaki_permutation` 按分量顺序拼接各个 `position`，其中 `result[k]` 表示占据 Bourbaki 位置 `k` 的 datum 顶点。因此，秩二分类中的保序或交换决定了对应顶点在置换结果中的顺序。^[dynkin.md:74-76]

已有测试确认：`cartan[0][1] = -2` 的 B2 分类为 B 且不换序，`cartan[0][1] = -1` 的 C2 分类为 C；标准 B2/C2 的 Bourbaki 置换均为平凡置换。G2 的两种指向分别覆盖交换与不交换位置的情况，均以短根在前为约定。^[dynkin.md:89-96]

## 输入与证据边界

上述规则处于 [[Cartan 矩阵输入契约与连通分量划分|Cartan 矩阵分类]] 的输入契约之内：输入须满足 based-datum Cartan 不变量。入口检查方形、对角元为 2、非对角元属于 `-3..=0` 以及零模式对称，但不校验非零条目的数值配对；不能把这些检查视为完整的数学有效性验收。^[dynkin.md:38-41]

本来源属于结构性阅读，未核对上游源码字节；“精确复现 upstream”是注释中的意图声明。现有六个测试均覆盖成功路径，错误路径没有测试覆盖，相关结论应结合 [[Dynkin 分类器的测试覆盖与证据边界]] 理解。^[dynkin.md:10-12, dynkin.md:100-102]

## Sources

- [dynkin.md](dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）
