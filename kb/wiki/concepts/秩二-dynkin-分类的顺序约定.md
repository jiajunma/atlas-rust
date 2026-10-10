---
title: 秩二 Dynkin 分类的顺序约定
summary: 秩二按非对角条目乘积分型，B2/C2 标签由输入顶点顺序决定且不换序，G2 则按条目方向调整为短根在前。
sources:
  - dynkin.md
kind: concept
createdAt: "2026-10-09T14:45:26.234Z"
updatedAt: "2026-10-10T00:30:39.471Z"
tags:
  - Dynkin分类
  - Bourbaki编号
aliases:
  - 秩二-dynkin-分类的顺序约定
  - 秩D分
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 秩二 Dynkin 分类的顺序约定
summary: 秩二分类按非对角 Cartan 条目的乘积分型；B2/C2 标签由给定顶点顺序决定且保序，G2 则按条目方向调整为短根在前。
sources:
  - dynkin.md
kind: concept
tags:
  - Dynkin分类
  - 编号约定
  - 兼容性
---

# 秩二 Dynkin 分类的顺序约定

秩二 Dynkin 分类同时确定类型字母与顶点顺序。在 `dynkin.rs` 的单分量分类中，B2/C2 根据有序 Cartan 条目选择标签，并保留给定顺序；G2 则可能交换顶点，使短根在前。^[dynkin.md:49-56]

## 分类规则

对秩二连通分量，分类器按 `support` 的升序取顶点 `(i, j)`，再根据乘积 `cartan[i][j] * cartan[j][i]` 分派：乘积为 1 时判为 A；为 2 时，若 `cartan[i][j] == -1` 则判为 C，否则判为 B，两者均不交换位置；为 3 时判为 G，并在 `cartan[i][j] != -1` 时交换位置，使短根在前。^[dynkin.md:51-56]

这里的“给定顺序”指分量中顶点编号的升序。B2/C2 的特殊性在于：同构的秩二系统，其 B 或 C 标签由这一顺序下的 Cartan 条目决定。来源将该规则明确联系到历史 B2/C2 编号教训。^[dynkin.md:51-56]

## 与 Bourbaki 置换的关系

分类器为各分量生成 `position` 向量，`bourbaki_permutation` 将这些向量按分量顺序拼接。返回值 `result[k]` 表示占据 Bourbaki 位置 `k` 的 datum 顶点，因此秩二分类中的保序或交换会反映在最终置换中，详见 [[Bourbaki 顶点排序与置换语义]]。^[dynkin.md:51-56, dynkin.md:74-76]

`bourbaki_permutation` 是该模块在 crate 外的唯一入口，语言层使用它按 Bourbaki 编号打印 grading；`classify` 与 `DynkinComponent` 仅在 crate 内可见。^[dynkin.md:19-21]

## 测试锚点

来源列出的测试覆盖 A2 分类为 A、`cartan[0][1] = -2` 的 B2 分类为 B 且不换序，以及 `cartan[0][1] = -1` 的 C2 分类为 C。标准 B2/C2 的 Bourbaki 置换均为平凡置换。G2 的两种指向分别覆盖交换和不交换位置的情况，均遵循短根在前的约定。^[dynkin.md:89-96]

## 输入与证据边界

上述规则以输入满足 based-datum Cartan 不变量为前提。入口检查矩阵方形、对角元为 2、非对角元属于 `-3..=0`，以及零模式对称，但不校验非零条目的数值配对。完整输入契约见 [[Cartan 矩阵输入契约与连通分量划分]]。^[dynkin.md:38-41]

来源属于结构性源码阅读，未核对上游源码字节，也不声称分类器通过数学验收。“精确复现 upstream”是注释中的意图声明；六个测试全部覆盖成功路径，错误路径没有测试覆盖。本次知识维护未执行 Atlas、Cargo、测试或 benchmark，测试锚点的描述不代表本次执行结果。相关范围见 [[Dynkin 分类器的测试覆盖与证据边界]]。^[dynkin.md:10-12, dynkin.md:100-102, dynkin.md:111-115]

## Sources

- [dynkin.md](../../sources/dynkin.md) — Dynkin 分类器：连通分量、Bourbaki 置换与折叠 Cartan（dynkin.rs）。
